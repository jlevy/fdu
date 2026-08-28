//! Coherent bounded projections over one opened root.

use std::collections::BTreeMap;
use std::ops::Bound;
use std::path::{Component, Path, PathBuf};

use super::OpenedIndex;
use super::continuation::{ChildPartition, ChildPosition, ContinuationKind, ContinuationRecord};
use crate::{
    Coverage, CoverageReason, EngineVersion, EntryId, Error, Knowledge, LimitedProjection,
    PageRequest, ProjectionResult, QueryLimit, ReadRequest, ReadResponse, Result, TreePage, Work,
};

pub(super) fn read(opened: &OpenedIndex, request: ReadRequest) -> Result<ReadResponse> {
    if request.projections.len() > crate::MAX_READ_PROJECTIONS {
        return Err(Error::ReadProjectionLimit {
            attempted: request.projections.len(),
            limit: crate::MAX_READ_PROJECTIONS,
        });
    }
    validate_request(&request)?;

    opened.state.index.read_with(|index| {
        let scope = index.scope();
        let version = EngineVersion {
            session: opened.state.session,
            sequence: index.clock(),
            scope: scope.scope_identity(),
            semantics: scope.semantic_identity(),
        };
        if let Some(expected) = request.expected {
            if expected != version {
                return Err(Error::VersionUnavailable {
                    requested: Box::new(expected),
                    current: Box::new(version),
                });
            }
        }

        let state = index.state();
        let mut work = Work::default();
        let mut results = Vec::with_capacity(request.projections.len());
        for projection in request.projections {
            match projection {
                crate::ReadProjection::Lookup { path } => {
                    let path = crate::scan::normalize_subtree(&path)?;
                    charge_path(&mut work, &path);
                    let value = match index.entry_value(&path) {
                        Some(entry) => {
                            work.rows_returned = work.rows_returned.saturating_add(1);
                            Knowledge::Present(entry)
                        }
                        None if absence_is_known(index, &path) => Knowledge::Absent,
                        None => Knowledge::Unknown {
                            reason: match state.coverage {
                                Coverage::Partial(reason) => reason,
                                Coverage::Complete => CoverageReason::Building,
                            },
                        },
                    };
                    results.push(ProjectionResult::Lookup(value));
                }
                crate::ReadProjection::RollUp { path } => {
                    let path = crate::scan::normalize_subtree(&path)?;
                    charge_path(&mut work, &path);
                    work.maintained_index_work = work.maintained_index_work.saturating_add(1);
                    let value = match index.partition_rollup_summary(&path) {
                        Some(rollup) => {
                            work.rows_returned = work.rows_returned.saturating_add(1);
                            Knowledge::Present(rollup)
                        }
                        None if absence_is_known(index, &path) => Knowledge::Absent,
                        None => Knowledge::Unknown {
                            reason: match state.coverage {
                                Coverage::Partial(reason) => reason,
                                Coverage::Complete => CoverageReason::Building,
                            },
                        },
                    };
                    results.push(ProjectionResult::RollUp(value));
                }
                crate::ReadProjection::Tree { path, page } => {
                    validate_page(page)?;
                    let path = crate::scan::normalize_subtree(&path)?;
                    results.push(tree_projection(
                        opened,
                        index,
                        &path,
                        page,
                        version,
                        state.coverage,
                        None,
                        &mut work,
                    )?);
                }
                crate::ReadProjection::Continue { continuation, page } => {
                    validate_page(page)?;
                    let record = {
                        let mut table = opened
                            .state
                            .continuations
                            .lock()
                            .map_err(|_| Error::OpenedLifecyclePoisoned)?;
                        table.take(opened.state.session, continuation)?
                    };
                    if record.version != version {
                        return Err(Error::ContinuationStale {
                            requested: Box::new(record.version),
                            current: Box::new(version),
                        });
                    }
                    let retry = record.clone();
                    let result = match record.kind {
                        ContinuationKind::Tree { path, next } => tree_projection(
                            opened,
                            index,
                            &path,
                            page,
                            version,
                            state.coverage,
                            Some(&next),
                            &mut work,
                        ),
                        ContinuationKind::Flat { selection, shape, next } => flat_projection(
                            opened,
                            index,
                            &selection,
                            shape,
                            page,
                            version,
                            Some(next.as_str()),
                            &mut work,
                        ),
                    };
                    if result.is_err() || matches!(result, Ok(ProjectionResult::Limit(_))) {
                        let mut table = opened
                            .state
                            .continuations
                            .lock()
                            .map_err(|_| Error::OpenedLifecyclePoisoned)?;
                        table.restore(continuation, retry);
                    }
                    results.push(result?);
                }
                crate::ReadProjection::Flat { selection, shape, page } => {
                    validate_page(page)?;
                    validate_flat_selection(&selection)?;
                    results.push(flat_projection(
                        opened, index, &selection, shape, page, version, None, &mut work,
                    )?);
                }
                crate::ReadProjection::Aggregate { selection, count_cap, max_work } => {
                    validate_flat_selection(&selection)?;
                    validate_count(count_cap, max_work)?;
                    results.push(aggregate_projection(
                        index, &selection, count_cap, max_work, &mut work,
                    ));
                }
                crate::ReadProjection::Report(request) => {
                    results.push(report_projection(index, &request, state, &mut work)?);
                }
                crate::ReadProjection::Diagnostics => {
                    results.push(ProjectionResult::Diagnostics(crate::ReadDiagnostics {
                        root: index.root_path().to_path_buf(),
                        scope,
                        entries: index.len(),
                        issues: index.issues().to_vec(),
                    }));
                }
            }
        }
        Ok(ReadResponse { version, state, results, work, change_cursor: version })
    })?
}

fn validate_request(request: &ReadRequest) -> Result<()> {
    for projection in &request.projections {
        match projection {
            crate::ReadProjection::Tree { page, .. }
            | crate::ReadProjection::Continue { page, .. } => validate_page(*page)?,
            crate::ReadProjection::Flat { selection, page, .. } => {
                validate_page(*page)?;
                validate_flat_selection(selection)?;
            }
            crate::ReadProjection::Aggregate { selection, count_cap, max_work } => {
                validate_flat_selection(selection)?;
                validate_count(*count_cap, *max_work)?;
            }
            crate::ReadProjection::Report(request) => validate_report(request)?,
            crate::ReadProjection::Lookup { .. }
            | crate::ReadProjection::RollUp { .. }
            | crate::ReadProjection::Diagnostics => {}
        }
    }
    Ok(())
}

fn report_projection(
    index: &crate::Index,
    request: &crate::ReportRequest,
    state: crate::IndexState,
    work: &mut Work,
) -> Result<ProjectionResult> {
    validate_report(request)?;

    let charge = report_work(index, &request.query);
    if charge.total() > request.max_work {
        work.rows_visited = work.rows_visited.saturating_add(request.max_work);
        return Ok(ProjectionResult::Limit(QueryLimit {
            projection: LimitedProjection::Report,
            max_work: request.max_work,
            rows_visited: request.max_work,
        }));
    }

    let provenance = crate::query::Provenance {
        scan_started_at: None,
        generated_at: request.generated_at,
        source: match state.source {
            crate::Source::Scanned => crate::query::ReportSource::ColdScan,
            crate::Source::Revalidated => crate::query::ReportSource::WarmRevalidate,
            crate::Source::JournalScoped | crate::Source::Cached => {
                crate::query::ReportSource::CacheOnly
            }
        },
        complete: state.coverage == Coverage::Complete,
        errors: index.issues().iter().map(|issue| issue.message.clone()).collect(),
    };
    let report = crate::query::report(index, &request.query, &provenance);
    work.rows_visited = work.rows_visited.saturating_add(charge.rows);
    work.maintained_index_work = work.maintained_index_work.saturating_add(charge.maintained);
    work.rows_returned = work.rows_returned.saturating_add(report_rows(&report));
    Ok(ProjectionResult::Report(report))
}

fn validate_report(request: &crate::ReportRequest) -> Result<()> {
    if request.max_work == 0 || request.max_work > crate::MAX_PAGE_WORK {
        return Err(Error::PageWorkLimit {
            attempted: request.max_work,
            limit: crate::MAX_PAGE_WORK,
        });
    }
    let views = request.query.views.len().saturating_add(request.query.omitted_views.len());
    if views > crate::MAX_REPORT_VIEWS {
        return Err(Error::ReportViewLimit { attempted: views, limit: crate::MAX_REPORT_VIEWS });
    }
    Ok(())
}

#[derive(Clone, Copy, Default)]
struct ReportWork {
    rows: u64,
    maintained: u64,
}

impl ReportWork {
    fn total(self) -> u64 {
        self.rows.saturating_add(self.maintained)
    }
}

fn report_work(index: &crate::Index, query: &crate::query::Query) -> ReportWork {
    let entries = index.len().saturating_sub(1);
    if !query.selection.is_unfiltered() {
        // The report implementation performs one shared filtered walk, then shapes each
        // requested view from that retained result. Charging one full pass for each
        // shaping step is conservative and keeps the bound independent of heap layout.
        let shaping = query.views.iter().fold(0_u64, |total, view| {
            let rows = match view {
                crate::query::ViewSpec::Summary => 1,
                crate::query::ViewSpec::Tree
                | crate::query::ViewSpec::Types
                | crate::query::ViewSpec::Extensions
                | crate::query::ViewSpec::Families
                | crate::query::ViewSpec::Languages
                | crate::query::ViewSpec::Documents
                | crate::query::ViewSpec::Files
                | crate::query::ViewSpec::Largest
                | crate::query::ViewSpec::Recent => entries,
            };
            total.saturating_add(rows)
        });
        return ReportWork { rows: entries.saturating_add(shaping), maintained: 0 };
    }

    query.views.iter().fold(ReportWork::default(), |mut work, view| {
        match view {
            crate::query::ViewSpec::Summary => {
                work.maintained = work.maintained.saturating_add(1);
            }
            crate::query::ViewSpec::Extensions => {
                // The exact extension cardinality is behind an intern table; charging
                // the entry count is a stable upper bound without cloning that table a
                // second time merely to price the query.
                work.maintained = work.maintained.saturating_add(entries);
            }
            crate::query::ViewSpec::Tree
            | crate::query::ViewSpec::Types
            | crate::query::ViewSpec::Families
            | crate::query::ViewSpec::Languages
            | crate::query::ViewSpec::Documents
            | crate::query::ViewSpec::Files
            | crate::query::ViewSpec::Largest
            | crate::query::ViewSpec::Recent => {
                work.rows = work.rows.saturating_add(entries);
            }
        }
        work
    })
}

fn report_rows(report: &crate::query::Report) -> u64 {
    report.sections.iter().fold(0_u64, |total, section| {
        let rows = match section {
            crate::query::Section::Tree(root) => tree_rows(root),
            crate::query::Section::Extensions { rows, .. } => rows.len() as u64,
            crate::query::Section::Metrics { summary, .. } => summary.rows.len() as u64,
            crate::query::Section::Files { rows, .. } => rows.len() as u64,
            crate::query::Section::Summary(_) => 1,
        };
        total.saturating_add(rows)
    })
}

fn tree_rows(root: &crate::query::TreeNode) -> u64 {
    let mut count = 0_u64;
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        count = count.saturating_add(1);
        pending.extend(&node.children);
    }
    count
}

fn validate_count(count_cap: u64, max_work: u64) -> Result<()> {
    if count_cap == 0 || count_cap > crate::MAX_COUNT_CAP {
        return Err(Error::CountCapLimit { attempted: count_cap, limit: crate::MAX_COUNT_CAP });
    }
    if max_work == 0 || max_work > crate::MAX_PAGE_WORK {
        return Err(Error::PageWorkLimit { attempted: max_work, limit: crate::MAX_PAGE_WORK });
    }
    Ok(())
}

fn validate_flat_selection(selection: &crate::query::EntrySelection) -> Result<()> {
    if selection.query.depth.is_some()
        || selection.query.limit.is_some()
        || selection.query.sort.is_some()
        || selection.query.reverse
    {
        return Err(Error::UnsupportedFlatSelection);
    }
    Ok(())
}

fn validate_page(page: PageRequest) -> Result<()> {
    if page.limit == 0 || page.limit > crate::MAX_PAGE_ROWS {
        return Err(Error::PageRowLimit { attempted: page.limit, limit: crate::MAX_PAGE_ROWS });
    }
    if page.max_work == 0 || page.max_work > crate::MAX_PAGE_WORK {
        return Err(Error::PageWorkLimit { attempted: page.max_work, limit: crate::MAX_PAGE_WORK });
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn tree_projection(
    opened: &OpenedIndex,
    index: &crate::Index,
    path: &Path,
    page: PageRequest,
    version: EngineVersion,
    coverage: Coverage,
    start: Option<&ChildPosition>,
    work: &mut Work,
) -> Result<ProjectionResult> {
    let path_work = path.components().count() as u64 + 1;
    if path_work > page.max_work {
        work.rows_visited = work.rows_visited.saturating_add(page.max_work);
        return Ok(ProjectionResult::Limit(QueryLimit {
            projection: LimitedProjection::Tree,
            max_work: page.max_work,
            rows_visited: page.max_work,
        }));
    }
    let Some(directory) = index.entry_value(path) else {
        work.rows_visited = work.rows_visited.saturating_add(path_work);
        return Ok(ProjectionResult::Tree(if absence_is_known(index, path) {
            Knowledge::Absent
        } else {
            Knowledge::Unknown { reason: coverage_reason(coverage) }
        }));
    };
    if !directory.kind.is_dir() {
        work.rows_visited = work.rows_visited.saturating_add(path_work);
        return Ok(ProjectionResult::Tree(Knowledge::Absent));
    }

    let mut rows = Vec::with_capacity(page.limit);
    let mut spent = path_work;
    let mut next = None;
    if let Some(children) = index.portable_children(path) {
        let start_partition = start.map(|position| position.partition);
        if start_partition != Some(ChildPartition::Nondirectories) {
            let start_name = start
                .filter(|position| position.partition == ChildPartition::Directories)
                .map(|position| position.name.as_str());
            next = collect_children(
                index,
                path,
                &children.directories,
                ChildPartition::Directories,
                start_name,
                page,
                &mut rows,
                &mut spent,
            );
        }
        if next.is_none() && rows.len() < page.limit {
            let start_name = start
                .filter(|position| position.partition == ChildPartition::Nondirectories)
                .map(|position| position.name.as_str());
            next = collect_children(
                index,
                path,
                &children.nondirectories,
                ChildPartition::Nondirectories,
                start_name,
                page,
                &mut rows,
                &mut spent,
            );
        } else if next.is_none() && rows.len() == page.limit {
            next = first_position(
                &children.nondirectories,
                ChildPartition::Nondirectories,
                None,
                page.max_work,
                &mut spent,
            );
        }

        if spent > page.max_work {
            work.rows_visited = work.rows_visited.saturating_add(page.max_work);
            work.maintained_index_work =
                work.maintained_index_work.saturating_add(page.max_work.saturating_sub(path_work));
            return Ok(ProjectionResult::Limit(QueryLimit {
                projection: LimitedProjection::Tree,
                max_work: page.max_work,
                rows_visited: page.max_work,
            }));
        }
    }

    work.rows_visited = work.rows_visited.saturating_add(spent);
    work.maintained_index_work =
        work.maintained_index_work.saturating_add(spent.saturating_sub(path_work));
    work.rows_returned = work
        .rows_returned
        .saturating_add(u64::try_from(rows.len()).unwrap_or(u64::MAX).saturating_add(1));

    let continuation = if let Some(next) = next {
        let mut table =
            opened.state.continuations.lock().map_err(|_| Error::OpenedLifecyclePoisoned)?;
        Some(table.insert(
            opened.state.session,
            ContinuationRecord {
                version,
                kind: ContinuationKind::Tree { path: path.to_path_buf(), next },
            },
        )?)
    } else {
        None
    };
    let (omitted, examples) = index.portable_children(path).map_or_else(
        || (0, Vec::new()),
        |children| (children.omitted, portable_examples(index, &children.examples)),
    );
    let native_complete = directory.children_complete.unwrap_or(false);
    let directory_portable = directory.portable_path.is_some();
    let portable_issue = (omitted > 0).then_some(crate::PortablePathIssue { omitted, examples });
    Ok(ProjectionResult::Tree(Knowledge::Present(TreePage {
        directory,
        rows,
        next: continuation,
        native_complete,
        portable_complete: native_complete && omitted == 0 && directory_portable,
        portable_issue,
    })))
}

#[allow(clippy::too_many_arguments)]
fn flat_projection(
    opened: &OpenedIndex,
    index: &crate::Index,
    selection: &crate::query::EntrySelection,
    shape: crate::RowShape,
    page: PageRequest,
    version: EngineVersion,
    start: Option<&str>,
    work: &mut Work,
) -> Result<ProjectionResult> {
    validate_flat_selection(selection)?;
    let entries = index.portable_entries();
    let iterator: Box<dyn Iterator<Item = (&crate::PortablePath, &EntryId)> + '_> = match start {
        Some(start) => {
            Box::new(entries.range::<str, _>((Bound::Included(start), Bound::Unbounded)))
        }
        None => Box::new(entries.iter()),
    };
    let mut rows = Vec::with_capacity(page.limit);
    let mut spent = 0_u64;
    let mut next = None;
    for (portable, id) in iterator {
        spent = spent.saturating_add(1);
        if spent > page.max_work {
            work.rows_visited = work.rows_visited.saturating_add(page.max_work);
            work.maintained_index_work = work.maintained_index_work.saturating_add(page.max_work);
            return Ok(ProjectionResult::Limit(QueryLimit {
                projection: LimitedProjection::Flat,
                max_work: page.max_work,
                rows_visited: page.max_work,
            }));
        }
        let native = portable.to_native_relative_path();
        let mut row = index.entry_value_of(*id, &native);
        let name = portable.as_str().rsplit('/').next().unwrap_or(portable.as_str());
        let candidate = crate::query::Candidate {
            relative: &native,
            name,
            kind: row.kind,
            bytes: row.attrs.size,
            allocated: row.attrs.allocated,
            mtime_ns: row.attrs.mtime_ns,
        };
        if !selection.admits(&candidate, row.ignored) {
            continue;
        }
        if rows.len() == page.limit {
            next = Some(portable.clone());
            break;
        }
        if shape == crate::RowShape::Compact {
            row.rollup = None;
            row.children_complete = None;
        }
        rows.push(row);
    }

    work.rows_visited = work.rows_visited.saturating_add(spent);
    work.maintained_index_work = work.maintained_index_work.saturating_add(spent);
    work.rows_returned =
        work.rows_returned.saturating_add(u64::try_from(rows.len()).unwrap_or(u64::MAX));
    let continuation = if let Some(next) = next {
        let mut table =
            opened.state.continuations.lock().map_err(|_| Error::OpenedLifecyclePoisoned)?;
        Some(table.insert(
            opened.state.session,
            ContinuationRecord {
                version,
                kind: ContinuationKind::Flat {
                    selection: Box::new(selection.clone()),
                    shape,
                    next,
                },
            },
        )?)
    } else {
        None
    };
    let (omitted, examples) = index.portable_issue();
    let portable_issue = (omitted > 0).then(|| crate::PortablePathIssue {
        omitted,
        examples: portable_examples(index, examples),
    });
    Ok(ProjectionResult::Flat(crate::FlatPage { rows, next: continuation, portable_issue }))
}

fn aggregate_projection(
    index: &crate::Index,
    selection: &crate::query::EntrySelection,
    count_cap: u64,
    max_work: u64,
    work: &mut Work,
) -> ProjectionResult {
    if selection.is_unfiltered() {
        work.maintained_index_work = work.maintained_index_work.saturating_add(1);
        work.rows_returned = work.rows_returned.saturating_add(1);
        let count = u64::try_from(index.portable_entries().len()).unwrap_or(u64::MAX);
        return ProjectionResult::Aggregate(crate::CountResult::Exact(count));
    }

    let mut spent = 0_u64;
    let mut matches = 0_u64;
    for (portable, id) in index.portable_entries() {
        spent = spent.saturating_add(1);
        if spent > max_work {
            work.rows_visited = work.rows_visited.saturating_add(max_work);
            work.maintained_index_work = work.maintained_index_work.saturating_add(max_work);
            return ProjectionResult::Limit(QueryLimit {
                projection: LimitedProjection::Aggregate,
                max_work,
                rows_visited: max_work,
            });
        }
        let native = portable.to_native_relative_path();
        let row = index.entry_value_of(*id, &native);
        let name = portable.as_str().rsplit('/').next().unwrap_or(portable.as_str());
        let candidate = crate::query::Candidate {
            relative: &native,
            name,
            kind: row.kind,
            bytes: row.attrs.size,
            allocated: row.attrs.allocated,
            mtime_ns: row.attrs.mtime_ns,
        };
        if !selection.admits(&candidate, row.ignored) {
            continue;
        }
        if matches == count_cap {
            work.rows_visited = work.rows_visited.saturating_add(spent);
            work.maintained_index_work = work.maintained_index_work.saturating_add(spent);
            work.rows_returned = work.rows_returned.saturating_add(1);
            return ProjectionResult::Aggregate(crate::CountResult::AtLeast(count_cap));
        }
        matches = matches.saturating_add(1);
    }
    work.rows_visited = work.rows_visited.saturating_add(spent);
    work.maintained_index_work = work.maintained_index_work.saturating_add(spent);
    work.rows_returned = work.rows_returned.saturating_add(1);
    ProjectionResult::Aggregate(crate::CountResult::Exact(matches))
}

#[allow(clippy::too_many_arguments)]
fn collect_children(
    index: &crate::Index,
    parent: &Path,
    children: &BTreeMap<String, EntryId>,
    partition: ChildPartition,
    start: Option<&str>,
    page: PageRequest,
    rows: &mut Vec<crate::EntryValue>,
    spent: &mut u64,
) -> Option<ChildPosition> {
    // Child names, not portable paths: one component, no separators, already relative to
    // `parent`. Keeping them `String` is what makes `parent.join(name)` below correct.
    let iterator: Box<dyn Iterator<Item = (&String, &EntryId)> + '_> = match start {
        Some(start) => Box::new(children.range(start.to_string()..)),
        None => Box::new(children.iter()),
    };
    for (name, id) in iterator {
        *spent = spent.saturating_add(1);
        if *spent > page.max_work || rows.len() == page.limit {
            return Some(ChildPosition { partition, name: name.clone() });
        }
        let path = parent.join(name);
        rows.push(index.entry_value_of(*id, &path));
    }
    None
}

fn first_position(
    children: &BTreeMap<String, EntryId>,
    partition: ChildPartition,
    start: Option<&str>,
    max_work: u64,
    spent: &mut u64,
) -> Option<ChildPosition> {
    let first = match start {
        Some(start) => children.range(start.to_string()..).next(),
        None => children.first_key_value(),
    }?;
    *spent = spent.saturating_add(1);
    if *spent > max_work {
        return Some(ChildPosition { partition, name: first.0.clone() });
    }
    Some(ChildPosition { partition, name: first.0.clone() })
}

fn charge_path(work: &mut Work, path: &Path) {
    work.rows_visited = work.rows_visited.saturating_add(path.components().count() as u64 + 1);
}

fn absence_is_known(index: &crate::Index, path: &Path) -> bool {
    let mut parent = PathBuf::new();
    let mut components = path.components().peekable();
    while let Some(Component::Normal(component)) = components.next() {
        let candidate = parent.join(component);
        let Some(kind) = index.kind(&candidate) else {
            return index.directory_complete(&parent) == Some(true)
                && index.portable_children(&parent).is_none_or(|children| children.omitted == 0);
        };
        if components.peek().is_some() && !kind.is_dir() {
            return true;
        }
        parent = candidate;
    }
    false
}

fn coverage_reason(coverage: Coverage) -> CoverageReason {
    match coverage {
        Coverage::Partial(reason) => reason,
        Coverage::Complete => CoverageReason::Building,
    }
}

fn portable_examples(
    index: &crate::Index,
    examples: &[EntryId],
) -> Vec<crate::PortablePathExample> {
    examples
        .iter()
        .filter_map(|id| index.path_of(*id))
        .map(|path| portable_path_example(&path))
        .collect()
}

#[cfg(unix)]
fn portable_path_example(path: &Path) -> crate::PortablePathExample {
    use std::os::unix::ffi::OsStrExt;

    encode_path_example(
        crate::PortablePathEncoding::UnixBytes,
        path.as_os_str().as_bytes().iter().copied(),
    )
}

#[cfg(windows)]
fn portable_path_example(path: &Path) -> crate::PortablePathExample {
    use std::os::windows::ffi::OsStrExt;

    encode_path_example(
        crate::PortablePathEncoding::WindowsWtf16Le,
        path.as_os_str().encode_wide().flat_map(u16::to_le_bytes),
    )
}

#[cfg(not(any(unix, windows)))]
fn portable_path_example(path: &Path) -> crate::PortablePathExample {
    encode_path_example(
        crate::PortablePathEncoding::PlatformBytes,
        path.as_os_str().as_encoded_bytes().iter().copied(),
    )
}

fn encode_path_example(
    encoding: crate::PortablePathEncoding,
    bytes: impl Iterator<Item = u8>,
) -> crate::PortablePathExample {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut encoded_hex = String::with_capacity(crate::MAX_PORTABLE_PATH_EXAMPLE_BYTES * 2);
    let mut truncated = false;
    for (position, byte) in bytes.enumerate() {
        if position == crate::MAX_PORTABLE_PATH_EXAMPLE_BYTES {
            truncated = true;
            break;
        }
        encoded_hex.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded_hex.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    crate::PortablePathExample { encoding, encoded_hex, truncated }
}

/// Derive the canonical POSIX-relative form of a native path, when it has one.
///
/// `None` means some component is not valid UTF-8, so no portable name exists and the
/// entry cannot enter any ordered projection. The whole decision is the `?` below: a
/// native path is bytes, and nothing obliges those bytes to be UTF-8.
///
/// The partiality is deliberate today — fdu declines to invent a name it cannot reverse.
/// It is also the reason ordered pages and native roll-ups answer over different
/// populations, which every consumer then has to reconcile. Making the encoding total by
/// escaping the offending bytes would collapse the two, and is tracked as a design
/// decision rather than assumed here.
pub(crate) fn portable_path(path: &Path) -> Option<crate::PortablePath> {
    let mut portable = String::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            continue;
        };
        let component = component.to_str()?;
        if !portable.is_empty() {
            portable.push('/');
        }
        portable.push_str(component);
    }
    Some(crate::PortablePath::new(portable))
}
