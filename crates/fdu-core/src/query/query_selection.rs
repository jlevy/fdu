//! Which retained entries a query considers, and how its results are shaped.
//!
//! Selection is evaluated at view time against the index that a scan already built, never
//! during the walk. That split is what makes filters cheap and the cache reusable: scope
//! decides what is observed and cached, so one snapshot answers every selection, and
//! changing `--include` never invalidates anything. It is the same reasoning as tagging
//! ignored entries rather than pruning them.

use std::path::Path;

use crate::engine_contract::EntryKind;
use crate::query::query_glob::Pattern;

/// Which size metric a report answers in.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SizeMetric {
    /// Bytes the file's contents occupy logically.
    #[default]
    Apparent,
    /// Bytes the filesystem allocated, which sparse files and clones make differ.
    Allocated,
}

/// Which key results are ordered by.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SortKey {
    /// Bytes, in the selected metric.
    #[default]
    Size,
    /// Entry counts.
    Count,
    /// Newest modification time.
    Mtime,
    /// Path or name, lexicographically.
    Name,
}

/// An inclusive-start, exclusive-end window over modification times, in nanoseconds.
///
/// Half-open on purpose: `[since, before)` composes without double-counting when a caller
/// walks a tree in windows, and the inclusive start is the safe side for sync — a file
/// whose mtime equals the watermark re-lists, because duplicates are cheap and omissions
/// are not.
#[derive(Clone, Copy, Debug, Default)]
pub struct ModifiedWindow {
    /// Inclusive lower bound.
    pub since: Option<i64>,
    /// Exclusive upper bound.
    pub before: Option<i64>,
}

impl ModifiedWindow {
    /// Whether a modification time falls inside the window.
    pub fn contains(&self, mtime_ns: i64) -> bool {
        self.since.is_none_or(|since| mtime_ns >= since)
            && self.before.is_none_or(|before| mtime_ns < before)
    }

    /// Whether the window constrains anything at all.
    pub fn is_unbounded(&self) -> bool {
        self.since.is_none() && self.before.is_none()
    }
}

/// A bound that may be unlimited.
///
/// `--depth all` and `-n all` are spelled the same way as their numeric forms rather than
/// as a separate flag, so "how deep" and "how many" stay single questions.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Bound {
    /// No limit.
    #[default]
    All,
    /// At most this many.
    Limit(usize),
}

impl Bound {
    /// Whether a zero-based index is within the bound.
    pub fn admits(self, index: usize) -> bool {
        match self {
            Self::All => true,
            Self::Limit(limit) => index < limit,
        }
    }

    /// The bound as a count, when it has one.
    pub fn limit(self) -> Option<usize> {
        match self {
            Self::All => None,
            Self::Limit(limit) => Some(limit),
        }
    }
}

/// Which retained entries a query considers, and how its results are shaped.
#[derive(Clone, Debug, Default)]
pub struct Selection {
    /// Patterns an entry must match at least one of, when non-empty.
    pub include: Vec<Pattern>,
    /// Patterns that exclude an entry outright; exclusion wins over inclusion.
    pub exclude: Vec<Pattern>,
    /// Smallest size, in the selected metric, an entry may have.
    pub min_size: Option<u64>,
    /// Entry kinds to consider; empty means every kind.
    pub kinds: Vec<EntryKind>,
    /// Modification-time window.
    pub modified: ModifiedWindow,
    /// How deep a rendered tree descends, or `None` to let each view apply its own.
    ///
    /// Optional for the same reason `limit` and `sort` are. The depth that suits a tree
    /// is not the depth that suits a flat enumeration, and while this was a plain
    /// `Bound` the only default the library could offer was "unbounded" -- so the CLI
    /// declared `default_value = "2"` itself and every other caller silently got a
    /// different report for the same request.
    pub depth: Option<Bound>,
    /// How many entries a view reports.
    /// Rows to keep, or `None` to let each view apply its own bound.
    ///
    /// Optional for the same reason `sort` is: a bound that suits a per-directory tree
    /// is not the bound that suits a complete enumeration, and a single shared default
    /// produced "the ten alphabetically-first entries" of a 192,871-entry tree.
    pub limit: Option<Bound>,
    /// Ordering key, or `None` to let each view apply its own default.
    ///
    /// Optional rather than defaulted here because the sensible default differs by view:
    /// a tree and a type breakdown rank by size, while a flat file listing reads in name
    /// order. One shared default would be wrong for one of them.
    pub sort: Option<SortKey>,
    /// Whether the ordering is reversed.
    pub reverse: bool,
    /// Which size metric the report answers in.
    pub size: SizeMetric,
}

/// The facts about one entry that selection examines.
///
/// Passing a small explicit record rather than an index handle keeps the predicate pure
/// and trivially testable, and keeps selection from reaching into index internals.
#[derive(Clone, Copy, Debug)]
pub struct Candidate<'a> {
    /// Path relative to the index root.
    pub relative: &'a Path,
    /// Final path component.
    pub name: &'a str,
    /// What the entry is.
    pub kind: EntryKind,
    /// Apparent size in bytes.
    pub bytes: u64,
    /// Allocated size in bytes.
    pub allocated: u64,
    /// Modification time in nanoseconds since the Unix epoch.
    pub mtime_ns: i64,
}

impl Selection {
    /// Whether this selection constrains which entries are considered.
    ///
    /// An unconstrained selection lets a view read pre-computed roll-up state directly
    /// instead of traversing entries, which is the difference between the two performance
    /// tiers a report can run in.
    pub fn is_unfiltered(&self) -> bool {
        self.include.is_empty()
            && self.exclude.is_empty()
            && self.min_size.is_none()
            && self.kinds.is_empty()
            && self.modified.is_unbounded()
    }

    /// Whether an entry passes every filter.
    pub fn admits(&self, candidate: &Candidate<'_>) -> bool {
        if !self.kinds.is_empty() && !self.kinds.contains(&candidate.kind) {
            return false;
        }
        if let Some(min_size) = self.min_size {
            if self.size_of(candidate) < min_size {
                return false;
            }
        }
        if !self.modified.contains(candidate.mtime_ns) {
            return false;
        }
        // Exclusion wins: a pattern the caller wrote to keep something out should not be
        // overridden by a broader pattern they wrote to let things in.
        if self.exclude.iter().any(|p| p.matches(candidate.relative, candidate.name)) {
            return false;
        }
        if self.include.is_empty() {
            return true;
        }
        self.include.iter().any(|p| p.matches(candidate.relative, candidate.name))
    }

    /// The size of an entry in the selected metric.
    pub fn size_of(&self, candidate: &Candidate<'_>) -> u64 {
        match self.size {
            SizeMetric::Apparent => candidate.bytes,
            SizeMetric::Allocated => candidate.allocated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn candidate(path: &str, kind: EntryKind, bytes: u64, mtime_ns: i64) -> (PathBuf, String) {
        let relative = PathBuf::from(path);
        let name = relative
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let _ = (kind, bytes, mtime_ns);
        (relative, name)
    }

    fn admits(selection: &Selection, path: &str, kind: EntryKind, bytes: u64, mtime: i64) -> bool {
        let (relative, name) = candidate(path, kind, bytes, mtime);
        selection.admits(&Candidate {
            relative: &relative,
            name: &name,
            kind,
            bytes,
            allocated: bytes.div_ceil(512) * 512,
            mtime_ns: mtime,
        })
    }

    fn pattern(source: &str) -> Pattern {
        Pattern::parse(source).expect("pattern compiles")
    }

    #[test]
    fn a_default_selection_admits_everything_and_reads_the_fast_tier() {
        let selection = Selection::default();
        assert!(selection.is_unfiltered());
        assert!(admits(&selection, "src/main.rs", EntryKind::File, 10, 5));
        assert!(admits(&selection, "src", EntryKind::Dir, 0, 5));
    }

    #[test]
    fn include_patterns_narrow_and_exclude_patterns_win() {
        let mut selection = Selection { include: vec![pattern("*.rs")], ..Selection::default() };
        assert!(!selection.is_unfiltered());
        assert!(admits(&selection, "src/main.rs", EntryKind::File, 10, 5));
        assert!(!admits(&selection, "src/main.toml", EntryKind::File, 10, 5));

        // An exclusion beats a matching inclusion, so a narrowing rule cannot be undone
        // by a broader one written elsewhere on the command line.
        selection.exclude.push(pattern("**/generated/**"));
        assert!(!admits(&selection, "src/generated/api.rs", EntryKind::File, 10, 5));
        assert!(admits(&selection, "src/hand/api.rs", EntryKind::File, 10, 5));
    }

    #[test]
    fn min_size_follows_the_selected_metric() {
        let apparent = Selection { min_size: Some(600), ..Selection::default() };
        // 100 apparent bytes occupy 512 allocated bytes: neither reaches 600.
        assert!(!admits(&apparent, "a.bin", EntryKind::File, 100, 0));

        let allocated =
            Selection { min_size: Some(600), size: SizeMetric::Allocated, ..Selection::default() };
        // 600 apparent bytes occupy 1024 allocated bytes, which does reach 600.
        assert!(admits(&allocated, "a.bin", EntryKind::File, 600, 0));
        assert!(!admits(&allocated, "b.bin", EntryKind::File, 100, 0));
    }

    #[test]
    fn the_modified_window_is_half_open() {
        let selection = Selection {
            modified: ModifiedWindow { since: Some(100), before: Some(200) },
            ..Selection::default()
        };
        // Inclusive start: a file at exactly the watermark re-lists, because for sync a
        // duplicate is safe and an omission is not.
        assert!(admits(&selection, "a", EntryKind::File, 1, 100));
        assert!(admits(&selection, "b", EntryKind::File, 1, 150));
        // Exclusive end.
        assert!(!admits(&selection, "c", EntryKind::File, 1, 200));
        assert!(!admits(&selection, "d", EntryKind::File, 1, 99));
    }

    #[test]
    fn kinds_filter_and_an_empty_list_means_every_kind() {
        let files = Selection { kinds: vec![EntryKind::File], ..Selection::default() };
        assert!(admits(&files, "a.rs", EntryKind::File, 1, 0));
        assert!(!admits(&files, "src", EntryKind::Dir, 0, 0));

        let both =
            Selection { kinds: vec![EntryKind::File, EntryKind::Dir], ..Selection::default() };
        assert!(admits(&both, "src", EntryKind::Dir, 0, 0));
        assert!(!admits(&both, "link", EntryKind::Symlink, 0, 0));
    }

    #[test]
    fn bounds_admit_by_index_and_report_their_limit() {
        assert!(Bound::All.admits(1_000_000));
        assert_eq!(Bound::All.limit(), None);
        assert!(Bound::Limit(2).admits(0));
        assert!(Bound::Limit(2).admits(1));
        assert!(!Bound::Limit(2).admits(2));
        assert_eq!(Bound::Limit(2).limit(), Some(2));
        // `--depth 0` keeps du's meaning: root totals only, nothing below.
        assert!(!Bound::Limit(0).admits(0));
    }

    #[test]
    fn an_unbounded_window_does_not_constrain() {
        assert!(ModifiedWindow::default().is_unbounded());
        assert!(ModifiedWindow::default().contains(i64::MIN));
        assert!(ModifiedWindow::default().contains(i64::MAX));
    }
}
