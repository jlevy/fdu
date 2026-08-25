//! Which retained entries a query considers, and how its results are shaped.
//!
//! Selection is evaluated at view time against the index that a scan already built, never
//! during the walk. That split is what makes filters cheap and the cache reusable: scope
//! decides what is observed and cached, so one snapshot answers every selection, and
//! changing `--include` never invalidates anything. It is the same reasoning as tagging
//! ignored entries rather than pruning them.

use std::ffi::OsString;
use std::path::{Component, Path};

use crate::engine_contract::{Bound, EntryKind};
use crate::query::query_glob::Pattern;
use crate::tags::TagBits;

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

/// Which tags an entry must and must not carry.
///
/// Two masks rather than a list of names because this is tested once per entry: the whole
/// predicate is two `and`s and two compares, and it stays that size however many rules the
/// engine grows. Masks are meaningful only alongside the [`TagRules`](crate::tags::TagRules)
/// that issued them, so every surface resolves names to bits once, where the request is
/// parsed, and a name that is not enabled is a rejected request rather than a filter that
/// silently matches nothing.
///
/// Deliberately not a set of `if tag == gitignore` branches anywhere downstream. A rule is
/// a bit; adding one adds a catalogue entry and nothing else.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TagFilter {
    /// Tags an entry must carry at least one of, when non-zero.
    ///
    /// Any-of rather than all-of, matching `include`: naming a second tag widens, and the
    /// way to narrow is to ask twice.
    pub any: TagBits,
    /// Tags that exclude an entry outright. Exclusion wins, as it does for patterns.
    pub none: TagBits,
}

impl TagFilter {
    /// Whether an entry's tags pass the filter.
    pub fn admits(self, tags: TagBits) -> bool {
        if tags & self.none != 0 {
            return false;
        }
        self.any == 0 || tags & self.any != 0
    }

    /// Whether the filter constrains anything at all.
    pub fn is_unconstrained(self) -> bool {
        self.any == 0 && self.none == 0
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
    /// Largest size to admit, inclusive, measured by the same metric as `min_size`.
    ///
    /// Inclusive, mirroring `min_size`, because that is what a person means by "at most
    /// 1M" on a command line. A consumer whose own contract has an *exclusive* upper
    /// bound translates at its boundary rather than pushing exclusivity into the engine's
    /// vocabulary: `size_less_than: n` becomes `max_size: n - 1`, with zero meaning an
    /// empty selection.
    ///
    /// Exists because filtering above the engine means moving every candidate across the
    /// boundary just to discard it, which is the cost a native provider is for.
    pub max_size: Option<u64>,
    /// Entry kinds to consider; empty means every kind.
    pub kinds: Vec<EntryKind>,
    /// Terminal suffixes to admit, lowercase and dotted, or empty for every suffix.
    ///
    /// A closed predicate rather than a glob, because it is not one. The rule is the *last*
    /// suffix, case-folded: `NOTES.TXT` and `notes.txt` both have terminal `.txt`, and
    /// `archive.tar.gz` has `.gz` rather than `.tar.gz`. A case-sensitive whole-name glob
    /// cannot express either half, and a case-insensitive glob dialect would be a second
    /// pattern language answering one question.
    ///
    /// A name with no suffix -- `Makefile`, or a leading-dot name like `.gitignore` -- has
    /// terminal `""`, which no entry in this list can equal, so a non-empty list excludes
    /// them. That is the same answer the consuming contract gives, and it is why the
    /// entries are required to carry their dot.
    pub terminal_extensions: Vec<String>,
    /// Directory names an entry must have among its ancestors, or empty for any.
    ///
    /// Exact whole components, case-sensitively, and *any* of them: `["src", "tests"]`
    /// admits anything under either. Ancestors only -- the entry's own name is never one --
    /// so a file called `src` does not match `src`.
    ///
    /// A glob cannot say this either. `**/src/**` comes close and differs on the cases that
    /// matter: it is case-sensitive but also substring-free only by accident of the
    /// separator, and it cannot express "any of these names at any depth" without one
    /// pattern per name, which changes `include`'s any-of semantics into something a caller
    /// has to reason about.
    pub ancestor_names: Vec<OsString>,
    /// Modification-time window.
    pub modified: ModifiedWindow,
    /// Tags an entry must and must not carry.
    ///
    /// A view-time filter like every other field here, which is what keeps enabling a tag
    /// rule from invalidating a cache the way changing scope does: the bits are recorded
    /// once at insert, and asking a different question of them re-reads, never re-walks
    /// the filesystem.
    pub tags: TagFilter,
    /// A maintained plane to answer in, or `None` for the whole subtree.
    ///
    /// Deliberately not part of [`is_unfiltered`](Self::is_unfiltered), which is the one
    /// thing about this field worth stating twice. Every other field here narrows *which
    /// entries are considered*, and narrowing means the precomputed roll-ups no longer
    /// answer, so the report re-aggregates by walking the whole index. A plane is the
    /// opposite: the engine has already maintained this exact restriction on the
    /// ancestor-merge path, so a plane-only request is still a roll-up read. Treating it
    /// as an ordinary filter would put every plane query on the walking tier -- the
    /// hundred-millisecond route this whole design exists to avoid -- and it would be
    /// invisible, because the answers would be identical.
    ///
    /// Combined with a real filter it falls to the walking tier as anything does, and
    /// there it acts as one more exclusion: an entry carrying the promoted tag is outside
    /// the plane, so it is not admitted. That the two tiers agree is the property test.
    ///
    /// Resolve the name against the index's rules with
    /// [`TagRules::plane_of`](crate::tags::TagRules::plane_of); a bit position from
    /// another rule set means something else here.
    pub plane: Option<crate::tags::Promoted>,
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
    /// Tag bits the index recorded for this entry when it was inserted.
    ///
    /// Read, never re-derived. The rules are pure functions of a name and a path, so
    /// re-running them here would usually agree — but "usually" is how a filtered view and
    /// a projected row come to disagree about one entry, and the bits are already in hand.
    pub tags: TagBits,
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
            && self.max_size.is_none()
            && self.kinds.is_empty()
            && self.terminal_extensions.is_empty()
            && self.ancestor_names.is_empty()
            && self.modified.is_unbounded()
            && self.tags.is_unconstrained()
    }

    /// Whether an entry passes every filter.
    pub fn admits(&self, candidate: &Candidate<'_>) -> bool {
        if !self.kinds.is_empty() && !self.kinds.contains(&candidate.kind) {
            return false;
        }
        if let Some(max_size) = self.max_size
            && self.size_of(candidate) > max_size
        {
            return false;
        }
        if let Some(min_size) = self.min_size
            && self.size_of(candidate) < min_size
        {
            return false;
        }
        if !self.modified.contains(candidate.mtime_ns) {
            return false;
        }
        if !self.tags.admits(candidate.tags) {
            return false;
        }
        // A plane holds the entries *without* its tag, so carrying it puts an entry
        // outside. This is what makes the walking tier answer the same question the
        // maintained plane answers, rather than a similar one.
        if let Some(plane) = self.plane
            && candidate.tags & (1 << plane.0) != 0
        {
            return false;
        }
        self.admits_by_path(candidate.relative, candidate.name)
    }

    /// Whether the path-shaped parts of this selection admit a path.
    ///
    /// Every axis decidable from a relative path and a name, named in one place. A live
    /// watcher has only those two when an event arrives -- no `stat`, no index row -- so it
    /// filters through this, and an axis missing here is one it silently forgets: a change
    /// row delivered for an entry the same query would never return. That has now happened
    /// on three separate axes, each time because the second copy of the list was written by
    /// hand. Size, time, kind and tags are deliberately absent, because they need the entry
    /// rather than its name.
    pub fn admits_by_path(&self, relative: &Path, name: &str) -> bool {
        if !self.terminal_extensions.is_empty() && !self.admits_terminal(name) {
            return false;
        }
        if !self.ancestor_names.is_empty() && !self.admits_ancestry(relative) {
            return false;
        }
        // Exclusion wins: a pattern the caller wrote to keep something out should not be
        // overridden by a broader pattern they wrote to let things in.
        if self.exclude.iter().any(|p| p.matches(relative, name)) {
            return false;
        }
        if self.include.is_empty() {
            return true;
        }
        self.include.iter().any(|p| p.matches(relative, name))
    }

    /// Add a terminal suffix to admit, refusing one that could never match.
    ///
    /// The rule is narrow enough that every way of getting it wrong produces silence
    /// instead of an error: `rs` matches nothing because a terminal carries its dot,
    /// `.tar.gz` matches nothing because a terminal is the *last* suffix, and `.RS` matches
    /// nothing because the comparison lowers only the name. Each is a caller who believes
    /// they narrowed a catalog and got an empty page back with nothing to read.
    ///
    /// So the list is built through this rather than pushed to directly, and it lives here
    /// rather than in each surface's argument parser, because a rule stated twice is two
    /// rules that agree until one is edited.
    pub fn admit_terminal_extension(&mut self, value: impl Into<String>) -> crate::Result<()> {
        let value = value.into();
        let hint = match value.strip_prefix('.') {
            None => Some(r#"expected a dotted suffix such as ".rs""#),
            Some("") => Some(r#"expected a suffix after the dot, such as ".rs""#),
            Some(rest) if rest.contains('.') => {
                Some(r#"expected the terminal suffix alone: ".gz" rather than ".tar.gz""#)
            }
            Some(rest) if !rest.chars().flat_map(char::to_lowercase).eq(rest.chars()) => {
                Some(r#"expected lowercase: ".rs" rather than ".RS""#)
            }
            Some(_) => None,
        };
        if let Some(hint) = hint {
            return Err(crate::Error::InvalidValue {
                kind: "terminal extension",
                value,
                hint: hint.to_string(),
            });
        }
        self.terminal_extensions.push(value);
        Ok(())
    }

    /// Add an ancestor directory name to admit, refusing anything that is not one component.
    ///
    /// [`Component::Normal`] is the whole test, which is why this is not a search for
    /// separator characters: it refuses `src/lib` and `.` and the empty name on every
    /// platform, and it refuses `src\lib` on the one platform where that is two names and
    /// admits it on the ones where it is a legal single name.
    pub fn admit_ancestor_name(&mut self, value: impl Into<OsString>) -> crate::Result<()> {
        let value = value.into();
        let mut components = Path::new(&value).components();
        let single =
            matches!((components.next(), components.next()), (Some(Component::Normal(_)), None));
        if !single {
            return Err(crate::Error::InvalidValue {
                kind: "ancestor name",
                value: value.to_string_lossy().into_owned(),
                hint: r#"expected one exact directory name, such as "src""#.to_string(),
            });
        }
        self.ancestor_names.push(value);
        Ok(())
    }

    /// Whether a name's terminal suffix is one this selection admits.
    ///
    /// The name's suffix is lowered and compared for equality; the list is required to be
    /// lowercase already, which is why this is not a case-insensitive match on both sides.
    /// Lowering is done by iterator rather than into a fresh `String`, so a catalog page
    /// over a large tree does not allocate once per row it rejects.
    fn admits_terminal(&self, name: &str) -> bool {
        let Some(suffix) = terminal_suffix(name) else {
            // No terminal suffix, so it is the empty string, which cannot equal a dotted
            // entry. A list that constrains anything therefore excludes these.
            return false;
        };
        self.terminal_extensions
            .iter()
            .any(|wanted| suffix.chars().flat_map(char::to_lowercase).eq(wanted.chars()))
    }

    /// Whether any ancestor component is one this selection admits.
    fn admits_ancestry(&self, relative: &Path) -> bool {
        let Some(ancestors) = relative.parent() else {
            return false;
        };
        ancestors
            .components()
            .any(|component| self.ancestor_names.iter().any(|name| component.as_os_str() == *name))
    }

    /// The size of an entry in the selected metric.
    pub fn size_of(&self, candidate: &Candidate<'_>) -> u64 {
        match self.size {
            SizeMetric::Apparent => candidate.bytes,
            SizeMetric::Allocated => candidate.allocated,
        }
    }
}

impl Selection {
    /// Identity of everything about this selection that shapes a result.
    ///
    /// Not `Hash`, and deliberately: `Selection` holds compiled patterns and a size metric
    /// that only affects rendering, so a derived hash would be an identity over the
    /// implementation rather than over the question. Each component is mixed in by hand,
    /// in a fixed order, from the value a caller supplied -- `Pattern::source` rather than
    /// its compiled alternatives, so two engines parsing one glob agree.
    ///
    /// This exists because a resumable page carries counts established under one selection.
    /// Replaying that cursor under another returns the second selection's rows with the
    /// first one's denominator, and nothing in the page says so -- which is a wrong answer
    /// rather than a missing one, and the reason the cursor binds this value.
    pub fn shape(&self) -> u64 {
        // A plain accumulator threaded through free functions rather than nested closures:
        // each helper needs `hash` mutably, and closures capturing it cannot call each
        // other.
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let hash = &mut hash;

        mix(hash, b"selection/1\x1f");
        for (label, patterns) in
            [(&b"include"[..], &self.include), (&b"exclude"[..], &self.exclude)]
        {
            mix(hash, label);
            // Sorted: `include` is any-of and `exclude` wins outright, so what decides is
            // the set. Two callers listing the same globs in different orders have not
            // asked different questions, and this is an identity over the question.
            let mut sources: Vec<&str> = patterns.iter().map(Pattern::source).collect();
            sources.sort_unstable();
            for source in sources {
                mix(hash, source.as_bytes());
                mix(hash, b"\x1f");
            }
            mix(hash, b"\x1e");
        }
        optional(hash, self.min_size);
        optional(hash, self.max_size);
        let mut kinds: Vec<u8> = self.kinds.iter().map(|kind| *kind as u8).collect();
        kinds.sort_unstable();
        mix(hash, &kinds);
        mix(hash, b"\x1e");
        let mut terminals: Vec<&str> =
            self.terminal_extensions.iter().map(String::as_str).collect();
        terminals.sort_unstable();
        for terminal in terminals {
            mix(hash, terminal.as_bytes());
            mix(hash, b"\x1f");
        }
        mix(hash, b"\x1e");
        let mut ancestors: Vec<&OsString> = self.ancestor_names.iter().collect();
        ancestors.sort_unstable();
        for ancestor in ancestors {
            mix(hash, ancestor.as_encoded_bytes());
            mix(hash, b"\x1f");
        }
        mix(hash, b"\x1e");
        optional(hash, self.modified.since.map(i64::cast_unsigned));
        optional(hash, self.modified.before.map(i64::cast_unsigned));
        scalar(hash, u64::from(self.tags.any));
        scalar(hash, u64::from(self.tags.none));
        optional(hash, self.plane.map(|plane| u64::from(plane.0)));
        optional(hash, self.depth.map(bound_shape));
        optional(hash, self.limit.map(bound_shape));
        optional(hash, self.sort.map(|sort| sort as u64));
        scalar(hash, u64::from(self.reverse));
        // `size` is deliberately absent: it decides which number a *report* renders, not
        // which entries a page admits, so binding it would refuse a continuation that asks
        // the same question in different units.
        *hash
    }
}

/// The terminal suffix of a file name, dot included, or `None` when it has none.
///
/// Spelled out rather than delegated to [`Path::extension`], because the two rules are not
/// the same one and the differences are exactly the names a catalog filter meets: a
/// leading-dot name (`.gitignore`) has no suffix, a trailing dot (`notes.`) leaves nothing
/// after it, and a compound tail keeps only its last part (`archive.tar.gz` is `.gz`).
/// `Path::extension` agrees on those today and returns the suffix *without* its dot, so
/// depending on it would make an exactness claim rest on a coincidence of two libraries
/// and a shed dot. This is the consuming contract's own rule, transcribed.
fn terminal_suffix(name: &str) -> Option<&str> {
    let dot = name.rfind('.')?;
    // A dot at the start marks a hidden name rather than a suffix, and a dot at the end
    // introduces an empty one. Both are "no terminal suffix".
    if dot == 0 || dot + 1 == name.len() { None } else { Some(&name[dot..]) }
}

/// Mix one field's bytes into a shape accumulator, FNV-1a.
fn mix(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

/// Mix one number in, width-tagged by construction.
fn scalar(hash: &mut u64, value: u64) {
    mix(hash, &value.to_le_bytes());
}

/// Mix an optional number in, distinguishing absent from any value it could hold.
fn optional(hash: &mut u64, value: Option<u64>) {
    match value {
        Some(inner) => {
            scalar(hash, 1);
            scalar(hash, inner);
        }
        None => scalar(hash, 0),
    }
}

/// One bound as a number, distinguishing "all" from any finite limit.
fn bound_shape(bound: Bound) -> u64 {
    match bound {
        Bound::All => u64::MAX,
        Bound::Limit(limit) => limit as u64,
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

    fn admits_tagged(selection: &Selection, path: &str, tags: TagBits) -> bool {
        let (relative, name) = candidate(path, EntryKind::File, 1, 0);
        selection.admits(&Candidate {
            relative: &relative,
            name: &name,
            kind: EntryKind::File,
            bytes: 1,
            allocated: 512,
            mtime_ns: 0,
            tags,
        })
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
            tags: 0,
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

    /// An upper bound admits what a lower bound would keep, from the other side.
    ///
    /// Exists because the alternative is filtering above the engine, which means carrying
    /// every candidate across a language boundary in order to throw it away -- the exact
    /// cost a native provider is supposed to remove. Inclusive on purpose, mirroring
    /// `min_size`: a caller whose own contract is exclusive translates at its boundary.
    #[test]
    fn max_size_bounds_from_above_the_way_min_size_bounds_from_below() {
        let capped = Selection { max_size: Some(600), ..Selection::default() };
        assert!(admits(&capped, "at.bin", EntryKind::File, 600, 0), "the bound is inclusive");
        assert!(admits(&capped, "under.bin", EntryKind::File, 599, 0));
        assert!(!admits(&capped, "over.bin", EntryKind::File, 601, 0));

        // Both ends together are a window.
        let window = Selection { min_size: Some(400), max_size: Some(600), ..Selection::default() };
        assert!(admits(&window, "inside.bin", EntryKind::File, 500, 0));
        assert!(!admits(&window, "small.bin", EntryKind::File, 399, 0));
        assert!(!admits(&window, "large.bin", EntryKind::File, 601, 0));

        // A reversed window admits nothing, rather than quietly preferring one bound.
        let empty = Selection { min_size: Some(600), max_size: Some(400), ..Selection::default() };
        assert!(!admits(&empty, "any.bin", EntryKind::File, 500, 0));

        // And it follows the selected metric, like every other size predicate: 100
        // apparent bytes occupy 512 allocated, which exceeds a 200-byte cap.
        let allocated =
            Selection { max_size: Some(200), size: SizeMetric::Allocated, ..Selection::default() };
        assert!(!admits(&allocated, "a.bin", EntryKind::File, 100, 0));
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
    fn tag_filters_narrow_by_any_of_and_exclusion_wins() {
        const DOTFILE: TagBits = 1;
        const VENDORED: TagBits = 1 << 1;

        let mut selection = Selection::default();
        assert!(selection.is_unfiltered(), "no tag constraint is no constraint");
        assert!(admits_tagged(&selection, "a", 0));
        assert!(admits_tagged(&selection, "b", DOTFILE));

        // Any-of, matching `include`: naming a second tag widens.
        selection.tags.any = DOTFILE;
        assert!(!selection.is_unfiltered(), "a tag filter drops the report off the fast tier");
        assert!(admits_tagged(&selection, "a", DOTFILE));
        assert!(!admits_tagged(&selection, "b", VENDORED));
        selection.tags.any = DOTFILE | VENDORED;
        assert!(admits_tagged(&selection, "b", VENDORED));

        // Exclusion wins over inclusion, as it does for patterns.
        selection.tags.none = VENDORED;
        assert!(!admits_tagged(&selection, "b", VENDORED));
        assert!(admits_tagged(&selection, "a", DOTFILE));
        assert!(
            !admits_tagged(&selection, "c", DOTFILE | VENDORED),
            "carrying an excluded tag is disqualifying even while carrying an included one"
        );
    }

    #[test]
    fn an_unbounded_window_does_not_constrain() {
        assert!(ModifiedWindow::default().is_unbounded());
        assert!(ModifiedWindow::default().contains(i64::MIN));
        assert!(ModifiedWindow::default().contains(i64::MAX));
    }

    /// The terminal-suffix rule, spelled out case by case.
    ///
    /// These are the names that separate the rule from the several plausible rules next to
    /// it: the *last* suffix rather than every suffix, a dotted hidden name rather than a
    /// suffix, and a trailing dot rather than an empty one that matches.
    #[test]
    fn the_terminal_suffix_is_the_last_one_and_only_when_there_is_one() {
        assert_eq!(terminal_suffix("notes.txt"), Some(".txt"));
        assert_eq!(terminal_suffix("archive.tar.gz"), Some(".gz"));
        assert_eq!(terminal_suffix("NOTES.TXT"), Some(".TXT"));
        assert_eq!(terminal_suffix("..foo"), Some(".foo"));
        assert_eq!(terminal_suffix("Makefile"), None);
        assert_eq!(terminal_suffix(".gitignore"), None);
        assert_eq!(terminal_suffix("notes."), None);
        assert_eq!(terminal_suffix("..."), None);
        assert_eq!(terminal_suffix(""), None);
    }

    #[test]
    fn a_terminal_extension_admits_by_last_suffix_case_folded() {
        let mut selection = Selection::default();
        selection.admit_terminal_extension(".txt").expect("dotted lowercase suffix");
        assert!(admits(&selection, "notes.txt", EntryKind::File, 1, 0));
        assert!(
            admits(&selection, "a/NOTES.TXT", EntryKind::File, 1, 0),
            "the name's own suffix is lowered before comparison"
        );
        assert!(
            !admits(&selection, "archive.txt.gz", EntryKind::File, 1, 0),
            "the terminal is the last suffix, so a compound tail is judged by its end"
        );
        assert!(!admits(&selection, "Makefile", EntryKind::File, 1, 0));
        assert!(
            !admits(&selection, ".txt", EntryKind::File, 1, 0),
            "a leading dot names a hidden file rather than a bare suffix"
        );
        assert!(
            !admits(&selection, "src", EntryKind::Dir, 0, 0),
            "a directory has no terminal suffix to admit either"
        );

        // Any-of, like every other repeatable axis.
        selection.admit_terminal_extension(".rs").expect("dotted lowercase suffix");
        assert!(admits(&selection, "src/main.rs", EntryKind::File, 1, 0));
        assert!(admits(&selection, "notes.txt", EntryKind::File, 1, 0));
    }

    #[test]
    fn an_ancestor_name_admits_by_whole_component_at_any_depth() {
        let mut selection = Selection::default();
        selection.admit_ancestor_name("src").expect("one component");
        assert!(admits(&selection, "src/main.rs", EntryKind::File, 1, 0));
        assert!(admits(&selection, "crates/a/src/lib.rs", EntryKind::File, 1, 0));
        assert!(
            !admits(&selection, "src", EntryKind::Dir, 0, 0),
            "the entry's own name is never one of its ancestors"
        );
        assert!(
            !admits(&selection, "srcs/main.rs", EntryKind::File, 1, 0),
            "whole components, so a longer name is not a match"
        );
        assert!(
            !admits(&selection, "SRC/main.rs", EntryKind::File, 1, 0),
            "components are compared case-sensitively"
        );
        assert!(!admits(&selection, "main.rs", EntryKind::File, 1, 0));

        selection.admit_ancestor_name("tests").expect("one component");
        assert!(admits(&selection, "tests/a.rs", EntryKind::File, 1, 0));
        assert!(admits(&selection, "src/main.rs", EntryKind::File, 1, 0));
    }

    /// Every rejected form is one that would otherwise match nothing in silence.
    #[test]
    fn a_predicate_value_that_could_never_match_is_refused_when_it_is_written() {
        let mut selection = Selection::default();
        for (value, expected) in [
            ("rs", "dotted"),
            (".", "after the dot"),
            (".tar.gz", "terminal suffix alone"),
            (".RS", "lowercase"),
        ] {
            let error = selection
                .admit_terminal_extension(value)
                .expect_err("a value that can never match is refused");
            let message = error.to_string();
            assert!(
                message.contains(expected),
                "{value:?} should be refused with a hint mentioning {expected:?}, got {message}"
            );
        }
        assert!(selection.terminal_extensions.is_empty(), "a refused value is not half-admitted");

        for value in ["", ".", "..", "src/lib"] {
            selection
                .admit_ancestor_name(value)
                .expect_err("only one exact directory name is a component");
        }
        assert!(selection.ancestor_names.is_empty());
    }

    /// Both axes are decidable from a path and a name, so the live watcher's filter and the
    /// index's own answer have to be the same function.
    #[test]
    fn the_new_axes_are_path_shaped_and_narrow_the_fast_tier() {
        let mut terminal = Selection::default();
        terminal.admit_terminal_extension(".rs").expect("suffix");
        assert!(!terminal.is_unfiltered(), "a roll-up read would ignore this filter entirely");
        assert!(terminal.admits_by_path(Path::new("src/main.rs"), "main.rs"));
        assert!(!terminal.admits_by_path(Path::new("src/main.txt"), "main.txt"));

        let mut ancestry = Selection::default();
        ancestry.admit_ancestor_name("src").expect("component");
        assert!(!ancestry.is_unfiltered());
        assert!(ancestry.admits_by_path(Path::new("src/main.rs"), "main.rs"));
        assert!(!ancestry.admits_by_path(Path::new("docs/main.rs"), "main.rs"));
    }

    /// A page continuation is only valid against the question it was issued for, so both
    /// axes have to move the shape -- including the difference between them.
    #[test]
    fn each_predicate_changes_the_selection_shape() {
        let plain = Selection::default().shape();

        let mut terminal = Selection::default();
        terminal.admit_terminal_extension(".rs").expect("suffix");
        assert_ne!(terminal.shape(), plain);

        let mut ancestry = Selection::default();
        ancestry.admit_ancestor_name(".rs").expect("component");
        assert_ne!(ancestry.shape(), plain);
        assert_ne!(
            ancestry.shape(),
            terminal.shape(),
            "the two axes are separate questions even when spelled identically"
        );

        // Order is not part of the question: the same set has to resume the same page.
        let mut one = Selection::default();
        one.admit_terminal_extension(".rs").expect("suffix");
        one.admit_terminal_extension(".txt").expect("suffix");
        let mut other = Selection::default();
        other.admit_terminal_extension(".txt").expect("suffix");
        other.admit_terminal_extension(".rs").expect("suffix");
        assert_eq!(one.shape(), other.shape());

        // But a *different* set is a different question, and adjacent sets must not
        // collide -- including two that concatenate to the same bytes.
        let mut third = Selection::default();
        third.admit_terminal_extension(".rst").expect("suffix");
        assert_ne!(third.shape(), terminal.shape());
        let mut split = Selection::default();
        split.admit_terminal_extension(".rs").expect("suffix");
        split.admit_terminal_extension(".t").expect("suffix");
        assert_ne!(
            split.shape(),
            third.shape(),
            "each value is delimited, so `.rs` + `.t` is not `.rst`"
        );

        // Ancestor names are arbitrary components rather than dotted suffixes, so nothing
        // about their spelling separates one from two. The delimiter is the whole of it.
        let mut joined = Selection::default();
        joined.admit_ancestor_name("ab").expect("component");
        let mut apart = Selection::default();
        apart.admit_ancestor_name("a").expect("component");
        apart.admit_ancestor_name("b").expect("component");
        assert_ne!(
            joined.shape(),
            apart.shape(),
            "`ab` and `a` + `b` are different questions and must not share a shape"
        );
    }
}
