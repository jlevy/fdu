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
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::classify::derive_ext;
use crate::types::{
    AppliedDelta, Attrs, Clock, EntryIdentity, EntryKind, Expectation, Freshness, InvalidateReason,
    Observation, Op, PathExpectation, PathState, ScanScope,
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

/// Per-extension tally within a roll-up.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ExtTally {
    pub files: u64,
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
    /// Per-extension file and byte tallies across the subtree.
    pub by_ext: BTreeMap<String, ExtTally>,
}

impl RollUp {
    /// Fold another roll-up into this one. Commutative and associative, which is what
    /// lets the walk merge subtrees in whatever order threads finish them.
    fn merge(&mut self, other: &RollUp) {
        self.files += other.files;
        self.dirs += other.dirs;
        self.bytes += other.bytes;
        self.allocated += other.allocated;
        self.newest_mtime_ns = self.newest_mtime_ns.max(other.newest_mtime_ns);
        for (ext, tally) in &other.by_ext {
            let slot = self.by_ext.entry(ext.clone()).or_default();
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
#[derive(Debug)]
pub struct Since<'a> {
    /// Deltas applied strictly after the requested clock, oldest first.
    pub deltas: Vec<&'a AppliedDelta>,
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
    pub stats: ApplyStats,
    /// Present only when at least one effective mutation was committed.
    pub applied: Option<AppliedDelta>,
}

impl std::ops::Deref for ApplyOutcome {
    type Target = ApplyStats;

    fn deref(&self) -> &Self::Target {
        &self.stats
    }
}

/// The in-memory hierarchical index.
#[derive(Debug)]
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
    pub fn new(index: Index) -> Self {
        Self { inner: Arc::new(RwLock::new(index)) }
    }

    /// Acquire a read view of the current index state.
    pub fn read(&self) -> crate::Result<RwLockReadGuard<'_, Index>> {
        self.inner.read().map_err(|_| crate::Error::IndexLockPoisoned)
    }

    pub(crate) fn write(&self) -> crate::Result<RwLockWriteGuard<'_, Index>> {
        self.inner.write().map_err(|_| crate::Error::IndexLockPoisoned)
    }

    /// Arbitrate and apply one observation under the single-writer lock.
    pub fn apply(&self, observation: &Observation) -> crate::Result<ApplyOutcome> {
        Ok(self.write()?.apply(observation))
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
    pub fn apply(&mut self, observation: &Observation) -> ApplyOutcome {
        let mut stats = ApplyStats::default();
        let mut effective = Vec::new();
        let mut accepted = Vec::with_capacity(observation.len());
        for observed in &observation.ops {
            let op = &observed.op;
            if normalize(op.path()).is_none() {
                stats.unchanged += 1;
                accepted.push(false);
                continue;
            }
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

        self.clock = self.clock.next();
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
    pub(crate) fn apply_baseline(&mut self, observation: &Observation) -> ApplyStats {
        let outcome = self.apply(observation);
        self.establish_baseline();
        outcome.stats
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
    pub fn since(&self, clock: Clock) -> Since<'_> {
        Since {
            deltas: self.journal.iter().filter(|d| d.clock > clock).collect(),
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
            current = *self.entry(current).children.get(&part)?;
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

    /// Direct children of a directory, as `(name, id)` pairs in name order.
    pub fn children(&self, path: &Path) -> Option<Vec<(&OsStr, EntryId)>> {
        let id = self.lookup(path)?;
        let entry = self.entry(id);
        entry
            .kind
            .is_dir()
            .then(|| entry.children.iter().map(|(n, id)| (n.as_os_str(), *id)).collect())
    }

    /// Direct children of an entry id, as `(name, id)` pairs in name order.
    ///
    /// Returns `None` for a stale handle. A live non-directory returns an empty vector.
    pub fn children_of(&self, id: EntryId) -> Option<Vec<(&OsStr, EntryId)>> {
        Some(
            self.try_entry(id)?
                .children
                .iter()
                .map(|(name, child)| (name.as_os_str(), *child))
                .collect(),
        )
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
            let Some(child) = self.entry(current).children.get(part).copied() else {
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

    /// What an entry contributes to each of its ancestors.
    fn contribution(&self, id: EntryId) -> RollUp {
        let entry = self.entry(id);
        match entry.kind {
            EntryKind::Dir => {
                let mut roll = entry.rollup.clone();
                roll.dirs += 1;
                roll
            }
            EntryKind::File | EntryKind::Symlink | EntryKind::Other => {
                let mut roll = RollUp {
                    files: 1,
                    dirs: 0,
                    bytes: entry.attrs.size,
                    allocated: entry.attrs.allocated,
                    newest_mtime_ns: entry.attrs.mtime_ns,
                    by_ext: BTreeMap::new(),
                };
                if let Some(ext) = entry.name.to_str().and_then(derive_ext) {
                    roll.by_ext.insert(ext, ExtTally { files: 1, bytes: entry.attrs.size });
                }
                roll
            }
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
            let mut newest = 0i64;
            for child in self.entry(id).children.values() {
                let child_entry = self.entry(*child);
                let candidate = if child_entry.kind.is_dir() {
                    child_entry.rollup.newest_mtime_ns
                } else {
                    child_entry.attrs.mtime_ns
                };
                newest = newest.max(candidate);
            }
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
    fn ensure_dir_chain(&mut self, parts: &[OsString], stats: &mut ApplyStats) -> EntryId {
        let mut current = EntryId::ROOT;
        for part in parts {
            if let Some(existing) = self.entry(current).children.get(part).copied() {
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
                name: part.clone(),
                kind: EntryKind::Dir,
                attrs: Attrs::default(),
                children: BTreeMap::new(),
                rollup: RollUp::default(),
                revision: 0,
                children_revision: 0,
            });
            self.insert_child(current, part.clone(), child);
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
        let existing = self.entry(parent).children.get(name).copied();

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

        let id = self.alloc(Entry {
            parent: Some(parent),
            name: name.clone(),
            kind,
            attrs,
            children: BTreeMap::new(),
            rollup: RollUp::default(),
            revision: 0,
            children_revision: 0,
        });
        self.insert_child(parent, name.clone(), id);
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

/// Split a relative path into its normal components, rejecting anything that escapes.
///
/// Returns `None` for paths containing `..`, a root, or a prefix — an index keyed by
/// relative path has no way to represent those, and silently normalizing them away would
/// let a delta write outside the tree it claims to describe.
fn normalize(path: &Path) -> Option<Vec<OsString>> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_os_string()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(parts)
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
    fn delayed_conditional_observation_cannot_overwrite_newer_state() {
        let mut index = Index::new("/root");
        index.apply(&Observation::new(vec![upsert(
            "file.txt",
            EntryKind::File,
            file_attrs(10, 1),
        )]));

        let baseline = index.expectation(Path::new("file.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("file.txt", EntryKind::File, file_attrs(20, 2)),
            baseline,
        )]);

        index.apply(&Observation::new(vec![upsert(
            "file.txt",
            EntryKind::File,
            file_attrs(30, 3),
        )]));
        let outcome = index.apply(&delayed);

        assert_eq!(outcome.stats.stale, 1);
        assert!(outcome.applied.is_none());
        assert_eq!(index.attrs(Path::new("file.txt")).expect("file").size, 30);
    }

    #[test]
    fn delayed_absent_child_cannot_replace_a_newer_parent_file() {
        let mut index = Index::new("/root");
        index.apply(&Observation::new(vec![upsert("parent", EntryKind::Dir, file_attrs(0, 1))]));
        let child_baseline = index.expectation(Path::new("parent/child.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("parent/child.txt", EntryKind::File, file_attrs(10, 2)),
            child_baseline,
        )]);

        index.apply(&Observation::new(vec![upsert("parent", EntryKind::File, file_attrs(20, 3))]));
        let outcome = index.apply(&delayed);

        assert_eq!(outcome.stats.stale, 1);
        assert!(outcome.applied.is_none());
        assert_eq!(index.kind(Path::new("parent")), Some(EntryKind::File));
        assert!(index.lookup(Path::new("parent/child.txt")).is_none());
    }

    #[test]
    fn conditional_observation_rejects_present_state_aba() {
        let mut index = Index::new("/root");
        index.apply(&Observation::new(vec![upsert(
            "file.txt",
            EntryKind::File,
            file_attrs(10, 1),
        )]));
        let baseline = index.expectation(Path::new("file.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("file.txt", EntryKind::File, file_attrs(20, 2)),
            baseline,
        )]);

        index.apply(&Observation::new(vec![upsert(
            "file.txt",
            EntryKind::File,
            file_attrs(30, 3),
        )]));
        index.apply(&Observation::new(vec![upsert(
            "file.txt",
            EntryKind::File,
            file_attrs(10, 1),
        )]));
        let outcome = index.apply(&delayed);

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

        index.apply(&Observation::new(vec![upsert(
            "file.txt",
            EntryKind::File,
            file_attrs(30, 3),
        )]));
        index.apply(&Observation::new(vec![Op::Remove { path: PathBuf::from("file.txt") }]));
        let outcome = index.apply(&delayed);

        assert_eq!(outcome.stats.stale, 1);
        assert!(outcome.applied.is_none());
        assert!(index.lookup(Path::new("file.txt")).is_none());
    }

    #[test]
    fn unrelated_mutation_does_not_stale_an_absent_path() {
        let mut index = Index::new("/root");
        index.apply(&Observation::new(vec![upsert("dir", EntryKind::Dir, file_attrs(0, 1))]));
        let baseline = index.expectation(Path::new("dir/new.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("dir/new.txt", EntryKind::File, file_attrs(20, 2)),
            baseline,
        )]);

        index.apply(&Observation::new(vec![upsert(
            "other.txt",
            EntryKind::File,
            file_attrs(30, 3),
        )]));
        let outcome = index.apply(&delayed);

        assert_eq!(outcome.stats.stale, 0);
        assert_eq!(outcome.stats.inserted, 1);
        assert_eq!(index.attrs(Path::new("dir/new.txt")).expect("file").size, 20);
    }

    #[test]
    fn directory_metadata_change_does_not_stale_an_absent_child() {
        let mut index = Index::new("/root");
        index.apply(&Observation::new(vec![upsert("dir", EntryKind::Dir, file_attrs(0, 1))]));
        let baseline = index.expectation(Path::new("dir/new.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("dir/new.txt", EntryKind::File, file_attrs(20, 2)),
            baseline,
        )]);

        index.apply(&Observation::new(vec![upsert("dir", EntryKind::Dir, file_attrs(0, 3))]));
        let outcome = index.apply(&delayed);

        assert_eq!(outcome.stats.stale, 0);
        assert_eq!(outcome.stats.inserted, 1);
    }

    #[test]
    fn file_parent_metadata_change_stales_an_absent_child() {
        let mut index = Index::new("/root");
        index.apply(&Observation::new(vec![upsert("parent", EntryKind::File, file_attrs(10, 1))]));
        let baseline = index.expectation(Path::new("parent/child.txt"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            upsert("parent/child.txt", EntryKind::File, file_attrs(20, 2)),
            baseline,
        )]);

        index.apply(&Observation::new(vec![upsert("parent", EntryKind::File, file_attrs(30, 3))]));
        let outcome = index.apply(&delayed);

        assert_eq!(outcome.stats.stale, 1);
        assert_eq!(index.kind(Path::new("parent")), Some(EntryKind::File));
        assert!(index.lookup(Path::new("parent/child.txt")).is_none());
    }

    #[test]
    fn delayed_directory_remove_cannot_delete_a_newer_child() {
        let mut index = Index::new("/root");
        index.apply(&Observation::new(vec![upsert("dir", EntryKind::Dir, file_attrs(0, 1))]));
        let baseline = index.expectation(Path::new("dir"));
        let delayed = Observation::from_ops(vec![ObservationOp::if_state(
            Op::Remove { path: PathBuf::from("dir") },
            baseline,
        )]);

        index.apply(&Observation::new(vec![upsert(
            "dir/new.txt",
            EntryKind::File,
            file_attrs(20, 2),
        )]));
        let outcome = index.apply(&delayed);

        assert_eq!(outcome.stats.stale, 1);
        assert!(index.lookup(Path::new("dir/new.txt")).is_some());
    }

    #[test]
    fn conditional_batch_is_validated_at_one_boundary() {
        let mut index = Index::new("/root");
        let first = index.expectation(Path::new("first.txt"));
        let second = index.expectation(Path::new("second.txt"));

        let outcome = index.apply(&Observation::from_ops(vec![
            ObservationOp::if_state(upsert("first.txt", EntryKind::File, file_attrs(10, 1)), first),
            ObservationOp::if_state(
                upsert("second.txt", EntryKind::File, file_attrs(20, 2)),
                second,
            ),
        ]));

        assert_eq!(outcome.stats.inserted, 2);
        assert_eq!(outcome.stats.stale, 0);
    }

    fn index_with_sample_tree() -> Index {
        let mut index = Index::new("/root");
        index.apply(&Observation::new(vec![
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
    fn per_extension_tallies_roll_up_hierarchically() {
        let index = index_with_sample_tree();

        let total = index.total();
        assert_eq!(total.by_ext[".rs"], ExtTally { files: 2, bytes: 300 });
        assert_eq!(total.by_ext[".md"], ExtTally { files: 1, bytes: 300 });

        // Per-directory breakdown, which no surveyed tool provides.
        let src = index.rollup(Path::new("src")).expect("src is a directory");
        assert_eq!(src.by_ext[".rs"], ExtTally { files: 2, bytes: 300 });
        assert!(!src.by_ext.contains_key(".md"));
    }

    #[test]
    fn upsert_with_matching_fingerprint_is_a_no_op() {
        let mut index = index_with_sample_tree();
        let before = index.total().clone();
        let mark = index.clock();

        let stats = index.apply(&Observation::new(vec![upsert(
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
        let outcome = index.apply(&Observation::new(vec![
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
        index.apply(&delta);
        let after_first = index.total().clone();
        let stats = index.apply(&delta);

        assert_eq!(stats.unchanged, 2);
        assert_eq!(index.total(), &after_first);
        assert_eq!(index.len(), 3);
    }

    #[test]
    fn changed_size_updates_every_ancestor() {
        let mut index = index_with_sample_tree();
        index.apply(&Observation::new(vec![upsert(
            "src/main.rs",
            EntryKind::File,
            file_attrs(150, 11),
        )]));

        assert_eq!(index.rollup(Path::new("src")).expect("dir").bytes, 350);
        assert_eq!(index.total().bytes, 650);
        assert_eq!(index.total().by_ext[".rs"], ExtTally { files: 2, bytes: 350 });
    }

    #[test]
    fn allocated_size_change_updates_rollups_even_when_fingerprint_matches() {
        let mut index = Index::new("/root");
        let original = Attrs { allocated: 512, ..file_attrs(100, 10) };
        index.apply(&Observation::new(vec![upsert("file.bin", EntryKind::File, original)]));

        let repacked = Attrs { allocated: 4096, ..original };
        let outcome =
            index.apply(&Observation::new(vec![upsert("file.bin", EntryKind::File, repacked)]));

        assert_eq!(outcome.stats.updated, 1);
        assert_eq!(outcome.stats.unchanged, 0);
        assert_eq!(index.total().allocated, 4096);
        assert_eq!(index.attrs(Path::new("file.bin")), Some(&repacked));
    }

    #[test]
    fn removing_a_file_corrects_sums_and_rebuilds_the_max() {
        let mut index = index_with_sample_tree();
        // guide.md holds the newest mtime for the whole tree.
        let stats = index
            .apply(&Observation::new(vec![Op::Remove { path: PathBuf::from("docs/guide.md") }]));

        assert_eq!(stats.removed, 1);
        let total = index.total();
        assert_eq!(total.files, 2);
        assert_eq!(total.bytes, 300);
        assert_eq!(total.newest_mtime_ns, 20, "max must fall back to src/lib.rs");
        assert!(!total.by_ext.contains_key(".md"), "emptied tallies are dropped");
    }

    #[test]
    fn removing_a_directory_cascades_to_descendants() {
        let mut index = index_with_sample_tree();
        let stats = index.apply(&Observation::new(vec![Op::Remove { path: PathBuf::from("src") }]));

        assert_eq!(stats.removed, 3, "the directory and both files");
        let total = index.total();
        assert_eq!(total.files, 1);
        assert_eq!(total.dirs, 1);
        assert_eq!(total.bytes, 300);
        assert!(index.lookup(Path::new("src/main.rs")).is_none());
        assert!(!total.by_ext.contains_key(".rs"));
    }

    #[test]
    fn freed_slots_are_reused() {
        let mut index = index_with_sample_tree();
        let before = index.len();
        index.apply(&Observation::new(vec![Op::Remove { path: PathBuf::from("src") }]));
        index.apply(&Observation::new(vec![
            upsert("other", EntryKind::Dir, Attrs::default()),
            upsert("other/x.rs", EntryKind::File, file_attrs(1, 1)),
            upsert("other/y.rs", EntryKind::File, file_attrs(1, 1)),
        ]));
        assert_eq!(index.len(), before, "three freed slots, three new entries");
    }

    #[test]
    fn stale_entry_handle_does_not_alias_a_reused_slot() {
        let mut index = Index::new("/root");
        index.apply(&Observation::new(vec![upsert(
            "first.txt",
            EntryKind::File,
            file_attrs(1, 1),
        )]));
        let stale = index.lookup(Path::new("first.txt")).expect("first id");
        index.apply(&Observation::new(vec![Op::Remove { path: PathBuf::from("first.txt") }]));
        index.apply(&Observation::new(vec![upsert(
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
        index.apply(&Observation::new(vec![upsert(
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
        index.apply(&Observation::new(vec![upsert("conflict", EntryKind::File, file_attrs(9, 1))]));

        let outcome = index.apply(&Observation::new(vec![upsert(
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
        index.apply(&Observation::new(vec![upsert("thing", EntryKind::File, file_attrs(50, 5))]));
        assert_eq!(index.total().files, 1);

        index.apply(&Observation::new(vec![upsert("thing", EntryKind::Dir, Attrs::default())]));
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
        index.apply(&Observation::new(vec![upsert("a.txt", EntryKind::File, file_attrs(1, 1))]));
        let mark = index.clock();
        index.apply(&Observation::new(vec![upsert("b.txt", EntryKind::File, file_attrs(2, 2))]));

        let since = index.since(mark);
        assert!(!since.truncated);
        assert_eq!(since.deltas.len(), 1);
        assert_eq!(since.deltas[0].ops[0].path(), Path::new("b.txt"));

        assert_eq!(index.since(index.clock()).deltas.len(), 0);
    }

    #[test]
    fn oversized_single_batch_is_not_retained() {
        let mut index = Index::with_journal_op_capacity("/root", 2);
        let outcome = index.apply(&Observation::new(vec![
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
        index.apply(&Observation::new(vec![
            upsert("a.txt", EntryKind::File, file_attrs(1, 1)),
            upsert("b.txt", EntryKind::File, file_attrs(2, 2)),
        ]));
        index.apply(&Observation::new(vec![
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
        let stats = index.apply(&Observation::new(vec![Op::InvalidateSubtree {
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
        index.apply(&Observation::new(vec![upsert(
            "../escape",
            EntryKind::File,
            file_attrs(1, 1),
        )]));
        assert_eq!(index.total().files, 0, "escaping upserts are dropped");

        index.apply(&Observation::new(vec![Op::InvalidateSubtree {
            path: PathBuf::from("../outside"),
            reason: InvalidateReason::Requested,
        }]));
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
        index.apply(&Observation::new(vec![
            Op::Upsert { path: first.clone(), kind: EntryKind::File, attrs: file_attrs(10, 1) },
            Op::Upsert { path: second.clone(), kind: EntryKind::File, attrs: file_attrs(20, 2) },
        ]));

        assert_eq!(index.total().files, 2);
        assert_eq!(index.total().bytes, 30);
        assert!(index.lookup(&first).is_some());
        assert!(index.lookup(&second).is_some());
    }
}
