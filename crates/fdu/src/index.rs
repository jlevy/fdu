//! The in-memory hierarchical index.
//!
//! The index is a parent-pointer tree in a flat arena. Entries store their **name only**
//! and paths are reconstructed by walking parents, so a path like
//! `srv/data/project/src/lib/utils.rs` costs six name strings across six entries with no
//! duplication — the fsearch/ncdu layout, deliberately not dut's full-path-per-entry.
//!
//! Every directory carries pre-computed roll-up state for its whole subtree, so a query
//! reads a field and never traverses. Applying an [`Observation`] re-merges that state up the
//! ancestor chain only. Producers submit observations; only effective, arbitrated
//! mutations become clocked deltas.
//!
//! Reducers split into two classes and the split is visible in the code, because it
//! decides the cost of an update:
//!
//! - **Invertible** (counts, byte sums, per-extension tallies) apply differentially in
//!   O(depth): add the new contribution, subtract the old one.
//! - **Non-invertible** ([`RollUp::newest_mtime_ns`]) absorb *additions* in O(depth) by
//!   taking a max, but a *removal* may need the directory's value rebuilt from its direct
//!   children — standard incremental-view-maintenance behaviour. Metabrowser's
//!   per-parent newest-mtime heaps are exactly this workaround, hand-written for one
//!   metric.
//!
//! # Concurrency
//!
//! This type is a single-writer structure. The intended deployment is one writer
//! applying deltas behind a `RwLock` with readers taking the read side: writes are short
//! (O(depth) applies) and reads are field lookups rather than queries that walk. The
//! delta contract being the only mutation path means escalating later to epoch or
//! arc-swap snapshots stays contained rather than becoming a rewrite.

use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::classify::derive_ext;
use crate::types::{
    AppliedDelta, Attrs, Clock, EntryIdentity, EntryKind, Expectation, Freshness, InvalidateReason,
    Observation, Op, PathExpectation, PathState, Provenance, ScanScope, Source, Status,
};

/// Maximum number of effective operations retained for [`Index::since`].
///
/// Bounded on purpose: an unbounded journal is a memory leak in a long-lived server. A
/// consumer that falls further behind than this is told so ([`Since::truncated`]) and is
/// expected to re-read state rather than silently miss changes.
const DEFAULT_JOURNAL_OP_CAPACITY: usize = 64 * 1024;

/// Identifier for an entry within an [`Index`] arena.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct EntryId {
    slot: u32,
    generation: u64,
}

impl EntryId {
    /// The root entry. Always present, never removed.
    pub const ROOT: EntryId = EntryId { slot: 0, generation: 0 };

    #[inline]
    const fn idx(self) -> usize {
        self.slot as usize
    }
}

/// Interned extension identity, valid only within the [`Index`] that issued it.
///
/// Never persisted: snapshots store entry names and roll-ups are rebuilt on load, so
/// the interner rebuilds with them and ids stay session-local by construction.
pub type ExtId = u32;

/// Per-extension tally within a roll-up.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ExtTally {
    /// Files with this extension.
    pub files: u64,
    /// Apparent bytes across those files.
    pub bytes: u64,
}

/// Pre-computed aggregate state for one directory's entire subtree.
///
/// # What is counted
///
/// `bytes` and `allocated` sum **files only**. Directories contribute their own subtree
/// plus one to `dirs`, but their own inode block usage is not added — unlike `du`, which
/// counts directory blocks. The difference is small and constant per directory, and
/// making it configurable is deferred rather than guessed at.
///
/// `newest_mtime_ns` is the newest modification time among descendant **files**.
/// Directory mtimes are excluded because they change on every child add or remove, which
/// makes "what changed recently" answer with directories instead of the edits a user
/// actually made.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RollUp {
    /// Descendant files.
    pub files: u64,
    /// Descendant directories, not counting the directory that owns this roll-up.
    pub dirs: u64,
    /// Apparent bytes across descendant files.
    pub bytes: u64,
    /// Allocated bytes across descendant files.
    pub allocated: u64,
    /// Newest mtime among descendant files, or 0 when there are none.
    pub newest_mtime_ns: i64,
    /// Per-extension file and byte tallies across the subtree, keyed by interned
    /// extension id.
    ///
    /// Ids, not strings, because these maps are merged along the ancestor chain for
    /// every file a scan applies: with owned `String` keys that was ~523k string
    /// clones and string-keyed B-tree descents per 60k-entry scan — the largest
    /// single cost in apply. An id is copied, compared as one integer, and resolved
    /// back to its name only at query time via [`Index::by_ext_named`].
    pub by_ext: BTreeMap<ExtId, ExtTally>,
}

impl RollUp {
    /// Fold another roll-up into this one. Commutative and associative, which is what
    /// lets the walk merge subtrees in whatever order threads finish them.
    fn merge(&mut self, other: &RollUp) {
        let had_files = self.files > 0;
        self.files += other.files;
        self.dirs += other.dirs;
        self.bytes += other.bytes;
        self.allocated += other.allocated;
        if other.files > 0 {
            self.newest_mtime_ns = if had_files {
                self.newest_mtime_ns.max(other.newest_mtime_ns)
            } else {
                other.newest_mtime_ns
            };
        }
        for (ext, tally) in &other.by_ext {
            let slot = self.by_ext.entry(*ext).or_default();
            slot.files += tally.files;
            slot.bytes += tally.bytes;
        }
    }

    /// Remove another roll-up's contribution from this one.
    ///
    /// Only the invertible reducers are corrected here. `newest_mtime_ns` is left stale
    /// on purpose and repaired by [`Index::recompute_newest_upward`], because a max
    /// cannot be un-merged without knowing what else contributed it.
    fn unmerge(&mut self, other: &RollUp) {
        self.files = self.files.saturating_sub(other.files);
        self.dirs = self.dirs.saturating_sub(other.dirs);
        self.bytes = self.bytes.saturating_sub(other.bytes);
        self.allocated = self.allocated.saturating_sub(other.allocated);
        for (ext, tally) in &other.by_ext {
            if let Some(slot) = self.by_ext.get_mut(ext) {
                slot.files = slot.files.saturating_sub(tally.files);
                slot.bytes = slot.bytes.saturating_sub(tally.bytes);
                if slot.files == 0 && slot.bytes == 0 {
                    self.by_ext.remove(ext);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Entry {
    parent: Option<EntryId>,
    name: OsString,
    /// Interned extension, computed once at insert. Files only; `None` elsewhere and
    /// for files without an extension. Precomputing it here is what lets
    /// `contribution` run without a string allocation or an interner borrow.
    ext_id: Option<ExtId>,
    /// Where this entry's metadata came from.
    ///
    /// One byte, not a `Provenance` struct: the timestamps that complete the picture
    /// are shared by nearly every entry in a tree, so they live once on the index
    /// while only the source genuinely varies per entry. See `Index::provenance`.
    source: Source,
    kind: EntryKind,
    attrs: Attrs,
    /// Populated for directories only.
    children: BTreeMap<OsString, EntryId>,
    /// Meaningful for directories only.
    rollup: RollUp,
    /// Changes on direct metadata updates. Together with the arena generation this
    /// detects present-state ABA races.
    revision: u64,
    /// Changes only on direct child-map mutations. This is the narrow structural guard
    /// for absent paths and destructive subtree operations.
    children_revision: u64,
}

#[derive(Clone, Debug)]
enum Slot {
    Occupied { generation: u64, entry: Box<Entry> },
    Free { generation: u64, next_free: Option<u32> },
}

/// Result of [`Index::since`].
#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[must_use]
pub struct Since {
    /// Deltas applied strictly after the requested clock, oldest first.
    pub deltas: Vec<AppliedDelta>,
    /// True when the requested clock is older than the retained journal, meaning the
    /// caller has missed changes and must re-read state rather than trust `deltas`.
    pub truncated: bool,
}

/// Summary of what one [`Index::apply`] call did.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ApplyStats {
    /// Entries created.
    pub inserted: u64,
    /// Entries whose attributes changed.
    pub updated: u64,
    /// Entries removed, including cascaded descendants.
    pub removed: u64,
    /// Operations whose complete observed state already matched, so nothing changed.
    pub unchanged: u64,
    /// Subtrees escalated for re-scan.
    pub invalidated: u64,
    /// Conditional observations rejected because the indexed state changed after the
    /// producer captured its baseline.
    pub stale: u64,
}

/// Result of arbitrating and applying one producer observation.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ApplyOutcome {
    /// Per-operation arbitration and mutation counts.
    pub stats: ApplyStats,
    /// Present only when at least one effective mutation was committed.
    pub applied: Option<AppliedDelta>,
}

/// One direct child captured from a shared index at a single read boundary.
///
/// Every field is owned so retaining this value never retains an index lock. The
/// optional roll-up is present for directories; non-directories carry only `attrs`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ChildSnapshot {
    /// Generation-safe arena identity at the capture boundary.
    pub id: EntryId,
    /// Entry name relative to its direct parent.
    pub name: OsString,
    /// Filesystem entry kind.
    pub kind: EntryKind,
    /// Last observed metadata.
    pub attrs: Attrs,
    /// Pre-computed subtree totals for a directory.
    pub rollup: Option<RollUp>,
}

impl std::ops::Deref for ApplyOutcome {
    type Target = ApplyStats;

    fn deref(&self) -> &Self::Target {
        &self.stats
    }
}

/// The in-memory hierarchical index.
#[derive(Clone, Debug)]
pub struct Index {
    root_path: PathBuf,
    scope: ScanScope,
    arena: Vec<Slot>,
    free_head: Option<u32>,
    live: u64,
    clock: Clock,
    journal: VecDeque<AppliedDelta>,
    journal_ops: usize,
    journal_op_capacity: usize,
    /// Oldest clock still represented in `journal`.
    journal_floor: Clock,
    pending_invalidations: Vec<(PathBuf, InvalidateReason)>,
    /// Source recorded on entries that incoming deltas create or update.
    ///
    /// Producers do not carry provenance in the delta itself — an observation says
    /// what it saw, not how much to trust it — so the consumer stamps it, and a
    /// caller loading a snapshot sets this to `Cached` for the duration.
    applying_source: Source,
    /// When this session observed the filesystem, in nanoseconds since the epoch.
    scanned_at_ns: i64,
    /// When the snapshot this index was loaded from captured the tree. Zero when the
    /// index was never loaded from one.
    captured_at_ns: i64,
    /// Interner storage: id → name. Ids are indexes into this vector.
    ext_names: Vec<String>,
    /// Interner lookup: name → id.
    ext_ids: BTreeMap<String, ExtId>,
    freshness_epoch: u64,
    freshness_marks: BTreeMap<PathBuf, FreshnessMark>,
}

#[derive(Clone, Copy, Debug)]
struct FreshnessMark {
    state: Freshness,
    epoch: u64,
}

/// Shareable owner for serving readers while reconciliation applies short writes.
#[derive(Clone, Debug)]
pub struct IndexHandle {
    inner: Arc<RwLock<Index>>,
}

impl IndexHandle {
    /// Wrap an owned index in the shared single-writer owner.
    pub fn new(index: Index) -> Self {
        Self { inner: Arc::new(RwLock::new(index)) }
    }

    fn read_index(&self) -> crate::Result<std::sync::RwLockReadGuard<'_, Index>> {
        self.inner.read().map_err(|_| crate::Error::IndexLockPoisoned)
    }

    fn write_index(&self) -> crate::Result<std::sync::RwLockWriteGuard<'_, Index>> {
        self.inner.write().map_err(|_| crate::Error::IndexLockPoisoned)
    }

    /// Arbitrate and apply one observation under the single-writer lock.
    pub fn apply(&self, observation: &Observation) -> crate::Result<ApplyOutcome> {
        self.write_index()?.apply(observation)
    }

    /// Absolute filesystem root, copied without retaining the read lock.
    pub fn root_path(&self) -> crate::Result<PathBuf> {
        Ok(self.read_index()?.root_path().to_path_buf())
    }

    /// Semantic scan scope represented by the shared index.
    pub fn scope(&self) -> crate::Result<ScanScope> {
        Ok(self.read_index()?.scope())
    }

    /// Trust state for the whole index.
    pub fn freshness(&self) -> crate::Result<Freshness> {
        Ok(self.read_index()?.freshness())
    }

    /// Trust state for one subtree.
    pub fn freshness_at(&self, path: &Path) -> crate::Result<Freshness> {
        Ok(self.read_index()?.freshness_at(path))
    }

    /// Clock of the most recently committed delta.
    pub fn clock(&self) -> crate::Result<Clock> {
        Ok(self.read_index()?.clock())
    }

    /// Number of live entries, including the root.
    pub fn len(&self) -> crate::Result<u64> {
        Ok(self.read_index()?.len())
    }

    /// Whether the index contains only its root.
    pub fn is_empty(&self) -> crate::Result<bool> {
        Ok(self.read_index()?.is_empty())
    }

    /// Owned roll-up totals for the whole tree.
    pub fn total(&self) -> crate::Result<RollUp> {
        Ok(self.read_index()?.total().clone())
    }

    /// Owned roll-up state for a relative directory path.
    pub fn rollup(&self, path: &Path) -> crate::Result<Option<RollUp>> {
        Ok(self.read_index()?.rollup(path).cloned())
    }

    /// Owned metadata for a relative path.
    pub fn attrs(&self, path: &Path) -> crate::Result<Option<Attrs>> {
        Ok(self.read_index()?.attrs(path).copied())
    }

    /// Entry kind for a relative path.
    pub fn kind(&self, path: &Path) -> crate::Result<Option<EntryKind>> {
        Ok(self.read_index()?.kind(path))
    }

    /// Current visible state for a relative path.
    pub fn path_state(&self, path: &Path) -> crate::Result<PathState> {
        Ok(self.read_index()?.path_state(path))
    }

    /// Conditional baseline for a producer operating on a shared index.
    pub fn expectation(&self, path: &Path) -> crate::Result<PathExpectation> {
        Ok(self.read_index()?.expectation(path))
    }

    /// Owned deltas committed after `clock`.
    pub fn since(&self, clock: Clock) -> crate::Result<Since> {
        Ok(self.read_index()?.since(clock))
    }

    /// Direct children captured coherently at one read boundary.
    pub fn children(&self, path: &Path) -> crate::Result<Option<Vec<ChildSnapshot>>> {
        let index = self.read_index()?;
        let Some(children) = index.children(path) else {
            return Ok(None);
        };
        Ok(Some(
            children
                .map(|(name, id)| {
                    let entry = index.entry(id);
                    ChildSnapshot {
                        id,
                        name: name.to_os_string(),
                        kind: entry.kind,
                        attrs: entry.attrs,
                        rollup: entry.kind.is_dir().then(|| entry.rollup.clone()),
                    }
                })
                .collect(),
        ))
    }

    /// Capture one coherent owned index image, releasing the lock before callers do
    /// serialization, filesystem I/O, conversion, or other potentially blocking work.
    pub fn snapshot(&self) -> crate::Result<Index> {
        Ok(self.read_index()?.clone())
    }

    pub(crate) fn child_states(
        &self,
        path: &Path,
    ) -> crate::Result<BTreeMap<OsString, PathExpectation>> {
        let index = self.read_index()?;
        Ok(collect_child_expectations(&index, path))
    }

    pub(crate) fn take_pending_invalidations(
        &self,
    ) -> crate::Result<Vec<(PathBuf, InvalidateReason)>> {
        Ok(self.write_index()?.take_pending_invalidations())
    }

    pub(crate) fn restore_pending_invalidations(
        &self,
        invalidations: Vec<(PathBuf, InvalidateReason)>,
    ) -> crate::Result<()> {
        self.write_index()?.restore_pending_invalidations(invalidations);
        Ok(())
    }

    pub(crate) fn begin_reconcile(&self, path: &Path) -> crate::Result<u64> {
        Ok(self.write_index()?.begin_reconcile(path))
    }

    pub(crate) fn finish_reconcile(
        &self,
        path: &Path,
        started_at: u64,
        complete: bool,
    ) -> crate::Result<()> {
        self.write_index()?.finish_reconcile(path, started_at, complete);
        Ok(())
    }

    #[cfg(feature = "watch")]
    pub(crate) fn apply_if_clock(
        &self,
        clock: Clock,
        observation: &Observation,
    ) -> crate::Result<Option<ApplyOutcome>> {
        let mut index = self.write_index()?;
        if index.clock() != clock {
            return Ok(None);
        }
        index.apply(observation).map(Some)
    }

    #[cfg(feature = "watch")]
    pub(crate) fn watch_boundary(&self) -> crate::Result<(PathBuf, ScanScope, Clock)> {
        let index = self.read_index()?;
        Ok((index.root_path().to_path_buf(), index.scope(), index.clock()))
    }

    #[cfg(feature = "watch")]
    pub(crate) fn invalidate_root(&self, reason: InvalidateReason) -> crate::Result<ApplyOutcome> {
        self.apply(&Observation::new(vec![Op::InvalidateSubtree { path: PathBuf::new(), reason }]))
    }
}

impl Index {
    /// Create an empty index rooted at `root_path`.
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        Self::new_with_scope(root_path, ScanScope::default())
    }

    /// Create an empty index with an explicit semantic scan scope.
    pub fn new_with_scope(root_path: impl Into<PathBuf>, scope: ScanScope) -> Self {
        Self::new_with_journal_op_capacity(root_path, scope, DEFAULT_JOURNAL_OP_CAPACITY)
    }

    fn new_with_journal_op_capacity(
        root_path: impl Into<PathBuf>,
        scope: ScanScope,
        journal_op_capacity: usize,
    ) -> Self {
        let root = Entry {
            parent: None,
            name: OsString::new(),
            ext_id: None,
            source: Source::Scanned,
            kind: EntryKind::Dir,
            attrs: Attrs::default(),
            children: BTreeMap::new(),
            rollup: RollUp::default(),
            revision: 0,
            children_revision: 0,
        };
        Self {
            root_path: root_path.into(),
            scope,
            arena: vec![Slot::Occupied { generation: 0, entry: Box::new(root) }],
            free_head: None,
            live: 1,
            clock: Clock::ZERO,
            journal: VecDeque::new(),
            journal_ops: 0,
            journal_op_capacity,
            journal_floor: Clock::ZERO,
            pending_invalidations: Vec::new(),
            freshness_epoch: 0,
            freshness_marks: BTreeMap::new(),
            applying_source: Source::Scanned,
            scanned_at_ns: Self::now_unix_nanos(),
            captured_at_ns: 0,
            ext_names: Vec::new(),
            ext_ids: BTreeMap::new(),
        }
    }

    #[cfg(test)]
    fn with_journal_op_capacity(root_path: impl Into<PathBuf>, journal_op_capacity: usize) -> Self {
        Self::new_with_journal_op_capacity(root_path, ScanScope::default(), journal_op_capacity)
    }

    /// The absolute path this index is rooted at.
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Semantic scope represented by this index and any snapshot written from it.
    pub const fn scope(&self) -> ScanScope {
        self.scope
    }

    /// Trust state for the whole index.
    pub fn freshness(&self) -> Freshness {
        self.freshness_at(Path::new(""))
    }

    /// Trust state for one subtree, including any stale descendant it contains.
    pub fn freshness_at(&self, path: &Path) -> Freshness {
        self.freshness_marks
            .iter()
            .filter(|(marked, _)| path.starts_with(marked) || marked.starts_with(path))
            .map(|(_, mark)| mark.state)
            .max_by_key(|state| state.rank())
            .unwrap_or(Freshness::Fresh)
    }

    /// The clock of the most recently applied delta.
    pub fn clock(&self) -> Clock {
        self.clock
    }

    /// Number of live entries, including the root.
    pub fn len(&self) -> u64 {
        self.live
    }

    /// True when the index holds nothing but its root.
    pub fn is_empty(&self) -> bool {
        self.live <= 1
    }

    /// Roll-up state for the whole tree.
    pub fn total(&self) -> &RollUp {
        &self.entry(EntryId::ROOT).rollup
    }

    /// Arbitrate a producer observation and commit its effective mutations.
    ///
    /// Conditional operations are accepted only while their baseline still matches.
    /// No-ops and stale operations do not advance the clock or enter the journal.
    pub fn apply(&mut self, observation: &Observation) -> crate::Result<ApplyOutcome> {
        validate_observation(observation)?;
        if observation.is_empty() {
            return Ok(ApplyOutcome::default());
        }
        let Some(next_clock) = self.clock.checked_next() else {
            // Clock exhaustion is relevant only if arbitration would commit a change.
            // Probe the otherwise infallible mutation phase on a clone so an all-no-op
            // or all-stale observation still reports its stats at the terminal clock,
            // while a real change fails before touching shared state. This path is
            // reachable only after 2^64 committed batches (or by an injected test).
            let mut probe = self.clone();
            let outcome = probe.apply_validated(observation, self.clock);
            return if outcome.applied.is_some() {
                Err(crate::Error::ClockExhausted)
            } else {
                Ok(outcome)
            };
        };

        Ok(self.apply_validated(observation, next_clock))
    }

    /// Apply an already validated observation with a clock known to be available.
    fn apply_validated(&mut self, observation: &Observation, next_clock: Clock) -> ApplyOutcome {
        let mut stats = ApplyStats::default();
        let mut effective = Vec::new();
        let mut accepted = Vec::with_capacity(observation.len());
        for observed in &observation.ops {
            let op = &observed.op;
            if let Expectation::State(expected) = observed.expectation {
                if !self.expectation_matches(op, expected) {
                    stats.stale += 1;
                    accepted.push(false);
                    continue;
                }
            }
            accepted.push(true);
        }

        for (observed, accepted) in observation.ops.iter().zip(accepted) {
            if !accepted {
                continue;
            }
            let op = &observed.op;
            let changed = match op {
                Op::Upsert { path, kind, attrs } => {
                    self.apply_upsert(path, *kind, *attrs, &mut stats)
                }
                Op::Remove { path } => self.apply_remove(path, &mut stats),
                Op::InvalidateSubtree { path, reason } => {
                    self.pending_invalidations.push((path.clone(), *reason));
                    self.mark_unfresh(path, Freshness::Stale);
                    stats.invalidated += 1;
                    true
                }
            };
            if changed {
                effective.push(op.clone());
            }
        }

        if effective.is_empty() {
            return ApplyOutcome { stats, applied: None };
        }

        self.clock = next_clock;
        let applied = AppliedDelta { clock: self.clock, ops: effective };
        if applied.len() > self.journal_op_capacity {
            self.journal.clear();
            self.journal_ops = 0;
            self.journal_floor = applied.clock;
            return ApplyOutcome { stats, applied: Some(applied) };
        }

        while self.journal_ops + applied.len() > self.journal_op_capacity {
            if let Some(dropped) = self.journal.pop_front() {
                self.journal_ops -= dropped.len();
                self.journal_floor = dropped.clock;
            }
        }
        self.journal_ops += applied.len();
        self.journal.push_back(applied.clone());
        ApplyOutcome { stats, applied: Some(applied) }
    }

    /// Apply trusted bootstrap data without exposing it as live change history.
    pub(crate) fn apply_baseline(
        &mut self,
        observation: &Observation,
    ) -> crate::Result<ApplyStats> {
        let outcome = self.apply(observation)?;
        self.establish_baseline();
        Ok(outcome.stats)
    }

    #[cfg(test)]
    pub(crate) fn apply_ok(&mut self, observation: &Observation) -> ApplyOutcome {
        self.apply(observation).expect("test observation must be valid")
    }

    #[cfg(test)]
    pub(crate) fn apply_baseline_ok(&mut self, observation: &Observation) -> ApplyStats {
        self.apply_baseline(observation).expect("test baseline must be valid")
    }

    /// Mark the current tree as the process baseline.
    pub(crate) fn establish_baseline(&mut self) {
        self.clock = Clock::ZERO;
        self.journal.clear();
        self.journal_ops = 0;
        self.journal_floor = Clock::ZERO;
        self.pending_invalidations.clear();
    }

    pub(crate) fn set_initial_freshness(&mut self, complete: bool) {
        self.freshness_marks.clear();
        if !complete {
            self.mark_unfresh(Path::new(""), Freshness::Partial);
        }
    }

    pub(crate) fn begin_reconcile(&mut self, path: &Path) -> u64 {
        self.mark_unfresh(path, Freshness::Reconciling)
    }

    pub(crate) fn finish_reconcile(&mut self, path: &Path, started_at: u64, complete: bool) {
        self.freshness_marks
            .retain(|marked, mark| !marked.starts_with(path) || mark.epoch > started_at);
        if !complete {
            self.mark_unfresh(path, Freshness::Partial);
        }
    }

    fn mark_unfresh(&mut self, path: &Path, state: Freshness) -> u64 {
        self.freshness_epoch =
            self.freshness_epoch.checked_add(1).expect("freshness epoch exhausted");
        let epoch = self.freshness_epoch;
        self.freshness_marks.insert(path.to_path_buf(), FreshnessMark { state, epoch });
        epoch
    }

    /// Current user-visible state for one path.
    ///
    /// Conditional producers should capture [`Self::expectation`] so ABA and structural
    /// races cannot return to the same visible state unnoticed.
    pub fn path_state(&self, path: &Path) -> PathState {
        let Some(id) = self.lookup(path) else {
            return PathState::Absent;
        };
        let entry = self.entry(id);
        PathState::Present { kind: entry.kind, attrs: entry.attrs }
    }

    /// Conditional baseline with target and nearest-ancestor ABA protection.
    pub fn expectation(&self, path: &Path) -> PathExpectation {
        let entry = self.entry_identity(path);
        PathExpectation::new(
            self.path_state(path),
            entry,
            entry.is_none().then(|| self.absence_guard_identity(path)).flatten(),
        )
    }

    pub(crate) fn relaxed_expectation(&self, path: &Path) -> PathExpectation {
        PathExpectation::new(self.path_state(path), self.entry_identity(path), None)
    }

    /// Deltas applied since `clock`, oldest first.
    pub fn since(&self, clock: Clock) -> Since {
        Since {
            deltas: self.journal.iter().filter(|d| d.clock > clock).cloned().collect(),
            truncated: clock < self.journal_floor,
        }
    }

    /// Take the subtrees that producers escalated for re-scan.
    ///
    /// The caller is expected to hand these to the scan layer, which turns them back
    /// into precise deltas. Escalation is closed-loop: draining this list without
    /// re-scanning is what makes an index silently diverge.
    pub fn take_pending_invalidations(&mut self) -> Vec<(PathBuf, InvalidateReason)> {
        std::mem::take(&mut self.pending_invalidations)
    }

    /// Put unresolved invalidations back without minting a second public change.
    pub(crate) fn restore_pending_invalidations(
        &mut self,
        invalidations: Vec<(PathBuf, InvalidateReason)>,
    ) {
        self.pending_invalidations.extend(invalidations);
    }

    /// Look up an entry id by path relative to the root.
    pub fn lookup(&self, path: &Path) -> Option<EntryId> {
        let mut current = EntryId::ROOT;
        for part in normalize(path)? {
            current = *self.entry(current).children.get(part)?;
        }
        Some(current)
    }

    /// Roll-up state for a directory, by relative path. The empty path is the root.
    pub fn rollup(&self, path: &Path) -> Option<&RollUp> {
        let id = self.lookup(path)?;
        let entry = self.entry(id);
        entry.kind.is_dir().then_some(&entry.rollup)
    }

    /// Attributes for any entry, by relative path.
    pub fn attrs(&self, path: &Path) -> Option<&Attrs> {
        Some(&self.entry(self.lookup(path)?).attrs)
    }

    /// Kind of an entry, by relative path.
    pub fn kind(&self, path: &Path) -> Option<EntryKind> {
        Some(self.entry(self.lookup(path)?).kind)
    }

    /// Borrow direct children of a directory as `(name, id)` pairs in name order.
    ///
    /// The iterator borrows this owned index and allocates nothing.
    pub fn children(
        &self,
        path: &Path,
    ) -> Option<impl DoubleEndedIterator<Item = (&OsStr, EntryId)> + ExactSizeIterator + '_> {
        let id = self.lookup(path)?;
        let entry = self.entry(id);
        entry
            .kind
            .is_dir()
            .then(|| entry.children.iter().map(|(name, child)| (name.as_os_str(), *child)))
    }

    /// Borrow direct children of an entry id as `(name, id)` pairs in name order.
    ///
    /// Returns `None` for a stale handle. A live non-directory returns an empty iterator.
    pub fn children_of(
        &self,
        id: EntryId,
    ) -> Option<impl DoubleEndedIterator<Item = (&OsStr, EntryId)> + ExactSizeIterator + '_> {
        Some(self.try_entry(id)?.children.iter().map(|(name, child)| (name.as_os_str(), *child)))
    }

    /// Reconstruct an entry's path relative to the root by walking parent pointers.
    pub fn path_of(&self, id: EntryId) -> Option<PathBuf> {
        let mut parts = Vec::new();
        let mut current = Some(id);
        while let Some(node) = current {
            let entry = self.try_entry(node)?;
            if entry.parent.is_some() {
                parts.push(entry.name.as_os_str());
            }
            current = entry.parent;
        }
        parts.reverse();
        Some(parts.iter().collect())
    }

    /// Roll-up state for an entry id, if it is a directory.
    pub fn rollup_of(&self, id: EntryId) -> Option<&RollUp> {
        let entry = self.try_entry(id)?;
        entry.kind.is_dir().then_some(&entry.rollup)
    }

    /// Attributes for an entry id, or `None` when the handle is stale.
    pub fn attrs_of(&self, id: EntryId) -> Option<&Attrs> {
        Some(&self.try_entry(id)?.attrs)
    }

    /// Kind for an entry id, or `None` when the handle is stale.
    pub fn kind_of(&self, id: EntryId) -> Option<EntryKind> {
        Some(self.try_entry(id)?.kind)
    }

    /// Name for an entry id. The root's name is empty; stale handles return `None`.
    pub fn name_of(&self, id: EntryId) -> Option<&OsStr> {
        Some(&self.try_entry(id)?.name)
    }

    // ---- internals ----

    fn try_entry(&self, id: EntryId) -> Option<&Entry> {
        match self.arena.get(id.idx())? {
            Slot::Occupied { generation, entry } if *generation == id.generation => Some(entry),
            Slot::Occupied { .. } | Slot::Free { .. } => None,
        }
    }

    fn expectation_matches(&self, op: &Op, expected: PathExpectation) -> bool {
        if self.path_state(op.path()) != expected.state {
            return false;
        }

        let require_structure = match (op, expected.state) {
            (Op::Remove { .. }, _) => true,
            (Op::Upsert { kind, .. }, PathState::Present { kind: baseline, .. }) => {
                *kind != baseline
            }
            (Op::Upsert { .. } | Op::InvalidateSubtree { .. }, _) => false,
        };
        if !same_target(self.entry_identity(op.path()), expected.entry(), require_structure) {
            return false;
        }

        match expected.absence_guard() {
            Some(expected) => self
                .absence_guard_identity(op.path())
                .is_some_and(|current| current.same_absence_guard(expected)),
            None => true,
        }
    }

    fn absence_guard_identity(&self, path: &Path) -> Option<EntryIdentity> {
        let parts = normalize(path)?;
        let (_, ancestors) = parts.split_last()?;
        let mut current = EntryId::ROOT;
        for part in ancestors {
            let Some(child) = self.entry(current).children.get(*part).copied() else {
                break;
            };
            current = child;
        }
        Some(self.identity(current))
    }

    fn entry_identity(&self, path: &Path) -> Option<EntryIdentity> {
        Some(self.identity(self.lookup(path)?))
    }

    fn identity(&self, id: EntryId) -> EntryIdentity {
        let entry = self.entry(id);
        EntryIdentity::new(
            id.slot,
            id.generation,
            entry.revision,
            entry.children_revision,
            entry.kind.is_dir(),
        )
    }

    fn bump_revision(entry: &mut Entry) {
        entry.revision = entry.revision.checked_add(1).expect("entry revision exhausted");
    }

    fn bump_children_revision(entry: &mut Entry) {
        entry.children_revision =
            entry.children_revision.checked_add(1).expect("entry children revision exhausted");
    }

    fn insert_child(&mut self, parent: EntryId, name: OsString, child: EntryId) {
        let entry = self.entry_mut(parent);
        entry.children.insert(name, child);
        Self::bump_children_revision(entry);
    }

    fn remove_child(&mut self, parent: EntryId, name: &OsStr) {
        let entry = self.entry_mut(parent);
        if entry.children.remove(name).is_some() {
            Self::bump_children_revision(entry);
        }
    }

    fn entry(&self, id: EntryId) -> &Entry {
        self.try_entry(id).expect("internal entry handle must be live")
    }

    fn entry_mut(&mut self, id: EntryId) -> &mut Entry {
        match self.arena.get_mut(id.idx()) {
            Some(Slot::Occupied { generation, entry }) if *generation == id.generation => entry,
            Some(Slot::Occupied { .. } | Slot::Free { .. }) | None => {
                panic!("internal entry handle must be live: {id:?}")
            }
        }
    }

    fn alloc(&mut self, entry: Entry) -> EntryId {
        self.live += 1;
        if let Some(free_slot) = self.free_head {
            let free_idx = free_slot as usize;
            let (generation, next) = match &self.arena[free_idx] {
                Slot::Free { generation, next_free } => (*generation, *next_free),
                Slot::Occupied { .. } => unreachable!("free list pointed at a live slot"),
            };
            self.free_head = next;
            self.arena[free_idx] = Slot::Occupied { generation, entry: Box::new(entry) };
            return EntryId { slot: free_slot, generation };
        }
        let slot = u32::try_from(self.arena.len()).expect("index arena exceeded u32 capacity");
        let id = EntryId { slot, generation: 0 };
        self.arena.push(Slot::Occupied { generation: 0, entry: Box::new(entry) });
        id
    }

    fn free(&mut self, id: EntryId) {
        let next_generation = match &self.arena[id.idx()] {
            Slot::Occupied { generation, .. } if *generation == id.generation => {
                generation.checked_add(1).expect("entry generation exhausted")
            }
            Slot::Occupied { .. } | Slot::Free { .. } => {
                panic!("internal entry handle must be live: {id:?}")
            }
        };
        self.arena[id.idx()] =
            Slot::Free { generation: next_generation, next_free: self.free_head };
        self.free_head = Some(id.slot);
        self.live -= 1;
    }

    /// Wall-clock now, in nanoseconds since the epoch, or zero if the clock is before
    /// it. Provenance timestamps are for display, so a nonsensical clock reads as
    /// "unknown" rather than propagating an error through every constructor.
    fn now_unix_nanos() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|since| i64::try_from(since.as_nanos()).ok())
            .unwrap_or(0)
    }

    /// Provenance of one path: where its value came from, when, and how settled.
    ///
    /// Built on demand from the entry's stored source and the index's timestamps
    /// rather than read from a field, because the timestamps are shared by nearly
    /// every entry and storing them per entry would cost far more than the
    /// information is worth. For a directory this reports the *composed* provenance
    /// of its whole subtree, so a directory is only as trustworthy as its least
    /// trustworthy descendant.
    pub fn provenance(&self, path: &Path) -> Option<Provenance> {
        let id = self.lookup(path)?;
        Some(self.provenance_of(id))
    }

    fn provenance_of(&self, id: EntryId) -> Provenance {
        let entry = self.entry(id);
        Provenance {
            source: entry.source,
            observed_at_ns: self.observed_at(entry.source),
            status: self.status_of(id),
        }
    }

    /// Whether this path's totals account for everything beneath it.
    ///
    /// Derived from the freshness marks rather than stored: an incomplete scan or a
    /// reconciliation in progress both leave a value that can still grow.
    fn status_of(&self, id: EntryId) -> Status {
        let Some(path) = self.path_of(id) else {
            return Status::Complete;
        };
        match self.freshness_at(&path) {
            Freshness::Fresh => Status::Complete,
            Freshness::Partial | Freshness::Reconciling | Freshness::Stale => Status::Partial,
        }
    }

    /// When an entry with this source was observed.
    const fn observed_at(&self, source: Source) -> i64 {
        match source {
            Source::Cached | Source::JournalConfirmed => self.captured_at_ns,
            Source::Scanned | Source::Revalidated => self.scanned_at_ns,
        }
    }

    /// Stamp deltas applied from here on with `source`, restoring the previous value
    /// when the returned guard value is passed back.
    ///
    /// Used by snapshot loading, which is replaying observations that describe a tree
    /// as it was, not as this process has seen it.
    pub(crate) fn set_applying_source(&mut self, source: Source, captured_at_ns: i64) -> Source {
        let previous = self.applying_source;
        self.applying_source = source;
        if captured_at_ns != 0 {
            self.captured_at_ns = captured_at_ns;
        }
        previous
    }

    /// Intern an extension name, returning its stable id within this index.
    fn intern_ext(&mut self, name: &str) -> ExtId {
        if let Some(id) = self.ext_ids.get(name) {
            return *id;
        }
        let id = ExtId::try_from(self.ext_names.len()).expect("extension interner exhausted");
        self.ext_names.push(name.to_string());
        self.ext_ids.insert(name.to_string(), id);
        id
    }

    /// Resolve a roll-up's interned extension tallies back to their names.
    ///
    /// This is the query-time boundary: merges run on integer ids, and the strings
    /// reappear only here, once per report rather than once per ancestor per file.
    pub fn by_ext_named(&self, rollup: &RollUp) -> BTreeMap<String, ExtTally> {
        rollup
            .by_ext
            .iter()
            .map(|(id, tally)| {
                let name =
                    self.ext_names.get(*id as usize).expect("extension id from a foreign index");
                (name.clone(), *tally)
            })
            .collect()
    }

    /// What an entry contributes to each of its ancestors.
    fn contribution(&self, id: EntryId) -> RollUp {
        let entry = self.entry(id);
        match entry.kind {
            EntryKind::Dir => {
                let mut roll = entry.rollup.clone();
                roll.dirs += 1;
                roll
            }
            EntryKind::File => {
                let mut roll = RollUp {
                    files: 1,
                    dirs: 0,
                    bytes: entry.attrs.size,
                    allocated: entry.attrs.allocated,
                    newest_mtime_ns: entry.attrs.mtime_ns,
                    by_ext: BTreeMap::new(),
                };
                if let Some(ext_id) = entry.ext_id {
                    roll.by_ext.insert(ext_id, ExtTally { files: 1, bytes: entry.attrs.size });
                }
                roll
            }
            EntryKind::Symlink | EntryKind::Other => RollUp::default(),
        }
    }

    fn merge_upward(&mut self, from_parent: Option<EntryId>, contribution: &RollUp) {
        let mut current = from_parent;
        while let Some(id) = current {
            let entry = self.entry_mut(id);
            entry.rollup.merge(contribution);
            current = entry.parent;
        }
    }

    fn unmerge_upward(&mut self, from_parent: Option<EntryId>, contribution: &RollUp) {
        let mut current = from_parent;
        while let Some(id) = current {
            let entry = self.entry_mut(id);
            entry.rollup.unmerge(contribution);
            current = entry.parent;
        }
    }

    /// Rebuild `newest_mtime_ns` from direct children, walking to the root.
    ///
    /// Stops early once a directory's value is unchanged, since nothing above it can
    /// change either. That early exit is what keeps the common removal O(depth) instead
    /// of O(depth x children).
    fn recompute_newest_upward(&mut self, from: Option<EntryId>) {
        let mut current = from;
        while let Some(id) = current {
            let mut newest: Option<i64> = None;
            for child in self.entry(id).children.values() {
                let child_entry = self.entry(*child);
                let candidate = if child_entry.kind.is_dir() {
                    (child_entry.rollup.files > 0).then_some(child_entry.rollup.newest_mtime_ns)
                } else {
                    Some(child_entry.attrs.mtime_ns)
                };
                if let Some(candidate) = candidate {
                    newest = Some(newest.map_or(candidate, |current| current.max(candidate)));
                }
            }
            let newest = newest.unwrap_or(0);
            let entry = self.entry_mut(id);
            if entry.rollup.newest_mtime_ns == newest {
                return;
            }
            entry.rollup.newest_mtime_ns = newest;
            current = entry.parent;
        }
    }

    /// Resolve a relative path to a directory id, creating missing ancestors.
    ///
    /// Watch events do not arrive parent-first the way a walk does, so an upsert deep in
    /// a tree may name ancestors the index has never seen. Creating them as directories
    /// with default attributes keeps the delta applicable; a later upsert or the
    /// revalidation sweep fills in their real attributes.
    fn ensure_dir_chain(&mut self, parts: &[&OsStr], stats: &mut ApplyStats) -> EntryId {
        let mut current = EntryId::ROOT;
        for part in parts {
            if let Some(existing) = self.entry(current).children.get(*part).copied() {
                if self.entry(existing).kind.is_dir() {
                    current = existing;
                    continue;
                }
                // A path cannot have children beneath a non-directory. Replace the
                // conflicting record with a placeholder directory; a later observation
                // for the ancestor fills in its real attributes.
                self.remove_entry(existing, stats);
            }
            let child = self.alloc(Entry {
                parent: Some(current),
                name: (*part).to_os_string(),
                ext_id: None,
                source: self.applying_source,
                kind: EntryKind::Dir,
                attrs: Attrs::default(),
                children: BTreeMap::new(),
                rollup: RollUp::default(),
                revision: 0,
                children_revision: 0,
            });
            self.insert_child(current, (*part).to_os_string(), child);
            // A new empty directory contributes one to `dirs` all the way up.
            let contribution = RollUp { dirs: 1, ..RollUp::default() };
            self.merge_upward(Some(current), &contribution);
            stats.inserted += 1;
            current = child;
        }
        current
    }

    fn apply_upsert(
        &mut self,
        path: &Path,
        kind: EntryKind,
        attrs: Attrs,
        stats: &mut ApplyStats,
    ) -> bool {
        let Some(parts) = normalize(path) else {
            return false;
        };
        let Some((name, ancestors)) = parts.split_last() else {
            // The root itself: only its own attributes can change.
            if self.entry(EntryId::ROOT).attrs == attrs {
                stats.unchanged += 1;
                return false;
            }
            let root = self.entry_mut(EntryId::ROOT);
            root.attrs = attrs;
            Self::bump_revision(root);
            stats.updated += 1;
            return true;
        };

        let parent = self.ensure_dir_chain(ancestors, stats);
        let existing = self.entry(parent).children.get(*name).copied();

        if let Some(id) = existing {
            let entry = self.entry(id);
            if entry.kind == kind {
                if entry.attrs == attrs {
                    stats.unchanged += 1;
                    return false;
                }
                if kind.is_dir() {
                    // A directory's own attributes do not reach its ancestors' roll-ups,
                    // so there is nothing to re-merge.
                    let entry = self.entry_mut(id);
                    entry.attrs = attrs;
                    Self::bump_revision(entry);
                    stats.updated += 1;
                    return true;
                }
                let old = self.contribution(id);
                self.unmerge_upward(Some(parent), &old);
                let entry = self.entry_mut(id);
                entry.attrs = attrs;
                Self::bump_revision(entry);
                let new = self.contribution(id);
                self.merge_upward(Some(parent), &new);
                if new.newest_mtime_ns < old.newest_mtime_ns {
                    self.recompute_newest_upward(Some(parent));
                }
                stats.updated += 1;
                return true;
            }
            // The kind changed (a file became a directory, say). Remove and re-insert
            // rather than trying to mutate one shape into the other.
            self.remove_entry(id, stats);
        }

        let ext_id = (kind == EntryKind::File)
            .then(|| name.to_str().and_then(derive_ext).map(|ext| self.intern_ext(&ext)))
            .flatten();
        let id = self.alloc(Entry {
            parent: Some(parent),
            name: (*name).to_os_string(),
            ext_id,
            source: self.applying_source,
            kind,
            attrs,
            children: BTreeMap::new(),
            rollup: RollUp::default(),
            revision: 0,
            children_revision: 0,
        });
        self.insert_child(parent, (*name).to_os_string(), id);
        let contribution = self.contribution(id);
        self.merge_upward(Some(parent), &contribution);
        stats.inserted += 1;
        true
    }

    fn apply_remove(&mut self, path: &Path, stats: &mut ApplyStats) -> bool {
        let Some(id) = self.lookup(path) else {
            stats.unchanged += 1;
            return false;
        };
        if id == EntryId::ROOT {
            stats.unchanged += 1;
            return false;
        }
        self.remove_entry(id, stats);
        true
    }

    fn remove_entry(&mut self, id: EntryId, stats: &mut ApplyStats) {
        let parent = self.entry(id).parent;
        let name = self.entry(id).name.clone();
        let contribution = self.contribution(id);

        self.unmerge_upward(parent, &contribution);
        if let Some(parent) = parent {
            self.remove_child(parent, &name);
        }

        // Free the subtree iteratively; a recursive drop would blow the stack on deep
        // trees, which is exactly the shape this engine is built for.
        let mut queue = vec![id];
        while let Some(node) = queue.pop() {
            let children: Vec<EntryId> = self.entry(node).children.values().copied().collect();
            queue.extend(children);
            self.free(node);
            stats.removed += 1;
        }

        // The max may have lived in what was just removed.
        self.recompute_newest_upward(parent);
    }
}

/// Capture every child's expectation directly off its live entry, with no path work.
///
/// Both reconcile targets use this. The exclusive path once had a twin in `scan.rs`
/// that re-derived each expectation by joining a `PathBuf` and descending from the
/// root — two full descents and ~13 allocations per child to recover an `EntryId`
/// the iterator already held. The equivalence test below is what lets the twin stay
/// deleted.
pub(crate) fn collect_child_expectations(
    index: &Index,
    path: &Path,
) -> BTreeMap<OsString, PathExpectation> {
    index.children(path).map_or_else(BTreeMap::new, |children| {
        children
            .map(|(name, id)| {
                let entry = index.entry(id);
                let expectation = PathExpectation::new(
                    PathState::Present { kind: entry.kind, attrs: entry.attrs },
                    Some(index.identity(id)),
                    None,
                );
                (name.to_os_string(), expectation)
            })
            .collect()
    })
}

/// Split a relative path into its normal components, rejecting anything that escapes.
///
/// Returns `None` for paths containing `..`, a root, or a prefix — an index keyed by
/// relative path has no way to represent those, and silently normalizing them away would
/// let a delta write outside the tree it claims to describe.
/// The components are borrowed from `path`, not copied out of it.
///
/// Owning them cost an allocation per component, and this runs twice for every
/// operation in every batch — once to validate the path and once to apply it. On a
/// tree averaging eight levels deep that was on the order of eighteen allocations per
/// entry, all of them holding bytes that the caller's `PathBuf` already owned and
/// outlives. Only the returned `Vec` allocates now, and only where a slice is
/// genuinely needed.
fn normalize(path: &Path) -> Option<Vec<&OsStr>> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(parts)
}

/// Whether a path can be represented by an index keyed on relative paths.
///
/// The same rule as [`normalize`] without building anything: validation asks a yes or
/// no question, and answering it by constructing a component list and dropping it was
/// pure allocation.
fn path_is_representable(path: &Path) -> bool {
    path.components().all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn validate_observation(observation: &Observation) -> crate::Result<()> {
    for observed in &observation.ops {
        let path = observed.op.path();
        if !path_is_representable(path) {
            return Err(crate::Error::PathEscapesRoot(path.to_path_buf()));
        }
    }
    Ok(())
}

fn same_target(
    current: Option<EntryIdentity>,
    expected: Option<EntryIdentity>,
    require_structure: bool,
) -> bool {
    match (current, expected) {
        (Some(current), Some(expected)) => current.same_target(expected, require_structure),
        (None, None) => true,
        (Some(_), None) | (None, Some(_)) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ObservationOp;
    use std::sync::{Arc, Barrier};

    fn file_attrs(size: u64, mtime_ns: i64) -> Attrs {
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

    #[test]
    fn shared_queries_return_owned_values_and_release_the_lock() {
        let handle = IndexHandle::new(index_with_sample_tree());
        let retained_total = handle.total().expect("total");
        let retained_history = handle.since(Clock::ZERO).expect("history");
        let retained_children = handle.children(Path::new("src")).expect("children");
        let retained_snapshot = handle.snapshot().expect("snapshot");

        let writer = handle.clone();
        let (done_tx, done_rx) = std::sync::mpsc::sync_channel(1);
        let thread = std::thread::spawn(move || {
            let result = writer.apply(&Observation::new(vec![upsert(
                "concurrent.txt",
                EntryKind::File,
                file_attrs(7, 30),
            )]));
            done_tx.send(result).expect("report writer result");
        });

        let outcome = done_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("owned query results must not retain the read lock")
            .expect("writer apply");
        thread.join().expect("writer thread");

        assert_eq!(outcome.inserted, 1);
        assert_eq!(retained_total.files, 3);
        assert!(!retained_history.deltas.is_empty());
        assert_eq!(retained_children.expect("src directory").len(), 2);
        assert!(retained_snapshot.lookup(Path::new("concurrent.txt")).is_none());
        assert!(handle.kind(Path::new("concurrent.txt")).expect("query").is_some());
    }

    #[test]
    fn captured_child_expectations_match_individual_path_lookups() {
        let index = index_with_sample_tree();
        let captured = collect_child_expectations(&index, Path::new("src"));

        assert!(!captured.is_empty());
        for (name, expectation) in captured {
            assert_eq!(expectation, index.expectation(&Path::new("src").join(name)));
        }
    }

    #[test]
    fn index_and_shared_handle_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<Index>();
        assert_send_sync::<IndexHandle>();
    }

    #[test]
    fn simultaneous_writers_commit_unique_contiguous_clocks_in_journal_order() {
        let writer_count: usize = 8;
        let handle: IndexHandle = IndexHandle::new(Index::new("/root"));
        let barrier: Arc<Barrier> = Arc::new(Barrier::new(writer_count));
        let (result_tx, result_rx) = std::sync::mpsc::sync_channel(writer_count);

        std::thread::scope(|scope| {
            for worker_id in 0..writer_count {
                let worker: IndexHandle = handle.clone();
                let start: Arc<Barrier> = Arc::clone(&barrier);
                let results = result_tx.clone();
                scope.spawn(move || {
                    let ordinal: u64 = u64::try_from(worker_id + 1).expect("small worker count");
                    let path: String = format!("writer-{ordinal}.txt");
                    start.wait();
                    let outcome: crate::Result<ApplyOutcome> =
                        worker.apply(&Observation::new(vec![upsert(
                            &path,
                            EntryKind::File,
                            file_attrs(ordinal, i64::try_from(ordinal).expect("small ordinal")),
                        )]));
                    results.send((path, outcome)).expect("report writer result");
                });
            }
        });
        drop(result_tx);

        let mut committed_clocks: Vec<u64> = Vec::with_capacity(writer_count);
        for (path, outcome) in result_rx {
            let applied: AppliedDelta =
                outcome.expect("writer apply").applied.expect("unique upsert must commit");
            committed_clocks.push(applied.clock.0);
            assert!(handle.kind(Path::new(&path)).expect("query committed path").is_some());
        }
        committed_clocks.sort_unstable();

        let last_clock: u64 = u64::try_from(writer_count).expect("small writer count");
        let expected_clocks: Vec<u64> = (1..=last_clock).collect();
        assert_eq!(committed_clocks, expected_clocks);
        assert_eq!(handle.clock().expect("clock"), Clock(last_clock));

        let journal_clocks: Vec<u64> = handle
            .since(Clock::ZERO)
            .expect("journal")
            .deltas
            .iter()
            .map(|delta| delta.clock.0)
            .collect();
        assert_eq!(journal_clocks, expected_clocks);
    }

    #[test]
    fn readers_observe_only_complete_states_around_a_large_batch() {
        let file_count: u64 = 2_048;
        let expected_bytes: u64 = file_count * (file_count + 1) / 2;
        let operations: Vec<Op> = (1..=file_count)
            .map(|ordinal| {
                upsert(
                    &format!("batch/file-{ordinal}.bin"),
                    EntryKind::File,
                    file_attrs(ordinal, i64::try_from(ordinal).expect("small ordinal")),
                )
            })
            .collect();
        let observation: Observation = Observation::new(operations);
        let handle: IndexHandle = IndexHandle::new(Index::new("/root"));
        let before: Index = handle.snapshot().expect("before snapshot");
        assert_eq!(before.total().files, 0);

        let barrier: Arc<Barrier> = Arc::new(Barrier::new(2));
        let (done_tx, done_rx) = std::sync::mpsc::sync_channel(1);
        std::thread::scope(|scope| {
            let writer: IndexHandle = handle.clone();
            let writer_start: Arc<Barrier> = Arc::clone(&barrier);
            scope.spawn(move || {
                writer_start.wait();
                let result: crate::Result<ApplyOutcome> = writer.apply(&observation);
                done_tx.send(result).expect("report batch result");
            });

            let reader: IndexHandle = handle.clone();
            let reader_start: Arc<Barrier> = Arc::clone(&barrier);
            scope.spawn(move || {
                reader_start.wait();
                let deadline: std::time::Instant =
                    std::time::Instant::now() + std::time::Duration::from_secs(10);
                loop {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "reader did not observe batch completion before the deadline"
                    );
                    let image: Index = reader.snapshot().expect("coherent reader snapshot");
                    let total: &RollUp = image.total();
                    match total.files {
                        0 => {
                            assert_eq!(total.bytes, 0);
                            assert_eq!(image.len(), 1);
                        }
                        count if count == file_count => {
                            assert_eq!(total.bytes, expected_bytes);
                            assert_eq!(total.dirs, 1);
                            assert_eq!(image.len(), file_count + 2);
                        }
                        partial => panic!("reader observed partial batch with {partial} files"),
                    }

                    match done_rx.try_recv() {
                        Ok(result) => {
                            assert_eq!(result.expect("batch apply").inserted, file_count + 1);
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {}
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            panic!("batch writer disconnected")
                        }
                    }
                }
            });
        });

        let after: Index = handle.snapshot().expect("after snapshot");
        assert_eq!(after.total().files, file_count);
        assert_eq!(after.total().bytes, expected_bytes);
        assert_eq!(after.len(), file_count + 2);
    }

    #[test]
    fn poisoned_shared_lock_returns_typed_errors() {
        let handle: IndexHandle = IndexHandle::new(Index::new("/root"));
        let poisoner: IndexHandle = handle.clone();
        let panic_result: std::thread::Result<()> = std::thread::spawn(move || {
            let _poison_guard = poisoner.write_index().expect("initial write lock");
            panic!("intentional lock poison");
        })
        .join();
        assert!(panic_result.is_err());

        assert!(matches!(handle.total(), Err(crate::Error::IndexLockPoisoned)));
        assert!(matches!(
            handle.apply(&Observation::new(vec![upsert(
                "never-applied.txt",
                EntryKind::File,
                file_attrs(1, 1),
            )])),
            Err(crate::Error::IndexLockPoisoned)
        ));
    }

    #[test]
    fn clock_exhaustion_rejects_before_any_mutation() {
        let mut index = index_with_sample_tree();
        index.clock = Clock(u64::MAX);
        let before_total = index.total().clone();
        let before_len = index.len();

        let error = index
            .apply(&Observation::new(vec![upsert(
                "too-late.txt",
                EntryKind::File,
                file_attrs(1, 1),
            )]))
            .expect_err("clock exhaustion must be typed");

        assert!(matches!(error, crate::Error::ClockExhausted));
        assert_eq!(index.clock(), Clock(u64::MAX));
        assert_eq!(index.len(), before_len);
        assert_eq!(index.total(), &before_total);
        assert!(index.lookup(Path::new("too-late.txt")).is_none());
    }

    #[test]
    fn terminal_clock_still_accepts_no_op_and_stale_observations() {
        let mut index = index_with_sample_tree();
        let current = *index.attrs(Path::new("src/main.rs")).expect("sample attributes");
        let stale_baseline = index.expectation(Path::new("src/main.rs"));
        index.apply_ok(&Observation::new(vec![upsert(
            "src/main.rs",
            EntryKind::File,
            file_attrs(99, 99),
        )]));
        index.clock = Clock(u64::MAX);
        let before_total = index.total().clone();
        let before_len = index.len();
        let before_journal = index.journal.clone();

        let no_op = index
            .apply(&Observation::new(vec![upsert(
                "src/main.rs",
                EntryKind::File,
                file_attrs(99, 99),
            )]))
            .expect("a no-op needs no new clock");
        let stale = index
            .apply(&Observation::from_ops(vec![ObservationOp::if_state(
                upsert("src/main.rs", EntryKind::File, current),
                stale_baseline,
            )]))
            .expect("a rejected stale observation needs no new clock");

        assert_eq!(no_op.unchanged, 1);
        assert!(no_op.applied.is_none());
        assert_eq!(stale.stale, 1);
        assert!(stale.applied.is_none());
        assert_eq!(index.clock(), Clock(u64::MAX));
        assert_eq!(index.len(), before_len);
        assert_eq!(index.total(), &before_total);
        assert_eq!(index.journal, before_journal);
    }

    #[test]
    fn delayed_conditional_observation_cannot_overwrite_newer_state() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert(
            "file.txt",
            EntryKind::File,
            file_attrs(10, 1),
        )]));

        let baseline = index.expectation(Path::new("file.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("file.txt", EntryKind::File, file_attrs(20, 2)),
            baseline,
        )]);

        index.apply_ok(&Observation::new(vec![upsert(
            "file.txt",
            EntryKind::File,
            file_attrs(30, 3),
        )]));
        let outcome = index.apply_ok(&delayed);

        assert_eq!(outcome.stats.stale, 1);
        assert!(outcome.applied.is_none());
        assert_eq!(index.attrs(Path::new("file.txt")).expect("file").size, 30);
    }

    #[test]
    fn delayed_absent_child_cannot_replace_a_newer_parent_file() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert("parent", EntryKind::Dir, file_attrs(0, 1))]));
        let child_baseline = index.expectation(Path::new("parent/child.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("parent/child.txt", EntryKind::File, file_attrs(10, 2)),
            child_baseline,
        )]);

        index.apply_ok(&Observation::new(vec![upsert(
            "parent",
            EntryKind::File,
            file_attrs(20, 3),
        )]));
        let outcome = index.apply_ok(&delayed);

        assert_eq!(outcome.stats.stale, 1);
        assert!(outcome.applied.is_none());
        assert_eq!(index.kind(Path::new("parent")), Some(EntryKind::File));
        assert!(index.lookup(Path::new("parent/child.txt")).is_none());
    }

    #[test]
    fn conditional_observation_rejects_present_state_aba() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert(
            "file.txt",
            EntryKind::File,
            file_attrs(10, 1),
        )]));
        let baseline = index.expectation(Path::new("file.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("file.txt", EntryKind::File, file_attrs(20, 2)),
            baseline,
        )]);

        index.apply_ok(&Observation::new(vec![upsert(
            "file.txt",
            EntryKind::File,
            file_attrs(30, 3),
        )]));
        index.apply_ok(&Observation::new(vec![upsert(
            "file.txt",
            EntryKind::File,
            file_attrs(10, 1),
        )]));
        let outcome = index.apply_ok(&delayed);

        assert_eq!(outcome.stats.stale, 1);
        assert!(outcome.applied.is_none());
        assert_eq!(index.attrs(Path::new("file.txt")).expect("file").size, 10);
    }

    #[test]
    fn conditional_observation_rejects_absent_state_aba() {
        let mut index = Index::new("/root");
        let baseline = index.expectation(Path::new("file.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("file.txt", EntryKind::File, file_attrs(20, 2)),
            baseline,
        )]);

        index.apply_ok(&Observation::new(vec![upsert(
            "file.txt",
            EntryKind::File,
            file_attrs(30, 3),
        )]));
        index.apply_ok(&Observation::new(vec![Op::Remove { path: PathBuf::from("file.txt") }]));
        let outcome = index.apply_ok(&delayed);

        assert_eq!(outcome.stats.stale, 1);
        assert!(outcome.applied.is_none());
        assert!(index.lookup(Path::new("file.txt")).is_none());
    }

    #[test]
    fn unrelated_mutation_does_not_stale_an_absent_path() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert("dir", EntryKind::Dir, file_attrs(0, 1))]));
        let baseline = index.expectation(Path::new("dir/new.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("dir/new.txt", EntryKind::File, file_attrs(20, 2)),
            baseline,
        )]);

        index.apply_ok(&Observation::new(vec![upsert(
            "other.txt",
            EntryKind::File,
            file_attrs(30, 3),
        )]));
        let outcome = index.apply_ok(&delayed);

        assert_eq!(outcome.stats.stale, 0);
        assert_eq!(outcome.stats.inserted, 1);
        assert_eq!(index.attrs(Path::new("dir/new.txt")).expect("file").size, 20);
    }

    #[test]
    fn directory_metadata_change_does_not_stale_an_absent_child() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert("dir", EntryKind::Dir, file_attrs(0, 1))]));
        let baseline = index.expectation(Path::new("dir/new.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("dir/new.txt", EntryKind::File, file_attrs(20, 2)),
            baseline,
        )]);

        index.apply_ok(&Observation::new(vec![upsert("dir", EntryKind::Dir, file_attrs(0, 3))]));
        let outcome = index.apply_ok(&delayed);

        assert_eq!(outcome.stats.stale, 0);
        assert_eq!(outcome.stats.inserted, 1);
    }

    #[test]
    fn file_parent_metadata_change_stales_an_absent_child() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert(
            "parent",
            EntryKind::File,
            file_attrs(10, 1),
        )]));
        let baseline = index.expectation(Path::new("parent/child.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("parent/child.txt", EntryKind::File, file_attrs(20, 2)),
            baseline,
        )]);

        index.apply_ok(&Observation::new(vec![upsert(
            "parent",
            EntryKind::File,
            file_attrs(30, 3),
        )]));
        let outcome = index.apply_ok(&delayed);

        assert_eq!(outcome.stats.stale, 1);
        assert_eq!(index.kind(Path::new("parent")), Some(EntryKind::File));
        assert!(index.lookup(Path::new("parent/child.txt")).is_none());
    }

    #[test]
    fn delayed_directory_remove_cannot_delete_a_newer_child() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert("dir", EntryKind::Dir, file_attrs(0, 1))]));
        let baseline = index.expectation(Path::new("dir"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            Op::Remove { path: PathBuf::from("dir") },
            baseline,
        )]);

        index.apply_ok(&Observation::new(vec![upsert(
            "dir/new.txt",
            EntryKind::File,
            file_attrs(20, 2),
        )]));
        let outcome = index.apply_ok(&delayed);

        assert_eq!(outcome.stats.stale, 1);
        assert!(index.lookup(Path::new("dir/new.txt")).is_some());
    }

    #[test]
    fn conditional_batch_is_validated_at_one_boundary() {
        let mut index = Index::new("/root");
        let first = index.expectation(Path::new("first.txt"));
        let second = index.expectation(Path::new("second.txt"));

        let outcome = index.apply_ok(&Observation::from_ops(vec![
            ObservationOp::if_state(upsert("first.txt", EntryKind::File, file_attrs(10, 1)), first),
            ObservationOp::if_state(
                upsert("second.txt", EntryKind::File, file_attrs(20, 2)),
                second,
            ),
        ]));

        assert_eq!(outcome.stats.inserted, 2);
        assert_eq!(outcome.stats.stale, 0);
    }

    #[test]
    fn malformed_batch_is_rejected_before_any_index_mutation() {
        let invalid_paths = [
            PathBuf::from("../escape"),
            PathBuf::from(format!("{}absolute", std::path::MAIN_SEPARATOR)),
        ];

        for invalid_path in invalid_paths {
            for invalid_first in [false, true] {
                let mut index = index_with_sample_tree();
                let before_clock = index.clock;
                let before_live = index.live;
                let before_total = index.total().clone();
                let before_journal = index.journal.clone();
                let before_journal_ops = index.journal_ops;
                let before_journal_floor = index.journal_floor;
                let before_invalidations = index.pending_invalidations.clone();
                let before_freshness_epoch = index.freshness_epoch;
                let before_freshness = index.freshness();
                let valid = upsert("new.txt", EntryKind::File, file_attrs(99, 99));
                let invalid = Op::InvalidateSubtree {
                    path: invalid_path.clone(),
                    reason: InvalidateReason::Requested,
                };
                let ops = if invalid_first { vec![invalid, valid] } else { vec![valid, invalid] };

                let error = index.apply(&Observation::new(ops)).expect_err("malformed batch");

                assert!(
                    matches!(error, crate::Error::PathEscapesRoot(path) if path == invalid_path)
                );
                assert_eq!(index.clock, before_clock);
                assert_eq!(index.live, before_live);
                assert_eq!(index.total(), &before_total);
                assert_eq!(index.journal, before_journal);
                assert_eq!(index.journal_ops, before_journal_ops);
                assert_eq!(index.journal_floor, before_journal_floor);
                assert_eq!(index.pending_invalidations, before_invalidations);
                assert_eq!(index.freshness_epoch, before_freshness_epoch);
                assert_eq!(index.freshness(), before_freshness);
                assert!(index.lookup(Path::new("new.txt")).is_none());
            }
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_prefix_is_rejected_before_mutation() {
        let mut index = index_with_sample_tree();
        let before_clock = index.clock();

        let error = index
            .apply(&Observation::new(vec![Op::Remove { path: PathBuf::from(r"C:\escape") }]))
            .expect_err("prefixed path");

        assert!(
            matches!(error, crate::Error::PathEscapesRoot(path) if path == Path::new(r"C:\escape"))
        );
        assert_eq!(index.clock(), before_clock);
    }

    fn index_with_sample_tree() -> Index {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("src", EntryKind::Dir, Attrs::default()),
            upsert("src/main.rs", EntryKind::File, file_attrs(100, 10)),
            upsert("src/lib.rs", EntryKind::File, file_attrs(200, 20)),
            upsert("docs", EntryKind::Dir, Attrs::default()),
            upsert("docs/guide.md", EntryKind::File, file_attrs(300, 30)),
        ]));
        index
    }

    #[test]
    fn rollups_aggregate_up_the_tree() {
        let index = index_with_sample_tree();

        let total = index.total();
        assert_eq!(total.files, 3);
        assert_eq!(total.dirs, 2);
        assert_eq!(total.bytes, 600);
        assert_eq!(total.newest_mtime_ns, 30);

        let src = index.rollup(Path::new("src")).expect("src is a directory");
        assert_eq!(src.files, 2);
        assert_eq!(src.dirs, 0);
        assert_eq!(src.bytes, 300);
        assert_eq!(src.newest_mtime_ns, 20);
    }

    #[test]
    fn symlinks_and_special_nodes_do_not_contribute_regular_file_tallies() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("regular.txt", EntryKind::File, file_attrs(10, 10)),
            upsert("link.rs", EntryKind::Symlink, file_attrs(99, 99)),
            upsert("socket.md", EntryKind::Other, file_attrs(88, 88)),
        ]));

        let total = index.total();
        assert_eq!(total.files, 1);
        assert_eq!(total.bytes, 10);
        assert_eq!(total.allocated, 512);
        assert_eq!(total.newest_mtime_ns, 10);
        assert_eq!(index.by_ext_named(total)[".txt"], ExtTally { files: 1, bytes: 10 });
        assert!(!index.by_ext_named(total).contains_key(".rs"));
        assert!(!index.by_ext_named(total).contains_key(".md"));

        index.apply_ok(&Observation::new(vec![upsert(
            "link.rs",
            EntryKind::File,
            file_attrs(99, 99),
        )]));
        assert_eq!(index.total().files, 2);
        assert_eq!(index.total().bytes, 109);

        index.apply_ok(&Observation::new(vec![upsert(
            "link.rs",
            EntryKind::Symlink,
            file_attrs(99, 99),
        )]));
        assert_eq!(index.total().files, 1);
        assert_eq!(index.total().bytes, 10);
    }

    #[test]
    fn per_extension_tallies_roll_up_hierarchically() {
        let index = index_with_sample_tree();

        let total = index.total();
        assert_eq!(index.by_ext_named(total)[".rs"], ExtTally { files: 2, bytes: 300 });
        assert_eq!(index.by_ext_named(total)[".md"], ExtTally { files: 1, bytes: 300 });

        // Per-directory breakdown, which no surveyed tool provides.
        let src = index.rollup(Path::new("src")).expect("src is a directory");
        assert_eq!(index.by_ext_named(src)[".rs"], ExtTally { files: 2, bytes: 300 });
        assert!(!index.by_ext_named(src).contains_key(".md"));
    }

    #[test]
    fn upsert_with_matching_fingerprint_is_a_no_op() {
        let mut index = index_with_sample_tree();
        let before = index.total().clone();
        let mark = index.clock();

        let stats = index.apply_ok(&Observation::new(vec![upsert(
            "src/main.rs",
            EntryKind::File,
            file_attrs(100, 10),
        )]));

        assert_eq!(stats.unchanged, 1);
        assert_eq!(stats.updated, 0);
        assert_eq!(index.total(), &before);
        assert_eq!(index.clock(), mark);
        assert!(index.since(mark).deltas.is_empty());
    }

    #[test]
    fn applied_delta_contains_only_effective_mutations() {
        let mut index = index_with_sample_tree();
        let outcome = index.apply_ok(&Observation::new(vec![
            upsert("src/main.rs", EntryKind::File, file_attrs(100, 10)),
            upsert("new.txt", EntryKind::File, file_attrs(4, 4)),
            Op::Remove { path: PathBuf::from("missing.txt") },
        ]));

        assert_eq!(outcome.stats.unchanged, 2);
        let applied = outcome.applied.expect("one effective insert");
        assert_eq!(applied.len(), 1);
        assert_eq!(applied.ops[0].path(), Path::new("new.txt"));
    }

    #[test]
    fn replaying_the_same_delta_twice_changes_nothing() {
        let mut index = Index::new("/root");
        let delta = Observation::new(vec![
            upsert("a", EntryKind::Dir, Attrs::default()),
            upsert("a/f.txt", EntryKind::File, file_attrs(10, 1)),
        ]);
        index.apply_ok(&delta);
        let after_first = index.total().clone();
        let stats = index.apply_ok(&delta);

        assert_eq!(stats.unchanged, 2);
        assert_eq!(index.total(), &after_first);
        assert_eq!(index.len(), 3);
    }

    #[test]
    fn changed_size_updates_every_ancestor() {
        let mut index = index_with_sample_tree();
        index.apply_ok(&Observation::new(vec![upsert(
            "src/main.rs",
            EntryKind::File,
            file_attrs(150, 11),
        )]));

        assert_eq!(index.rollup(Path::new("src")).expect("dir").bytes, 350);
        assert_eq!(index.total().bytes, 650);
        assert_eq!(index.by_ext_named(index.total())[".rs"], ExtTally { files: 2, bytes: 350 });
    }

    #[test]
    fn allocated_size_change_updates_rollups_even_when_fingerprint_matches() {
        let mut index = Index::new("/root");
        let original = Attrs { allocated: 512, ..file_attrs(100, 10) };
        index.apply_ok(&Observation::new(vec![upsert("file.bin", EntryKind::File, original)]));

        let repacked = Attrs { allocated: 4096, ..original };
        let outcome =
            index.apply_ok(&Observation::new(vec![upsert("file.bin", EntryKind::File, repacked)]));

        assert_eq!(outcome.stats.updated, 1);
        assert_eq!(outcome.stats.unchanged, 0);
        assert_eq!(index.total().allocated, 4096);
        assert_eq!(index.attrs(Path::new("file.bin")), Some(&repacked));
    }

    #[test]
    fn newest_mtime_preserves_pre_epoch_values_through_updates_and_removals() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("newer.txt", EntryKind::File, file_attrs(10, -10)),
            upsert("older.txt", EntryKind::File, file_attrs(20, -20)),
        ]));

        assert_eq!(index.total().newest_mtime_ns, -10);

        index.apply_ok(&Observation::new(vec![upsert(
            "newer.txt",
            EntryKind::File,
            file_attrs(10, -30),
        )]));
        assert_eq!(index.total().newest_mtime_ns, -20);

        index.apply_ok(&Observation::new(vec![Op::Remove { path: PathBuf::from("older.txt") }]));
        assert_eq!(index.total().newest_mtime_ns, -30);
    }

    #[test]
    fn removing_a_file_corrects_sums_and_rebuilds_the_max() {
        let mut index = index_with_sample_tree();
        // guide.md holds the newest mtime for the whole tree.
        let stats = index
            .apply_ok(&Observation::new(vec![Op::Remove { path: PathBuf::from("docs/guide.md") }]));

        assert_eq!(stats.removed, 1);
        let total = index.total();
        assert_eq!(total.files, 2);
        assert_eq!(total.bytes, 300);
        assert_eq!(total.newest_mtime_ns, 20, "max must fall back to src/lib.rs");
        assert!(!index.by_ext_named(total).contains_key(".md"), "emptied tallies are dropped");
    }

    #[test]
    fn removing_a_directory_cascades_to_descendants() {
        let mut index = index_with_sample_tree();
        let stats = index
            .apply(&Observation::new(vec![Op::Remove { path: PathBuf::from("src") }]))
            .expect("valid observation");

        assert_eq!(stats.removed, 3, "the directory and both files");
        let total = index.total();
        assert_eq!(total.files, 1);
        assert_eq!(total.dirs, 1);
        assert_eq!(total.bytes, 300);
        assert!(index.lookup(Path::new("src/main.rs")).is_none());
        assert!(!index.by_ext_named(total).contains_key(".rs"));
    }

    #[test]
    fn freed_slots_are_reused() {
        let mut index = index_with_sample_tree();
        let before = index.len();
        index.apply_ok(&Observation::new(vec![Op::Remove { path: PathBuf::from("src") }]));
        index.apply_ok(&Observation::new(vec![
            upsert("other", EntryKind::Dir, Attrs::default()),
            upsert("other/x.rs", EntryKind::File, file_attrs(1, 1)),
            upsert("other/y.rs", EntryKind::File, file_attrs(1, 1)),
        ]));
        assert_eq!(index.len(), before, "three freed slots, three new entries");
    }

    #[test]
    fn stale_entry_handle_does_not_alias_a_reused_slot() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert(
            "first.txt",
            EntryKind::File,
            file_attrs(1, 1),
        )]));
        let stale = index.lookup(Path::new("first.txt")).expect("first id");
        index.apply_ok(&Observation::new(vec![Op::Remove { path: PathBuf::from("first.txt") }]));
        index.apply_ok(&Observation::new(vec![upsert(
            "second.txt",
            EntryKind::File,
            file_attrs(2, 2),
        )]));
        let current = index.lookup(Path::new("second.txt")).expect("second id");

        assert_ne!(stale, current, "generation participates in handle identity");
        assert!(index.attrs_of(stale).is_none());
        assert!(index.kind_of(stale).is_none());
        assert!(index.name_of(stale).is_none());
        assert!(index.path_of(stale).is_none());
        assert!(index.children_of(stale).is_none());
        assert!(index.rollup_of(stale).is_none());
        assert_eq!(index.attrs_of(current).map(|attrs| attrs.size), Some(2));
    }

    #[test]
    fn missing_ancestors_are_created_for_out_of_order_upserts() {
        let mut index = Index::new("/root");
        // A watch event can name a deep path the index has never seen.
        index.apply_ok(&Observation::new(vec![upsert(
            "deep/nested/tree/file.txt",
            EntryKind::File,
            file_attrs(42, 7),
        )]));

        assert_eq!(index.total().files, 1);
        assert_eq!(index.total().dirs, 3);
        assert_eq!(index.total().bytes, 42);
        assert_eq!(index.rollup(Path::new("deep/nested")).expect("created").files, 1);
    }

    #[test]
    fn non_directory_ancestor_is_replaced_before_attaching_a_child() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert(
            "conflict",
            EntryKind::File,
            file_attrs(9, 1),
        )]));

        let outcome = index.apply_ok(&Observation::new(vec![upsert(
            "conflict/child.txt",
            EntryKind::File,
            file_attrs(4, 2),
        )]));

        assert_eq!(index.kind(Path::new("conflict")), Some(EntryKind::Dir));
        assert!(index.lookup(Path::new("conflict/child.txt")).is_some());
        assert_eq!(index.total().files, 1);
        assert_eq!(index.total().dirs, 1);
        assert_eq!(index.total().bytes, 4);
        assert_eq!(outcome.removed, 1);
    }

    #[test]
    fn kind_change_replaces_the_entry() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert(
            "thing",
            EntryKind::File,
            file_attrs(50, 5),
        )]));
        assert_eq!(index.total().files, 1);

        index.apply_ok(&Observation::new(vec![upsert("thing", EntryKind::Dir, Attrs::default())]));
        let total = index.total();
        assert_eq!(total.files, 0);
        assert_eq!(total.dirs, 1);
        assert_eq!(total.bytes, 0);
    }

    #[test]
    fn paths_are_reconstructed_from_parent_pointers() {
        let index = index_with_sample_tree();
        let id = index.lookup(Path::new("src/main.rs")).expect("present");
        assert_eq!(index.path_of(id), Some(PathBuf::from("src/main.rs")));
        assert_eq!(index.path_of(EntryId::ROOT), Some(PathBuf::new()));
    }

    #[test]
    fn since_returns_deltas_after_a_clock() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert("a.txt", EntryKind::File, file_attrs(1, 1))]));
        let mark = index.clock();
        index.apply_ok(&Observation::new(vec![upsert("b.txt", EntryKind::File, file_attrs(2, 2))]));

        let since = index.since(mark);
        assert!(!since.truncated);
        assert_eq!(since.deltas.len(), 1);
        assert_eq!(since.deltas[0].ops[0].path(), Path::new("b.txt"));

        assert_eq!(index.since(index.clock()).deltas.len(), 0);
    }

    #[test]
    fn oversized_single_batch_is_not_retained() {
        let mut index = Index::with_journal_op_capacity("/root", 2);
        let outcome = index.apply_ok(&Observation::new(vec![
            upsert("a.txt", EntryKind::File, file_attrs(1, 1)),
            upsert("b.txt", EntryKind::File, file_attrs(2, 2)),
            upsert("c.txt", EntryKind::File, file_attrs(3, 3)),
        ]));

        assert_eq!(outcome.applied.as_ref().expect("committed").len(), 3);
        let since = index.since(Clock::ZERO);
        assert!(since.truncated);
        assert!(since.deltas.is_empty());
    }

    #[test]
    fn journal_eviction_uses_the_operation_budget() {
        let mut index = Index::with_journal_op_capacity("/root", 3);
        index.apply_ok(&Observation::new(vec![
            upsert("a.txt", EntryKind::File, file_attrs(1, 1)),
            upsert("b.txt", EntryKind::File, file_attrs(2, 2)),
        ]));
        index.apply_ok(&Observation::new(vec![
            upsert("c.txt", EntryKind::File, file_attrs(3, 3)),
            upsert("d.txt", EntryKind::File, file_attrs(4, 4)),
        ]));

        let since = index.since(Clock::ZERO);
        assert!(since.truncated);
        assert_eq!(since.deltas.len(), 1);
        assert_eq!(since.deltas[0].len(), 2);
        assert_eq!(since.deltas[0].ops[0].path(), Path::new("c.txt"));
    }

    #[test]
    fn invalidations_are_queued_for_the_scan_layer() {
        let mut index = Index::new("/root");
        let stats = index.apply_ok(&Observation::new(vec![Op::InvalidateSubtree {
            path: PathBuf::from("src"),
            reason: InvalidateReason::WatchOverflow,
        }]));

        assert_eq!(stats.invalidated, 1);
        let pending = index.take_pending_invalidations();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, PathBuf::from("src"));
        assert_eq!(pending[0].1, InvalidateReason::WatchOverflow);
        assert!(index.take_pending_invalidations().is_empty(), "drained once");
    }

    #[test]
    fn paths_escaping_the_root_are_rejected() {
        assert!(normalize(Path::new("../escape")).is_none());
        assert!(normalize(Path::new("/absolute")).is_none());
        assert_eq!(
            normalize(Path::new("./a/b")).expect("relative"),
            vec![OsString::from("a"), OsString::from("b")]
        );

        let mut index = Index::new("/root");
        let upsert_error = index
            .apply(&Observation::new(vec![upsert("../escape", EntryKind::File, file_attrs(1, 1))]))
            .expect_err("escaping upsert");
        assert!(matches!(upsert_error, crate::Error::PathEscapesRoot(_)));
        assert_eq!(index.total().files, 0);

        let invalidation_error = index
            .apply(&Observation::new(vec![Op::InvalidateSubtree {
                path: PathBuf::from("../outside"),
                reason: InvalidateReason::Requested,
            }]))
            .expect_err("escaping invalidation");
        assert!(matches!(invalidation_error, crate::Error::PathEscapesRoot(_)));
        assert!(index.take_pending_invalidations().is_empty());
        assert_eq!(index.freshness(), Freshness::Fresh);
    }

    #[cfg(unix)]
    #[test]
    fn distinct_non_utf8_names_have_distinct_identity() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let first = PathBuf::from(OsString::from_vec(vec![b'n', 0x80]));
        let second = PathBuf::from(OsString::from_vec(vec![b'n', 0x81]));
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            Op::Upsert { path: first.clone(), kind: EntryKind::File, attrs: file_attrs(10, 1) },
            Op::Upsert { path: second.clone(), kind: EntryKind::File, attrs: file_attrs(20, 2) },
        ]));

        assert_eq!(index.total().files, 2);
        assert_eq!(index.total().bytes, 30);
        assert!(index.lookup(&first).is_some());
        assert!(index.lookup(&second).is_some());
    }
}
