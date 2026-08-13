//! Views over a built index, and the report they produce.
//!
//! Every view is a pure function of an index and a [`Selection`]: they read, and nothing
//! else. Producers submit observations and the index commits them; a report can never
//! become a third way to change state.
//!
//! # Two performance tiers
//!
//! An unfiltered request reads the roll-up state the index already maintains, so it costs
//! O(directories) for a tree and O(1) for a summary regardless of how many files the tree
//! holds. Any selection filter forces the other tier: the report walks the retained
//! entries and re-aggregates only what the filter admits, because a pre-computed roll-up
//! cannot answer a question about a subset. Both tiers are milliseconds warm and neither
//! touches the filesystem; the difference is visible in a profile, not in a user's wait.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::classify::derive_ext;
use crate::index::{EntryId, ExtTally, Index, RollUp};
use crate::query::selection::{Bound, Candidate, Selection, SizeMetric, SortKey};
use crate::types::{EntryKind, Freshness, ScanScope};

/// Which roll-up or listing a view reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewSpec {
    /// Per-directory roll-ups down the hierarchy.
    Tree,
    /// One row per derived extension.
    Types,
    /// A flat listing of matching entries.
    Files,
    /// One aggregate row for everything selected.
    Summary,
}

impl ViewSpec {
    /// The ordering this view uses when the caller did not choose one.
    fn default_sort(self) -> SortKey {
        match self {
            // Size-ranked by default, because "what is big" is the question these answer.
            Self::Tree | Self::Types | Self::Summary => SortKey::Size,
            // A flat listing is a file list first, so it reads and diffs in name order.
            Self::Files => SortKey::Name,
        }
    }
}

/// What a report was asked for.
#[derive(Clone, Debug, Default)]
pub struct Query {
    /// Which entries to consider and how to shape results.
    pub selection: Selection,
    /// Which views to report, in the order they were requested.
    pub views: Vec<ViewSpec>,
}

/// Which tier of the freshness ladder produced the index behind a report.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReportSource {
    /// The tree was walked from scratch.
    ColdScan,
    /// A snapshot was loaded and revalidated against the filesystem.
    WarmRevalidate,
    /// A snapshot answered without the filesystem being consulted.
    CacheOnly,
}

/// Facts about how a report's index was produced.
///
/// Passed in rather than sampled inside [`report`] so the view layer stays a pure
/// function of its inputs: the same index and query must always produce the same report,
/// which is what makes the goldens meaningful and the tests deterministic.
#[derive(Clone, Debug)]
pub struct Provenance {
    /// When the walk or revalidation behind this index began.
    ///
    /// This, not the finish time, is the sound watermark for an incremental follow-up
    /// query: a file modified mid-scan may have been observed before the modification,
    /// so only the start bound is conservative.
    pub scan_started_at: Option<SystemTime>,
    /// When the report was rendered.
    pub generated_at: SystemTime,
    /// Which cache tier answered.
    pub source: ReportSource,
    /// Whether every path in scope was read successfully.
    pub complete: bool,
    /// Per-path failures that made this result partial, already rendered.
    ///
    /// Rendered strings rather than error values: this crosses into serialization, and a
    /// report is evidence about a run, not a place to re-handle its errors.
    pub errors: Vec<String>,
}

/// One directory's row in a tree view.
#[derive(Clone, Debug)]
pub struct TreeNode {
    /// Path relative to the index root; empty for the root itself.
    pub path: PathBuf,
    /// Final path component, or `.` for the root.
    pub name: String,
    /// What the entry is.
    pub kind: EntryKind,
    /// Apparent bytes in this subtree.
    pub bytes: u64,
    /// Allocated bytes in this subtree.
    pub allocated: u64,
    /// Files in this subtree.
    pub files: u64,
    /// Directories in this subtree.
    pub dirs: u64,
    /// Newest modification time in this subtree, when it holds any files.
    pub newest_mtime_ns: Option<i64>,
    /// Children reported beneath this node.
    pub children: Vec<TreeNode>,
    /// Whether children were withheld by the depth or limit bound.
    pub truncated: bool,
}

impl Drop for TreeNode {
    /// Release children iteratively.
    ///
    /// The derived drop glue recurses once per level, so a deeply nested tree would
    /// exhaust the stack on release even after every renderer was made iterative — the
    /// same hazard the index avoids when freeing a subtree. Taking the children out first
    /// turns that recursion into a loop.
    fn drop(&mut self) {
        let mut pending = std::mem::take(&mut self.children);
        while let Some(mut node) = pending.pop() {
            pending.extend(std::mem::take(&mut node.children));
        }
    }
}

/// One extension's row in a types view.
#[derive(Clone, Debug)]
pub struct TypeRow {
    /// The derived extension, including its leading dot.
    pub extension: String,
    /// Files with this extension.
    pub files: u64,
    /// Apparent bytes across those files.
    pub bytes: u64,
    /// Allocated bytes across those files.
    pub allocated: u64,
}

/// One entry's row in a files view.
#[derive(Clone, Debug)]
pub struct FileRow {
    /// Path relative to the index root.
    pub path: PathBuf,
    /// What the entry is.
    pub kind: EntryKind,
    /// Apparent bytes.
    pub bytes: u64,
    /// Allocated bytes.
    pub allocated: u64,
    /// Modification time in nanoseconds since the Unix epoch.
    pub mtime_ns: i64,
}

/// The aggregate row of a summary view.
#[derive(Clone, Copy, Debug, Default)]
pub struct SummaryRow {
    /// Files selected.
    pub files: u64,
    /// Directories selected.
    pub dirs: u64,
    /// Apparent bytes.
    pub bytes: u64,
    /// Allocated bytes.
    pub allocated: u64,
    /// Newest modification time, when anything was selected.
    pub newest_mtime_ns: Option<i64>,
}

/// One view's results.
#[derive(Clone, Debug)]
pub enum Section {
    /// A tree view.
    Tree(TreeNode),
    /// A types view.
    Types(Vec<TypeRow>),
    /// A files view.
    Files(Vec<FileRow>),
    /// A summary view.
    Summary(SummaryRow),
}

impl Section {
    /// Which view produced this section.
    pub fn view(&self) -> ViewSpec {
        match self {
            Self::Tree(_) => ViewSpec::Tree,
            Self::Types(_) => ViewSpec::Types,
            Self::Files(_) => ViewSpec::Files,
            Self::Summary(_) => ViewSpec::Summary,
        }
    }
}

/// A rendered answer: provenance, plus one section per requested view.
#[derive(Clone, Debug)]
pub struct Report {
    /// When the walk behind this index began.
    pub scan_started_at: Option<SystemTime>,
    /// When this report was rendered.
    pub generated_at: SystemTime,
    /// Which cache tier answered.
    pub source: ReportSource,
    /// Whether every path in scope was read successfully.
    pub complete: bool,
    /// Per-path failures that made this result partial.
    pub errors: Vec<String>,
    /// How current the index is.
    pub freshness: Freshness,
    /// The immutable scan scope the index represents.
    pub scope: ScanScope,
    /// Absolute path of the indexed root.
    pub root: PathBuf,
    /// Which size metric this report answers in.
    ///
    /// Carried on the report so a renderer shows the same number the ordering used;
    /// printing apparent bytes beside an allocated-bytes ranking looks like a sorting
    /// bug and is worse than either metric alone.
    pub size: SizeMetric,
    /// One section per requested view, in request order.
    pub sections: Vec<Section>,
}

/// Build a report from an index.
///
/// Pure: the same index, query, and provenance always produce the same report, and
/// nothing here reads the filesystem or mutates the index.
pub fn report(index: &Index, query: &Query, provenance: &Provenance) -> Report {
    // One traversal serves every filtered view in the request, so asking for three views
    // costs one pass rather than three.
    let walked = (!query.selection.is_unfiltered()).then(|| walk(index, &query.selection));

    let sections = query
        .views
        .iter()
        .map(|view| build_section(*view, index, query, walked.as_ref()))
        .collect();

    Report {
        scan_started_at: provenance.scan_started_at,
        generated_at: provenance.generated_at,
        source: provenance.source,
        complete: provenance.complete,
        errors: provenance.errors.clone(),
        freshness: index.freshness(),
        scope: index.scope(),
        root: index.root_path().to_path_buf(),
        size: query.selection.size,
        sections,
    }
}

/// Aggregates gathered by one filtered traversal.
struct Walked {
    /// Filtered subtree aggregates, keyed by directory id.
    per_directory: BTreeMap<EntryId, SummaryRow>,
    /// Filtered per-extension tallies.
    by_ext: BTreeMap<String, ExtTally>,
    /// Entries the selection admitted.
    rows: Vec<FileRow>,
}

/// Walk the retained index once, aggregating only what the selection admits.
///
/// Iterative rather than recursive: this engine is built for trees deep enough that a
/// recursive post-order would exhaust the stack.
fn walk(index: &Index, selection: &Selection) -> Walked {
    let mut walked =
        Walked { per_directory: BTreeMap::new(), by_ext: BTreeMap::new(), rows: Vec::new() };

    // (id, path, whether its children have already been pushed)
    let mut stack: Vec<(EntryId, PathBuf, bool)> = vec![(EntryId::ROOT, PathBuf::new(), false)];
    while let Some((id, path, expanded)) = stack.pop() {
        if expanded {
            // Post-order: every child has finished, so fold their totals into this one.
            let mut total = walked.per_directory.remove(&id).unwrap_or_default();
            if let Some(children) = index.children_of(id) {
                for (name, child) in children {
                    let child_path = path.join(name);
                    if let Some(sub) = walked.per_directory.get(&child) {
                        let sub = *sub;
                        merge_summary(&mut total, &sub);
                        if index.kind_of(child) == Some(EntryKind::Dir) {
                            total.dirs += 1;
                        }
                    }
                    let _ = child_path;
                }
            }
            walked.per_directory.insert(id, total);
            continue;
        }

        stack.push((id, path.clone(), true));
        let Some(children) = index.children_of(id) else {
            continue;
        };
        let children: Vec<(PathBuf, EntryId)> =
            children.map(|(name, child)| (path.join(name), child)).collect();

        for (child_path, child) in children {
            let (Some(kind), Some(attrs)) = (index.kind_of(child), index.attrs_of(child)) else {
                continue;
            };
            let name = child_path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            let candidate = Candidate {
                relative: &child_path,
                name: &name,
                kind,
                bytes: attrs.size,
                allocated: attrs.allocated,
                mtime_ns: attrs.mtime_ns,
            };

            if selection.admits(&candidate) {
                walked.rows.push(FileRow {
                    path: child_path.clone(),
                    kind,
                    bytes: attrs.size,
                    allocated: attrs.allocated,
                    mtime_ns: attrs.mtime_ns,
                });

                if kind == EntryKind::File {
                    let own = walked.per_directory.entry(id).or_default();
                    own.files += 1;
                    own.bytes += attrs.size;
                    own.allocated += attrs.allocated;
                    own.newest_mtime_ns = Some(
                        own.newest_mtime_ns.map_or(attrs.mtime_ns, |seen| seen.max(attrs.mtime_ns)),
                    );

                    if let Some(ext) = derive_ext(&name) {
                        let tally = walked.by_ext.entry(ext).or_default();
                        tally.files += 1;
                        tally.bytes += attrs.size;
                        tally.allocated += attrs.allocated;
                    }
                }
            }

            if kind == EntryKind::Dir {
                stack.push((child, child_path, false));
            }
        }
    }

    walked
}

/// Fold one subtree's filtered totals into another's.
fn merge_summary(into: &mut SummaryRow, from: &SummaryRow) {
    into.files += from.files;
    into.dirs += from.dirs;
    into.bytes += from.bytes;
    into.allocated += from.allocated;
    into.newest_mtime_ns = match (into.newest_mtime_ns, from.newest_mtime_ns) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (left, right) => left.or(right),
    };
}

/// Build one view's section, using the pre-computed tier when the selection allows.
fn build_section(view: ViewSpec, index: &Index, query: &Query, walked: Option<&Walked>) -> Section {
    match view {
        ViewSpec::Summary => Section::Summary(match walked {
            None => summary_from_rollup(index.total()),
            Some(walked) => walked.per_directory.get(&EntryId::ROOT).copied().unwrap_or_default(),
        }),
        ViewSpec::Types => Section::Types(types_rows(index, query, walked)),
        ViewSpec::Files => Section::Files(file_rows(index, query, walked)),
        ViewSpec::Tree => Section::Tree(tree_node(index, query, walked)),
    }
}

/// A summary row taken straight from pre-computed roll-up state.
fn summary_from_rollup(rollup: &RollUp) -> SummaryRow {
    SummaryRow {
        files: rollup.files,
        dirs: rollup.dirs,
        bytes: rollup.bytes,
        allocated: rollup.allocated,
        newest_mtime_ns: (rollup.files > 0).then_some(rollup.newest_mtime_ns),
    }
}

/// Rows for the types view.
fn types_rows(index: &Index, query: &Query, walked: Option<&Walked>) -> Vec<TypeRow> {
    let tallies: BTreeMap<String, ExtTally> = match walked {
        None => index.by_ext_named(index.total()),
        Some(walked) => walked.by_ext.clone(),
    };

    let mut rows: Vec<TypeRow> = tallies
        .into_iter()
        .map(|(extension, tally)| TypeRow {
            extension,
            files: tally.files,
            bytes: tally.bytes,
            allocated: tally.allocated,
        })
        .collect();

    sort_rows(
        &mut rows,
        query,
        ViewSpec::Types,
        |row, metric| match metric {
            SizeMetric::Apparent => row.bytes,
            SizeMetric::Allocated => row.allocated,
        },
        |row| row.files,
        |_| None,
        |row| row.extension.clone(),
    );
    truncate(&mut rows, query.selection.limit);
    rows
}

/// Rows for the files view.
fn file_rows(index: &Index, query: &Query, walked: Option<&Walked>) -> Vec<FileRow> {
    let mut rows = match walked {
        Some(walked) => walked.rows.clone(),
        None => every_entry(index),
    };

    sort_rows(
        &mut rows,
        query,
        ViewSpec::Files,
        |row, metric| match metric {
            SizeMetric::Apparent => row.bytes,
            SizeMetric::Allocated => row.allocated,
        },
        |_| 1,
        |row| Some(row.mtime_ns),
        |row| row.path.to_string_lossy().into_owned(),
    );
    truncate(&mut rows, query.selection.limit);
    rows
}

/// Every entry in the index, for an unfiltered files view.
fn every_entry(index: &Index) -> Vec<FileRow> {
    let mut rows = Vec::new();
    let mut stack: Vec<(EntryId, PathBuf)> = vec![(EntryId::ROOT, PathBuf::new())];
    while let Some((id, path)) = stack.pop() {
        let Some(children) = index.children_of(id) else {
            continue;
        };
        let children: Vec<(PathBuf, EntryId)> =
            children.map(|(name, child)| (path.join(name), child)).collect();
        for (child_path, child) in children {
            let (Some(kind), Some(attrs)) = (index.kind_of(child), index.attrs_of(child)) else {
                continue;
            };
            rows.push(FileRow {
                path: child_path.clone(),
                kind,
                bytes: attrs.size,
                allocated: attrs.allocated,
                mtime_ns: attrs.mtime_ns,
            });
            if kind == EntryKind::Dir {
                stack.push((child, child_path));
            }
        }
    }
    rows
}

/// The tree view's root node, expanded to the requested depth.
fn tree_node(index: &Index, query: &Query, walked: Option<&Walked>) -> TreeNode {
    let root_summary = match walked {
        None => summary_from_rollup(index.total()),
        Some(walked) => walked.per_directory.get(&EntryId::ROOT).copied().unwrap_or_default(),
    };

    let mut root = TreeNode {
        path: PathBuf::new(),
        name: ".".to_string(),
        kind: EntryKind::Dir,
        bytes: root_summary.bytes,
        allocated: root_summary.allocated,
        files: root_summary.files,
        dirs: root_summary.dirs,
        newest_mtime_ns: root_summary.newest_mtime_ns,
        children: Vec::new(),
        truncated: false,
    };
    expand(index, query, walked, EntryId::ROOT, &PathBuf::new(), &mut root, 0);
    root
}

/// Attach a node's children, honoring the depth and per-directory limit bounds.
///
/// Iterative rather than recursive: this engine indexes trees deep enough that recursive
/// expansion would exhaust the stack, and a report that panics on a deep tree fails
/// exactly where the tool is most useful. Nodes are built flat with parent links in
/// pre-order, then folded together from the leaves up.
fn expand(
    index: &Index,
    query: &Query,
    walked: Option<&Walked>,
    root_id: EntryId,
    root_path: &Path,
    node: &mut TreeNode,
    start_depth: usize,
) {
    /// One node awaiting its children.
    struct Pending {
        node: TreeNode,
        id: EntryId,
        depth: usize,
        parent: Option<usize>,
    }

    let mut built = vec![Pending {
        // Only identity and bounds matter while expanding; the caller keeps the
        // populated root and receives its children back at the end.
        node: TreeNode {
            path: root_path.to_path_buf(),
            name: node.name.clone(),
            kind: node.kind,
            bytes: node.bytes,
            allocated: node.allocated,
            files: node.files,
            dirs: node.dirs,
            newest_mtime_ns: node.newest_mtime_ns,
            children: Vec::new(),
            truncated: false,
        },
        id: root_id,
        depth: start_depth,
        parent: None,
    }];

    let mut cursor = 0;
    while cursor < built.len() {
        let (id, depth) = (built[cursor].id, built[cursor].depth);
        let path = built[cursor].node.path.clone();

        if !query.selection.depth.admits(depth) {
            // `--depth 0` keeps du's meaning: totals for this node, nothing beneath it.
            // Files are already represented in this directory's totals and never become
            // tree rows. Only a directory child hidden by the depth bound makes the
            // rendered hierarchy incomplete.
            built[cursor].node.truncated = index.children_of(id).is_some_and(|mut children| {
                children.any(|(_, child)| index.kind_of(child) == Some(EntryKind::Dir))
            });
            cursor += 1;
            continue;
        }

        let mut rows = child_rows(index, query, walked, id, &path);
        let kept = query.selection.limit.limit().unwrap_or(rows.len()).min(rows.len());
        built[cursor].node.truncated = kept < rows.len();
        rows.truncate(kept);

        for (child_node, child_id) in rows {
            built.push(Pending {
                node: child_node,
                id: child_id,
                depth: depth + 1,
                parent: Some(cursor),
            });
        }
        cursor += 1;
    }

    // Fold from the end: every parent index is smaller than its child's, so removing the
    // last element never disturbs an index still to be used.
    for position in (1..built.len()).rev() {
        let child = built.remove(position);
        let parent = child.parent.expect("only the root has no parent");
        built[parent].node.children.insert(0, child.node);
    }

    let mut root = built.pop().expect("the root is always present");
    node.children = std::mem::take(&mut root.node.children);
    node.truncated = root.node.truncated;
}

/// The directory children of one node, shaped and sorted but not yet expanded.
fn child_rows(
    index: &Index,
    query: &Query,
    walked: Option<&Walked>,
    id: EntryId,
    path: &Path,
) -> Vec<(TreeNode, EntryId)> {
    let Some(children) = index.children_of(id) else {
        return Vec::new();
    };
    let children: Vec<(PathBuf, EntryId)> =
        children.map(|(name, child)| (path.join(name), child)).collect();

    let mut rows: Vec<(TreeNode, EntryId)> = Vec::new();
    for (child_path, child) in children {
        let Some(kind) = index.kind_of(child) else {
            continue;
        };
        // The tree view is a directory hierarchy: a file contributes its bytes to the
        // directory holding it rather than appearing as its own row.
        if kind != EntryKind::Dir {
            continue;
        }
        let summary = match walked {
            None => index.rollup_of(child).map(summary_from_rollup).unwrap_or_default(),
            Some(walked) => walked.per_directory.get(&child).copied().unwrap_or_default(),
        };
        let name = child_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        rows.push((
            TreeNode {
                path: child_path,
                name,
                kind,
                bytes: summary.bytes,
                allocated: summary.allocated,
                files: summary.files,
                dirs: summary.dirs,
                newest_mtime_ns: summary.newest_mtime_ns,
                children: Vec::new(),
                truncated: false,
            },
            child,
        ));
    }

    sort_rows_by(
        &mut rows,
        query,
        ViewSpec::Tree,
        |(row, _), metric| match metric {
            SizeMetric::Apparent => row.bytes,
            SizeMetric::Allocated => row.allocated,
        },
        |(row, _)| row.files,
        |(row, _)| row.newest_mtime_ns,
        |(row, _)| row.name.clone(),
    );
    rows
}

/// Trim a row list to the configured limit.
fn truncate<T>(rows: &mut Vec<T>, limit: Bound) {
    if let Some(limit) = limit.limit() {
        rows.truncate(limit);
    }
}

/// Sort rows by the effective key for a view.
fn sort_rows<T>(
    rows: &mut [T],
    query: &Query,
    view: ViewSpec,
    size: impl Fn(&T, SizeMetric) -> u64,
    count: impl Fn(&T) -> u64,
    mtime: impl Fn(&T) -> Option<i64>,
    name: impl Fn(&T) -> String,
) {
    sort_rows_by(rows, query, view, size, count, mtime, name);
}

/// Sort rows by the effective key, with a stable name tiebreak.
fn sort_rows_by<T>(
    rows: &mut [T],
    query: &Query,
    view: ViewSpec,
    size: impl Fn(&T, SizeMetric) -> u64,
    count: impl Fn(&T) -> u64,
    mtime: impl Fn(&T) -> Option<i64>,
    name: impl Fn(&T) -> String,
) {
    let key = query.selection.sort.unwrap_or_else(|| view.default_sort());
    let metric = query.selection.size;

    rows.sort_by(|left, right| {
        let ordering = match key {
            // Size, count, and recency read most-first: the interesting end is the top.
            SortKey::Size => size(right, metric).cmp(&size(left, metric)),
            SortKey::Count => count(right).cmp(&count(left)),
            SortKey::Mtime => mtime(right).cmp(&mtime(left)),
            SortKey::Name => name(left).cmp(&name(right)),
        };
        // A name tiebreak keeps equal rows in a deterministic order, which is what makes
        // the goldens stable across runs and platforms.
        ordering.then_with(|| name(left).cmp(&name(right)))
    });

    if query.selection.reverse {
        rows.reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::glob::Pattern;
    use crate::query::selection::ModifiedWindow;
    use crate::types::{Attrs, Observation, Op};
    use std::time::{Duration, UNIX_EPOCH};

    fn attrs(size: u64, mtime_ns: i64) -> Attrs {
        Attrs {
            size,
            allocated: size.div_ceil(512) * 512,
            mtime_ns,
            ctime_ns: mtime_ns,
            inode: size.wrapping_mul(31).wrapping_add(mtime_ns.unsigned_abs()),
            dev: 1,
        }
    }

    fn upsert(path: &str, kind: EntryKind, attrs: Attrs) -> Op {
        Op::Upsert { path: PathBuf::from(path), kind, attrs }
    }

    /// A tree with two top-level directories, a nested level, and three extensions.
    fn sample() -> Index {
        let mut index = Index::new("/root");
        index
            .apply(&Observation::new(vec![
                upsert("src", EntryKind::Dir, Attrs::default()),
                upsert("src/main.rs", EntryKind::File, attrs(100, 10)),
                upsert("src/lib.rs", EntryKind::File, attrs(200, 20)),
                upsert("src/deep", EntryKind::Dir, Attrs::default()),
                upsert("src/deep/nested.rs", EntryKind::File, attrs(50, 40)),
                upsert("docs", EntryKind::Dir, Attrs::default()),
                upsert("docs/guide.md", EntryKind::File, attrs(300, 30)),
                upsert("notes.txt", EntryKind::File, attrs(7, 5)),
            ]))
            .expect("apply");
        index
    }

    fn provenance() -> Provenance {
        Provenance {
            scan_started_at: Some(UNIX_EPOCH + Duration::from_secs(1_000)),
            generated_at: UNIX_EPOCH + Duration::from_secs(1_001),
            source: ReportSource::ColdScan,
            complete: true,
            errors: Vec::new(),
        }
    }

    fn run(index: &Index, request: &Query) -> Report {
        report(index, request, &provenance())
    }

    fn query(views: &[ViewSpec], selection: Selection) -> Query {
        Query { selection, views: views.to_vec() }
    }

    fn pattern(source: &str) -> Pattern {
        Pattern::parse(source).expect("pattern compiles")
    }

    fn summary_of(report: &Report) -> SummaryRow {
        match report.sections.first().expect("a section") {
            Section::Summary(row) => *row,
            other => panic!("expected a summary, got {other:?}"),
        }
    }

    fn files_of(report: &Report) -> Vec<FileRow> {
        match report.sections.first().expect("a section") {
            Section::Files(rows) => rows.clone(),
            other => panic!("expected files, got {other:?}"),
        }
    }

    fn types_of(report: &Report) -> Vec<TypeRow> {
        match report.sections.first().expect("a section") {
            Section::Types(rows) => rows.clone(),
            other => panic!("expected types, got {other:?}"),
        }
    }

    fn tree_of(report: &Report) -> TreeNode {
        match report.sections.first().expect("a section") {
            Section::Tree(node) => node.clone(),
            other => panic!("expected a tree, got {other:?}"),
        }
    }

    #[test]
    fn an_unfiltered_summary_matches_the_precomputed_rollup() {
        let index = sample();
        let row = summary_of(&run(&index, &query(&[ViewSpec::Summary], Selection::default())));
        assert_eq!(row.files, 5);
        assert_eq!(row.dirs, 3);
        assert_eq!(row.bytes, 657);
        assert_eq!(row.newest_mtime_ns, Some(40));
    }

    #[test]
    fn the_two_tiers_agree_on_the_same_question() {
        // The load-bearing property: reading pre-computed roll-ups and re-aggregating a
        // filtered walk must answer identically when the filter admits everything.
        let index = sample();
        let fast = summary_of(&run(&index, &query(&[ViewSpec::Summary], Selection::default())));

        // A filter that excludes nothing still forces the traversal tier.
        let admits_everything = Selection { min_size: Some(0), ..Selection::default() };
        assert!(!admits_everything.is_unfiltered());
        let slow = summary_of(&run(&index, &query(&[ViewSpec::Summary], admits_everything)));

        assert_eq!(
            (fast.files, fast.bytes, fast.allocated, fast.newest_mtime_ns),
            (slow.files, slow.bytes, slow.allocated, slow.newest_mtime_ns)
        );
    }

    #[test]
    fn selection_narrows_a_summary_to_what_it_admits() {
        let index = sample();
        let selection = Selection { include: vec![pattern("*.rs")], ..Selection::default() };
        let row = summary_of(&run(&index, &query(&[ViewSpec::Summary], selection)));
        assert_eq!(row.files, 3, "three .rs files");
        assert_eq!(row.bytes, 350);
    }

    #[test]
    fn a_files_view_lists_matching_entries_in_name_order_by_default() {
        let index = sample();
        let selection = Selection { include: vec![pattern("*.rs")], ..Selection::default() };
        let rows = files_of(&run(&index, &query(&[ViewSpec::Files], selection)));
        // Built from components so the expectation carries the native separator: a
        // literal "src/main.rs" passes on Unix and fails on Windows for a reason that
        // has nothing to do with the view under test.
        let paths: Vec<PathBuf> = rows.iter().map(|row| row.path.clone()).collect();
        let expected: Vec<PathBuf> = [["src", "deep", "nested.rs"].iter().collect::<PathBuf>()]
            .into_iter()
            .chain([["src", "lib.rs"].iter().collect::<PathBuf>()])
            .chain([["src", "main.rs"].iter().collect::<PathBuf>()])
            .collect();
        assert_eq!(paths, expected);
    }

    #[test]
    fn sorting_and_limiting_compose_without_a_dedicated_view() {
        // "Largest files" is not a view; it is files plus sort plus limit.
        let index = sample();
        let selection = Selection {
            kinds: vec![EntryKind::File],
            sort: Some(SortKey::Size),
            limit: Bound::Limit(2),
            ..Selection::default()
        };
        let rows = files_of(&run(&index, &query(&[ViewSpec::Files], selection)));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].bytes, 300, "largest first");
        assert_eq!(rows[1].bytes, 200);
    }

    #[test]
    fn reverse_flips_whatever_order_is_in_effect() {
        let index = sample();
        let selection = Selection {
            kinds: vec![EntryKind::File],
            sort: Some(SortKey::Size),
            reverse: true,
            ..Selection::default()
        };
        let rows = files_of(&run(&index, &query(&[ViewSpec::Files], selection)));
        assert_eq!(rows[0].bytes, 7, "smallest first once reversed");
    }

    #[test]
    fn a_modified_window_selects_by_time() {
        let index = sample();
        let selection = Selection {
            kinds: vec![EntryKind::File],
            modified: ModifiedWindow { since: Some(20), before: Some(40) },
            sort: Some(SortKey::Mtime),
            ..Selection::default()
        };
        let rows = files_of(&run(&index, &query(&[ViewSpec::Files], selection)));
        let mut times: Vec<i64> = rows.iter().map(|row| row.mtime_ns).collect();
        times.sort_unstable();
        assert_eq!(times, vec![20, 30], "inclusive start, exclusive end");
    }

    #[test]
    fn a_types_view_reports_both_size_metrics_per_extension() {
        let index = sample();
        let rows = types_of(&run(&index, &query(&[ViewSpec::Types], Selection::default())));
        let rs = rows.iter().find(|row| row.extension == ".rs").expect(".rs present");
        assert_eq!((rs.files, rs.bytes), (3, 350));
        assert_eq!(rs.allocated, 1536, "three files, one 512-byte block each");
        // Size-ranked by default: .rs (350) then .md (300) then .txt (7).
        let order: Vec<&str> = rows.iter().map(|row| row.extension.as_str()).collect();
        assert_eq!(order, vec![".rs", ".md", ".txt"]);
    }

    #[test]
    fn a_tree_view_reports_directories_with_their_subtree_totals() {
        let index = sample();
        let tree = tree_of(&run(&index, &query(&[ViewSpec::Tree], Selection::default())));
        assert_eq!(tree.name, ".");
        assert_eq!(tree.bytes, 657);
        // Size-ranked children: src (350) before docs (300).
        let names: Vec<&str> = tree.children.iter().map(|child| child.name.as_str()).collect();
        assert_eq!(names, vec!["src", "docs"]);
        let src = &tree.children[0];
        assert_eq!(src.bytes, 350);
        let nested: Vec<&str> = src.children.iter().map(|child| child.name.as_str()).collect();
        assert_eq!(nested, vec!["deep"]);
    }

    #[test]
    fn depth_zero_keeps_dus_meaning_of_root_totals_only() {
        let index = sample();
        let selection = Selection { depth: Bound::Limit(0), ..Selection::default() };
        let tree = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert_eq!(tree.bytes, 657, "totals still cover the whole tree");
        assert!(tree.children.is_empty(), "but nothing below the root is listed");
        assert!(tree.truncated, "and the report says so rather than implying emptiness");
    }

    #[test]
    fn a_depth_bound_marks_only_hidden_directory_rows_as_truncated() {
        let index = sample();
        let selection = Selection { depth: Bound::Limit(1), ..Selection::default() };
        let tree = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));

        let src = tree.children.iter().find(|child| child.name == "src").expect("src");
        assert!(src.truncated, "src with a hidden directory child is truncated");

        let docs = tree.children.iter().find(|child| child.name == "docs").expect("docs");
        assert!(docs.children.is_empty());
        assert!(
            !docs.truncated,
            "file children contribute to a directory row; they are not hidden tree rows"
        );
    }

    #[test]
    fn a_tree_limit_bounds_entries_per_directory_and_marks_truncation() {
        let index = sample();
        let selection = Selection { limit: Bound::Limit(1), ..Selection::default() };
        let tree = tree_of(&run(&index, &query(&[ViewSpec::Tree], selection)));
        assert_eq!(tree.children.len(), 1);
        assert!(tree.truncated);
    }

    #[test]
    fn requesting_more_views_never_changes_another_views_answer() {
        // The property that makes `--view types,tree` one scan and one consistent state.
        let index = sample();
        let alone = types_of(&run(&index, &query(&[ViewSpec::Types], Selection::default())));
        let together = run(
            &index,
            &query(&[ViewSpec::Types, ViewSpec::Tree, ViewSpec::Summary], Selection::default()),
        );
        let with_others = match &together.sections[0] {
            Section::Types(rows) => rows.clone(),
            other => panic!("expected types first, got {other:?}"),
        };

        assert_eq!(alone.len(), with_others.len());
        for (left, right) in alone.iter().zip(with_others.iter()) {
            assert_eq!(
                (&left.extension, left.files, left.bytes),
                (&right.extension, right.files, right.bytes)
            );
        }
        assert_eq!(together.sections.len(), 3, "one section per view, in request order");
        assert_eq!(together.sections[1].view(), ViewSpec::Tree);
        assert_eq!(together.sections[2].view(), ViewSpec::Summary);
    }

    #[test]
    fn a_report_carries_the_provenance_it_was_given() {
        let index = sample();
        let report = run(&index, &query(&[ViewSpec::Summary], Selection::default()));
        assert_eq!(report.source, ReportSource::ColdScan);
        assert!(report.complete);
        assert_eq!(report.scan_started_at, Some(UNIX_EPOCH + Duration::from_secs(1_000)));
        assert_eq!(report.generated_at, UNIX_EPOCH + Duration::from_secs(1_001));
        assert_eq!(report.root, Path::new("/root"));
    }

    #[test]
    fn reporting_is_pure_and_repeatable() {
        let index = sample();
        let request = query(&[ViewSpec::Tree, ViewSpec::Types], Selection::default());
        assert_eq!(format!("{:?}", run(&index, &request)), format!("{:?}", run(&index, &request)));
    }
}
