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

/// Additive selection for portable opened-root entry projections.
///
/// The established [`Selection`] remains the shared one-shot query contract. This value
/// composes it instead of adding fields to that public struct, preserving source
/// compatibility for existing Rust callers while keeping interactive row predicates in
/// one pure engine-owned value.
#[derive(Clone, Debug, Default)]
pub struct EntrySelection {
    /// Existing fdu query predicates.
    pub query: Selection,
    /// Largest size, inclusive, in the selected metric.
    ///
    /// A caller with an exclusive upper bound translates `less_than: n` to `n - 1`.
    pub max_size: Option<u64>,
    /// Exclude entries in the fixed ignored partition.
    pub exclude_ignored: bool,
    /// Logical extensions to admit, including the leading dot.
    ///
    /// This is name identity, not the registry's canonical classification bucket.
    pub logical_extensions: Vec<String>,
    /// Exact basenames to admit, compared case-insensitively.
    ///
    /// When either this or `logical_extensions` is nonempty, matching either admits the
    /// name. That represents one identity filter rather than two intersected filters.
    pub exact_names: Vec<String>,
    /// Lowercase terminal suffixes to admit, including the leading dot.
    ///
    /// Unlike a logical extension, only the final dotted component participates.
    pub terminal_extensions: Vec<String>,
    /// Exact ancestor path-component names to admit.
    pub ancestor_names: Vec<String>,
}

impl From<Selection> for EntrySelection {
    fn from(query: Selection) -> Self {
        Self { query, ..Self::default() }
    }
}

impl Selection {
    /// Heap payload retained when an opened-root continuation owns this selection.
    pub(crate) fn retained_heap_bytes(&self) -> usize {
        let pattern_bytes =
            self.include.iter().chain(&self.exclude).fold(0_usize, |total, pattern| {
                total.saturating_add(pattern.retained_heap_bytes())
            });
        self.include
            .capacity()
            .saturating_add(self.exclude.capacity())
            .saturating_mul(std::mem::size_of::<Pattern>())
            .saturating_add(pattern_bytes)
            .saturating_add(self.kinds.capacity().saturating_mul(std::mem::size_of::<EntryKind>()))
    }

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

impl EntrySelection {
    /// Heap payload retained when an opened-root continuation owns this selection.
    pub(crate) fn retained_heap_bytes(&self) -> usize {
        self.query
            .retained_heap_bytes()
            .saturating_add(retained_strings(
                &self.logical_extensions,
                self.logical_extensions.capacity(),
            ))
            .saturating_add(retained_strings(&self.exact_names, self.exact_names.capacity()))
            .saturating_add(retained_strings(
                &self.terminal_extensions,
                self.terminal_extensions.capacity(),
            ))
            .saturating_add(retained_strings(&self.ancestor_names, self.ancestor_names.capacity()))
    }

    /// Whether this portable entry selection constrains any row.
    pub fn is_unfiltered(&self) -> bool {
        self.query.is_unfiltered()
            && self.max_size.is_none()
            && !self.exclude_ignored
            && self.logical_extensions.is_empty()
            && self.exact_names.is_empty()
            && self.terminal_extensions.is_empty()
            && self.ancestor_names.is_empty()
    }

    /// Whether an entry passes the base query and every opened-row predicate.
    pub fn admits(&self, candidate: &Candidate<'_>, ignored: bool) -> bool {
        if !self.query.admits(candidate) {
            return false;
        }
        if let Some(max_size) = self.max_size {
            if self.query.size_of(candidate) > max_size {
                return false;
            }
        }
        if self.exclude_ignored && ignored {
            return false;
        }
        if !self.logical_extensions.is_empty() || !self.exact_names.is_empty() {
            if candidate.kind != EntryKind::File {
                return false;
            }
            let extension_matches = crate::classify::logical_ext(candidate.name.as_ref())
                .is_some_and(|extension| {
                    self.logical_extensions
                        .iter()
                        .any(|expected| extension.eq_ignore_ascii_case(expected))
                });
            let name_matches = self
                .exact_names
                .iter()
                .any(|expected| candidate.name.eq_ignore_ascii_case(expected));
            if !extension_matches && !name_matches {
                return false;
            }
        }
        if !self.terminal_extensions.is_empty() {
            if candidate.kind != EntryKind::File {
                return false;
            }
            let Some(suffix) = terminal_suffix(candidate.name) else {
                return false;
            };
            if !self
                .terminal_extensions
                .iter()
                .any(|expected| suffix.eq_ignore_ascii_case(expected))
            {
                return false;
            }
        }
        if !self.ancestor_names.is_empty()
            && !candidate.relative.parent().is_some_and(|parent| {
                parent.components().any(|component| {
                    let std::path::Component::Normal(name) = component else {
                        return false;
                    };
                    self.ancestor_names.iter().any(|expected| name == expected.as_str())
                })
            })
        {
            return false;
        }
        true
    }
}

fn retained_strings(values: &[String], capacity: usize) -> usize {
    capacity.saturating_mul(std::mem::size_of::<String>()).saturating_add(
        values.iter().fold(0_usize, |total, value| total.saturating_add(value.capacity())),
    )
}

fn terminal_suffix(name: &str) -> Option<&str> {
    let dot = name.rfind('.')?;
    (dot > 0 && dot + 1 < name.len()).then_some(&name[dot..])
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

    fn entry_admits(
        selection: &EntrySelection,
        path: &str,
        kind: EntryKind,
        bytes: u64,
        mtime: i64,
        ignored: bool,
    ) -> bool {
        let (relative, name) = candidate(path, kind, bytes, mtime);
        selection.admits(
            &Candidate {
                relative: &relative,
                name: &name,
                kind,
                bytes,
                allocated: bytes.div_ceil(512) * 512,
                mtime_ns: mtime,
            },
            ignored,
        )
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
    fn portable_catalog_predicates_compose_without_client_side_filtering() {
        let selection = EntrySelection {
            max_size: Some(10),
            exclude_ignored: true,
            terminal_extensions: vec![".rs".to_string(), ".md".to_string()],
            ancestor_names: vec!["src".to_string(), "docs".to_string()],
            ..EntrySelection::default()
        };
        assert!(entry_admits(&selection, "src/lib.rs", EntryKind::File, 10, 0, false));
        assert!(entry_admits(&selection, "docs/readme.md", EntryKind::File, 9, 0, false));
        assert!(!entry_admits(&selection, "src/lib.RS", EntryKind::File, 11, 0, false));
        assert!(!entry_admits(&selection, "tests/lib.rs", EntryKind::File, 9, 0, false));
        assert!(!entry_admits(&selection, "src/lib.rs", EntryKind::File, 9, 0, true));
        assert!(!entry_admits(&selection, "src/.gitignore", EntryKind::File, 1, 0, false));
    }

    #[test]
    fn logical_extensions_and_exact_names_form_one_name_identity_filter() {
        let selection = EntrySelection {
            logical_extensions: vec![".v2.zip".to_string()],
            exact_names: vec!["makefile".to_string()],
            ..EntrySelection::default()
        };
        assert!(entry_admits(&selection, "release.v2.zip", EntryKind::File, 1, 0, false));
        assert!(entry_admits(&selection, "Makefile", EntryKind::File, 1, 0, false));
        assert!(!entry_admits(&selection, "plain.zip", EntryKind::File, 1, 0, false));
        assert!(!entry_admits(&selection, "README", EntryKind::File, 1, 0, false));
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
