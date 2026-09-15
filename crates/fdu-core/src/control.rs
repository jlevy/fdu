//! Bounded, removal-aware control state used to classify retained filesystem facts.
//!
//! A control file is producer input, not something the index discovers by walking the
//! filesystem. The table stores the exact bytes a producer verified and the matcher
//! derived from them. That keeps cold discovery, refresh, observation, and snapshot load
//! on one semantic path and makes deleting the last control file an ordinary state
//! transition rather than a special rebuild.
//!
//! **Which of git's ignore inputs count.** Exactly one: every regular file named
//! [`CONTROL_FILE_NAME`] inside the scanned root, each governing its own directory and
//! everything below it, with deeper files taking precedence. Nothing else git consults is
//! read. `.git/info/exclude` and `core.excludesFile` are ignored, so a `.DS_Store` excluded
//! only globally lands in the unignored partition. A nested repository is not a boundary:
//! its `.gitignore` files join the outer ones as if the tree were one repository. And
//! unlike git, which never looks inside an ignored directory, the walk reads a
//! `.gitignore` there too; the ignored partition is still right, because git's rule that
//! an excluded parent cannot be re-included is applied when matching, but the file's
//! bytes are retained against the table bound. Matching is case-sensitive regardless of
//! `core.ignorecase`.

mod gitignore;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gitignore::Gitignore;

/// Name of the fixed control file understood by the first engine version.
pub const CONTROL_FILE_NAME: &str = ".gitignore";

/// Default retained control-table charge for one index, in bytes.
///
/// The charge includes a fixed amount per directory as well as each distinct source's
/// bytes and matcher, so a hostile tree of empty control files cannot evade it. Four MiB is
/// far above ordinary repositories while remaining small relative to the inventory it
/// governs; a source past it is refused, not an error. Callers lift it with
/// [`crate::ScanConfig::control_budget`], which the command line spells
/// `--gitignore-budget`.
pub const DEFAULT_CONTROL_BUDGET: usize = 4 * 1024 * 1024;

/// Longest line a bounded control table admits, in bytes.
///
/// A control file may hold many ordinary rules up to the budget, but a single adversarial
/// rule must not impose unbounded matching work on every entry, so a source with a longer
/// line is refused whole. An unbounded budget (`control_budget: None`, or
/// `--gitignore-budget all`) lifts this guard too: it is one knob for both bounds.
pub const CONTROL_LINE_GUARD_BYTES: usize = 16 * 1024;

/// Conservative retained charge for one key, identity, and matcher shell.
pub(crate) const CONTROL_SOURCE_OVERHEAD: usize = 64;

/// Stable, non-sensitive identity of one retained control source.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ControlIdentity {
    /// Exact source length.
    pub bytes: u64,
    /// Stable FNV-1a digest of the source bytes.
    pub fingerprint: u64,
}

/// One distinct control content, parsed once and shared by every directory holding it.
#[derive(Debug)]
struct SharedContent {
    bytes: Vec<u8>,
    identity: ControlIdentity,
    matcher: Gitignore,
    /// Charge for the exact bytes and the parsed matcher, paid once per distinct content.
    content_cost: usize,
}

/// A distinct content and how many directories of one table hold it.
///
/// The count belongs to the table, not to the `Arc`: a projected clone of a table shares
/// every content with it, so the reference count cannot say which holders one table has.
#[derive(Clone, Debug)]
struct Holding {
    content: Arc<SharedContent>,
    holders: usize,
}

/// Why a control table refused a control source instead of applying its rules.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ControlRefusalReason {
    /// Retaining the source would have taken the table past its control budget.
    Budget,
    /// A line of the source is longer than [`CONTROL_LINE_GUARD_BYTES`].
    LineGuard,
}

impl ControlRefusalReason {
    /// The stable name every structured output and binding uses for this reason.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Budget => "budget",
            Self::LineGuard => "line_guard",
        }
    }
}

/// What a control table did with one verified control source.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ControlAdmission {
    /// The rules apply. `changed` is false when the directory already retained exactly
    /// these bytes.
    Retained {
        /// Whether the table's retained sources changed.
        changed: bool,
    },
    /// The rules do not apply, and the directory retains no source. The refusal is
    /// recorded, so the table's coverage names it.
    Refused(ControlRefusalReason),
}

/// One control file whose rules an index refused, relative to the index root.
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct RefusedControl {
    /// The refused `.gitignore`.
    pub path: PathBuf,
    /// Which bound refused it.
    pub reason: ControlRefusalReason,
}

/// Whether an index's ignore classification applies every control file in its scope.
///
/// Sizes and counts never depend on this: a refused control file costs only the
/// ignored and unignored split. Below a refused file that split is not exact in either
/// direction, because the file may have held negations as well as ignore rules.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ControlCoverage {
    /// The index read no control file, so it classifies nothing as ignored or unignored.
    NotObserved,
    /// The index read every control file in its scope and applied the ones it admitted.
    Observed(ControlObservation),
}

/// The control files an observing index applied and refused.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ControlObservation {
    /// Retained-charge budget in bytes, or `None` when unbounded.
    pub budget: Option<usize>,
    /// Control files whose rules apply.
    pub applied: u64,
    /// Control files refused, counted exactly.
    pub refused: u64,
    /// Refused control files in path order, at most [`crate::MAX_RETAINED_ISSUES`] of
    /// them. A list shorter than [`Self::refused`] is truncated.
    pub refusals: Vec<RefusedControl>,
}

impl ControlObservation {
    /// Whether every control file in scope applies, so ignore classification is exact.
    pub const fn is_complete(&self) -> bool {
        self.refused == 0
    }

    /// Whether [`Self::refusals`] names every refused control file.
    pub fn lists_every_refusal(&self) -> bool {
        u64::try_from(self.refusals.len()).is_ok_and(|listed| listed == self.refused)
    }
}

/// Exact `.gitignore` sources and parsed matchers, keyed by governing directory.
///
/// Identical sources are stored and parsed once. A tree of package checkouts repeats a few
/// `.gitignore` files thousands of times, and charging every copy for its bytes and matcher
/// crossed the bound at a fraction of the distinct rules the tree holds (fdu-szkg). Each
/// directory still pays for its own key, so a tree of empty control files cannot evade the
/// bound, and removing the last holder of a content releases the content's charge.
///
/// A source the bounds cannot admit is refused, not an error: the table records the
/// refusal and keeps no rules for that directory, and the scan that read it continues
/// (fdu-1onj). A refusal ends when the directory's control file is removed or a later
/// read admits it.
#[derive(Clone, Debug)]
pub struct ControlTable {
    by_directory: BTreeMap<PathBuf, Arc<SharedContent>>,
    /// Distinct contents by identity. A list, because an equal length and FNV-1a digest do
    /// not prove equal bytes, and a collision must never share another source's matcher.
    shared: HashMap<ControlIdentity, Vec<Holding>>,
    /// Every refused source, by governing directory. Kept whole, not bounded like issue
    /// details, so removing a refused file keeps the refused count exact.
    refused: BTreeMap<PathBuf, ControlRefusalReason>,
    budget: Option<usize>,
    source_bytes: usize,
    retained_cost: usize,
}

impl Default for ControlTable {
    fn default() -> Self {
        Self::with_budget(Some(DEFAULT_CONTROL_BUDGET))
    }
}

impl ControlTable {
    /// An empty table that refuses sources past `budget` bytes of retained charge, or
    /// none when `budget` is `None`.
    pub(crate) fn with_budget(budget: Option<usize>) -> Self {
        Self {
            by_directory: BTreeMap::new(),
            shared: HashMap::new(),
            refused: BTreeMap::new(),
            budget,
            source_bytes: 0,
            retained_cost: 0,
        }
    }

    /// Insert or replace one verified control source.
    ///
    /// `path` names the control file relative to the index root. The source is retained
    /// exactly, while matching state is derived once per distinct content rather than per
    /// directory or per entry. A source the bounds cannot admit is refused and recorded,
    /// and a source it replaces is dropped with it: rules no longer on disk must not keep
    /// applying.
    ///
    /// # Errors
    ///
    /// [`crate::Error::InvalidControlPath`] when `path` does not name a control file.
    pub fn upsert(&mut self, path: &Path, source: Vec<u8>) -> crate::Result<ControlAdmission> {
        let identity = identity(&source);
        self.upsert_identified(path, source, identity)
    }

    fn upsert_identified(
        &mut self,
        path: &Path,
        source: Vec<u8>,
        identity: ControlIdentity,
    ) -> crate::Result<ControlAdmission> {
        let directory = control_directory(path)?;
        if self.by_directory.get(directory).is_some_and(|current| current.bytes == source) {
            return Ok(ControlAdmission::Retained { changed: false });
        }
        if self.budget.is_some()
            && source.split(|byte| *byte == b'\n').any(|line| line.len() > CONTROL_LINE_GUARD_BYTES)
        {
            return Ok(self.refuse(directory, ControlRefusalReason::LineGuard));
        }
        let content_charge =
            if self.holding(identity, &source).is_some() { 0 } else { content_cost(&source) };
        let next = self
            .retained_cost
            .checked_sub(self.release_charge(directory))
            .and_then(|bytes| bytes.checked_add(directory_cost(directory)))
            .and_then(|bytes| bytes.checked_add(content_charge));
        let Some(next) = next.filter(|next| self.budget.is_none_or(|budget| *next <= budget))
        else {
            return Ok(self.refuse(directory, ControlRefusalReason::Budget));
        };

        self.detach(directory);
        self.attach(directory, source, identity);
        self.refused.remove(directory);
        debug_assert_eq!(self.retained_cost, next);
        Ok(ControlAdmission::Retained { changed: true })
    }

    /// Restore a refusal a snapshot recorded, without the source that was refused.
    pub(crate) fn record_refusal(
        &mut self,
        path: &Path,
        reason: ControlRefusalReason,
    ) -> crate::Result<()> {
        let directory = control_directory(path)?;
        self.refuse(directory, reason);
        Ok(())
    }

    fn refuse(&mut self, directory: &Path, reason: ControlRefusalReason) -> ControlAdmission {
        self.detach(directory);
        self.refused.insert(directory.to_path_buf(), reason);
        ControlAdmission::Refused(reason)
    }

    /// Remove one control source, or the record of its refusal. Missing sources are no-ops.
    ///
    /// # Errors
    ///
    /// [`crate::Error::InvalidControlPath`] when `path` does not name a control file.
    pub fn remove(&mut self, path: &Path) -> crate::Result<bool> {
        let directory = control_directory(path)?;
        let retained = self.detach(directory);
        let refused = self.refused.remove(directory).is_some();
        Ok(retained || refused)
    }

    /// Remove every control file, and every refusal, at or below `subtree`.
    pub(crate) fn remove_subtree(&mut self, subtree: &Path) {
        let directories: Vec<PathBuf> = self
            .by_directory
            .keys()
            .filter(|directory| directory.starts_with(subtree))
            .cloned()
            .collect();
        for directory in directories {
            self.detach(&directory);
        }
        self.refused.retain(|directory, _| !directory.starts_with(subtree));
    }

    /// The holding for exactly `source`, when some directory already retains it.
    fn holding(&self, identity: ControlIdentity, source: &[u8]) -> Option<&Holding> {
        self.shared
            .get(&identity)?
            .iter()
            .find(|holding| holding.content.bytes.as_slice() == source)
    }

    /// The charge that dropping `directory`'s current source would release.
    fn release_charge(&self, directory: &Path) -> usize {
        let Some(content) = self.by_directory.get(directory) else {
            return 0;
        };
        let last_holder = self.shared.get(&content.identity).is_some_and(|holdings| {
            holdings
                .iter()
                .any(|holding| Arc::ptr_eq(&holding.content, content) && holding.holders == 1)
        });
        directory_cost(directory).saturating_add(if last_holder { content.content_cost } else { 0 })
    }

    /// Drop `directory`'s source, releasing its content when this was the last holder.
    fn detach(&mut self, directory: &Path) -> bool {
        let Some(content) = self.by_directory.remove(directory) else {
            return false;
        };
        let holdings = self.shared.get_mut(&content.identity).expect("a retained content is held");
        let position = holdings
            .iter()
            .position(|holding| Arc::ptr_eq(&holding.content, &content))
            .expect("a retained content is listed under its own identity");
        holdings[position].holders -= 1;
        if holdings[position].holders == 0 {
            holdings.swap_remove(position);
            self.retained_cost -= content.content_cost;
            if holdings.is_empty() {
                self.shared.remove(&content.identity);
            }
        }
        self.retained_cost -= directory_cost(directory);
        self.source_bytes -= content.bytes.len();
        true
    }

    /// Hold `source` at `directory`, sharing an identical retained content when one exists.
    fn attach(&mut self, directory: &Path, source: Vec<u8>, identity: ControlIdentity) {
        let holdings = self.shared.entry(identity).or_default();
        let content = if let Some(holding) =
            holdings.iter_mut().find(|holding| holding.content.bytes == source)
        {
            holding.holders += 1;
            Arc::clone(&holding.content)
        } else {
            let content_cost = content_cost(&source);
            let matcher = Gitignore::parse(&source);
            let content =
                Arc::new(SharedContent { bytes: source, identity, matcher, content_cost });
            holdings.push(Holding { content: Arc::clone(&content), holders: 1 });
            self.retained_cost += content_cost;
            content
        };
        self.retained_cost += directory_cost(directory);
        self.source_bytes += content.bytes.len();
        self.by_directory.insert(directory.to_path_buf(), content);
    }

    /// Matcher view for one retained path.
    pub fn matcher_for<'a>(&'a self, path: &'a Path) -> ControlMatcher<'a> {
        ControlMatcher { table: self, path }
    }

    /// Evaluate complete ignore semantics without relying on retained parent facts.
    ///
    /// The index hot path uses [`ControlMatcher::is_ignored`] with the parent's stored
    /// classification. This standalone form evaluates each directory prefix so callers
    /// and tests receive the same answer even without an index entry in hand.
    pub fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        let components: Vec<_> = path.components().collect();
        let mut current = PathBuf::new();
        let mut parent_ignored = false;
        for (position, component) in components.iter().enumerate() {
            current.push(component.as_os_str());
            if parent_ignored {
                return true;
            }
            let current_is_dir = position + 1 < components.len() || is_dir;
            parent_ignored = self.matcher_for(&current).is_ignored(current_is_dir);
        }
        parent_ignored
    }

    /// Relative subtree whose classification may move when `path` changes.
    pub fn affected_subtree(path: &Path) -> crate::Result<PathBuf> {
        Ok(control_directory(path)?.to_path_buf())
    }

    /// Stable identities changed between two complete table states.
    pub(crate) fn changes_from(
        &self,
        previous: &Self,
    ) -> Vec<(PathBuf, Option<ControlIdentity>, Option<ControlIdentity>)> {
        let directories: BTreeSet<&Path> = previous
            .by_directory
            .keys()
            .chain(self.by_directory.keys())
            .map(PathBuf::as_path)
            .collect();
        directories
            .into_iter()
            .filter_map(|directory| {
                let before = previous.by_directory.get(directory);
                let after = self.by_directory.get(directory);
                let changed = match (before, after) {
                    (Some(before), Some(after)) => {
                        !Arc::ptr_eq(before, after) && before.bytes != after.bytes
                    }
                    (None, None) => false,
                    (Some(_), None) | (None, Some(_)) => true,
                };
                changed.then(|| {
                    (
                        control_path(directory),
                        before.map(|source| source.identity),
                        after.map(|source| source.identity),
                    )
                })
            })
            .collect()
    }

    /// Refusals recorded or lifted between two complete table states.
    pub(crate) fn refusal_changes_from(
        &self,
        previous: &Self,
    ) -> Vec<(PathBuf, Option<ControlRefusalReason>, Option<ControlRefusalReason>)> {
        let directories: BTreeSet<&Path> =
            previous.refused.keys().chain(self.refused.keys()).map(PathBuf::as_path).collect();
        directories
            .into_iter()
            .filter_map(|directory| {
                let before = previous.refused.get(directory).copied();
                let after = self.refused.get(directory).copied();
                (before != after).then(|| (control_path(directory), before, after))
            })
            .collect()
    }

    /// Exact sources in deterministic governing-directory order.
    pub(crate) fn sources(&self) -> impl ExactSizeIterator<Item = (PathBuf, &[u8])> {
        self.by_directory
            .iter()
            .map(|(directory, source)| (control_path(directory), source.bytes.as_slice()))
    }

    /// Every refused control file and its reason, in governing-directory order.
    pub fn refusals(&self) -> impl ExactSizeIterator<Item = RefusedControl> + '_ {
        self.refused.iter().map(|(directory, reason)| RefusedControl {
            path: control_path(directory),
            reason: *reason,
        })
    }

    /// Number of refused control files.
    pub fn refused_len(&self) -> usize {
        self.refused.len()
    }

    /// Retained-charge budget in bytes, or `None` when unbounded.
    pub const fn budget(&self) -> Option<usize> {
        self.budget
    }

    /// This table's coverage, listing at most [`crate::MAX_RETAINED_ISSUES`] refusals.
    pub fn observation(&self) -> ControlObservation {
        ControlObservation {
            budget: self.budget,
            applied: u64::try_from(self.len()).unwrap_or(u64::MAX),
            refused: u64::try_from(self.refused_len()).unwrap_or(u64::MAX),
            refusals: self.refusals().take(crate::MAX_RETAINED_ISSUES).collect(),
        }
    }

    /// Exact retained source bytes across the whole table.
    pub const fn source_bytes(&self) -> usize {
        self.source_bytes
    }

    /// Bounded retained charge for the complete table.
    pub const fn retained_cost(&self) -> usize {
        self.retained_cost
    }

    /// Whether `path` already retains exactly `source`.
    pub fn source_is(&self, path: &Path, source: &[u8]) -> bool {
        control_directory(path)
            .ok()
            .and_then(|directory| self.by_directory.get(directory))
            .is_some_and(|current| current.bytes == source)
    }

    /// Whether the table holds a record at `path`: a retained source or a refusal.
    ///
    /// Reconciliation asks this to decide whether a control file missing from a listing
    /// needs a removal, and a refused file that disappears must lift its refusal.
    pub(crate) fn contains(&self, path: &Path) -> bool {
        control_directory(path).ok().is_some_and(|directory| {
            self.by_directory.contains_key(directory) || self.refused.contains_key(directory)
        })
    }

    /// Number of retained control files.
    pub fn len(&self) -> usize {
        self.by_directory.len()
    }

    /// Whether no control file is retained. A table may still record refusals.
    pub fn is_empty(&self) -> bool {
        self.by_directory.is_empty()
    }

    /// Whether the table records nothing at all: no retained source and no refusal.
    pub(crate) fn is_vacant(&self) -> bool {
        self.by_directory.is_empty() && self.refused.is_empty()
    }
}

/// A path-bound view over the controls that may govern it.
pub struct ControlMatcher<'a> {
    table: &'a ControlTable,
    path: &'a Path,
}

impl ControlMatcher<'_> {
    /// Decide this path assuming its retained parent is not ignored.
    ///
    /// A caller with retained facts already knows the parent's effective classification,
    /// so it can stop immediately when that parent is ignored. Otherwise every control
    /// directory on this path is active and the deepest matching opinion wins.
    pub fn is_ignored(&self, is_dir: bool) -> bool {
        // The empty table answers without collecting the ancestor list. Every entry of
        // a scan that observes no control state asks this question exactly once, so the
        // allocation below would be the last per-entry cost of a machinery the scan
        // opted out of (fdu-etfj).
        if self.table.by_directory.is_empty() {
            return false;
        }
        // A deeper control source overrides every matching ancestor. Walk from the
        // immediate directory toward the root and return the first opinion instead of
        // allocating and reversing an ancestor vector for every classified entry.
        for directory in self.path.parent().into_iter().flat_map(Path::ancestors) {
            let Some(source) = self.table.by_directory.get(directory) else {
                continue;
            };
            let relative = self.path.strip_prefix(directory).unwrap_or(self.path);
            if let Some(ignored) = source.matcher.matches(relative, is_dir) {
                return ignored;
            }
        }
        false
    }
}

/// Whether a relative path names the fixed control file.
pub fn is_control_file(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == CONTROL_FILE_NAME)
}

fn control_directory(path: &Path) -> crate::Result<&Path> {
    if !is_control_file(path) {
        return Err(crate::Error::InvalidControlPath(path.to_path_buf()));
    }
    Ok(path.parent().unwrap_or_else(|| Path::new("")))
}

fn control_path(directory: &Path) -> PathBuf {
    directory.join(CONTROL_FILE_NAME)
}

fn identity(bytes: &[u8]) -> ControlIdentity {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x100_0000_01b3;

    let mut fingerprint = FNV_OFFSET_BASIS;
    for byte in bytes {
        fingerprint ^= u64::from(*byte);
        fingerprint = fingerprint.wrapping_mul(FNV_PRIME);
    }
    ControlIdentity { bytes: u64::try_from(bytes.len()).unwrap_or(u64::MAX), fingerprint }
}

/// Charge for one directory's key and identity, whatever content it holds.
fn directory_cost(directory: &Path) -> usize {
    CONTROL_SOURCE_OVERHEAD.saturating_add(directory.as_os_str().as_encoded_bytes().len())
}

/// Charge for one distinct content: its exact bytes and parsed glob bytes, plus the
/// matcher's per-pattern and per-segment shells.
fn content_cost(source: &[u8]) -> usize {
    let (newlines, segment_shells) = source.iter().fold((0usize, 0usize), |counts, byte| {
        (counts.0 + usize::from(*byte == b'\n'), counts.1 + usize::from(*byte == b'/'))
    });
    let pattern_shells = newlines.saturating_add(1);
    source
        .len()
        .saturating_mul(2)
        .saturating_add(pattern_shells.saturating_mul(64))
        .saturating_add(segment_shells.saturating_mul(24))
}

/// Charge for one source whose content no other directory holds.
#[cfg(test)]
fn retained_source_cost(directory: &Path, source: &[u8]) -> usize {
    directory_cost(directory).saturating_add(content_cost(source))
}

#[cfg(test)]
pub(crate) fn source_at_test_limit() -> Vec<u8> {
    let mut source = Vec::new();
    loop {
        let previous_len = source.len();
        source.extend(std::iter::repeat_n(b'a', CONTROL_LINE_GUARD_BYTES));
        source.push(b'\n');
        if retained_source_cost(Path::new(""), &source) > DEFAULT_CONTROL_BUDGET {
            source.truncate(previous_len);
            break;
        }
    }
    let remaining = DEFAULT_CONTROL_BUDGET - retained_source_cost(Path::new(""), &source);
    source.extend(std::iter::repeat_n(b'a', (remaining / 2).min(CONTROL_LINE_GUARD_BYTES)));
    assert_eq!(retained_source_cost(Path::new(""), &source), DEFAULT_CONTROL_BUDGET);
    source
}

#[cfg(test)]
impl ControlTable {
    /// Recompute every charge and holder count from the directories alone.
    fn assert_consistent(&self) {
        let mut distinct: Vec<&Arc<SharedContent>> = Vec::new();
        let mut directory_charges = 0;
        let mut source_bytes = 0;
        for (directory, content) in &self.by_directory {
            directory_charges += directory_cost(directory);
            source_bytes += content.bytes.len();
            if !distinct.iter().any(|seen| Arc::ptr_eq(seen, content)) {
                distinct.push(content);
            }
        }
        let content_charges: usize = distinct.iter().map(|content| content.content_cost).sum();
        assert_eq!(self.retained_cost, directory_charges + content_charges, "retained cost");
        assert_eq!(self.source_bytes, source_bytes, "source bytes");
        let holdings: usize = self.shared.values().map(Vec::len).sum();
        assert_eq!(holdings, distinct.len(), "one holding per distinct content");
        for (identity, holdings) in &self.shared {
            assert!(!holdings.is_empty(), "no empty identity list survives");
            for holding in holdings {
                assert_eq!(holding.content.identity, *identity);
                let holders = self
                    .by_directory
                    .values()
                    .filter(|content| Arc::ptr_eq(content, &holding.content))
                    .count();
                assert_eq!(holding.holders, holders, "holder count");
            }
            for (index, left) in holdings.iter().enumerate() {
                for right in &holdings[index + 1..] {
                    assert_ne!(left.content.bytes, right.content.bytes, "equal bytes are shared");
                }
            }
        }
        assert!(
            self.refused.keys().all(|directory| !self.by_directory.contains_key(directory)),
            "a refused directory retains no source"
        );
        assert!(self.budget.is_none_or(|budget| self.retained_cost <= budget), "within budget");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic `SplitMix64`, so a failing sequence replays from its printed seed.
    struct SplitMix(u64);

    impl SplitMix {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut value = self.0;
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            value ^ (value >> 31)
        }

        fn below(&mut self, bound: usize) -> usize {
            usize::try_from(self.next() % u64::try_from(bound).expect("small bound")).expect("fits")
        }
    }

    /// Every directory's last operation decides whether it holds a record, whatever the
    /// bounds decided: an upsert leaves exactly one of a source or a refusal, and a removal
    /// leaves neither. Charges and holder counts are recomputed after every step.
    #[test]
    fn charges_and_refusals_stay_exact_through_random_upserts_and_removals() {
        const DIRECTORIES: [&str; 6] = ["", "a", "a/b", "b", "b/c/d", "c"];
        let long_line = [vec![b'x'; CONTROL_LINE_GUARD_BYTES + 1], b"\n".to_vec()].concat();
        let large = b"pattern/\n".repeat(40);
        let contents: [&[u8]; 6] =
            [b"*.log\n", b"target/\n", b"!keep\n*.tmp\n", b"", &large, &long_line];
        // Room for a few small sources and one large one, so the budget refuses often.
        let budget = 2 * retained_source_cost(Path::new("b/c/d"), &large);
        for seed in 0..64 {
            let mut random = SplitMix(seed);
            let mut table = ControlTable::with_budget(Some(budget));
            let mut holds_record: BTreeMap<&Path, bool> = BTreeMap::new();
            for step in 0..300 {
                let directory = Path::new(DIRECTORIES[random.below(DIRECTORIES.len())]);
                let path = directory.join(CONTROL_FILE_NAME);
                match random.below(8) {
                    0 => {
                        table.remove_subtree(directory);
                        for (held, record) in &mut holds_record {
                            if held.starts_with(directory) {
                                *record = false;
                            }
                        }
                    }
                    1 | 2 => {
                        table.remove(&path).expect("control path");
                        holds_record.insert(directory, false);
                    }
                    _ => {
                        let content = contents[random.below(contents.len())].to_vec();
                        let admission = table.upsert(&path, content.clone()).expect("control path");
                        if content == long_line {
                            assert_eq!(
                                admission,
                                ControlAdmission::Refused(ControlRefusalReason::LineGuard)
                            );
                        }
                        holds_record.insert(directory, true);
                    }
                }
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    table.assert_consistent();
                    for (directory, record) in &holds_record {
                        let path = directory.join(CONTROL_FILE_NAME);
                        assert_eq!(table.contains(&path), *record, "{}", path.display());
                    }
                    let records = holds_record.values().filter(|record| **record).count();
                    assert_eq!(table.len() + table.refused_len(), records);
                }))
                .unwrap_or_else(|_| panic!("seed {seed}, step {step}: inconsistent table"));
            }
            for directory in DIRECTORIES {
                table.remove(&Path::new(directory).join(CONTROL_FILE_NAME)).expect("control path");
            }
            assert_eq!(table.retained_cost(), 0, "seed {seed}");
            assert_eq!(table.source_bytes(), 0, "seed {seed}");
            assert!(table.shared.is_empty(), "seed {seed}");
            assert!(table.is_vacant(), "seed {seed}");
        }
    }

    #[test]
    fn a_fingerprint_collision_never_shares_a_matcher() {
        let collision = ControlIdentity { bytes: 6, fingerprint: 7 };
        let mut table = ControlTable::default();
        table
            .upsert_identified(Path::new("a/.gitignore"), b"*.log\n".to_vec(), collision)
            .expect("first");
        table
            .upsert_identified(Path::new("b/.gitignore"), b"*.tmp\n".to_vec(), collision)
            .expect("second");
        table.assert_consistent();

        assert_eq!(table.shared[&collision].len(), 2);
        assert!(table.is_ignored(Path::new("a/x.log"), false));
        assert!(!table.is_ignored(Path::new("a/x.tmp"), false));
        assert!(table.is_ignored(Path::new("b/x.tmp"), false));
        assert!(!table.is_ignored(Path::new("b/x.log"), false));
        assert_eq!(
            table.retained_cost(),
            retained_source_cost(Path::new("a"), b"*.log\n")
                + retained_source_cost(Path::new("b"), b"*.tmp\n")
        );

        table.remove(Path::new("a/.gitignore")).expect("remove");
        table.assert_consistent();
        assert!(table.is_ignored(Path::new("b/x.tmp"), false));
    }

    const CHANGED: ControlAdmission = ControlAdmission::Retained { changed: true };
    const UNCHANGED: ControlAdmission = ControlAdmission::Retained { changed: false };
    const OVER_BUDGET: ControlAdmission = ControlAdmission::Refused(ControlRefusalReason::Budget);
    const OVER_LINE_GUARD: ControlAdmission =
        ControlAdmission::Refused(ControlRefusalReason::LineGuard);

    #[test]
    fn replacing_the_last_holder_releases_its_content_for_the_bound() {
        let mut table = ControlTable::default();
        let first = source_at_test_limit();
        assert_eq!(
            table.upsert(Path::new(".gitignore"), first.clone()).expect("control path"),
            CHANGED
        );
        // The same content elsewhere costs only a key, which still crosses a full table.
        assert_eq!(table.upsert(Path::new("copy/.gitignore"), first).expect("path"), OVER_BUDGET);
        assert_eq!(
            table.upsert(Path::new(".gitignore"), b"small\n".to_vec()).expect("control path"),
            CHANGED
        );
        table.assert_consistent();
        assert_eq!(table.retained_cost(), retained_source_cost(Path::new(""), b"small\n"));
    }

    #[test]
    fn creation_edit_and_last_removal_are_exact() {
        let mut table = ControlTable::default();
        assert_eq!(
            table.upsert(Path::new(".gitignore"), b"*.log\n".to_vec()).expect("control path"),
            CHANGED
        );
        let original = table.clone();
        assert_eq!(
            table.upsert(Path::new(".gitignore"), b"*.log\n".to_vec()).expect("control path"),
            UNCHANGED
        );
        assert_eq!(
            table.upsert(Path::new(".gitignore"), b"*.tmp\n".to_vec()).expect("control path"),
            CHANGED
        );
        assert_eq!(table.changes_from(&original).len(), 1);
        assert!(table.remove(Path::new(".gitignore")).expect("remove"));
        assert!(table.is_empty());
        assert_eq!(table.source_bytes(), 0);
        assert!(!table.remove(Path::new(".gitignore")).expect("missing is a no-op"));
    }

    /// The budget admits a table charged exactly to it and refuses one byte more.
    #[test]
    fn the_budget_admits_its_own_size_and_refuses_one_byte_over_it() {
        let source = b"*.log\n".to_vec();
        let exact = retained_source_cost(Path::new("a"), &source);
        let mut at_budget = ControlTable::with_budget(Some(exact));
        assert_eq!(
            at_budget.upsert(Path::new("a/.gitignore"), source.clone()).expect("control path"),
            CHANGED
        );
        assert_eq!(at_budget.retained_cost(), exact);

        let mut under_budget = ControlTable::with_budget(Some(exact - 1));
        assert_eq!(
            under_budget.upsert(Path::new("a/.gitignore"), source).expect("control path"),
            OVER_BUDGET
        );
        assert_eq!(under_budget.retained_cost(), 0);
        assert!(under_budget.contains(Path::new("a/.gitignore")));
        assert_eq!(
            under_budget.observation(),
            ControlObservation {
                budget: Some(exact - 1),
                applied: 0,
                refused: 1,
                refusals: vec![RefusedControl {
                    path: PathBuf::from("a/.gitignore"),
                    reason: ControlRefusalReason::Budget,
                }],
            }
        );
    }

    #[test]
    fn a_refused_source_drops_the_rules_it_replaces_and_freed_budget_admits_it_later() {
        let mut table = ControlTable::default();
        let first = source_at_test_limit();
        assert_eq!(
            table.upsert(Path::new(".gitignore"), first.clone()).expect("control path"),
            CHANGED
        );
        assert_eq!(
            table
                .upsert(Path::new("nested/.gitignore"), b"*.log\n".to_vec())
                .expect("control path"),
            OVER_BUDGET
        );
        let before = table.clone();

        // Replacing the root's source with one that no longer fits drops the old rules
        // rather than keeping rules that are no longer on disk.
        let mut grown = first;
        grown.push(b'\n');
        assert_eq!(
            table.upsert(Path::new(".gitignore"), grown).expect("control path"),
            OVER_BUDGET
        );
        assert!(table.is_empty());
        assert_eq!(table.changes_from(&before).len(), 1);
        assert_eq!(
            table.refusal_changes_from(&before),
            vec![(PathBuf::from(".gitignore"), None, Some(ControlRefusalReason::Budget))]
        );

        // With the budget free, reading the nested file again admits it and lifts its refusal.
        assert_eq!(
            table
                .upsert(Path::new("nested/.gitignore"), b"*.log\n".to_vec())
                .expect("control path"),
            CHANGED
        );
        assert_eq!(table.refused_len(), 1);
        // Removing a refused file lifts its refusal too.
        assert!(table.remove(Path::new(".gitignore")).expect("remove refused"));
        assert_eq!(table.refused_len(), 0);
        table.assert_consistent();
    }

    #[test]
    fn identical_sources_share_one_content_charge() {
        let source = b"target/\n*.log\nnode_modules/\n".to_vec();
        let mut table = ControlTable::default();
        table.upsert(Path::new("a/.gitignore"), source.clone()).expect("first holder");
        let one = table.retained_cost();
        table.upsert(Path::new("bb/.gitignore"), source.clone()).expect("second holder");

        // The second directory pays for its key and nothing for content it shares.
        assert_eq!(table.retained_cost() - one, CONTROL_SOURCE_OVERHEAD + "bb".len());
        assert_eq!(table.source_bytes(), 2 * source.len());
        assert!(table.is_ignored(Path::new("a/debug.log"), false));
        assert!(table.is_ignored(Path::new("bb/debug.log"), false));
    }

    /// One line at the guard applies; one byte longer refuses the whole source, before any
    /// parsing, so a hostile rule costs no matching work.
    #[test]
    fn the_line_guard_admits_its_own_length_and_refuses_one_byte_over_it() {
        let mut table = ControlTable::default();
        let at_guard = vec![b'a'; CONTROL_LINE_GUARD_BYTES];
        assert_eq!(
            table.upsert(Path::new("a/.gitignore"), at_guard).expect("control path"),
            CHANGED
        );
        let over = [b"*.log\n".as_slice(), &vec![b'a'; CONTROL_LINE_GUARD_BYTES + 1]].concat();
        assert_eq!(
            table.upsert(Path::new("b/.gitignore"), over).expect("control path"),
            OVER_LINE_GUARD
        );
        assert!(!table.is_ignored(Path::new("b/debug.log"), false), "no rule of it applies");
        assert_eq!(
            table.refusals().collect::<Vec<_>>(),
            vec![RefusedControl {
                path: PathBuf::from("b/.gitignore"),
                reason: ControlRefusalReason::LineGuard,
            }]
        );
    }

    #[test]
    fn an_unbounded_table_applies_what_the_bounds_would_refuse() {
        let mut table = ControlTable::with_budget(None);
        let long = [b"*.log\n".as_slice(), &vec![b'a'; CONTROL_LINE_GUARD_BYTES + 1]].concat();
        assert_eq!(table.upsert(Path::new(".gitignore"), long).expect("control path"), CHANGED);
        assert_eq!(
            table
                .upsert(Path::new("big/.gitignore"), source_at_test_limit())
                .expect("control path"),
            CHANGED
        );
        assert!(table.retained_cost() > DEFAULT_CONTROL_BUDGET);
        assert!(table.is_ignored(Path::new("debug.log"), false));
        assert_eq!(table.refused_len(), 0);
    }

    #[test]
    fn a_listing_of_refusals_is_bounded_and_says_when_it_is_truncated() {
        let mut table = ControlTable::with_budget(Some(0));
        let refused = crate::MAX_RETAINED_ISSUES + 1;
        for directory in 0..refused {
            let path = PathBuf::from(format!("d{directory:03}/.gitignore"));
            assert_eq!(table.upsert(&path, b"*\n".to_vec()).expect("control path"), OVER_BUDGET);
        }
        let observation = table.observation();
        assert_eq!(observation.refused, u64::try_from(refused).expect("small"));
        assert_eq!(observation.refusals.len(), crate::MAX_RETAINED_ISSUES);
        assert_eq!(observation.refusals[0].path, Path::new("d000/.gitignore"));
        assert!(!observation.lists_every_refusal());
        assert!(!observation.is_complete());
    }

    #[test]
    fn control_identity_uses_standard_fnv1a_vectors() {
        assert_eq!(identity(b"").fingerprint, 0xcbf2_9ce4_8422_2325);
        assert_eq!(identity(b"a").fingerprint, 0xaf63_dc4c_8601_ec8c);
        assert_eq!(identity(b"foobar").fingerprint, 0x8594_4171_f739_67e8);
    }

    #[test]
    fn nested_negation_and_control_removal_change_the_governed_subtree() {
        let mut table = ControlTable::default();
        table.upsert(Path::new(".gitignore"), b"*.log\n".to_vec()).expect("root");
        table.upsert(Path::new("docs/.gitignore"), b"!keep.log\n".to_vec()).expect("nested");

        assert!(table.is_ignored(Path::new("debug.log"), false));
        assert!(table.is_ignored(Path::new("docs/other.log"), false));
        assert!(!table.is_ignored(Path::new("docs/keep.log"), false));
        assert_eq!(
            ControlTable::affected_subtree(Path::new("docs/.gitignore")).expect("scope"),
            Path::new("docs")
        );

        table.remove(Path::new("docs/.gitignore")).expect("remove nested");
        assert!(table.is_ignored(Path::new("docs/keep.log"), false));
    }

    #[test]
    fn ignored_parent_cannot_be_reincluded_from_inside_it() {
        let mut table = ControlTable::default();
        table.upsert(Path::new(".gitignore"), b"vendor/\n".to_vec()).expect("root");
        table
            .upsert(Path::new("vendor/.gitignore"), b"!keep.txt\n".to_vec())
            .expect("retained but inactive nested control");

        assert!(table.is_ignored(Path::new("vendor"), true));
        assert!(table.is_ignored(Path::new("vendor/keep.txt"), false));
    }
}
