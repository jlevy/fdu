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
    // One completeness, because there is now one population. While a portable name was
    // optional this had to distinguish "the native child set is authoritative" from
    // "a portable consumer may trust it", since an entry with no portable name was in the
    // first and absent from the second. Every entry has a portable name now, so the two
    // questions have the same answer and asking twice would only invite them to diverge.
    Ok(ProjectionResult::Tree(Knowledge::Present(TreePage {
        complete: directory.children_complete.unwrap_or(false),
        directory,
        rows,
        next: continuation,
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
        let native = index.path_of(*id).unwrap_or_default();
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
    Ok(ProjectionResult::Flat(crate::FlatPage { rows, next: continuation }))
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
        let native = index.path_of(*id).unwrap_or_default();
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
        // The arena owns the native path. Joining the child *name* here would be joining
        // an escaped component onto a native parent, and the row's portable form would
        // then be derived from an already-escaped string and escaped a second time —
        // `x%FF` became `x%25FF`. The names in this map are portable by construction, so
        // they are for ordering and resumption, never for addressing.
        let path = index.path_of(*id).unwrap_or_else(|| parent.join(name));
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
            // Directory completeness alone settles this now. It once had to be paired
            // with "and that directory omitted nothing", because a sibling whose name had
            // no portable form was retained but unlistable, so the name asked about might
            // have been hiding in that invisible set and absence could not be claimed.
            // Every entry is listable now, so a complete directory that does not hold the
            // name genuinely does not hold it.
            return index.directory_complete(&parent) == Some(true);
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

/// Derive the canonical POSIX-relative form of a native path.
///
/// Total: every path has one. Components join with `/`, and two kinds of byte are
/// percent-escaped — those that are not valid UTF-8, and `%` itself.
///
/// Escaping `%` everywhere is not decoration, it is what makes the mapping injective.
/// A file literally named `caf%FF.txt` is valid UTF-8; a file named `caf<0xFF>.txt` is
/// not. Escaping only the invalid byte maps both to `caf%FF.txt`, and two distinct files
/// would share one wire name — the aliasing bug of lossy conversion wearing better
/// output. So a literal `%` becomes `%25`, and the cost is that `100%.txt` transmits as
/// `100%25.txt`.
///
/// This is not URI encoding. The result is a JSON string, not a URL, so spaces, `#`, `?`
/// and every non-ASCII scalar pass through untouched: `café/naïve.txt` is unchanged.
///
/// The partial version this replaced returned `None` for a non-UTF-8 component, which is
/// why ordered pages and native roll-ups used to answer over different populations, why a
/// directory with one stray byte in its name hid its whole subtree from browsing, and why
/// omission counts, bounded escaped examples and a second completeness flag existed at
/// all. Every mature system that meets this problem — git's quoted paths, Python's
/// surrogate escapes, the `file://` URIs that LSP and the desktop file managers exchange
/// — makes the derived name total. None of them tells a caller that a file has no name.
pub(crate) fn portable_path(path: &Path) -> crate::PortablePath {
    let mut portable = String::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            continue;
        };
        if !portable.is_empty() {
            portable.push('/');
        }
        push_component(&mut portable, component);
    }
    crate::PortablePath::new(portable)
}

/// The canonical form of one path component, escaped by the same rules.
///
/// A child-name map is keyed by this rather than by the native name, so a component whose
/// bytes are not UTF-8 still has a key and its directory can still be listed.
pub(crate) fn portable_component(component: &std::ffi::OsStr) -> String {
    let mut out = String::new();
    push_component(&mut out, component);
    out
}

fn push_component(out: &mut String, component: &std::ffi::OsStr) {
    match component.to_str() {
        Some(text) => push_text(out, text),
        None => push_unrepresentable(out, component),
    }
}

/// Append valid UTF-8, escaping only `%`.
fn push_text(out: &mut String, text: &str) {
    if !text.contains('%') {
        out.push_str(text);
        return;
    }
    for character in text.chars() {
        if character == '%' {
            out.push_str("%25");
        } else {
            out.push(character);
        }
    }
}

fn push_byte(out: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    out.push('%');
    out.push(char::from(HEX[usize::from(byte >> 4)]));
    out.push(char::from(HEX[usize::from(byte & 0x0f)]));
}

/// Append platform bytes that are not wholly valid UTF-8.
///
/// Valid runs are kept as text so a name that is mostly readable stays mostly readable;
/// only the bytes that cannot be decoded are escaped.
fn push_lossy_bytes(out: &mut String, mut bytes: &[u8]) {
    loop {
        match std::str::from_utf8(bytes) {
            Ok(text) => {
                push_text(out, text);
                return;
            }
            Err(error) => {
                let valid = error.valid_up_to();
                if valid > 0 {
                    let text = std::str::from_utf8(&bytes[..valid])
                        .expect("the prefix utf8 validation just accepted");
                    push_text(out, text);
                }
                // `None` means the input ended mid-sequence, so the remainder is escaped.
                let invalid = error.error_len().unwrap_or(bytes.len() - valid);
                for byte in &bytes[valid..valid + invalid] {
                    push_byte(out, *byte);
                }
                bytes = &bytes[valid + invalid..];
            }
        }
    }
}

#[cfg(unix)]
fn push_unrepresentable(out: &mut String, component: &std::ffi::OsStr) {
    use std::os::unix::ffi::OsStrExt;

    push_lossy_bytes(out, component.as_bytes());
}

/// Windows names are UTF-16 that need not be well formed.
///
/// An unpaired surrogate has no UTF-8 encoding at all, so its two code-unit bytes are
/// escaped big-endian, matching how `PortablePathEncoding::WindowsWtf16Le` already
/// describes native bytes elsewhere.
#[cfg(windows)]
fn push_unrepresentable(out: &mut String, component: &std::ffi::OsStr) {
    use std::os::windows::ffi::OsStrExt;

    for unit in char::decode_utf16(component.encode_wide()) {
        match unit {
            Ok('%') => out.push_str("%25"),
            Ok(character) => out.push(character),
            Err(unpaired) => {
                for byte in unpaired.unpaired_surrogate().to_be_bytes() {
                    push_byte(out, byte);
                }
            }
        }
    }
}

#[cfg(not(any(unix, windows)))]
fn push_unrepresentable(out: &mut String, component: &std::ffi::OsStr) {
    push_lossy_bytes(out, component.as_encoded_bytes());
}
