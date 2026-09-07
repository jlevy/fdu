//! The in-memory hierarchical index.
//!
//! The index is a parent-pointer tree in a flat arena. Entries store their **name only**
//! and paths are reconstructed by walking parents, so a path like
//! `srv/data/project/src/lib/utils.rs` costs six name strings across six entries with no
//! duplication — the fsearch/ncdu layout, deliberately not dut's full-path-per-entry.
//!
//! Every directory carries pre-computed roll-up state for its whole subtree, so a query
//! reads a field and never traverses. Applying an [`Observation`] re-merges that state up the
//! ancestor chain only. Producers submit observations; only effective, arbitrated fact
//! or state changes become exact clocked commits.
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
//! applying commits behind a `RwLock` with readers taking the read side: writes are short
//! (O(depth) applies) and reads are field lookups rather than queries that walk. The
//! delta contract being the only mutation path means escalating later to epoch or
//! arc-swap snapshots stays contained rather than becoming a rewrite.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::content::{
    AnalysisApplyOutcome, AnalysisCandidate, AnalysisObservation, AnalysisSet, ContentIndex,
    ContentRollUp,
};
use crate::engine_contract::{
    AppliedDelta, Attrs, Clock, Commit, Coverage, CoverageReason, DiscoveryProgress,
    EffectiveChange, EntryIdentity, EntryKind, Expectation, Freshness, Impact, ImpactDomain,
    IndexState, InvalidateReason, Issue, LifecyclePhase, MAX_DIRTY_PATHS, MAX_RETAINED_ISSUES,
    Observation, ObservationOp, Op, PathExpectation, PathState, Provenance, ScanScope, Source,
    StateTransition, Status, Work,
};

/// Verification intervals kept before the oldest are dropped.
///
/// Bounds the memory a long-lived session can accumulate through repeated scoped
/// reconciliation. Dropping an interval only ever moves a path back to reporting
/// `Cached`, so the bound costs precision, never correctness.
const MAX_VERIFIED_INTERVALS: usize = 256;

/// Maximum retained-cost units in the exact commit history used by [`Index::since`].
///
/// Bounded on purpose: an unbounded journal is a memory leak in a long-lived server. A
/// consumer that falls further behind than this is told so ([`Since::truncated`]) and is
/// expected to re-read state rather than silently miss changes.
pub const DEFAULT_JOURNAL_CAPACITY: usize = 64 * 1024;

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

/// Index-private extension identity.
type ExtId = u32;

/// Per-extension tally within a roll-up.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ExtTally {
    /// Files with this extension.
    pub files: u64,
    /// Apparent bytes across those files.
    pub bytes: u64,
    /// Allocated bytes across those files.
    ///
    /// Carried alongside `bytes` so a per-type report can answer in either metric. A
    /// tally that tracked only apparent size would force a report asked for allocated
    /// bytes to either switch metrics silently or drop the breakdown.
    pub allocated: u64,
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

/// The two fixed aggregate partitions maintained for inventory reads.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PartitionRollUp {
    /// Every retained descendant.
    pub all: RollUp,
    /// Retained descendants outside the effective ignored partition.
    pub unignored: RollUp,
}

/// Constant-size directory totals suitable for bounded interactive rows.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct RollUpSummary {
    /// Descendant regular files.
    pub files: u64,
    /// Descendant directories, excluding the directory that owns this summary.
    pub dirs: u64,
    /// Apparent bytes across descendant regular files.
    pub bytes: u64,
    /// Allocated bytes across descendant regular files.
    pub allocated: u64,
    /// Newest descendant-file modification time, or `None` for an empty subtree.
    pub newest_mtime_ns: Option<i64>,
}

/// Constant-size totals for the fixed all and unignored partitions.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PartitionRollUpSummary {
    /// Every retained descendant.
    pub all: RollUpSummary,
    /// Retained descendants outside the effective ignored partition.
    pub unignored: RollUpSummary,
}

/// Hot-path aggregate state owned by one index.
///
/// Integer extension keys make ancestor merges cheap, but they are meaningful only
/// while held by the index that issued them. Public query methods convert this into a
/// self-describing [`RollUp`] so a retained result cannot be relabelled when an interner
/// slot is reused.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct InternedRollUp {
    files: u64,
    dirs: u64,
    bytes: u64,
    allocated: u64,
    newest_mtime_ns: i64,
    by_ext: BTreeMap<ExtId, ExtTally>,
}

/// Hot-path form of the fixed `all` and `unignored` partitions.
///
/// Dereferencing yields `all`, keeping existing unrestricted query code direct while
/// mutation helpers update both partitions explicitly.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct InternedPartitionRollUp {
    all: InternedRollUp,
    unignored: InternedRollUp,
}

impl std::ops::Deref for InternedPartitionRollUp {
    type Target = InternedRollUp;

    fn deref(&self) -> &Self::Target {
        &self.all
    }
}

impl std::ops::DerefMut for InternedPartitionRollUp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.all
    }
}

impl InternedPartitionRollUp {
    fn merge(&mut self, other: &Self) {
        self.all.merge(&other.all);
        self.unignored.merge(&other.unignored);
    }

    fn unmerge(&mut self, other: &Self) {
        self.all.unmerge(&other.all);
        self.unignored.unmerge(&other.unignored);
    }
}

fn rollup_summary(rollup: &InternedRollUp) -> RollUpSummary {
    RollUpSummary {
        files: rollup.files,
        dirs: rollup.dirs,
        bytes: rollup.bytes,
        allocated: rollup.allocated,
        newest_mtime_ns: (rollup.files > 0).then_some(rollup.newest_mtime_ns),
    }
}

fn partition_summary(rollup: &InternedPartitionRollUp) -> PartitionRollUpSummary {
    PartitionRollUpSummary {
        all: rollup_summary(&rollup.all),
        unignored: rollup_summary(&rollup.unignored),
    }
}

/// Map-free roll-up fields for internal reports that do not need extension names.
///
/// Keeping this view separate avoids cloning every extension string for summary and
/// tree queries while the public [`RollUp`] remains safe to retain independently.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct RollUpScalars {
    pub(crate) files: u64,
    pub(crate) dirs: u64,
    pub(crate) bytes: u64,
    pub(crate) allocated: u64,
    pub(crate) newest_mtime_ns: i64,
}

impl From<&InternedRollUp> for RollUpScalars {
    fn from(rollup: &InternedRollUp) -> Self {
        Self {
            files: rollup.files,
            dirs: rollup.dirs,
            bytes: rollup.bytes,
            allocated: rollup.allocated,
            newest_mtime_ns: rollup.newest_mtime_ns,
        }
    }
}

impl InternedRollUp {
    /// Fold another roll-up into this one. Commutative and associative, which is what
    /// lets the walk merge subtrees in whatever order threads finish them.
    fn merge(&mut self, other: &InternedRollUp) {
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
            slot.allocated += tally.allocated;
        }
    }

    /// Remove another roll-up's contribution from this one.
    ///
    /// Only the invertible reducers are corrected here. `newest_mtime_ns` is left stale
    /// on purpose and repaired by [`Index::recompute_newest_upward`], because a max
    /// cannot be un-merged without knowing what else contributed it.
    fn unmerge(&mut self, other: &InternedRollUp) {
        self.files = self.files.saturating_sub(other.files);
        self.dirs = self.dirs.saturating_sub(other.dirs);
        self.bytes = self.bytes.saturating_sub(other.bytes);
        self.allocated = self.allocated.saturating_sub(other.allocated);
        for (ext, tally) in &other.by_ext {
            if let Some(slot) = self.by_ext.get_mut(ext) {
                slot.files = slot.files.saturating_sub(tally.files);
                slot.bytes = slot.bytes.saturating_sub(tally.bytes);
                slot.allocated = slot.allocated.saturating_sub(tally.allocated);
                if slot.files == 0 && slot.bytes == 0 && slot.allocated == 0 {
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
    /// Effective fixed-control classification, including an ignored ancestor.
    ignored: bool,
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
    rollup: InternedPartitionRollUp,
    /// Changes on direct metadata updates. Together with the arena generation this
    /// detects present-state ABA races.
    revision: u64,
    /// Changes only on direct child-map mutations. This is the narrow structural guard
    /// for absent paths and destructive subtree operations.
    children_revision: u64,
    /// Whether every in-scope child of this directory has been enumerated.
    ///
    /// Meaningful only for directories. Keeping the bit on the entry makes absence a
    /// local fact instead of forcing readers to reconstruct the discovery frontier.
    children_complete: bool,
}

/// Portable direct children retained in the order interactive tree pages emit them.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct PortableChildren {
    pub(crate) directories: BTreeMap<String, EntryId>,
    pub(crate) nondirectories: BTreeMap<String, EntryId>,
}

/// Commit-maintained orders and diagnostics used only while serving an opened root.
///
/// A detached [`Index`] is the storage and one-shot execution shape used by the CLI,
/// snapshots, and ordinary library callers. Keeping these maps behind one optional
/// allocation makes interactive reads additive without charging those paths one copied
/// portable string and child-map node per entry.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct ServingIndexes {
    portable_children: BTreeMap<PathBuf, PortableChildren>,
    portable_entries: BTreeMap<crate::PortablePath, EntryId>,
    recent_files: BTreeSet<RecentKey>,
    semantic_names: Vec<Option<String>>,
    semantic_ids: BTreeMap<String, u32>,
    semantic_refcounts: Vec<u64>,
    free_semantic_ids: Vec<u32>,
    semantic_by_directory: BTreeMap<EntryId, InternedSemanticPartitions>,
    exact_name_ids: BTreeMap<String, u32>,
    exact_names: Vec<String>,
    exact_name_by_directory: BTreeMap<EntryId, InternedSemanticPartitions>,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct InternedSemanticPartitions {
    all: BTreeMap<u32, ExtTally>,
    unignored: BTreeMap<u32, ExtTally>,
}

/// One regular file in global newest-first order.
#[derive(Clone, PartialEq, Eq, Debug)]
struct RecentKey {
    mtime_ns: i64,
    portable_path: crate::PortablePath,
    id: EntryId,
}

impl PartialOrd for RecentKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RecentKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .mtime_ns
            .cmp(&self.mtime_ns)
            .then_with(|| self.portable_path.cmp(&other.portable_path))
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl ServingIndexes {
    fn for_types(types: &crate::classify::TypeRegistry) -> Self {
        let exact_names: Vec<_> = types
            .exact_filenames()
            .map(str::to_ascii_lowercase)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let exact_name_ids = exact_names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let id = u32::try_from(index)
                    .expect("a registry declares fewer than four billion exact filenames");
                (name.clone(), id)
            })
            .collect();
        Self { exact_name_ids, exact_names, ..Self::default() }
    }

    fn exact_name_id(&self, name: &OsStr) -> Option<u32> {
        let name = name.to_str()?;
        if let Some(id) = self.exact_name_ids.get(name) {
            return Some(*id);
        }
        name.bytes()
            .any(|byte| byte.is_ascii_uppercase())
            .then(|| name.to_ascii_lowercase())
            .and_then(|name| self.exact_name_ids.get(&name).copied())
    }

    fn intern_semantic(&mut self, name: &str) -> u32 {
        if let Some(id) = self.semantic_ids.get(name).copied() {
            let refcount = self
                .semantic_refcounts
                .get_mut(id as usize)
                .expect("a live semantic id has a refcount");
            *refcount = refcount.checked_add(1).expect("semantic refcount exhausted");
            return id;
        }
        let id = if let Some(id) = self.free_semantic_ids.pop() {
            self.semantic_names[id as usize] = Some(name.to_string());
            self.semantic_refcounts[id as usize] = 1;
            id
        } else {
            let id = u32::try_from(self.semantic_names.len())
                .expect("fewer than four billion semantic types are live");
            self.semantic_names.push(Some(name.to_string()));
            self.semantic_refcounts.push(1);
            id
        };
        self.semantic_ids.insert(name.to_string(), id);
        id
    }

    fn release_semantic(&mut self, id: u32, count: u64) {
        let slot = self
            .semantic_refcounts
            .get_mut(id as usize)
            .expect("a live semantic id has a refcount");
        *slot = slot.checked_sub(count).expect("semantic reference released twice");
        if *slot != 0 {
            return;
        }
        let name =
            self.semantic_names[id as usize].take().expect("a referenced semantic id has a name");
        let removed = self.semantic_ids.remove(&name);
        debug_assert_eq!(removed, Some(id), "the semantic interner's two maps disagreed");
        self.free_semantic_ids.push(id);
    }
}

fn merge_semantic(map: &mut BTreeMap<u32, ExtTally>, id: u32, attrs: Attrs) {
    let tally = map.entry(id).or_default();
    tally.files = tally.files.saturating_add(1);
    tally.bytes = tally.bytes.saturating_add(attrs.size);
    tally.allocated = tally.allocated.saturating_add(attrs.allocated);
}

fn unmerge_semantic(map: &mut BTreeMap<u32, ExtTally>, id: u32, attrs: Attrs) {
    let tally = map.get_mut(&id).expect("a semantic contribution must exist before removal");
    tally.files = tally.files.saturating_sub(1);
    tally.bytes = tally.bytes.saturating_sub(attrs.size);
    tally.allocated = tally.allocated.saturating_sub(attrs.allocated);
    if tally.files == 0 && tally.bytes == 0 && tally.allocated == 0 {
        map.remove(&id);
    }
}

fn unmerge_semantic_map(
    destination: &mut BTreeMap<u32, ExtTally>,
    contribution: &BTreeMap<u32, ExtTally>,
) {
    for (id, removed) in contribution {
        let tally = destination
            .get_mut(id)
            .expect("a semantic subtree contribution must exist before removal");
        tally.files = tally.files.saturating_sub(removed.files);
        tally.bytes = tally.bytes.saturating_sub(removed.bytes);
        tally.allocated = tally.allocated.saturating_sub(removed.allocated);
        if tally.files == 0 && tally.bytes == 0 && tally.allocated == 0 {
            destination.remove(id);
        }
    }
}

#[derive(Clone, Debug)]
enum Slot {
    Occupied { generation: u64, entry: Box<Entry> },
    Free { generation: u64, next_free: Option<u32> },
}

fn retained_parent(arena: &[Slot], id: EntryId) -> Option<EntryId> {
    match arena.get(id.idx()) {
        Some(Slot::Occupied { generation, entry }) if *generation == id.generation => entry.parent,
        Some(Slot::Occupied { .. } | Slot::Free { .. }) | None => {
            panic!("internal entry handle must be live: {id:?}")
        }
    }
}

/// Result of [`Index::since`].
#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[must_use]
pub struct Since {
    /// Exact commits applied strictly after the requested clock, oldest first.
    pub commits: Vec<Commit>,
    /// Deltas applied strictly after the requested clock, oldest first.
    ///
    /// This is derived from `commits` for compatibility and excludes state-only commits.
    pub deltas: Vec<AppliedDelta>,
    /// Terminal clock captured under the same read boundary as `commits`.
    pub clock: Clock,
    /// Complete public state at `clock`.
    pub state: IndexState,
    /// True when the requested clock is older than the retained journal, meaning the
    /// caller has missed commits and must re-read state rather than trust either view.
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
    /// Exact control sources inserted, replaced, or removed.
    pub controls: u64,
    /// Retained entries moved between ignored and unignored partitions.
    pub reclassified: u64,
    /// Conditional observations rejected because the indexed state changed after the
    /// producer captured its baseline.
    pub stale: u64,
    /// File upserts refused because their exact effect would exceed an opened-root
    /// resource budget.
    pub resource_refused: u64,
}

impl ApplyStats {
    /// True when any operation changed indexed state.
    pub const fn mutated(&self) -> bool {
        self.inserted > 0
            || self.updated > 0
            || self.removed > 0
            || self.invalidated > 0
            || self.controls > 0
            || self.reclassified > 0
    }
}

/// Result of arbitrating and applying one producer observation.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ApplyOutcome {
    /// Per-operation arbitration and mutation counts.
    pub stats: ApplyStats,
    /// Present only when at least one exact fact or state transition was committed.
    pub commit: Option<Commit>,
    /// Legacy entry-operation projection derived from `commit`.
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
    /// Effective fixed-control classification.
    pub ignored: bool,
    /// Pre-computed subtree totals for a directory.
    pub rollup: Option<RollUp>,
    /// Both maintained aggregate partitions for a directory.
    pub partitions: Option<PartitionRollUp>,
}

impl std::ops::Deref for ApplyOutcome {
    type Target = ApplyStats;

    fn deref(&self) -> &Self::Target {
        &self.stats
    }
}

impl ApplyOutcome {
    fn from_commit(stats: ApplyStats, commit: Option<Commit>) -> Self {
        let applied = commit.as_ref().and_then(Commit::applied_delta);
        Self { stats, commit, applied }
    }

    /// Borrow the legacy entry-operation projection derived from the exact commit.
    ///
    /// Existing callers may continue to read the public [`Self::applied`] field. This
    /// method is useful when code wants to make the derivation explicit.
    pub fn applied(&self) -> Option<&AppliedDelta> {
        self.applied.as_ref()
    }
}

/// Validated, canonical producer input ready for arbitration under the write guard.
#[derive(Clone, Debug)]
struct PreparedObservation {
    ops: Vec<ObservationOp>,
    #[cfg(test)]
    reject_before_apply: bool,
}

#[derive(Default)]
struct MutationEffects {
    changes: Vec<EffectiveChange>,
    state: Vec<StateTransition>,
}

impl MutationEffects {
    fn is_empty(&self) -> bool {
        self.changes.is_empty() && self.state.is_empty()
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
    journal: VecDeque<Commit>,
    journal_cost: usize,
    journal_capacity: usize,
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
    /// Subtrees a completed reconciliation has verified, with when it finished.
    ///
    /// Kept as intervals rather than per-entry flags because a sweep verifies
    /// everything beneath a path at once, including entries the producer elided as
    /// no-ops, and because one record per sweep costs nothing against millions of
    /// entries. Nested and repeated sweeps collapse: a new record replaces any it
    /// covers.
    verified: Vec<(PathBuf, i64)>,
    /// Interner storage: id → live name. Ids are indexes into this vector, and a
    /// vacant slot holds `None` until it is reissued.
    ext_names: Vec<Option<String>>,
    /// Interner lookup: name → id.
    ext_ids: BTreeMap<String, ExtId>,
    /// Live file entries holding each extension id, parallel to `ext_names`.
    ///
    /// Interning without a matching release is a leak in the case this engine is built
    /// for: a watched tree that churns through editor temporaries, build outputs, and
    /// content-hashed asset names keeps minting extensions the tree no longer contains,
    /// and both maps grow for the life of the process.
    ext_refcounts: Vec<u64>,
    /// Slots whose last referencing file went away, available for reissue.
    free_ext_ids: Vec<ExtId>,
    /// Sparse derived-data tier, allocated only after analysis is enabled.
    content: Option<Box<ContentIndex>>,
    /// File-type rules this index classifies against.
    ///
    /// Held rather than reached for globally, because a caller may run two indexes under
    /// different taxonomies in one process. It must agree with `scope`'s type-rule
    /// fingerprint: an index that classified under one set of rules while claiming
    /// another would serve a snapshot that is wrong in a way nothing checks.
    types: std::sync::Arc<crate::classify::TypeRegistry>,
    /// Exact fixed control sources and their derived matchers.
    controls: crate::control::ControlTable,
    freshness_epoch: u64,
    freshness_marks: BTreeMap<PathBuf, FreshnessMark>,
    /// Coherent opened-root state. Detached indexes retain the settled default and do
    /// not acquire live identity or worker ownership by carrying this value.
    state: IndexState,
    /// Bounded diagnostic details summarized by `state.issues`.
    issues: Vec<Issue>,
    /// Optional commit-maintained state for interactive opened-root projections.
    ///
    /// Detached indexes deliberately carry `None`, including the standalone CLI's
    /// one-shot scan. Only [`crate::OpenedIndex`] enables this allocation.
    serving: Option<Box<ServingIndexes>>,
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

/// Index-owned part of one progressive discovery commit.
///
/// The producer may combine one of these with entry observations; the opened commit
/// policy updates exact file progress and publishes one atomic fact-and-state commit.
#[derive(Clone, Debug, Default)]
pub(crate) struct DiscoveryCommit {
    pub(crate) directory_complete: Option<PathBuf>,
    pub(crate) transition: Option<DiscoveryTransition>,
}

#[derive(Clone, Debug)]
pub(crate) enum DiscoveryTransition {
    Begin,
    Finish,
    BudgetRefused(Issue),
    Inaccessible { issues: Vec<Issue>, omitted: u64 },
    Cancelled,
    Failed(Issue),
}

/// Index-owned lifecycle transitions for the optional observation producer.
#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "watch"), allow(dead_code))]
pub(crate) enum ObservationTransition {
    /// Baseline discovery finished and the observer is closing its registration gap.
    Reconciling,
    /// The observer is active and its baseline handoff has been verified.
    ///
    /// Persistent inaccessible boundaries do not prevent observation of the readable
    /// scope, but they keep coverage partial and their causes remain inspectable.
    Watching { issues: Vec<Issue>, omitted: u64 },
    /// Observation could not establish or retain a trustworthy live boundary.
    Failed(Issue),
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

    /// Evaluate one owned result while holding exactly one coherent read boundary.
    pub(crate) fn read_with<T>(&self, read: impl FnOnce(&Index) -> T) -> crate::Result<T> {
        let index = self.read_index()?;
        Ok(read(&index))
    }

    #[cfg(test)]
    pub(crate) fn poison_for_test(&self) {
        let inner = std::sync::Arc::clone(&self.inner);
        std::thread::spawn(move || {
            let _guard = inner.write().expect("test index write lock");
            panic!("inject index poison");
        })
        .join()
        .expect_err("injected index panic");
    }

    /// Arbitrate and apply one observation under the single-writer lock.
    pub fn apply(&self, observation: &Observation) -> crate::Result<ApplyOutcome> {
        let prepared = prepare_observation(observation)?;
        self.write_index()?.commit_prepared(prepared, true)
    }

    pub(crate) fn apply_discovery(
        &self,
        observation: &Observation,
        discovery: DiscoveryCommit,
    ) -> crate::Result<ApplyOutcome> {
        let prepared = prepare_observation(observation)?;
        self.write_index()?.commit_prepared_with(prepared, true, Some(discovery), None, None, true)
    }

    pub(crate) fn apply_opened(
        &self,
        observation: &Observation,
        max_files: Option<u64>,
    ) -> crate::Result<ApplyOutcome> {
        let prepared = prepare_observation(observation)?;
        self.write_index()?.commit_prepared_with(prepared, true, None, None, max_files, true)
    }

    pub(crate) fn apply_discovery_bounded(
        &self,
        observation: &Observation,
        discovery: DiscoveryCommit,
        max_files: Option<u64>,
    ) -> crate::Result<ApplyOutcome> {
        let prepared = prepare_observation(observation)?;
        self.write_index()?.commit_prepared_with(
            prepared,
            true,
            Some(discovery),
            None,
            max_files,
            true,
        )
    }

    pub(crate) fn transition_discovery(
        &self,
        transition: DiscoveryTransition,
    ) -> crate::Result<ApplyOutcome> {
        self.apply_discovery(
            &Observation::new(Vec::new()),
            DiscoveryCommit { directory_complete: None, transition: Some(transition) },
        )
    }

    #[cfg(feature = "watch")]
    pub(crate) fn transition_observation(
        &self,
        transition: ObservationTransition,
    ) -> crate::Result<ApplyOutcome> {
        let prepared = prepare_observation(&Observation::default())?;
        self.write_index()?.commit_prepared_with(prepared, true, None, Some(transition), None, true)
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
        Ok(self.read_index()?.total())
    }

    /// Coherent opened-root state at the returned clock.
    pub(crate) fn state(&self) -> crate::Result<IndexState> {
        Ok(self.read_index()?.state())
    }

    #[allow(dead_code)] // Consumed by the opened-root coherent read checkpoint.
    pub(crate) fn issues(&self) -> crate::Result<Vec<Issue>> {
        Ok(self.read_index()?.issues().to_vec())
    }

    /// Whether a known directory has an authoritative in-scope child set.
    #[allow(dead_code)] // Consumed by the opened-root coherent read checkpoint.
    pub(crate) fn directory_complete(&self, path: &Path) -> crate::Result<Option<bool>> {
        Ok(self.read_index()?.directory_complete(path))
    }

    /// Owned roll-up state for a relative directory path.
    pub fn rollup(&self, path: &Path) -> crate::Result<Option<RollUp>> {
        Ok(self.read_index()?.rollup(path))
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

    /// Owned exact commits and legacy deltas after `clock`.
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
                        ignored: entry.ignored,
                        rollup: entry.kind.is_dir().then(|| index.named_rollup(&entry.rollup.all)),
                        partitions: entry
                            .kind
                            .is_dir()
                            .then(|| index.named_partitions(&entry.rollup)),
                    }
                })
                .collect(),
        ))
    }

    /// Capture one coherent owned index image, releasing the lock before callers do
    /// serialization, filesystem I/O, conversion, or other potentially blocking work.
    pub fn snapshot(&self) -> crate::Result<Index> {
        let mut snapshot = self.read_index()?.clone();
        snapshot.serving = None;
        Ok(snapshot)
    }

    pub(crate) fn child_states(
        &self,
        path: &Path,
    ) -> crate::Result<BTreeMap<OsString, PathExpectation>> {
        let index = self.read_index()?;
        Ok(collect_child_expectations(&index, path))
    }

    pub(crate) fn has_control(&self, path: &Path) -> crate::Result<bool> {
        Ok(self.read_index()?.controls().contains(path))
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

    pub(crate) fn begin_reconcile(&self, path: &Path) -> crate::Result<(u64, Option<Commit>)> {
        self.write_index()?.begin_reconcile(path)
    }

    pub(crate) fn finish_reconcile(
        &self,
        path: &Path,
        started_at: u64,
        complete: bool,
    ) -> crate::Result<Option<Commit>> {
        self.write_index()?.finish_reconcile(path, started_at, complete)
    }

    #[cfg(feature = "watch")]
    pub(crate) fn apply_if_clock(
        &self,
        clock: Clock,
        observation: &Observation,
    ) -> crate::Result<Option<ApplyOutcome>> {
        let prepared = prepare_observation(observation)?;
        let mut index = self.write_index()?;
        if index.clock() != clock {
            return Ok(None);
        }
        index.commit_prepared(prepared, true).map(Some)
    }

    #[cfg(feature = "watch")]
    pub(crate) fn apply_opened_if_clock(
        &self,
        clock: Clock,
        observation: &Observation,
        max_files: Option<u64>,
    ) -> crate::Result<Option<ApplyOutcome>> {
        let prepared = prepare_observation(observation)?;
        let mut index = self.write_index()?;
        if index.clock() != clock {
            return Ok(None);
        }
        index.commit_prepared_with(prepared, true, None, None, max_files, true).map(Some)
    }

    #[cfg(feature = "watch")]
    pub(crate) fn unknown_ancestry(
        &self,
        observation: &Observation,
    ) -> crate::Result<Vec<(PathBuf, PathBuf)>> {
        let prepared = prepare_observation(observation)?;
        let index = self.read_index()?;
        let accepted = index.accepted_operations(&prepared.ops);
        Ok(index.unknown_ancestry(&prepared.ops, &accepted))
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
        Self::new_with_scope_and_types(
            root_path,
            scope,
            crate::classify::TypeRegistry::compiled_shared(),
        )
    }

    /// Create an index whose registry is part of its validated semantic scope.
    pub(crate) fn new_with_scope_and_types(
        root_path: impl Into<PathBuf>,
        scope: ScanScope,
        types: std::sync::Arc<crate::classify::TypeRegistry>,
    ) -> Self {
        Self::new_with_scope_types_and_journal_capacity(
            root_path,
            scope,
            types,
            DEFAULT_JOURNAL_CAPACITY,
        )
    }

    pub(crate) fn new_with_scope_types_and_journal_capacity(
        root_path: impl Into<PathBuf>,
        scope: ScanScope,
        types: std::sync::Arc<crate::classify::TypeRegistry>,
        journal_capacity: usize,
    ) -> Self {
        assert_eq!(
            scope.type_rules_fingerprint,
            types.fingerprint(),
            "an index's registry must match its semantic scope"
        );
        Self::new_with_journal_capacity(root_path, scope, journal_capacity, types, None)
    }

    /// Create the retained index behind an opened root, including its serving orders.
    pub(crate) fn new_opened_with_scope_types_and_journal_capacity(
        root_path: impl Into<PathBuf>,
        scope: ScanScope,
        types: std::sync::Arc<crate::classify::TypeRegistry>,
        journal_capacity: usize,
    ) -> Self {
        assert_eq!(
            scope.type_rules_fingerprint,
            types.fingerprint(),
            "an index's registry must match its semantic scope"
        );
        let serving = ServingIndexes::for_types(&types);
        Self::new_with_journal_capacity(
            root_path,
            scope,
            journal_capacity,
            types,
            Some(Box::new(serving)),
        )
    }

    fn new_with_journal_capacity(
        root_path: impl Into<PathBuf>,
        scope: ScanScope,
        journal_capacity: usize,
        types: std::sync::Arc<crate::classify::TypeRegistry>,
        serving: Option<Box<ServingIndexes>>,
    ) -> Self {
        let root = Entry {
            parent: None,
            name: OsString::new(),
            ext_id: None,
            ignored: false,
            source: Source::Scanned,
            kind: EntryKind::Dir,
            attrs: Attrs::default(),
            children: BTreeMap::new(),
            rollup: InternedPartitionRollUp::default(),
            revision: 0,
            children_revision: 0,
            children_complete: true,
        };
        Self {
            root_path: root_path.into(),
            scope,
            arena: vec![Slot::Occupied { generation: 0, entry: Box::new(root) }],
            free_head: None,
            live: 1,
            clock: Clock::ZERO,
            journal: VecDeque::new(),
            journal_cost: 0,
            journal_capacity,
            journal_floor: Clock::ZERO,
            pending_invalidations: Vec::new(),
            freshness_epoch: 0,
            freshness_marks: BTreeMap::new(),
            state: IndexState::default(),
            issues: Vec::new(),
            serving,
            applying_source: Source::Scanned,
            scanned_at_ns: Self::now_unix_nanos(),
            captured_at_ns: 0,
            verified: Vec::new(),
            ext_names: Vec::new(),
            ext_ids: BTreeMap::new(),
            ext_refcounts: Vec::new(),
            free_ext_ids: Vec::new(),
            content: None,
            types,
            controls: crate::control::ControlTable::default(),
        }
    }

    /// The file-type rules this index classifies against.
    pub fn types(&self) -> &crate::classify::TypeRegistry {
        self.types.as_ref()
    }

    /// Exact fixed control state retained by this detached index.
    pub fn controls(&self) -> &crate::control::ControlTable {
        &self.controls
    }

    /// Install a complete bounded control table while restoring a detached snapshot.
    pub(crate) fn install_controls(
        &mut self,
        controls: crate::control::ControlTable,
    ) -> crate::Result<()> {
        if controls.retained_cost() > crate::control::MAX_CONTROL_TABLE_BYTES {
            return Err(crate::Error::ControlSourceLimit {
                attempted: controls.retained_cost(),
                limit: crate::control::MAX_CONTROL_TABLE_BYTES,
            });
        }
        self.controls = controls;
        let mut stats = ApplyStats::default();
        let mut effects = MutationEffects::default();
        self.reclassify_controlled_subtrees(&[PathBuf::new()], &mut stats, &mut effects);
        Ok(())
    }

    /// Share the registry with background analysis workers.
    pub(crate) fn types_shared(&self) -> std::sync::Arc<crate::classify::TypeRegistry> {
        std::sync::Arc::clone(&self.types)
    }

    /// Classify one relative path under this index's rules, without opening the file.
    pub fn classify(&self, relative_path: &Path) -> crate::classify::Classification {
        crate::classify::classify_with(&self.types, relative_path, None)
    }

    #[cfg(test)]
    fn with_journal_capacity(root_path: impl Into<PathBuf>, journal_capacity: usize) -> Self {
        Self::new_with_journal_capacity(
            root_path,
            ScanScope::default(),
            journal_capacity,
            crate::classify::TypeRegistry::compiled_shared(),
            None,
        )
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

    /// Coherent state at the current clock.
    pub(crate) const fn state(&self) -> IndexState {
        self.state
    }

    #[allow(dead_code)] // Consumed through `IndexHandle` by the next vertical slice.
    pub(crate) fn issues(&self) -> &[Issue] {
        &self.issues
    }

    /// Whether the complete in-scope child set of a known directory is authoritative.
    #[allow(dead_code)] // Consumed through `IndexHandle` by the next vertical slice.
    pub(crate) fn directory_complete(&self, path: &Path) -> Option<bool> {
        let id = self.lookup(path)?;
        let entry = self.entry(id);
        (entry.kind == EntryKind::Dir).then_some(entry.children_complete)
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

    /// The clock of the most recently applied commit.
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

    /// Owned, self-describing roll-up state for the whole tree.
    pub fn total(&self) -> RollUp {
        self.named_rollup(&self.entry(EntryId::ROOT).rollup.all)
    }

    /// Both fixed aggregate partitions for the complete tree.
    pub fn partition_total(&self) -> PartitionRollUp {
        self.named_partitions(&self.entry(EntryId::ROOT).rollup)
    }

    /// Map-free whole-tree totals for in-crate reporting paths.
    pub(crate) fn total_scalars(&self) -> RollUpScalars {
        RollUpScalars::from(&self.entry(EntryId::ROOT).rollup.all)
    }

    /// Arbitrate a producer observation and commit its effective mutations.
    ///
    /// Conditional operations are accepted only while their baseline still matches.
    /// No-ops and stale operations do not advance the clock or enter the journal.
    pub fn apply(&mut self, observation: &Observation) -> crate::Result<ApplyOutcome> {
        let prepared = prepare_observation(observation)?;
        self.commit_prepared(prepared, true)
    }

    /// [`Self::apply`] with the change-history capture optional.
    ///
    /// `journal: false` exists for the bootstrap path, whose history
    /// [`Self::establish_baseline`] clears after every batch: capturing it first
    /// cost one effective-change clone per changed entry plus one commit clone per
    /// batch, all
    /// freed unread. Arbitration, validation, guards, and stats are identical in
    /// both modes; only what is retained afterwards differs.
    fn apply_with(
        &mut self,
        observation: &Observation,
        journal: bool,
    ) -> crate::Result<ApplyOutcome> {
        let prepared = prepare_observation(observation)?;
        self.commit_prepared(prepared, journal)
    }

    /// Arbitrate and atomically apply normalized producer input.
    fn commit_prepared(
        &mut self,
        prepared: PreparedObservation,
        journal: bool,
    ) -> crate::Result<ApplyOutcome> {
        self.commit_prepared_with(prepared, journal, None, None, None, false)
    }

    fn commit_prepared_with(
        &mut self,
        prepared: PreparedObservation,
        journal: bool,
        discovery: Option<DiscoveryCommit>,
        observation: Option<ObservationTransition>,
        max_files: Option<u64>,
        track_file_progress: bool,
    ) -> crate::Result<ApplyOutcome> {
        if prepared.ops.is_empty()
            && discovery.as_ref().is_none_or(|discovery| {
                discovery.directory_complete.is_none() && discovery.transition.is_none()
            })
            && observation.is_none()
        {
            return Ok(ApplyOutcome::default());
        }

        if let Some(path) =
            discovery.as_ref().and_then(|discovery| discovery.directory_complete.as_ref())
        {
            let path = canonical_relative_path(path)?;
            let Some(id) = self.lookup(&path) else {
                return Err(crate::Error::InvalidDirectoryCompletion(path));
            };
            if self.entry(id).kind != EntryKind::Dir {
                return Err(crate::Error::InvalidDirectoryCompletion(path));
            }
        }

        #[cfg(test)]
        if prepared.reject_before_apply {
            return Err(crate::Error::CommitRejected("injected reducer preflight"));
        }

        let Some(next_clock) = self.clock.checked_next() else {
            // At the terminal clock, an all-no-op or all-stale batch is still a valid
            // observation. Probe on a detached clone to distinguish it from a real
            // change without touching the original index.
            let mut probe = self.clone();
            probe.clock = Clock(self.clock.0 - 1);
            let outcome = probe.commit_prepared_with(
                prepared,
                false,
                discovery,
                observation,
                max_files,
                track_file_progress,
            )?;
            return if outcome.commit.is_some() {
                Err(crate::Error::ClockExhausted)
            } else {
                Ok(outcome)
            };
        };

        let observed = u64::try_from(prepared.ops.len()).unwrap_or(u64::MAX);
        let mut stats = ApplyStats::default();
        let mut effects = MutationEffects::default();
        let mut parent_memo = ParentMemo::default();
        let accepted = self.accepted_operations(&prepared.ops);
        stats.stale = u64::try_from(accepted.iter().filter(|accepted| !**accepted).count())
            .unwrap_or(u64::MAX);
        self.validate_known_ancestry(&prepared.ops, &accepted)?;
        let projected_controls = self.projected_controls(&prepared.ops, &accepted)?;

        for (observed, accepted) in prepared.ops.iter().zip(accepted) {
            if !accepted {
                continue;
            }
            let op = &observed.op;
            if let (Some(max_files), Op::Upsert { path, kind, .. }) = (max_files, op) {
                if self.files_after_upsert(path, *kind) > max_files {
                    stats.resource_refused = stats.resource_refused.saturating_add(1);
                    continue;
                }
            }
            match op {
                Op::Upsert { path, kind, attrs } => {
                    self.apply_upsert(
                        path,
                        *kind,
                        *attrs,
                        &mut stats,
                        &mut effects,
                        &mut parent_memo,
                    );
                }
                Op::Remove { path } => {
                    // A removal takes a subtree with it, so a remembered id inside that
                    // subtree would dangle. Both of the non-upsert arms drop the memo
                    // rather than reason about whether this particular path could be an
                    // ancestor of it: the memo is refilled by the next upsert, so the
                    // cost of being conservative is one path resolution.
                    parent_memo.clear();
                    self.apply_remove(path, &mut stats, &mut effects);
                }
                Op::ControlUpsert { .. } | Op::ControlRemove { .. } => {
                    // The complete table was already prepared above. It is installed
                    // once, after ordinary structural mutations, so classification and
                    // both reducer partitions become visible atomically.
                    parent_memo.clear();
                }
                Op::InvalidateSubtree { path, reason } => {
                    parent_memo.clear();
                    let previous_index_state = self.state;
                    let previous = self.freshness_at(path);
                    self.pending_invalidations.push((path.clone(), *reason));
                    self.mark_unfresh(path, Freshness::Stale);
                    let current = self.freshness_at(path);
                    self.state.freshness = self.freshness();
                    if matches!(
                        reason,
                        InvalidateReason::WatchOverflow
                            | InvalidateReason::UnpairedRename
                            | InvalidateReason::WatchSetupRace
                            | InvalidateReason::VerificationFailed
                            | InvalidateReason::UnknownAncestry
                            | InvalidateReason::WatchContention
                    ) {
                        self.retain_issue(Issue::observation_gap(path, *reason));
                    }
                    stats.invalidated += 1;
                    effects
                        .changes
                        .push(EffectiveChange::Invalidated { path: path.clone(), reason: *reason });
                    if previous != current {
                        effects.state.push(StateTransition::Freshness {
                            path: path.clone(),
                            previous,
                            current,
                        });
                    }
                    if previous_index_state != self.state {
                        effects.state.push(StateTransition::IndexState {
                            previous: previous_index_state,
                            current: self.state,
                        });
                    }
                }
            }
        }

        self.apply_control_transition(projected_controls, &mut stats, &mut effects);
        let mut discovery = discovery;
        if stats.resource_refused > 0 {
            let max_files = max_files.expect("resource refusal requires a file limit");
            let discovery = discovery.get_or_insert_with(DiscoveryCommit::default);
            discovery.directory_complete = None;
            discovery.transition =
                Some(DiscoveryTransition::BudgetRefused(Issue::resource_budget(max_files)));
        }
        self.apply_opened_state(discovery, observation, track_file_progress, &mut effects);

        if effects.is_empty() {
            return Ok(ApplyOutcome::from_commit(stats, None));
        }

        let commit =
            self.publish_effects(next_clock, effects, commit_work(observed, stats), journal);
        Ok(ApplyOutcome::from_commit(stats, Some(commit)))
    }

    fn apply_opened_state(
        &mut self,
        discovery: Option<DiscoveryCommit>,
        observation: Option<ObservationTransition>,
        track_file_progress: bool,
        effects: &mut MutationEffects,
    ) {
        if discovery.is_none() && observation.is_none() && !track_file_progress {
            return;
        }
        let previous = self.state;
        if track_file_progress {
            self.state.progress.files_retained = self.total_scalars().files;
        }

        if let Some(discovery) = discovery {
            if let Some(path) = discovery.directory_complete {
                let id = self.lookup(&path).expect("discovery completion was preflighted");
                if !self.entry(id).children_complete {
                    self.entry_mut(id).children_complete = true;
                    self.state.progress.directories_complete =
                        self.state.progress.directories_complete.saturating_add(1);
                    effects.state.push(StateTransition::DirectoryComplete { path });
                }
            }

            if let Some(transition) = discovery.transition {
                match transition {
                    DiscoveryTransition::Begin => {
                        for slot in &mut self.arena {
                            if let Slot::Occupied { entry, .. } = slot {
                                if entry.kind == EntryKind::Dir {
                                    entry.children_complete = false;
                                }
                            }
                        }
                        self.state = IndexState {
                            phase: LifecyclePhase::Discovering,
                            coverage: Coverage::Partial(CoverageReason::Building),
                            freshness: Freshness::Fresh,
                            source: Source::Scanned,
                            progress: DiscoveryProgress::default(),
                            issues: crate::IssueSummary::default(),
                        };
                        self.issues.clear();
                    }
                    DiscoveryTransition::Finish => {
                        self.state.phase = LifecyclePhase::Ready;
                        if self.state.coverage == Coverage::Partial(CoverageReason::Building) {
                            self.state.coverage = Coverage::Complete;
                        }
                        self.state.freshness = if self.state.coverage == Coverage::Complete {
                            Freshness::Fresh
                        } else {
                            Freshness::Partial
                        };
                    }
                    DiscoveryTransition::BudgetRefused(issue) => {
                        let already_stopped_for_budget = self.state.phase
                            == LifecyclePhase::Stopped
                            && self.state.coverage == Coverage::Partial(CoverageReason::Budget);
                        self.state.phase = LifecyclePhase::Stopped;
                        self.state.coverage = Coverage::Partial(CoverageReason::Budget);
                        self.state.freshness = Freshness::Fresh;
                        if !already_stopped_for_budget {
                            self.retain_issue(issue);
                        }
                    }
                    DiscoveryTransition::Inaccessible { issues, omitted } => {
                        if self.state.coverage != Coverage::Partial(CoverageReason::Budget) {
                            self.state.coverage = Coverage::Partial(CoverageReason::Inaccessible);
                            self.state.freshness = Freshness::Partial;
                        }
                        for issue in issues {
                            self.retain_issue(issue);
                        }
                        self.state.issues.omitted =
                            self.state.issues.omitted.saturating_add(omitted);
                    }
                    DiscoveryTransition::Cancelled => {
                        self.state.phase = LifecyclePhase::Stopped;
                        if self.state.coverage != Coverage::Complete {
                            self.state.coverage = Coverage::Partial(CoverageReason::Cancelled);
                        }
                    }
                    DiscoveryTransition::Failed(issue) => {
                        self.state.phase = LifecyclePhase::Failed;
                        self.state.coverage = Coverage::Partial(CoverageReason::Failed);
                        self.state.freshness = Freshness::Partial;
                        self.retain_issue(issue);
                    }
                }
            }
        }

        if let Some(observation) = observation {
            match observation {
                ObservationTransition::Reconciling => {
                    if self.state.phase == LifecyclePhase::Ready {
                        self.state.phase = LifecyclePhase::Reconciling;
                        self.state.freshness = Freshness::Reconciling;
                    }
                }
                ObservationTransition::Watching { issues, omitted } => {
                    if self.state.phase == LifecyclePhase::Reconciling {
                        self.state.phase = LifecyclePhase::Watching;
                        if !issues.is_empty() || omitted > 0 {
                            self.state.coverage = Coverage::Partial(CoverageReason::Inaccessible);
                            for issue in issues {
                                self.retain_issue(issue);
                            }
                            self.state.issues.omitted =
                                self.state.issues.omitted.saturating_add(omitted);
                        }
                        self.state.freshness = self.freshness();
                        if self.state.coverage != Coverage::Complete {
                            self.state.freshness = Freshness::Partial;
                        }
                    }
                }
                ObservationTransition::Failed(issue) => {
                    if self.state.phase != LifecyclePhase::Stopped {
                        self.state.phase = LifecyclePhase::Failed;
                        self.state.freshness = Freshness::Partial;
                        self.retain_issue(issue);
                    }
                }
            }
        }

        if previous != self.state {
            effects.state.push(StateTransition::IndexState { previous, current: self.state });
        }
    }

    /// Exact regular-file total that would remain after one upsert at the current
    /// commit boundary.
    fn files_after_upsert(&self, path: &Path, kind: EntryKind) -> u64 {
        let current_total = self.total_scalars().files;
        let Some(id) = self.lookup(path) else {
            return current_total.saturating_add(u64::from(kind == EntryKind::File));
        };
        let current = self.entry(id);
        if current.kind == kind {
            return current_total;
        }
        let removed = match current.kind {
            EntryKind::File => 1,
            EntryKind::Dir => current.rollup.all.files,
            _ => 0,
        };
        current_total.saturating_sub(removed).saturating_add(u64::from(kind == EntryKind::File))
    }

    fn retain_issue(&mut self, issue: Issue) {
        if self.issues.len() < MAX_RETAINED_ISSUES {
            self.issues.push(issue);
            self.state.issues.retained = u64::try_from(self.issues.len()).unwrap_or(u64::MAX);
        } else {
            self.state.issues.omitted = self.state.issues.omitted.saturating_add(1);
        }
    }

    /// Mint and optionally retain one fully evaluated transition.
    ///
    /// Every fact-only, state-only, or combined mutation reaches this function after
    /// its fallible validation and preflight work is complete.
    fn publish_effects(
        &mut self,
        next_clock: Clock,
        effects: MutationEffects,
        work: Work,
        journal: bool,
    ) -> Commit {
        debug_assert!(!effects.is_empty());
        let commit = Commit {
            clock: next_clock,
            impact: derive_impact(&effects.changes, &effects.state),
            changes: effects.changes,
            state: effects.state,
            work,
        };
        self.clock = next_clock;
        if journal {
            self.retain_commit(commit.clone());
        }
        commit
    }

    fn retain_commit(&mut self, commit: Commit) {
        let cost = commit.retained_cost();
        if cost > self.journal_capacity {
            self.journal.clear();
            self.journal_cost = 0;
            self.journal_floor = commit.clock;
            return;
        }

        while self.journal_cost + cost > self.journal_capacity {
            if let Some(dropped) = self.journal.pop_front() {
                self.journal_cost -= dropped.retained_cost();
                self.journal_floor = dropped.clock;
            }
        }
        self.journal_cost += cost;
        self.journal.push_back(commit);
    }

    /// Apply trusted bootstrap data without exposing it as live change history.
    pub(crate) fn apply_baseline(
        &mut self,
        observation: &Observation,
    ) -> crate::Result<ApplyStats> {
        let outcome = self.apply_with(observation, false)?;
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
        self.journal_cost = 0;
        self.journal_floor = Clock::ZERO;
        self.pending_invalidations.clear();
    }

    /// Mark the whole index as not verified against the filesystem.
    ///
    /// Used by the cache-only open path: a snapshot records the freshness it had when it
    /// was written, and replaying that verbatim would let an unverified answer claim
    /// currency it has not earned.
    pub(crate) fn mark_unverified(&mut self) {
        self.freshness_marks.clear();
        self.mark_unfresh(Path::new(""), Freshness::Stale);
    }

    pub(crate) fn set_initial_freshness(&mut self, complete: bool) {
        self.freshness_marks.clear();
        if complete {
            for slot in &mut self.arena {
                if let Slot::Occupied { entry, .. } = slot {
                    if entry.kind == EntryKind::Dir {
                        entry.children_complete = true;
                    }
                }
            }
            self.state.phase = LifecyclePhase::Ready;
            self.state.coverage = Coverage::Complete;
            self.state.freshness = Freshness::Fresh;
        } else {
            self.mark_unfresh(Path::new(""), Freshness::Partial);
            self.state.phase = LifecyclePhase::Ready;
            self.state.coverage = Coverage::Partial(CoverageReason::Inaccessible);
            self.state.freshness = Freshness::Partial;
        }
    }

    pub(crate) fn begin_reconcile(&mut self, path: &Path) -> crate::Result<(u64, Option<Commit>)> {
        let path = canonical_relative_path(path)?;
        let next_clock = self.clock.checked_next().ok_or(crate::Error::ClockExhausted)?;
        let previous_index_state = self.state;
        let previous = self.freshness_at(&path);
        let epoch = self.mark_unfresh(&path, Freshness::Reconciling);
        let current = self.freshness_at(&path);
        self.state.freshness = self.freshness();
        let commit = if previous == current && previous_index_state == self.state {
            None
        } else {
            let mut state = Vec::new();
            if previous != current {
                state.push(StateTransition::Freshness { path, previous, current });
            }
            if previous_index_state != self.state {
                state.push(StateTransition::IndexState {
                    previous: previous_index_state,
                    current: self.state,
                });
            }
            let effects = MutationEffects { state, ..MutationEffects::default() };
            Some(self.publish_effects(next_clock, effects, Work::default(), true))
        };
        Ok((epoch, commit))
    }

    pub(crate) fn finish_reconcile(
        &mut self,
        path: &Path,
        started_at: u64,
        complete: bool,
    ) -> crate::Result<Option<Commit>> {
        let path = canonical_relative_path(path)?;
        let next_clock = self.clock.checked_next().ok_or(crate::Error::ClockExhausted)?;
        let previous_index_state = self.state;
        let previous = self.freshness_at(&path);
        self.freshness_marks
            .retain(|marked, mark| !marked.starts_with(&path) || mark.epoch > started_at);
        let mut state = Vec::new();
        if complete {
            // A completed sweep stat'd every entry beneath `path`, including the ones
            // the producer elided as no-ops. Record that interval as one exact state
            // transition rather than manufacturing millions of entry updates.
            let now = Self::now_unix_nanos();
            self.verified.retain(|(verified_path, _)| !verified_path.starts_with(&path));
            self.verified.push((path.clone(), now));
            if self.verified.len() > MAX_VERIFIED_INTERVALS {
                let excess = self.verified.len() - MAX_VERIFIED_INTERVALS;
                self.verified.sort_by_key(|(_, at)| *at);
                self.verified.drain(..excess);
            }
            state.push(StateTransition::Verified { path: path.clone() });
        } else {
            self.mark_unfresh(&path, Freshness::Partial);
        }

        let current = self.freshness_at(&path);
        self.state.freshness = self.freshness();
        if previous != current {
            state.push(StateTransition::Freshness { path: path.clone(), previous, current });
        }
        if previous_index_state != self.state {
            state.push(StateTransition::IndexState {
                previous: previous_index_state,
                current: self.state,
            });
        }
        if state.is_empty() {
            return Ok(None);
        }
        let effects = MutationEffects { state, ..MutationEffects::default() };
        Ok(Some(self.publish_effects(next_clock, effects, Work::default(), true)))
    }

    /// When a completed reconciliation last covered this path, if one did.
    fn verified_at(&self, path: &Path) -> Option<i64> {
        self.verified
            .iter()
            .filter(|(covered, _)| path.starts_with(covered))
            .map(|(_, at)| *at)
            .max()
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

    /// Exact commits and legacy deltas applied since `clock`, oldest first.
    pub fn since(&self, clock: Clock) -> Since {
        let commits: Vec<Commit> =
            self.journal.iter().filter(|commit| commit.clock > clock).cloned().collect();
        Since {
            deltas: commits.iter().filter_map(Commit::applied_delta).collect(),
            commits,
            clock: self.clock,
            state: self.state,
            truncated: clock < self.journal_floor,
        }
    }

    /// Take the subtrees that producers escalated for re-scan.
    ///
    /// The caller is expected to hand these to the scan layer, which turns them back
    /// into precise commits. Escalation is closed-loop: draining this list without
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

    /// Owned, self-describing roll-up state for a directory by relative path.
    /// The empty path is the root.
    pub fn rollup(&self, path: &Path) -> Option<RollUp> {
        let id = self.lookup(path)?;
        let entry = self.entry(id);
        entry.kind.is_dir().then(|| self.named_rollup(&entry.rollup.all))
    }

    /// Both fixed aggregate partitions for a directory by relative path.
    pub fn partition_rollup(&self, path: &Path) -> Option<PartitionRollUp> {
        let entry = self.entry(self.lookup(path)?);
        entry.kind.is_dir().then(|| self.named_partitions(&entry.rollup))
    }

    /// Both constant-size aggregate partitions for a directory.
    pub fn partition_rollup_summary(&self, path: &Path) -> Option<PartitionRollUpSummary> {
        let entry = self.entry(self.lookup(path)?);
        entry.kind.is_dir().then(|| partition_summary(&entry.rollup))
    }

    /// Capture one retained entry without repeating path lookup in a consumer.
    pub(crate) fn entry_value(&self, path: &Path) -> Option<crate::EntryValue> {
        let id = self.lookup(path)?;
        Some(self.entry_value_of(id, path))
    }

    pub(crate) fn entry_value_of(&self, id: EntryId, path: &Path) -> crate::EntryValue {
        let entry = self.entry(id);
        crate::EntryValue {
            path: path.to_path_buf(),
            portable_path: crate::opened::read::portable_path(path),
            kind: entry.kind,
            attrs: entry.attrs,
            ignored: entry.ignored,
            classification: (entry.kind == EntryKind::File)
                .then(|| self.types.classify_name(path.file_name().unwrap_or_default())),
            rollup: entry.kind.is_dir().then(|| partition_summary(&entry.rollup)),
            children_complete: entry.kind.is_dir().then_some(entry.children_complete),
        }
    }

    pub(crate) fn portable_children(&self, path: &Path) -> Option<&PortableChildren> {
        self.serving.as_ref()?.portable_children.get(path)
    }

    pub(crate) fn portable_entries(&self) -> &BTreeMap<crate::PortablePath, EntryId> {
        &self.serving.as_ref().expect("opened-root reads require serving indexes").portable_entries
    }

    #[cfg(test)]
    pub(crate) const fn serving_indexes_enabled(&self) -> bool {
        self.serving.is_some()
    }

    fn insert_serving_entry(&mut self, path: &Path, kind: EntryKind, attrs: Attrs, id: EntryId) {
        if path.as_os_str().is_empty() || self.serving.is_none() {
            return;
        }
        let file = (kind == EntryKind::File).then(|| {
            (
                self.classify(path).file_type.as_str().to_string(),
                path.file_name(),
                self.entry(id).ignored,
                self.entry(id).parent,
            )
        });
        let arena = &self.arena;
        let Some(serving) = self.serving.as_mut() else {
            return;
        };
        if let Some((name, exact_name, ignored, parent)) = file {
            let semantic = serving.intern_semantic(&name);
            let mut ancestor = parent;
            while let Some(directory) = ancestor {
                let partition = serving.semantic_by_directory.entry(directory).or_default();
                merge_semantic(&mut partition.all, semantic, attrs);
                if !ignored {
                    merge_semantic(&mut partition.unignored, semantic, attrs);
                }
                ancestor = retained_parent(arena, directory);
            }
            if let Some(exact_name) = exact_name.and_then(|name| serving.exact_name_id(name)) {
                let mut ancestor = parent;
                while let Some(directory) = ancestor {
                    let partition = serving.exact_name_by_directory.entry(directory).or_default();
                    merge_semantic(&mut partition.all, exact_name, attrs);
                    if !ignored {
                        merge_semantic(&mut partition.unignored, exact_name, attrs);
                    }
                    ancestor = retained_parent(arena, directory);
                }
            }
        }
        let portable = crate::opened::read::portable_path(path);
        serving.portable_entries.insert(portable.clone(), id);
        if kind == EntryKind::File {
            serving.recent_files.insert(RecentKey {
                mtime_ns: attrs.mtime_ns,
                portable_path: portable,
                id,
            });
        }
        let parent = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();
        let Some(name) = path.file_name().map(crate::opened::read::portable_component) else {
            return;
        };
        let children = serving.portable_children.entry(parent).or_default();
        if kind.is_dir() {
            children.directories.insert(name, id);
        } else {
            children.nondirectories.insert(name, id);
        }
    }

    fn remove_serving_entry(&mut self, path: &Path, kind: EntryKind, attrs: Attrs, id: EntryId) {
        if path.as_os_str().is_empty() {
            return;
        }
        let Some(serving) = self.serving.as_mut() else {
            return;
        };
        let portable = crate::opened::read::portable_path(path);
        serving.portable_entries.remove(&portable);
        if kind == EntryKind::File {
            serving.recent_files.remove(&RecentKey {
                mtime_ns: attrs.mtime_ns,
                portable_path: portable,
                id,
            });
        }
        let parent = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();
        let remove_parent = if let Some(children) = serving.portable_children.get_mut(&parent) {
            if let Some(name) = path.file_name().map(crate::opened::read::portable_component) {
                if kind.is_dir() {
                    children.directories.remove(&name);
                } else {
                    children.nondirectories.remove(&name);
                }
            }
            children.directories.is_empty() && children.nondirectories.is_empty()
        } else {
            false
        };
        if remove_parent {
            serving.portable_children.remove(&parent);
        }
        if kind.is_dir() {
            serving.portable_children.remove(path);
        }
    }

    fn remove_serving_file_semantics(&mut self, path: &Path, id: EntryId, attrs: Attrs) {
        if self.serving.is_none() {
            return;
        }
        let name = self.classify(path).file_type.as_str().to_string();
        let exact_name = path.file_name();
        let ignored = self.entry(id).ignored;
        let parent = self.entry(id).parent;
        let arena = &self.arena;
        let serving = self.serving.as_mut().expect("checked above");
        let semantic = *serving
            .semantic_ids
            .get(&name)
            .expect("every served file has an interned semantic type");
        let mut empty = Vec::new();
        let mut ancestor = parent;
        while let Some(directory) = ancestor {
            let partition = serving
                .semantic_by_directory
                .get_mut(&directory)
                .expect("every served file contributes to every ancestor");
            unmerge_semantic(&mut partition.all, semantic, attrs);
            if !ignored {
                unmerge_semantic(&mut partition.unignored, semantic, attrs);
            }
            if partition.all.is_empty() && partition.unignored.is_empty() {
                empty.push(directory);
            }
            ancestor = retained_parent(arena, directory);
        }
        for ancestor in empty {
            serving.semantic_by_directory.remove(&ancestor);
        }
        serving.release_semantic(semantic, 1);
        if let Some(exact_name) = exact_name.and_then(|name| serving.exact_name_id(name)) {
            let mut exact_empty = Vec::new();
            let mut ancestor = parent;
            while let Some(directory) = ancestor {
                let partition = serving
                    .exact_name_by_directory
                    .get_mut(&directory)
                    .expect("every declared exact-name file contributes to every ancestor");
                unmerge_semantic(&mut partition.all, exact_name, attrs);
                if !ignored {
                    unmerge_semantic(&mut partition.unignored, exact_name, attrs);
                }
                if partition.all.is_empty() && partition.unignored.is_empty() {
                    exact_empty.push(directory);
                }
                ancestor = retained_parent(arena, directory);
            }
            for ancestor in exact_empty {
                serving.exact_name_by_directory.remove(&ancestor);
            }
        }
    }

    fn remove_serving_subtree_semantics(&mut self, root: EntryId, path: &Path) {
        if self.serving.is_none() {
            return;
        }
        match self.entry(root).kind {
            EntryKind::File => {
                let attrs = self.entry(root).attrs;
                self.remove_serving_file_semantics(path, root, attrs);
            }
            EntryKind::Dir => {
                let parent = self.entry(root).parent;
                let mut stack = vec![root];
                let mut directories = Vec::new();
                while let Some(id) = stack.pop() {
                    let entry = self.entry(id);
                    if !entry.kind.is_dir() {
                        continue;
                    }
                    directories.push(id);
                    stack.extend(entry.children.values().copied());
                }
                let arena = &self.arena;
                let serving = self.serving.as_mut().expect("checked above");
                let contribution =
                    serving.semantic_by_directory.get(&root).cloned().unwrap_or_default();
                let exact_contribution =
                    serving.exact_name_by_directory.get(&root).cloned().unwrap_or_default();
                let mut empty = Vec::new();
                if !contribution.all.is_empty() || !contribution.unignored.is_empty() {
                    let mut ancestor = parent;
                    while let Some(directory) = ancestor {
                        let partition = serving
                            .semantic_by_directory
                            .get_mut(&directory)
                            .expect("a semantic subtree contributes to every ancestor");
                        unmerge_semantic_map(&mut partition.all, &contribution.all);
                        unmerge_semantic_map(&mut partition.unignored, &contribution.unignored);
                        if partition.all.is_empty() && partition.unignored.is_empty() {
                            empty.push(directory);
                        }
                        ancestor = retained_parent(arena, directory);
                    }
                }
                let mut exact_empty = Vec::new();
                if !exact_contribution.all.is_empty() || !exact_contribution.unignored.is_empty() {
                    let mut ancestor = parent;
                    while let Some(directory) = ancestor {
                        let partition = serving
                            .exact_name_by_directory
                            .get_mut(&directory)
                            .expect("an exact-name subtree contributes to every ancestor");
                        unmerge_semantic_map(&mut partition.all, &exact_contribution.all);
                        unmerge_semantic_map(
                            &mut partition.unignored,
                            &exact_contribution.unignored,
                        );
                        if partition.all.is_empty() && partition.unignored.is_empty() {
                            exact_empty.push(directory);
                        }
                        ancestor = retained_parent(arena, directory);
                    }
                }
                for ancestor in empty {
                    serving.semantic_by_directory.remove(&ancestor);
                }
                for ancestor in exact_empty {
                    serving.exact_name_by_directory.remove(&ancestor);
                }
                for directory in directories {
                    serving.semantic_by_directory.remove(&directory);
                    serving.exact_name_by_directory.remove(&directory);
                }
                for (semantic, tally) in contribution.all {
                    serving.release_semantic(semantic, tally.files);
                }
            }
            EntryKind::Symlink | EntryKind::Other => {}
        }
    }

    fn move_serving_file_partition(
        &mut self,
        path: &Path,
        id: EntryId,
        previous_ignored: bool,
        current_ignored: bool,
    ) {
        if previous_ignored == current_ignored
            || self.serving.is_none()
            || self.entry(id).kind != EntryKind::File
        {
            return;
        }
        let name = self.classify(path).file_type.as_str().to_string();
        let exact_name = path.file_name();
        let attrs = self.entry(id).attrs;
        let parent = self.entry(id).parent;
        let arena = &self.arena;
        let serving = self.serving.as_mut().expect("checked above");
        let semantic = *serving
            .semantic_ids
            .get(&name)
            .expect("every served file has an interned semantic type");
        let mut ancestor = parent;
        while let Some(directory) = ancestor {
            let partition = serving
                .semantic_by_directory
                .get_mut(&directory)
                .expect("every served file contributes to every ancestor");
            if current_ignored {
                unmerge_semantic(&mut partition.unignored, semantic, attrs);
            } else {
                merge_semantic(&mut partition.unignored, semantic, attrs);
            }
            ancestor = retained_parent(arena, directory);
        }
        if let Some(exact_name) = exact_name.and_then(|name| serving.exact_name_id(name)) {
            let mut ancestor = parent;
            while let Some(directory) = ancestor {
                let partition = serving
                    .exact_name_by_directory
                    .get_mut(&directory)
                    .expect("every declared exact-name file contributes to every ancestor");
                if current_ignored {
                    unmerge_semantic(&mut partition.unignored, exact_name, attrs);
                } else {
                    merge_semantic(&mut partition.unignored, exact_name, attrs);
                }
                ancestor = retained_parent(arena, directory);
            }
        }
    }

    /// Attributes for any entry, by relative path.
    pub fn attrs(&self, path: &Path) -> Option<&Attrs> {
        Some(&self.entry(self.lookup(path)?).attrs)
    }

    /// Kind of an entry, by relative path.
    pub fn kind(&self, path: &Path) -> Option<EntryKind> {
        Some(self.entry(self.lookup(path)?).kind)
    }

    /// Effective fixed-control classification for one retained entry.
    pub fn is_ignored(&self, path: &Path) -> Option<bool> {
        Some(self.entry(self.lookup(path)?).ignored)
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

    /// Owned, self-describing roll-up state for an entry id, if it is a directory.
    pub fn rollup_of(&self, id: EntryId) -> Option<RollUp> {
        let entry = self.try_entry(id)?;
        entry.kind.is_dir().then(|| self.named_rollup(&entry.rollup.all))
    }

    /// Map-free directory totals for in-crate reporting paths.
    pub(crate) fn rollup_scalars_of(&self, id: EntryId) -> Option<RollUpScalars> {
        let entry = self.try_entry(id)?;
        entry.kind.is_dir().then(|| RollUpScalars::from(&entry.rollup.all))
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

    /// Sparse content tier, when analysis has been enabled.
    pub fn content(&self) -> Option<&ContentIndex> {
        self.content.as_deref()
    }

    /// Precomputed content rollup for one relative directory.
    pub fn content_rollup(&self, path: &Path) -> Option<&ContentRollUp> {
        self.content()?.rollup(path)
    }

    pub(crate) fn prepare_content_analysis(&mut self, request: crate::content::AnalysisRequest) {
        if !request.profile.is_enabled() {
            return;
        }
        self.content.get_or_insert_with(|| Box::new(ContentIndex::default())).prepare(
            request.profile,
            crate::content::ContentProvenance::for_request(request, self.types.fingerprint()),
        );
    }

    /// Capture every regular-file analysis candidate without retaining a lock or entry
    /// borrow across filesystem I/O.
    pub fn analysis_candidates(&self, profile: AnalysisSet) -> Vec<AnalysisCandidate> {
        if !profile.is_enabled() {
            return Vec::new();
        }
        let root_files = self.entry(EntryId::ROOT).rollup.files;
        let mut candidates = Vec::with_capacity(usize::try_from(root_files).unwrap_or(0));
        let mut stack = vec![EntryId::ROOT];
        while let Some(parent) = stack.pop() {
            for (_, id) in self.children_of(parent).into_iter().flatten() {
                let entry = self.entry(id);
                if entry.kind == EntryKind::Dir {
                    stack.push(id);
                    continue;
                }
                if entry.kind != EntryKind::File {
                    continue;
                }
                let relative_path = self.path_of(id).expect("live entry has a path");
                candidates.push(AnalysisCandidate {
                    entry_id: id,
                    revision: entry.revision,
                    absolute_path: self.root_path.join(&relative_path),
                    classification: self.classify(&relative_path),
                    relative_path,
                    attrs: entry.attrs,
                    profile,
                });
            }
        }
        candidates
    }

    pub(crate) fn pending_analysis_candidates(
        &self,
        request: crate::content::AnalysisRequest,
    ) -> Vec<AnalysisCandidate> {
        self.analysis_candidates(request.profile)
            .into_iter()
            .filter(|candidate| {
                self.content()
                    .and_then(|content| content.file(&candidate.relative_path))
                    .is_none_or(|record| {
                        record.fingerprint != candidate.attrs.fingerprint()
                            || !record.provenance.satisfies(
                                record.profile,
                                request.profile,
                                self.types.fingerprint(),
                            )
                    })
            })
            .collect()
    }

    /// Conditionally commit a worker result if its entry and metadata expectation still
    /// match.
    pub fn apply_analysis(&mut self, observation: AnalysisObservation) -> AnalysisApplyOutcome {
        let candidate = &observation.candidate;
        let Some(entry) = self.try_entry(candidate.entry_id) else {
            return AnalysisApplyOutcome::Stale;
        };
        if entry.kind != EntryKind::File
            || entry.revision != candidate.revision
            || entry.attrs.fingerprint() != candidate.attrs.fingerprint()
            || self.classify(&candidate.relative_path) != candidate.classification
        {
            return AnalysisApplyOutcome::Stale;
        }
        self.content
            .get_or_insert_with(|| Box::new(ContentIndex::default()))
            .commit(candidate.relative_path.clone(), observation.analysis);
        AnalysisApplyOutcome::Applied
    }

    /// Drop all derived content while preserving metadata and snapshot compatibility.
    pub fn clear_content(&mut self) {
        self.content = None;
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
            (
                Op::Upsert { .. }
                | Op::ControlUpsert { .. }
                | Op::ControlRemove { .. }
                | Op::InvalidateSubtree { .. },
                _,
            ) => false,
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

    /// Prove that every accepted live upsert has a verified parent chain.
    ///
    /// The overlay follows batch order without touching the real index. That admits
    /// parent-first discovery batches and rejects a child whose missing or non-directory
    /// ancestry would otherwise be filled with guessed metadata.
    fn validate_known_ancestry(
        &self,
        ops: &[ObservationOp],
        accepted: &[bool],
    ) -> crate::Result<()> {
        if let Some((path, reconcile_from)) =
            self.unknown_ancestry(ops, accepted).into_iter().next()
        {
            return Err(crate::Error::UnknownAncestry { path, reconcile_from });
        }
        Ok(())
    }

    fn accepted_operations(&self, ops: &[ObservationOp]) -> Vec<bool> {
        ops.iter()
            .map(|observed| match observed.expectation {
                Expectation::Any => true,
                Expectation::State(expected) => self.expectation_matches(&observed.op, expected),
            })
            .collect()
    }

    /// Evaluate the complete resulting control table before any fact or reducer moves.
    ///
    /// Parsing is infallible, but the shared source bound is not. Building the projected
    /// table here makes an over-limit observation fault-atomic even when the same batch
    /// also inserts or removes ordinary entries.
    fn projected_controls(
        &self,
        ops: &[ObservationOp],
        accepted: &[bool],
    ) -> crate::Result<crate::control::ControlTable> {
        // When the retained table is empty and the batch carries no control op of any
        // kind, projection cannot change anything: only control ops write the table, and
        // there is nothing retained for a structural removal to prune. A cold scan of a
        // tree without control files takes this lane for every batch, instead of paying
        // one structural-overlay insertion per op to project a table that was empty onto
        // a table that stays empty (fdu-pro1).
        //
        // Both control op kinds disqualify, not just upserts: with the capability
        // compiled out, `ControlTable::remove` is what rejects a `ControlRemove`, and a
        // fast lane that skipped it would accept input the slow lane fails closed on --
        // which is precisely what `control_input_fails_closed_when_the_capability_is_absent`
        // caught when this lane tested only for upserts.
        if self.controls.is_empty()
            && !ops.iter().zip(accepted).any(|(observed, accepted)| {
                *accepted
                    && matches!(observed.op, Op::ControlUpsert { .. } | Op::ControlRemove { .. })
            })
        {
            return Ok(self.controls.clone());
        }
        let mut projected = self.controls.clone();
        let mut structure = StructuralOverlay::default();
        for (observed, accepted) in ops.iter().zip(accepted) {
            if !accepted {
                continue;
            }
            match &observed.op {
                Op::Upsert { path, kind, .. } => {
                    if crate::control::is_control_file(path) && *kind != EntryKind::File {
                        projected.remove(path)?;
                    }
                    if structure.kind(self, path) == Some(EntryKind::Dir) && !kind.is_dir() {
                        projected.remove_subtree(path);
                    }
                    structure.upsert(self, path, *kind);
                }
                Op::Remove { path } => {
                    if crate::control::is_control_file(path) {
                        projected.remove(path)?;
                    }
                    projected.remove_subtree(path);
                    structure.remove(self, path);
                }
                Op::ControlUpsert { path, source } => {
                    projected.upsert(path, source.clone())?;
                }
                Op::ControlRemove { path } => {
                    #[cfg(not(feature = "gitignore"))]
                    {
                        let _ = path;
                        return Err(crate::Error::UnsupportedScanConfig(
                            "control observations require the fdu-core `gitignore` feature",
                        ));
                    }
                    #[cfg(feature = "gitignore")]
                    projected.remove(path)?;
                }
                Op::InvalidateSubtree { .. } => {}
            }
        }
        Ok(projected)
    }

    fn apply_control_transition(
        &mut self,
        projected: crate::control::ControlTable,
        stats: &mut ApplyStats,
        effects: &mut MutationEffects,
    ) {
        let changes = projected.changes_from(&self.controls);
        if changes.is_empty() {
            return;
        }
        let affected: Vec<PathBuf> = changes
            .iter()
            .filter_map(|(path, _, _)| crate::control::ControlTable::affected_subtree(path).ok())
            .collect();
        self.controls = projected;
        stats.controls = u64::try_from(changes.len()).unwrap_or(u64::MAX);
        effects.changes.extend(changes.into_iter().map(|(path, previous, current)| {
            EffectiveChange::ControlUpdated { path, previous, current }
        }));
        self.reclassify_controlled_subtrees(&affected, stats, effects);
    }

    /// Re-evaluate only subtrees governed by changed controls, then rebuild the fixed
    /// unignored reducer from the resulting facts.
    fn reclassify_controlled_subtrees(
        &mut self,
        affected: &[PathBuf],
        stats: &mut ApplyStats,
        effects: &mut MutationEffects,
    ) {
        let mut roots: Vec<PathBuf> = affected.to_vec();
        roots.sort();
        roots.dedup();
        let mut collapsed = Vec::new();
        for root in roots {
            if collapsed.iter().any(|ancestor: &PathBuf| root.starts_with(ancestor)) {
                continue;
            }
            collapsed.push(root);
        }

        let mut moved = false;
        for root in collapsed {
            let Some(root_id) = self.lookup(&root) else {
                continue;
            };
            let children: Vec<(PathBuf, EntryId)> = self
                .entry(root_id)
                .children
                .iter()
                .map(|(name, id)| (root.join(name), *id))
                .collect();
            let mut queue = VecDeque::from(children);
            while let Some((path, id)) = queue.pop_front() {
                let entry = self.entry(id);
                let parent_ignored = entry.parent.is_some_and(|parent| self.entry(parent).ignored);
                let current = entry.ignored;
                let next = parent_ignored
                    || self.controls.matcher_for(&path).is_ignored(entry.kind.is_dir());
                let descendants: Vec<(PathBuf, EntryId)> =
                    entry.children.iter().map(|(name, child)| (path.join(name), *child)).collect();
                if current != next {
                    self.move_serving_file_partition(&path, id, current, next);
                    self.entry_mut(id).ignored = next;
                    stats.reclassified += 1;
                    effects.changes.push(EffectiveChange::Reclassified {
                        path: path.clone(),
                        previous_ignored: current,
                        current_ignored: next,
                    });
                    moved = true;
                }
                queue.extend(descendants);
            }
        }
        if moved {
            self.rebuild_unignored_rollups();
        }
    }

    fn rebuild_unignored_rollups(&mut self) {
        let mut order = Vec::with_capacity(usize::try_from(self.live).unwrap_or(0));
        let mut stack = vec![EntryId::ROOT];
        while let Some(id) = stack.pop() {
            order.push(id);
            stack.extend(self.entry(id).children.values().copied());
            self.entry_mut(id).rollup.unignored = InternedRollUp::default();
        }
        for id in order.into_iter().rev() {
            let Some(parent) = self.entry(id).parent else {
                continue;
            };
            let contribution = self.contribution(id).unignored;
            self.entry_mut(parent).rollup.unignored.merge(&contribution);
        }
    }

    fn unknown_ancestry(
        &self,
        ops: &[ObservationOp],
        accepted: &[bool],
    ) -> Vec<(PathBuf, PathBuf)> {
        let mut structure = StructuralOverlay::default();
        let mut unknown = Vec::new();
        // The last directory this pass proved, with every ancestor of it. A producer
        // emits a directory's children together, so consecutive ops overwhelmingly
        // share a parent, and re-proving the same chain per op was the largest single
        // allocation cost of a cold scan (fdu-pro1): one component vector plus one
        // ancestor path rebuilt push-by-push, per entry, for an answer that had not
        // changed since the previous entry. The memo is invalidated wherever this loop
        // learns something that could change an answer -- a non-directory upsert or a
        // removal -- exactly like `ParentMemo` in the apply loop below.
        let mut proven_dir: Option<PathBuf> = None;
        let mut ancestor = PathBuf::new();
        for (observed, accepted) in ops.iter().zip(accepted) {
            if !accepted {
                continue;
            }
            match &observed.op {
                Op::Upsert { path, .. } | Op::ControlUpsert { path, .. }
                    if !path.as_os_str().is_empty() =>
                {
                    let same_proven_parent = matches!(
                        (path.parent(), proven_dir.as_deref()),
                        (Some(parent), Some(proven)) if parent == proven
                    );
                    if !same_proven_parent {
                        let mut reconcile_from = PathBuf::new();
                        let mut ancestry_known = true;
                        let parts = normalize(path).expect("prepared paths are canonical");
                        let (_, ancestors) = parts.split_last().expect("non-root path has a name");
                        ancestor.clear();
                        for part in ancestors {
                            ancestor.push(part);
                            if structure.kind(self, &ancestor) != Some(EntryKind::Dir) {
                                unknown.push((path.clone(), reconcile_from));
                                ancestry_known = false;
                                break;
                            }
                            reconcile_from.clone_from(&ancestor);
                        }
                        if !ancestry_known {
                            proven_dir = None;
                            continue;
                        }
                        match &mut proven_dir {
                            Some(proven) => {
                                proven.clear();
                                path.parent().unwrap_or(Path::new("")).clone_into(proven);
                            }
                            None => {
                                proven_dir =
                                    Some(path.parent().unwrap_or(Path::new("")).to_path_buf());
                            }
                        }
                    }
                    if let Op::Upsert { kind, .. } = &observed.op {
                        structure.upsert(self, path, *kind);
                        if !kind.is_dir() {
                            // This path may itself have been somebody's proven ancestor
                            // only if it was a directory before; the overlay knows, but
                            // the memo does not, so it forgets rather than reasons.
                            if proven_dir.as_deref().is_some_and(|proven| proven.starts_with(path))
                            {
                                proven_dir = None;
                            }
                        }
                    }
                }
                Op::Remove { path } if !path.as_os_str().is_empty() => {
                    structure.remove(self, path);
                    if proven_dir.as_deref().is_some_and(|proven| proven.starts_with(path)) {
                        proven_dir = None;
                    }
                }
                Op::Upsert { .. }
                | Op::Remove { .. }
                | Op::ControlUpsert { .. }
                | Op::ControlRemove { .. }
                | Op::InvalidateSubtree { .. } => {}
            }
        }
        unknown
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
        crate::counters::bump(|c| c.entries_allocated += 1);
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
    /// information is worth.
    ///
    /// # Two limitations, both tracked
    ///
    /// **This reports the entry's own provenance, not its subtree's.** A directory
    /// whose descendants are less trustworthy than itself will still report its own
    /// source, so a `Complete`/`Revalidated` directory can contain `Cached` children.
    /// Composition belongs in the roll-up, where it costs one merge rather than an
    /// O(subtree) walk per query, and it is not implemented yet (`fdu-fka6`,
    /// `fdu-b1ts`). Do not read a directory's provenance as a subtree guarantee.
    ///
    /// A completed reconciliation records one clocked [`StateTransition::Verified`]
    /// for its subtree, including when every entry was unchanged. Consumers of exact
    /// commits therefore observe the same provenance movement as readers of this view;
    /// the legacy [`AppliedDelta`] projection intentionally remains entry-only.
    pub fn provenance(&self, path: &Path) -> Option<Provenance> {
        let id = self.lookup(path)?;
        Some(self.provenance_of(id))
    }

    fn provenance_of(&self, id: EntryId) -> Provenance {
        let entry = self.entry(id);
        let status = self.status_of(id);
        // A completed sweep over an ancestor verified this entry even if no delta ever
        // named it, so an interval beats the entry's own stamp.
        //
        // Only while the index still considers the path fresh, though. An
        // `InvalidateSubtree` marks paths `Stale` and a running sweep marks them
        // `Reconciling`; in both cases trust has been withdrawn since the interval was
        // recorded, and promoting anyway would produce the self-contradicting answer
        // "partial, and verified".
        //
        // This applies to entries a delta *did* name, too, not only the ones it
        // skipped. Those were stamped `Revalidated` by the sweep, but their timestamp
        // would otherwise come from `observed_at`, which dates `Revalidated` to when
        // the index was constructed. One sweep would then report two different "as of"
        // times for equally verified paths — the elided siblings dated correctly to the
        // sweep, the touched entries dated to construction — and a consumer comparing
        // two rows could not tell which discipline it was reading.
        //
        // `Scanned` is excluded because it is *stronger* than `Revalidated`: a path
        // walked fresh this session is not improved by a sweep having covered it, and
        // its own scan time is already the right answer.
        if entry.source >= Source::Revalidated {
            if let Some(path) = self.path_of(id) {
                if self.freshness_at(&path) == Freshness::Fresh {
                    if let Some(verified_at) = self.verified_at(&path) {
                        return Provenance {
                            source: Source::Revalidated,
                            observed_at_ns: verified_at,
                            status,
                        };
                    }
                }
            }
        }
        Provenance { source: entry.source, observed_at_ns: self.observed_at(entry.source), status }
    }

    /// Whether this path's totals account for everything beneath it.
    ///
    /// Derived from the freshness marks rather than stored, and answering the coverage
    /// question only. `Reconciling` and `Stale` describe values whose *trust* is in
    /// doubt while their coverage is not: a cached subtree still accounts for every
    /// entry it knows about, and saying otherwise would report a complete cached
    /// baseline as if it were half-built. That distinction is [`Source`]'s job, and
    /// collapsing the two axes is what let a value that may shrink advertise itself as
    /// a lower bound that can only grow.
    ///
    /// Only [`Freshness::Partial`] — reconciliation errors left some of the subtree
    /// unread — is genuinely missing coverage.
    fn status_of(&self, id: EntryId) -> Status {
        let Some(path) = self.path_of(id) else {
            return Status::Complete;
        };
        match self.freshness_at(&path) {
            Freshness::Fresh | Freshness::Reconciling | Freshness::Stale => Status::Complete,
            Freshness::Partial => Status::Partial,
        }
    }

    /// When an entry with this source was observed.
    const fn observed_at(&self, source: Source) -> i64 {
        match source {
            Source::Cached | Source::JournalScoped => self.captured_at_ns,
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

    /// Intern an extension name and retain one file's reference to it.
    ///
    /// Every call must be matched by a [`Self::release_ext`] when that file leaves the
    /// index, which is what keeps the interner proportional to the extensions the tree
    /// currently holds rather than to every extension it has ever held.
    fn intern_ext(&mut self, name: &str) -> ExtId {
        if let Some(&id) = self.ext_ids.get(name) {
            let refcount =
                self.ext_refcounts.get_mut(id as usize).expect("a live id has a refcount");
            *refcount = refcount.checked_add(1).expect("extension refcount exhausted");
            return id;
        }
        let id = if let Some(id) = self.free_ext_ids.pop() {
            let slot = id as usize;
            self.ext_names[slot] = Some(name.to_string());
            self.ext_refcounts[slot] = 1;
            id
        } else {
            let id = ExtId::try_from(self.ext_names.len()).expect("extension interner exhausted");
            self.ext_names.push(Some(name.to_string()));
            self.ext_refcounts.push(1);
            id
        };
        self.ext_ids.insert(name.to_string(), id);
        id
    }

    /// Drop one file's reference, freeing the id and its name after the last one.
    fn release_ext(&mut self, id: ExtId) {
        let slot = id as usize;
        let refcount = self.ext_refcounts.get_mut(slot).expect("a live id has a refcount");
        debug_assert!(*refcount > 0, "extension reference released twice");
        *refcount -= 1;
        if *refcount != 0 {
            return;
        }
        let name = self.ext_names[slot].take().expect("a live id has a name");
        let removed = self.ext_ids.remove(&name);
        debug_assert_eq!(removed, Some(id), "the interner's two maps disagreed");
        self.free_ext_ids.push(id);
    }

    /// Resolve hot-path integer keys exactly once at a public query boundary.
    fn named_rollup(&self, rollup: &InternedRollUp) -> RollUp {
        let by_ext = rollup
            .by_ext
            .iter()
            .map(|(id, tally)| {
                let name = self
                    .ext_names
                    .get(*id as usize)
                    .and_then(Option::as_ref)
                    .expect("a live roll-up's extension id has a name");
                (name.clone(), *tally)
            })
            .collect();
        RollUp {
            files: rollup.files,
            dirs: rollup.dirs,
            bytes: rollup.bytes,
            allocated: rollup.allocated,
            newest_mtime_ns: rollup.newest_mtime_ns,
            by_ext,
        }
    }

    fn named_partitions(&self, rollup: &InternedPartitionRollUp) -> PartitionRollUp {
        PartitionRollUp {
            all: self.named_rollup(&rollup.all),
            unignored: self.named_rollup(&rollup.unignored),
        }
    }

    /// What an entry contributes to each of its ancestors.
    fn contribution(&self, id: EntryId) -> InternedPartitionRollUp {
        let entry = self.entry(id);
        match entry.kind {
            EntryKind::Dir => {
                let mut all = entry.rollup.all.clone();
                all.dirs += 1;
                let mut unignored = InternedRollUp::default();
                if !entry.ignored {
                    unignored = entry.rollup.unignored.clone();
                    unignored.dirs += 1;
                }
                InternedPartitionRollUp { all, unignored }
            }
            EntryKind::File => {
                let mut all = InternedRollUp {
                    files: 1,
                    dirs: 0,
                    bytes: entry.attrs.size,
                    allocated: entry.attrs.allocated,
                    newest_mtime_ns: entry.attrs.mtime_ns,
                    by_ext: BTreeMap::new(),
                };
                if let Some(ext_id) = entry.ext_id {
                    all.by_ext.insert(
                        ext_id,
                        ExtTally {
                            files: 1,
                            bytes: entry.attrs.size,
                            allocated: entry.attrs.allocated,
                        },
                    );
                }
                let unignored = if entry.ignored { InternedRollUp::default() } else { all.clone() };
                InternedPartitionRollUp { all, unignored }
            }
            EntryKind::Symlink | EntryKind::Other => InternedPartitionRollUp::default(),
        }
    }

    fn merge_upward(
        &mut self,
        from_parent: Option<EntryId>,
        contribution: &InternedPartitionRollUp,
    ) {
        let mut current = from_parent;
        while let Some(id) = current {
            // Counted per level rather than per call: the O(depth) shape is the thing
            // worth seeing, and it is what S4's bottom-up pass would collapse.
            crate::counters::bump(|c| c.rollup_merges += 1);
            let entry = self.entry_mut(id);
            entry.rollup.merge(contribution);
            current = entry.parent;
        }
    }

    fn unmerge_upward(
        &mut self,
        from_parent: Option<EntryId>,
        contribution: &InternedPartitionRollUp,
    ) {
        let mut current = from_parent;
        while let Some(id) = current {
            let entry = self.entry_mut(id);
            entry.rollup.unmerge(contribution);
            current = entry.parent;
        }
    }

    /// Rebuild `newest_mtime_ns` from direct children, walking to the root.
    ///
    /// Every ancestor must be visited even when the nearest directory is already
    /// correct. Differential unmerge/re-merge can repair a single-child directory as
    /// it goes while leaving an ancestor with other contributors holding the removed
    /// maximum. Stopping at the first unchanged directory therefore strands a stale
    /// value higher in the tree.
    fn recompute_newest_upward(&mut self, from: Option<EntryId>) {
        let mut current = from;
        while let Some(id) = current {
            let mut newest: Option<i64> = None;
            for child in self.entry(id).children.values() {
                let child_entry = self.entry(*child);
                let candidate = match child_entry.kind {
                    EntryKind::Dir => {
                        (child_entry.rollup.files > 0).then_some(child_entry.rollup.newest_mtime_ns)
                    }
                    EntryKind::File => Some(child_entry.attrs.mtime_ns),
                    EntryKind::Symlink | EntryKind::Other => None,
                };
                if let Some(candidate) = candidate {
                    newest = Some(newest.map_or(candidate, |current| current.max(candidate)));
                }
            }
            let newest = newest.unwrap_or(0);
            self.entry_mut(id).rollup.all.newest_mtime_ns = newest;

            let mut newest_unignored: Option<i64> = None;
            for child in self.entry(id).children.values() {
                let child_entry = self.entry(*child);
                let candidate = match child_entry.kind {
                    EntryKind::Dir => (!child_entry.ignored
                        && child_entry.rollup.unignored.files > 0)
                        .then_some(child_entry.rollup.unignored.newest_mtime_ns),
                    EntryKind::File => (!child_entry.ignored).then_some(child_entry.attrs.mtime_ns),
                    EntryKind::Symlink | EntryKind::Other => None,
                };
                if let Some(candidate) = candidate {
                    newest_unignored =
                        Some(newest_unignored.map_or(candidate, |current| current.max(candidate)));
                }
            }
            let entry = self.entry_mut(id);
            entry.rollup.unignored.newest_mtime_ns = newest_unignored.unwrap_or(0);
            current = entry.parent;
        }
    }

    /// Resolve a parent chain already proved by [`Self::validate_known_ancestry`].
    fn resolve_dir_chain(&self, parts: &[&OsStr]) -> EntryId {
        let mut current = EntryId::ROOT;
        for part in parts {
            current = *self
                .entry(current)
                .children
                .get(*part)
                .expect("validated ancestry remains present under the writer lock");
            debug_assert!(self.entry(current).kind.is_dir());
        }
        current
    }

    fn apply_upsert(
        &mut self,
        path: &Path,
        kind: EntryKind,
        attrs: Attrs,
        stats: &mut ApplyStats,
        effects: &mut MutationEffects,
        parent_memo: &mut ParentMemo,
    ) -> bool {
        // A walker reports a directory's children consecutively, because that is the
        // order one `getdents64` batch hands them over, so the parent resolved for the
        // previous entry is almost always the parent of this one. Checking that first
        // turns the common case into a single path comparison and skips both the
        // component vector and the descent below.
        crate::counters::bump(|c| c.upserts += 1);
        if let (Some(dir), Some(name)) = (path.parent(), path.file_name()) {
            if let Some(parent) = parent_memo.get(dir) {
                crate::counters::bump(|c| c.parent_memo_hits += 1);
                return self.upsert_beneath(parent, name, path, kind, attrs, stats, effects);
            }
        }
        crate::counters::bump(|c| c.parent_resolutions += 1);

        let Some(parts) = normalize(path) else {
            return false;
        };
        let source = self.applying_source;

        let Some((name, ancestors)) = parts.split_last() else {
            // The root itself: only its own attributes can change. Its source is
            // stamped on both paths for the same reason every other entry's is — a
            // producer just looked at it — and the root is the entry where getting this
            // wrong costs the most, because the whole-tree totals hang off it and a
            // consumer reads its provenance to label the headline number.
            if self.entry(EntryId::ROOT).attrs == attrs {
                self.entry_mut(EntryId::ROOT).source = source;
                stats.unchanged += 1;
                return false;
            }
            let root = self.entry_mut(EntryId::ROOT);
            let previous = root.attrs;
            root.attrs = attrs;
            root.source = source;
            Self::bump_revision(root);
            stats.updated += 1;
            effects.changes.push(EffectiveChange::Updated {
                path: PathBuf::new(),
                kind: EntryKind::Dir,
                previous,
                current: attrs,
            });
            return true;
        };
        let parent = self.resolve_dir_chain(ancestors);
        if let Some(dir) = path.parent() {
            parent_memo.set(dir, parent);
        }
        self.upsert_beneath(parent, name, path, kind, attrs, stats, effects)
    }

    /// Apply one upsert beneath a parent whose id is already resolved.
    ///
    /// This is the whole of [`apply_upsert`] except for finding the parent, split out so
    /// that the memoized and the resolved paths share one body rather than two copies of
    /// the arbitration rules.  Every guard the delta contract requires still runs here:
    /// the caller has supplied a parent, not a decision.
    #[allow(clippy::too_many_arguments)]
    fn upsert_beneath(
        &mut self,
        parent: EntryId,
        name: &OsStr,
        path: &Path,
        kind: EntryKind,
        attrs: Attrs,
        stats: &mut ApplyStats,
        effects: &mut MutationEffects,
    ) -> bool {
        let source = self.applying_source;
        let existing = self.entry(parent).children.get(name).copied();

        if let Some(id) = existing {
            let entry = self.entry(id);
            if entry.kind == kind {
                if entry.attrs == attrs {
                    // Nothing about the value changed, but a producer just looked at
                    // it, and that is exactly what provenance records. Without this an
                    // entry verified by a revalidation sweep keeps reporting the source
                    // it was loaded with, and a consumer could never clear a
                    // stale-value indicator no matter how much checking happened.
                    self.entry_mut(id).source = source;
                    stats.unchanged += 1;
                    return false;
                }
                if kind.is_dir() {
                    // A directory's own attributes do not reach its ancestors' roll-ups,
                    // so there is nothing to re-merge.
                    let entry = self.entry_mut(id);
                    let previous = entry.attrs;
                    entry.attrs = attrs;
                    entry.source = source;
                    Self::bump_revision(entry);
                    stats.updated += 1;
                    effects.changes.push(EffectiveChange::Updated {
                        path: path.to_path_buf(),
                        kind,
                        previous,
                        current: attrs,
                    });
                    return true;
                }
                let previous_attrs = entry.attrs;
                self.invalidate_content(path);
                self.remove_serving_file_semantics(path, id, previous_attrs);
                self.remove_serving_entry(path, kind, previous_attrs, id);
                let old = self.contribution(id);
                self.unmerge_upward(Some(parent), &old);
                let entry = self.entry_mut(id);
                let previous = entry.attrs;
                entry.attrs = attrs;
                entry.source = source;
                Self::bump_revision(entry);
                let new = self.contribution(id);
                self.merge_upward(Some(parent), &new);
                self.insert_serving_entry(path, kind, attrs, id);
                if new.newest_mtime_ns < old.newest_mtime_ns {
                    self.recompute_newest_upward(Some(parent));
                }
                stats.updated += 1;
                effects.changes.push(EffectiveChange::Updated {
                    path: path.to_path_buf(),
                    kind,
                    previous,
                    current: attrs,
                });
                return true;
            }
            // The kind changed (a file became a directory, say). Remove and re-insert
            // rather than trying to mutate one shape into the other.
            //
            // This drops a subtree but cannot invalidate the memo: the memo holds this
            // entry's *parent*, and the subtree removed is rooted at the entry itself.
            // Clearing here would be untestable defensive code, which reads as a hazard
            // that does not exist.
            self.remove_entry(id, stats, effects);
        }

        let ext_id =
            (kind == EntryKind::File).then(|| self.intern_ext(&self.types.ext_bucket(name)));
        let ignored =
            self.entry(parent).ignored || self.controls.matcher_for(path).is_ignored(kind.is_dir());
        let id = self.alloc(Entry {
            parent: Some(parent),
            name: name.to_os_string(),
            ext_id,
            ignored,
            source,
            kind,
            attrs,
            children: BTreeMap::new(),
            rollup: InternedPartitionRollUp::default(),
            revision: 0,
            children_revision: 0,
            children_complete: kind != EntryKind::Dir,
        });
        self.insert_child(parent, name.to_os_string(), id);
        let contribution = self.contribution(id);
        self.merge_upward(Some(parent), &contribution);
        stats.inserted += 1;
        effects.changes.push(EffectiveChange::Inserted { path: path.to_path_buf(), kind, attrs });
        self.insert_serving_entry(path, kind, attrs, id);
        true
    }

    /// Insert one snapshot record beneath a parent whose id the caller already holds.
    ///
    /// The snapshot loader is not a producer.  It restores state that the delta contract
    /// already arbitrated and serialized, in the order it was written, with parents
    /// always preceding their children — so every fact [`apply_upsert`] rediscovers by
    /// resolving a path is a fact the loader was handed.  Routing it through the
    /// observation path made the loader pay, per record, a `PathBuf` join, an
    /// `Observation` vector, a `normalize` vector, and a descent from the root through
    /// one `BTreeMap` lookup per level, to arrive at a parent it had in a local variable.
    /// A callgrind profile of a 450k-entry load put the allocator at about 27% of the
    /// work and path-component iteration at about 15%; this removes both.
    ///
    /// It stays `pub(crate)` and takes an `EntryId` rather than a path precisely so it
    /// cannot become a second mutation surface: no external producer can reach it, and
    /// the guarantee that a loaded index equals the saved one is enforced by round-trip
    /// tests rather than by making deserialization impersonate a producer.
    ///
    /// Returns `None` when the parent is not a live directory or already holds `name`,
    /// which is how a corrupt snapshot fails closed.
    pub(crate) fn insert_loaded_child(
        &mut self,
        parent: EntryId,
        name: OsString,
        kind: EntryKind,
        attrs: Attrs,
    ) -> Option<EntryId> {
        let parent_entry = self.try_entry(parent)?;
        if parent_entry.kind != EntryKind::Dir || parent_entry.children.contains_key(&name) {
            return None;
        }
        let source = self.applying_source;
        let ext_id =
            (kind == EntryKind::File).then(|| self.intern_ext(&self.types.ext_bucket(&name)));
        let id = self.alloc(Entry {
            parent: Some(parent),
            name: name.clone(),
            ext_id,
            ignored: false,
            source,
            kind,
            attrs,
            children: BTreeMap::new(),
            rollup: InternedPartitionRollUp::default(),
            revision: 0,
            children_revision: 0,
            children_complete: true,
        });
        self.insert_child(parent, name, id);
        // Roll-ups stay eager. The same profile put `merge_upward` at about 3.5%, so
        // deferring it to a bottom-up pass would buy little and would introduce a window
        // in which the index is structurally complete but numerically wrong.
        let contribution = self.contribution(id);
        self.merge_upward(Some(parent), &contribution);
        let path = self.path_of(id).expect("a newly loaded entry has a path");
        self.insert_serving_entry(&path, kind, attrs, id);
        Some(id)
    }

    fn apply_remove(
        &mut self,
        path: &Path,
        stats: &mut ApplyStats,
        effects: &mut MutationEffects,
    ) -> bool {
        let Some(id) = self.lookup(path) else {
            stats.unchanged += 1;
            return false;
        };
        if id == EntryId::ROOT {
            stats.unchanged += 1;
            return false;
        }
        self.remove_entry(id, stats, effects);
        true
    }

    fn remove_entry(&mut self, id: EntryId, stats: &mut ApplyStats, effects: &mut MutationEffects) {
        let removed_root = self.path_of(id).expect("a live entry has a path");
        self.invalidate_content(&removed_root);
        self.remove_serving_subtree_semantics(id, &removed_root);
        let parent = self.entry(id).parent;
        let name = self.entry(id).name.clone();
        let contribution = self.contribution(id);

        self.unmerge_upward(parent, &contribution);
        if let Some(parent) = parent {
            self.remove_child(parent, &name);
        }

        // Free the subtree iteratively; a recursive drop would blow the stack on deep
        // trees, which is exactly the shape this engine is built for.
        let mut queue = VecDeque::from([(id, removed_root)]);
        while let Some((node, path)) = queue.pop_front() {
            let entry = self.entry(node);
            let kind = entry.kind;
            let attrs = entry.attrs;
            let children: Vec<(OsString, EntryId)> =
                entry.children.iter().map(|(name, child)| (name.clone(), *child)).collect();
            let ext_id = entry.ext_id;
            for (name, child) in children {
                queue.push_back((child, path.join(name)));
            }
            self.remove_serving_entry(&path, kind, attrs, node);
            effects.changes.push(EffectiveChange::Removed { path, kind, attrs });
            // Give the extension back before the entry itself goes, so the interner
            // holds only what the tree still contains.
            if let Some(ext_id) = ext_id {
                self.release_ext(ext_id);
            }
            self.free(node);
            stats.removed += 1;
        }

        // The max may have lived in what was just removed.
        self.recompute_newest_upward(parent);
    }

    fn invalidate_content(&mut self, path: &Path) {
        if let Some(content) = self.content.as_mut() {
            content.invalidate(path);
        }
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
/// The parent directory resolved for the previous upsert in a batch.
///
/// A walker reports a directory's children consecutively, so resolving the parent path
/// once per directory rather than once per entry removes the dominant cost of applying a
/// cold scan: a callgrind profile attributed about 25 path-component comparisons per
/// entry to the descent, and the component vector `normalize` builds is an allocation
/// per entry on top of that.
///
/// It is a single slot rather than a map on purpose.  A map would keep entries alive
/// across structural changes and turn every miss into a hash, where consecutive runs are
/// what the walker actually produces; one slot captures those and costs a path
/// comparison when it misses.  The slot holds an id, so it must be cleared whenever a
/// removal could unmake it — [`Index::apply_remove`], an invalidation, and the
/// kind-change removal inside an upsert all do.
#[derive(Default)]
struct ParentMemo {
    entry: Option<(PathBuf, EntryId)>,
}

impl ParentMemo {
    /// The id remembered for `dir`, if the last resolved parent was that directory.
    fn get(&self, dir: &Path) -> Option<EntryId> {
        self.entry.as_ref().filter(|(cached, _)| cached == dir).map(|&(_, id)| id)
    }

    fn set(&mut self, dir: &Path, id: EntryId) {
        match &mut self.entry {
            // Overwriting in place keeps this to one allocation per directory rather
            // than one per run, which matters because a wide tree alternates often.
            Some((cached, cached_id)) => {
                cached.clear();
                cached.push(dir);
                *cached_id = id;
            }
            slot => *slot = Some((dir.to_path_buf(), id)),
        }
    }

    fn clear(&mut self) {
        self.entry = None;
    }
}

/// Structural effects of accepted operations evaluated before the real mutation.
#[derive(Default)]
struct StructuralOverlay {
    entries: BTreeMap<PathBuf, EntryKind>,
    removed_roots: Vec<PathBuf>,
}

impl StructuralOverlay {
    fn kind(&self, index: &Index, path: &Path) -> Option<EntryKind> {
        self.entries.get(path).copied().or_else(|| {
            (!self.removed_roots.iter().any(|removed| path.starts_with(removed)))
                .then(|| index.kind(path))
                .flatten()
        })
    }

    fn upsert(&mut self, index: &Index, path: &Path, kind: EntryKind) {
        if self.kind(index, path).is_some_and(|current| current != kind) {
            self.remove(index, path);
        }
        self.entries.insert(path.to_path_buf(), kind);
    }

    fn remove(&mut self, index: &Index, path: &Path) {
        if self.kind(index, path).is_none() {
            return;
        }
        self.entries.retain(|candidate, _| !candidate.starts_with(path));
        self.removed_roots.retain(|candidate| !candidate.starts_with(path));
        if !self.removed_roots.iter().any(|removed| path.starts_with(removed)) {
            self.removed_roots.push(path.to_path_buf());
        }
    }
}

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

/// Whether a path is relative and never ascends, so joining it cannot leave a base.
///
/// Every component must be `Normal` or `CurDir`. That rejects `..`, a root, and a
/// Windows prefix, which is both what an index keyed on relative paths can store and
/// what keeps a joined path inside the directory it was joined to.
///
/// Deliberately *not* named for representability. This project already uses
/// "representable" for a different question — whether a native path has a canonical
/// UTF-8 portable form, which is what decides whether an entry appears in portable
/// projections. That predicate is [`crate::opened::read::portable_path`]. A path can
/// satisfy one and fail the other in either direction, so one word for both invites
/// exactly the confusion it caused.
///
/// The same rule as [`normalize`] without building anything: validation asks a yes or
/// no question, and answering it by constructing a component list and dropping it was
/// pure allocation.
pub(crate) fn path_is_relative_normal(path: &Path) -> bool {
    path.components().all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn prepare_observation(observation: &Observation) -> crate::Result<PreparedObservation> {
    let mut ops = Vec::with_capacity(observation.len());
    for observed in &observation.ops {
        let path = canonical_relative_path(observed.op.path())?;
        let op = match &observed.op {
            Op::Upsert { kind, attrs, .. } => Op::Upsert { path, kind: *kind, attrs: *attrs },
            Op::Remove { .. } => Op::Remove { path },
            Op::ControlUpsert { source, .. } => {
                if !crate::control::is_control_file(&path) {
                    return Err(crate::Error::InvalidControlPath(path));
                }
                Op::ControlUpsert { path, source: source.clone() }
            }
            Op::ControlRemove { .. } => {
                if !crate::control::is_control_file(&path) {
                    return Err(crate::Error::InvalidControlPath(path));
                }
                Op::ControlRemove { path }
            }
            Op::InvalidateSubtree { reason, .. } => Op::InvalidateSubtree { path, reason: *reason },
        };
        ops.push(ObservationOp { op, expectation: observed.expectation });
    }
    Ok(PreparedObservation {
        ops,
        #[cfg(test)]
        reject_before_apply: false,
    })
}

fn canonical_relative_path(path: &Path) -> crate::Result<PathBuf> {
    if !path_is_relative_normal(path) {
        return Err(crate::Error::PathEscapesRoot(path.to_path_buf()));
    }
    // A path whose every component is `Normal` rebuilds to itself, so copy it in one
    // allocation instead of reconstructing it component-by-component -- `collect` on a
    // `PathBuf` grows by repeated push, which showed up as the dominant reallocation on
    // whole-scan profiles (fdu-pro1). Every path fdu's own walker produces takes this
    // lane; only input that actually contains `.` pays the rebuild.
    if path.components().all(|component| matches!(component, Component::Normal(_))) {
        return Ok(path.to_path_buf());
    }
    Ok(normalize(path).expect("representable paths normalize").into_iter().collect::<PathBuf>())
}

fn derive_impact(changes: &[EffectiveChange], state: &[StateTransition]) -> Impact {
    let mut domains = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut all_dirty = false;

    for change in changes {
        match change {
            EffectiveChange::Inserted { .. } | EffectiveChange::Removed { .. } => {
                domains.extend([
                    ImpactDomain::Topology,
                    ImpactDomain::Metadata,
                    ImpactDomain::Classification,
                    ImpactDomain::Aggregates,
                    ImpactDomain::Content,
                ]);
            }
            EffectiveChange::Updated { .. } => {
                domains.extend([
                    ImpactDomain::Metadata,
                    ImpactDomain::Aggregates,
                    ImpactDomain::Content,
                ]);
            }
            EffectiveChange::ControlUpdated { .. } | EffectiveChange::Reclassified { .. } => {
                domains.extend([ImpactDomain::Classification, ImpactDomain::Aggregates]);
            }
            EffectiveChange::Invalidated { .. } => {
                domains.insert(ImpactDomain::State);
            }
        }
        insert_dirty_ancestors(change.path(), &mut paths, &mut all_dirty);
    }
    for transition in state {
        domains.insert(ImpactDomain::State);
        insert_dirty_ancestors(transition.path(), &mut paths, &mut all_dirty);
    }

    Impact {
        domains: domains.into_iter().collect(),
        dirty_paths: if all_dirty { Vec::new() } else { paths.into_iter().collect() },
        all_dirty,
    }
}

fn commit_work(observations: u64, stats: ApplyStats) -> Work {
    Work {
        observations,
        unchanged: stats.unchanged,
        stale: stats.stale,
        resource_refused: stats.resource_refused,
        ..Work::default()
    }
}

fn insert_dirty_ancestors(path: &Path, paths: &mut BTreeSet<PathBuf>, all_dirty: &mut bool) {
    if *all_dirty {
        return;
    }
    for ancestor in path.ancestors() {
        paths.insert(ancestor.to_path_buf());
        if paths.len() > MAX_DIRTY_PATHS {
            paths.clear();
            *all_dirty = true;
            return;
        }
    }
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
    use crate::engine_contract::ObservationOp;
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

    fn assert_serving_indexes(index: &Index) {
        let serving = index.serving.as_ref().expect("test index has serving state");
        let mut entries = BTreeMap::new();
        let mut children = BTreeMap::<PathBuf, PortableChildren>::new();
        let mut recent_files = BTreeSet::new();
        let mut semantic_by_directory =
            BTreeMap::<EntryId, (BTreeMap<String, ExtTally>, BTreeMap<String, ExtTally>)>::new();
        let declared_exact_names: BTreeSet<_> =
            index.types.exact_filenames().map(str::to_ascii_lowercase).collect();
        let mut exact_name_by_directory =
            BTreeMap::<EntryId, (BTreeMap<String, ExtTally>, BTreeMap<String, ExtTally>)>::new();
        let mut semantic_refcounts = BTreeMap::<String, u64>::new();
        let mut pending = vec![(EntryId::ROOT, PathBuf::new(), vec![EntryId::ROOT])];
        while let Some((parent_id, parent_path, ancestors)) = pending.pop() {
            let facts: Vec<_> = index
                .children_of(parent_id)
                .expect("live directory")
                .map(|(name, id)| (name.to_os_string(), id))
                .collect();
            for (name, id) in facts {
                let path = parent_path.join(&name);
                let kind = index.kind_of(id).expect("live child");
                let portable = crate::opened::read::portable_path(&path);
                entries.insert(portable.clone(), id);
                if kind == EntryKind::File {
                    recent_files.insert(RecentKey {
                        mtime_ns: index.attrs(&path).expect("live child has attributes").mtime_ns,
                        portable_path: portable,
                        id,
                    });
                }
                if kind == EntryKind::File {
                    let semantic = index.classify(&path).file_type.as_str().to_string();
                    *semantic_refcounts.entry(semantic.clone()).or_default() += 1;
                    let attrs = *index.attrs(&path).expect("live child has attributes");
                    let ignored = index.is_ignored(&path).expect("live child is classified");
                    for ancestor in &ancestors {
                        let partition = semantic_by_directory.entry(*ancestor).or_default();
                        let all = partition.0.entry(semantic.clone()).or_default();
                        all.files += 1;
                        all.bytes += attrs.size;
                        all.allocated += attrs.allocated;
                        if !ignored {
                            let unignored = partition.1.entry(semantic.clone()).or_default();
                            unignored.files += 1;
                            unignored.bytes += attrs.size;
                            unignored.allocated += attrs.allocated;
                        }
                    }
                    if let Some(exact_name) = name
                        .to_str()
                        .map(str::to_ascii_lowercase)
                        .filter(|name| declared_exact_names.contains(name))
                    {
                        for ancestor in &ancestors {
                            let partition = exact_name_by_directory.entry(*ancestor).or_default();
                            let all = partition.0.entry(exact_name.clone()).or_default();
                            all.files += 1;
                            all.bytes += attrs.size;
                            all.allocated += attrs.allocated;
                            if !ignored {
                                let unignored = partition.1.entry(exact_name.clone()).or_default();
                                unignored.files += 1;
                                unignored.bytes += attrs.size;
                                unignored.allocated += attrs.allocated;
                            }
                        }
                    }
                }
                let partition = children.entry(parent_path.clone()).or_default();
                let portable_name = crate::opened::read::portable_component(&name);
                if kind.is_dir() {
                    partition.directories.insert(portable_name, id);
                } else {
                    partition.nondirectories.insert(portable_name, id);
                }
                if kind.is_dir() {
                    let mut child_ancestors = ancestors.clone();
                    child_ancestors.insert(0, id);
                    pending.push((id, path, child_ancestors));
                }
            }
        }

        assert_eq!(serving.portable_entries, entries);
        assert_eq!(serving.recent_files, recent_files);
        let actual_semantics: BTreeMap<_, _> = serving
            .semantic_by_directory
            .iter()
            .map(|(directory, partitions)| {
                let named = |source: &BTreeMap<u32, ExtTally>| {
                    source
                        .iter()
                        .map(|(semantic, tally)| {
                            let name = serving.semantic_names[*semantic as usize]
                                .as_ref()
                                .expect("live semantic has a name")
                                .clone();
                            (name, *tally)
                        })
                        .collect()
                };
                (*directory, (named(&partitions.all), named(&partitions.unignored)))
            })
            .collect();
        assert_eq!(actual_semantics, semantic_by_directory);
        let actual_exact_names: BTreeMap<_, _> = serving
            .exact_name_by_directory
            .iter()
            .map(|(directory, partitions)| {
                let named = |source: &BTreeMap<u32, ExtTally>| {
                    source
                        .iter()
                        .map(|(exact_name, tally)| {
                            (serving.exact_names[*exact_name as usize].clone(), *tally)
                        })
                        .collect()
                };
                (*directory, (named(&partitions.all), named(&partitions.unignored)))
            })
            .collect();
        assert_eq!(actual_exact_names, exact_name_by_directory);
        assert_eq!(
            serving.exact_names.iter().cloned().collect::<BTreeSet<_>>(),
            declared_exact_names
        );
        assert_eq!(
            serving.exact_name_ids,
            serving
                .exact_names
                .iter()
                .enumerate()
                .map(|(position, name)| {
                    (
                        name.clone(),
                        u32::try_from(position).expect("the exact-name vocabulary fits u32"),
                    )
                })
                .collect()
        );
        assert!(serving.exact_name_by_directory.len() <= index.arena.len());
        assert!(serving.exact_name_by_directory.values().all(|partitions| {
            partitions.all.len() <= serving.exact_names.len()
                && partitions.unignored.len() <= serving.exact_names.len()
                && partitions
                    .all
                    .keys()
                    .chain(partitions.unignored.keys())
                    .all(|name| (*name as usize) < serving.exact_names.len())
        }));
        let actual_refcounts: BTreeMap<_, _> = serving
            .semantic_ids
            .iter()
            .map(|(name, semantic)| (name.clone(), serving.semantic_refcounts[*semantic as usize]))
            .collect();
        assert_eq!(actual_refcounts, semantic_refcounts);
        assert_eq!(serving.portable_children, children);

        // Every retained entry has a portable name, and the names are unique. The second
        // half is what the escaping has to earn: `%` is escaped in every name precisely so
        // a file called `x%FF` and one whose bytes are `x\xff` cannot collide here.
        assert_eq!(
            u64::try_from(serving.portable_entries.len()).expect("entry count fits u64"),
            index.len().saturating_sub(1),
            "every retained non-root entry has exactly one portable name"
        );
    }

    #[test]
    fn portable_indexes_conserve_insert_kind_change_and_subtree_removal() {
        let mut index = Index::new_opened_with_scope_types_and_journal_capacity(
            "/root",
            ScanScope::default(),
            crate::classify::TypeRegistry::compiled_shared(),
            DEFAULT_JOURNAL_CAPACITY,
        );
        index.apply_ok(&Observation::new(vec![
            upsert("dir", EntryKind::Dir, Attrs::default()),
            upsert("dir/a", EntryKind::File, file_attrs(1, 1)),
            upsert("replace", EntryKind::File, file_attrs(2, 2)),
        ]));
        assert_serving_indexes(&index);

        index.apply_ok(&Observation::new(vec![
            upsert("replace", EntryKind::Dir, Attrs::default()),
            upsert("replace/child", EntryKind::File, file_attrs(3, 3)),
        ]));
        assert_serving_indexes(&index);

        index.apply_ok(&Observation::new(vec![upsert("dir/a", EntryKind::File, file_attrs(4, 9))]));
        assert_serving_indexes(&index);
        assert_eq!(
            index
                .serving
                .as_ref()
                .expect("opened test index")
                .recent_files
                .iter()
                .map(|entry| entry.portable_path.as_str())
                .collect::<Vec<_>>(),
            vec!["dir/a", "replace/child"]
        );

        index.apply_ok(&Observation::new(vec![Op::Remove { path: PathBuf::from("replace") }]));
        assert_serving_indexes(&index);
        assert_eq!(
            index.portable_entries().keys().map(crate::PortablePath::as_str).collect::<Vec<_>>(),
            vec!["dir", "dir/a"]
        );
    }

    /// Escaping touches exactly two things and leaves everything else byte-identical.
    ///
    /// The rule is narrow on purpose: a byte that is not valid UTF-8, and `%` itself.
    /// Everything else — spaces, non-ASCII scalars, punctuation — passes through, because
    /// this produces a JSON string rather than a URL and mangling readable names would be
    /// a cost with no benefit.
    ///
    /// This test used to assert the opposite property, that the derived name could be
    /// turned back into a filesystem path with `PathBuf::from`. That held only while the
    /// derivation was the identity, and it is now unsound: `100%.txt` derives to
    /// `100%25.txt`, which names no file. The conversion was deleted rather than kept
    /// working, and callers ask the arena for a native path instead.
    #[test]
    fn escaping_touches_only_invalid_bytes_and_percent() {
        let mut index = Index::new_opened_with_scope_types_and_journal_capacity(
            "/root",
            ScanScope::default(),
            crate::classify::TypeRegistry::compiled_shared(),
            DEFAULT_JOURNAL_CAPACITY,
        );
        index.apply_ok(&Observation::new(vec![
            upsert("dir", EntryKind::Dir, Attrs::default()),
            upsert("dir/plain.txt", EntryKind::File, file_attrs(1, 1)),
            upsert("café", EntryKind::Dir, Attrs::default()),
            upsert("café/naïve.txt", EntryKind::File, file_attrs(2, 2)),
            upsert("日本語.md", EntryKind::File, file_attrs(3, 3)),
            upsert("a b", EntryKind::Dir, Attrs::default()),
            upsert("a b/c d.txt", EntryKind::File, file_attrs(5, 5)),
            upsert("100%.txt", EntryKind::File, file_attrs(4, 4)),
        ]));

        let names: Vec<_> =
            index.portable_entries().keys().map(crate::PortablePath::as_str).collect();
        assert_eq!(
            names,
            vec![
                "100%25.txt",
                "a b",
                "a b/c d.txt",
                "café",
                "café/naïve.txt",
                "dir",
                "dir/plain.txt",
                "日本語.md",
            ],
            "only the literal percent is rewritten; separators, spaces and non-ASCII are not"
        );
    }

    #[test]
    fn declared_exact_names_roll_up_by_ancestor_and_partition() {
        let types = Arc::new(
            crate::classify::TypeRegistry::from_manifest(
                "[[kind]]\nid = \"make\"\nfamily = \"code\"\nfilenames = [\"Makefile\"]\n",
            )
            .expect("custom registry"),
        );
        let scope =
            ScanScope { type_rules_fingerprint: types.fingerprint(), ..ScanScope::default() };
        let mut index = Index::new_opened_with_scope_types_and_journal_capacity(
            "/root",
            scope,
            types,
            DEFAULT_JOURNAL_CAPACITY,
        );
        index.apply_ok(&Observation::new(vec![
            upsert("Makefile", EntryKind::File, file_attrs(2, 1)),
            upsert("dir", EntryKind::Dir, Attrs::default()),
            upsert("dir/makefile", EntryKind::File, file_attrs(3, 2)),
            upsert("dir/notes", EntryKind::File, file_attrs(5, 3)),
        ]));

        let serving = index.serving.as_ref().expect("opened test index");
        let exact_name = serving.exact_name_ids["makefile"];
        let root = &serving.exact_name_by_directory[&EntryId::ROOT];
        assert_eq!(root.all[&exact_name], ExtTally { files: 2, bytes: 5, allocated: 1_024 });
        assert_eq!(root.unignored, root.all);

        let directory = index.lookup(Path::new("dir")).expect("directory");
        let nested = &serving.exact_name_by_directory[&directory];
        assert_eq!(nested.all[&exact_name], ExtTally { files: 1, bytes: 3, allocated: 512 });
        assert_eq!(nested.unignored, nested.all);
    }

    #[test]
    #[ignore = "manual opened-root commit-cost evidence"]
    fn measure_opened_serving_commit_cost() {
        const DIRECTORY_COUNT: usize = 100;
        const FILES_PER_DIRECTORY: usize = 100;
        const SAMPLE_COUNT: usize = 7;

        let mut operations = Vec::with_capacity(
            DIRECTORY_COUNT.saturating_mul(FILES_PER_DIRECTORY.saturating_add(1)),
        );
        for directory in 0..DIRECTORY_COUNT {
            let parent = format!("d{directory:03}");
            operations.push(upsert(&parent, EntryKind::Dir, Attrs::default()));
            for file in 0..FILES_PER_DIRECTORY {
                let size = u64::try_from(file).expect("the probe file count fits u64") + 1;
                let mtime = i64::try_from(file).expect("the probe file count fits i64");
                let name = if file == 0 {
                    format!("{parent}/Makefile")
                } else {
                    format!("{parent}/f{file:03}.rs")
                };
                operations.push(upsert(&name, EntryKind::File, file_attrs(size, mtime)));
            }
        }
        let observation = Observation::new(operations);
        let types = crate::classify::TypeRegistry::compiled_shared();
        let scope =
            ScanScope { type_rules_fingerprint: types.fingerprint(), ..ScanScope::default() };
        let measure = |opened: bool| {
            let mut index = if opened {
                Index::new_opened_with_scope_types_and_journal_capacity(
                    "/root",
                    scope,
                    Arc::clone(&types),
                    DEFAULT_JOURNAL_CAPACITY,
                )
            } else {
                Index::new_with_scope_types_and_journal_capacity(
                    "/root",
                    scope,
                    Arc::clone(&types),
                    DEFAULT_JOURNAL_CAPACITY,
                )
            };
            let started = std::time::Instant::now();
            index.apply_ok(&observation);
            let elapsed = started.elapsed();
            std::hint::black_box(index.len());
            (elapsed, index)
        };

        let _ = measure(false);
        let _ = measure(true);
        let mut detached = Vec::with_capacity(SAMPLE_COUNT);
        let mut opened = Vec::with_capacity(SAMPLE_COUNT);
        let mut last_opened = None;
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                detached.push(measure(false).0);
                let (duration, index) = measure(true);
                opened.push(duration);
                last_opened = Some(index);
            } else {
                let (duration, index) = measure(true);
                opened.push(duration);
                last_opened = Some(index);
                detached.push(measure(false).0);
            }
        }
        detached.sort_unstable();
        opened.sort_unstable();
        let detached_median = detached[SAMPLE_COUNT / 2];
        let opened_median = opened[SAMPLE_COUNT / 2];
        let ratio = opened_median.as_secs_f64() / detached_median.as_secs_f64();

        let index = last_opened.expect("an opened sample ran");
        let serving = index.serving.as_ref().expect("opened sample has serving indexes");
        let semantic_rows: usize = serving
            .semantic_by_directory
            .values()
            .map(|partitions| partitions.all.len() + partitions.unignored.len())
            .sum();
        let exact_name_rows: usize = serving
            .exact_name_by_directory
            .values()
            .map(|partitions| partitions.all.len() + partitions.unignored.len())
            .sum();
        eprintln!(
            "entries={} detached_median_us={} opened_median_us={} ratio={ratio:.3} \
             portable_rows={} child_rows={} recent_rows={} semantic_rows={} exact_name_rows={} \
             exact_name_vocabulary={}",
            index.len(),
            detached_median.as_micros(),
            opened_median.as_micros(),
            serving.portable_entries.len(),
            serving
                .portable_children
                .values()
                .map(|children| children.directories.len() + children.nondirectories.len())
                .sum::<usize>(),
            serving.recent_files.len(),
            semantic_rows,
            exact_name_rows,
            serving.exact_names.len(),
        );
    }

    #[test]
    fn detached_indexes_never_allocate_or_populate_serving_state() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("dir", EntryKind::Dir, Attrs::default()),
            upsert("dir/a", EntryKind::File, file_attrs(1, 1)),
        ]));

        assert!(index.serving.is_none());
        assert!(index.portable_children(Path::new("dir")).is_none());
    }

    #[test]
    fn opened_entry_values_project_name_identity_without_retaining_it_on_detached_entries() {
        let mut index = Index::new_opened_with_scope_types_and_journal_capacity(
            "/root",
            ScanScope::default(),
            crate::classify::TypeRegistry::compiled_shared(),
            DEFAULT_JOURNAL_CAPACITY,
        );
        index.apply_ok(&Observation::new(vec![upsert(
            "bundle.umd.min.js",
            EntryKind::File,
            file_attrs(7, 1),
        )]));

        let row = index.entry_value(Path::new("bundle.umd.min.js")).expect("entry value");
        let identity = row.classification.expect("regular files carry name identity");
        assert_eq!(identity.logical_extension(), Some(".min.js"));
        assert_eq!(identity.canonical_extension(), Some(".js"));
        assert_eq!(identity.kind_id(), Some("javascript"));
        assert_eq!(identity.content_family(), crate::classify::ContentFamily::Code);
    }

    #[test]
    fn a_shared_snapshot_drops_opened_root_serving_state() {
        let mut index = Index::new_opened_with_scope_types_and_journal_capacity(
            "/root",
            ScanScope::default(),
            crate::classify::TypeRegistry::compiled_shared(),
            DEFAULT_JOURNAL_CAPACITY,
        );
        index.apply_ok(&Observation::new(vec![upsert("a.txt", EntryKind::File, file_attrs(1, 1))]));
        assert!(index.serving_indexes_enabled());

        let snapshot = IndexHandle::new(index).snapshot().expect("detached snapshot");

        assert!(!snapshot.serving_indexes_enabled());
        assert_eq!(snapshot.total().files, 1);
    }

    #[test]
    #[should_panic(expected = "an index's registry must match its semantic scope")]
    fn an_index_cannot_claim_a_registry_different_from_its_scope() {
        let types = Arc::new(
            crate::classify::TypeRegistry::from_manifest(
                "[[kind]]\nid = \"notes\"\nfamily = \"prose\"\nextensions = [\"rs\"]\n",
            )
            .expect("custom registry"),
        );

        let _ = Index::new_with_scope_and_types("/root", ScanScope::default(), types);
    }

    /// The parent memo skips resolving a path when consecutive upserts share a parent,
    /// so every test below puts the op that could invalidate it *between* two upserts
    /// into the same directory — the arrangement where a stale hit would be believed.
    /// A memo that never cleared would still pass an ordinary scan-shaped workload,
    /// which is why these are written as batches rather than as separate applies: one
    /// `apply_validated` call is the memo's whole lifetime.
    #[test]
    fn parent_memo_does_not_survive_removing_the_directory_it_remembers() {
        let mut index = Index::new(PathBuf::from("/root"));
        index.apply_ok(&Observation::new(vec![
            upsert("dir", EntryKind::Dir, file_attrs(0, 1)),
            upsert("dir/a.txt", EntryKind::File, file_attrs(10, 1)),
        ]));

        // Rebuild the directory explicitly after the removal. The following child
        // must resolve through that new entry rather than a memoized id for the entry
        // that was just removed.
        index.apply_ok(&Observation::new(vec![
            upsert("dir/b.txt", EntryKind::File, file_attrs(20, 1)),
            Op::Remove { path: PathBuf::from("dir") },
            upsert("dir", EntryKind::Dir, file_attrs(0, 2)),
            upsert("dir/c.txt", EntryKind::File, file_attrs(30, 2)),
        ]));

        let children = index.children(Path::new("dir")).expect("dir survives");
        let names: Vec<_> = children.map(|(name, _)| name.to_os_string()).collect();
        assert_eq!(names, vec![OsString::from("c.txt")], "only the re-added child remains");
        assert_eq!(index.total().bytes, 30, "totals match the surviving child");
    }

    #[test]
    fn a_kind_change_mid_batch_leaves_the_memo_usable_for_the_next_sibling() {
        let mut index = Index::new(PathBuf::from("/root"));
        index.apply_ok(&Observation::new(vec![
            upsert("swap", EntryKind::Dir, file_attrs(0, 1)),
            upsert("swap/inner", EntryKind::Dir, file_attrs(0, 1)),
            upsert("swap/inner/deep.txt", EntryKind::File, file_attrs(40, 1)),
        ]));

        // A kind change drops a subtree, which looks like it should invalidate the memo
        // and does not: the memo holds the changed entry's parent, and the subtree
        // removed is rooted at the entry itself. This pins that reasoning, so that if
        // the removal ever widens to touch the parent the failure lands here rather
        // than as a dangling id in a scan.
        index.apply_ok(&Observation::new(vec![
            upsert("swap/inner/other.txt", EntryKind::File, file_attrs(50, 1)),
            upsert("swap/inner", EntryKind::File, file_attrs(60, 2)),
            upsert("swap/sibling.txt", EntryKind::File, file_attrs(70, 2)),
        ]));

        let children = index.children(Path::new("swap")).expect("swap survives");
        let names: Vec<_> = children.map(|(name, _)| name.to_os_string()).collect();
        assert_eq!(names, vec![OsString::from("inner"), OsString::from("sibling.txt")]);
        assert_eq!(index.total().files, 2, "inner counts once, as a file");
        assert_eq!(index.total().bytes, 130, "the dropped subtree's bytes are gone");
    }

    #[test]
    fn parent_memo_distinguishes_directories_that_share_a_name_prefix() {
        let mut index = Index::new(PathBuf::from("/root"));
        // `src` and `src2` differ only after the memo's stored bytes end, which is the
        // comparison a prefix check rather than an equality check would get wrong.
        index.apply_ok(&Observation::new(vec![
            upsert("src", EntryKind::Dir, file_attrs(0, 1)),
            upsert("src2", EntryKind::Dir, file_attrs(0, 1)),
            upsert("src/one.txt", EntryKind::File, file_attrs(11, 1)),
            upsert("src2/two.txt", EntryKind::File, file_attrs(22, 1)),
            upsert("src/three.txt", EntryKind::File, file_attrs(33, 1)),
        ]));

        let in_src: Vec<_> = index
            .children(Path::new("src"))
            .expect("src")
            .map(|(name, _)| name.to_os_string())
            .collect();
        let in_src2: Vec<_> = index
            .children(Path::new("src2"))
            .expect("src2")
            .map(|(name, _)| name.to_os_string())
            .collect();
        assert_eq!(in_src, vec![OsString::from("one.txt"), OsString::from("three.txt")]);
        assert_eq!(in_src2, vec![OsString::from("two.txt")]);
    }

    #[test]
    fn parent_memo_leaves_root_level_entries_alone() {
        // A root-level path has `Some("")` as its parent, which must not be confused
        // with the root entry itself or with a sibling's empty-parent lookup.
        let mut index = Index::new(PathBuf::from("/root"));
        index.apply_ok(&Observation::new(vec![
            upsert("a.txt", EntryKind::File, file_attrs(5, 1)),
            upsert("b.txt", EntryKind::File, file_attrs(6, 1)),
            upsert("dir", EntryKind::Dir, file_attrs(0, 1)),
            upsert("dir/c.txt", EntryKind::File, file_attrs(7, 1)),
            upsert("d.txt", EntryKind::File, file_attrs(8, 1)),
        ]));

        let top: Vec<_> = index
            .children(Path::new(""))
            .expect("root children")
            .map(|(name, _)| name.to_os_string())
            .collect();
        assert_eq!(
            top,
            vec![
                OsString::from("a.txt"),
                OsString::from("b.txt"),
                OsString::from("d.txt"),
                OsString::from("dir"),
            ]
        );
        assert_eq!(index.total().files, 4);
        assert_eq!(index.total().bytes, 26);
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
    fn cloned_indexes_are_independent_detached_images() {
        let original = index_with_sample_tree();
        let original_clock = original.clock();
        let mut detached = original.clone();

        detached.apply_ok(&Observation::new(vec![upsert(
            "detached-only.txt",
            EntryKind::File,
            file_attrs(7, 30),
        )]));

        assert_eq!(original.clock(), original_clock);
        assert!(original.lookup(Path::new("detached-only.txt")).is_none());
        assert!(detached.lookup(Path::new("detached-only.txt")).is_some());
        assert_eq!(detached.total().files, original.total().files + 1);
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
            let applied: AppliedDelta = outcome
                .expect("writer apply")
                .applied()
                .cloned()
                .expect("unique upsert must commit");
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
        let operations: Vec<Op> =
            std::iter::once(upsert("batch", EntryKind::Dir, file_attrs(0, 1)))
                .chain((1..=file_count).map(|ordinal| {
                    upsert(
                        &format!("batch/file-{ordinal}.bin"),
                        EntryKind::File,
                        file_attrs(ordinal, i64::try_from(ordinal).expect("small ordinal")),
                    )
                }))
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
                    let total = image.total();
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
        let before_total = index.total();
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
        assert_eq!(index.total(), before_total);
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
        let before_total = index.total();
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
        assert!(no_op.applied().is_none());
        assert_eq!(stale.stale, 1);
        assert!(stale.applied().is_none());
        assert_eq!(index.clock(), Clock(u64::MAX));
        assert_eq!(index.len(), before_len);
        assert_eq!(index.total(), before_total);
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
        assert!(outcome.applied().is_none());
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
        assert!(outcome.applied().is_none());
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
        assert!(outcome.applied().is_none());
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
        assert!(outcome.applied().is_none());
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
                let before_total = index.total();
                let before_journal = index.journal.clone();
                let before_journal_cost = index.journal_cost;
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
                assert_eq!(index.total(), before_total);
                assert_eq!(index.journal, before_journal);
                assert_eq!(index.journal_cost, before_journal_cost);
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
    fn the_extension_interner_reclaims_ids_after_churn() {
        // The long-lived case: a watched tree that keeps creating and deleting files
        // with distinct extensions. Without reclamation both interner maps grow for the
        // life of the process, which for `fdu --watch` means forever.
        let mut index = Index::new("/root");
        for sequence in 0..128 {
            let path = PathBuf::from(format!("build.out-{sequence}"));
            index.apply_ok(&Observation::new(vec![Op::Upsert {
                path: path.clone(),
                kind: EntryKind::File,
                attrs: file_attrs(1, sequence),
            }]));
            index.apply_ok(&Observation::new(vec![Op::Remove { path }]));
        }

        assert!(index.ext_ids.is_empty(), "no extension survives the file that named it");
        assert_eq!(index.ext_names.len(), 1, "128 dead extensions reuse one interner slot");
        assert!(index.total().by_ext.is_empty());
    }

    #[test]
    fn a_reclaimed_extension_id_does_not_alias_a_live_tally() {
        // Reissuing a slot is only safe if nothing still points at it. Keep one file on
        // the recycled extension while another one comes and goes.
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("keep.rs", EntryKind::File, file_attrs(10, 1)),
            upsert("drop.tmp", EntryKind::File, file_attrs(20, 2)),
        ]));
        let retained = index.total();
        index.apply_ok(&Observation::new(vec![Op::Remove { path: PathBuf::from("drop.tmp") }]));
        index.apply_ok(&Observation::new(vec![upsert(
            "next.bak",
            EntryKind::File,
            file_attrs(30, 3),
        )]));

        let tallies = index.total().by_ext;
        assert_eq!(tallies[".rs"], ExtTally { files: 1, bytes: 10, allocated: 512 });
        assert_eq!(tallies[".bak"], ExtTally { files: 1, bytes: 30, allocated: 512 });
        assert!(!tallies.contains_key(".tmp"), "the removed extension is gone");
        assert_eq!(
            retained.by_ext[".tmp"],
            ExtTally { files: 1, bytes: 20, allocated: 512 },
            "an owned roll-up stays self-describing after its interner slot is reused"
        );
        assert!(!retained.by_ext.contains_key(".bak"));
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
        assert_eq!(total.by_ext[".txt"], ExtTally { files: 1, bytes: 10, allocated: 512 });
        assert!(!total.by_ext.contains_key(".rs"));
        assert!(!total.by_ext.contains_key(".md"));

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
        assert_eq!(total.by_ext[".rs"], ExtTally { files: 2, bytes: 300, allocated: 1024 });
        assert_eq!(total.by_ext[".md"], ExtTally { files: 1, bytes: 300, allocated: 512 });

        // Per-directory breakdown, which no surveyed tool provides.
        let src = index.rollup(Path::new("src")).expect("src is a directory");
        assert_eq!(src.by_ext[".rs"], ExtTally { files: 2, bytes: 300, allocated: 1024 });
        assert!(!src.by_ext.contains_key(".md"));
    }

    #[test]
    fn per_extension_allocated_tracks_apparent_bytes_separately() {
        // Both size metrics ride in the same tally, so a report asked for allocated bytes
        // keeps its per-type breakdown instead of silently answering in apparent bytes.
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            // Two small files: apparent bytes are tiny, allocation rounds each to a block.
            upsert("a.rs", EntryKind::File, file_attrs(1, 10)),
            upsert("b.rs", EntryKind::File, file_attrs(2, 20)),
        ]));

        let rs = index.total().by_ext[".rs"];
        assert_eq!((rs.files, rs.bytes), (2, 3));
        assert_eq!(rs.allocated, 1024, "each file occupies one 512-byte block");

        // Removing one file withdraws its allocation from the tally, not just its bytes.
        index.apply_ok(&Observation::new(vec![Op::Remove { path: PathBuf::from("b.rs") }]));
        let rs = index.total().by_ext[".rs"];
        assert_eq!((rs.files, rs.bytes, rs.allocated), (1, 1, 512));
    }

    #[test]
    fn upsert_with_matching_fingerprint_is_a_no_op() {
        let mut index = index_with_sample_tree();
        let before = index.total();
        let mark = index.clock();

        let stats = index.apply_ok(&Observation::new(vec![upsert(
            "src/main.rs",
            EntryKind::File,
            file_attrs(100, 10),
        )]));

        assert_eq!(stats.unchanged, 1);
        assert_eq!(stats.updated, 0);
        assert_eq!(index.total(), before);
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
        let applied = outcome.applied().expect("one effective insert");
        assert_eq!(applied.len(), 1);
        assert_eq!(applied.ops[0].path(), Path::new("new.txt"));
    }

    #[test]
    fn exact_commit_records_verified_ancestry_and_kind_replacement() {
        let mut index = Index::new("/root");
        let inserted = index.apply_ok(&Observation::new(vec![
            upsert("unknown", EntryKind::Dir, file_attrs(0, 1)),
            upsert("unknown/deep", EntryKind::Dir, file_attrs(0, 2)),
            upsert("unknown/deep/file.txt", EntryKind::File, file_attrs(10, 3)),
        ]));
        let inserted = inserted.commit.expect("ancestry commit");
        assert_eq!(
            inserted.changes.iter().map(EffectiveChange::path).collect::<Vec<_>>(),
            [Path::new("unknown"), Path::new("unknown/deep"), Path::new("unknown/deep/file.txt")]
        );
        assert!(
            inserted
                .changes
                .iter()
                .all(|change| matches!(change, EffectiveChange::Inserted { .. }))
        );

        let replaced = index.apply_ok(&Observation::new(vec![upsert(
            "unknown",
            EntryKind::File,
            file_attrs(20, 2),
        )]));
        let replaced = replaced.commit.expect("replacement commit");
        assert_eq!(
            replaced.changes,
            vec![
                EffectiveChange::Removed {
                    path: "unknown".into(),
                    kind: EntryKind::Dir,
                    attrs: file_attrs(0, 1),
                },
                EffectiveChange::Removed {
                    path: "unknown/deep".into(),
                    kind: EntryKind::Dir,
                    attrs: file_attrs(0, 2),
                },
                EffectiveChange::Removed {
                    path: "unknown/deep/file.txt".into(),
                    kind: EntryKind::File,
                    attrs: file_attrs(10, 3),
                },
                EffectiveChange::Inserted {
                    path: "unknown".into(),
                    kind: EntryKind::File,
                    attrs: file_attrs(20, 2),
                },
            ]
        );
        assert_eq!(
            replaced.impact.domains,
            vec![
                ImpactDomain::Topology,
                ImpactDomain::Metadata,
                ImpactDomain::Classification,
                ImpactDomain::Aggregates,
                ImpactDomain::Content,
            ]
        );
        assert_eq!(
            replaced.impact.dirty_paths,
            vec![
                PathBuf::new(),
                "unknown".into(),
                "unknown/deep".into(),
                "unknown/deep/file.txt".into(),
            ]
        );
    }

    #[test]
    fn rejected_prepared_commit_is_fault_atomic() {
        let mut index = index_with_sample_tree();
        let before_clock = index.clock();
        let before_total = index.total();
        let before_len = index.len();
        let before_history = index.since(Clock::ZERO);
        let mut prepared = prepare_observation(&Observation::new(vec![upsert(
            "new/deep.txt",
            EntryKind::File,
            file_attrs(99, 99),
        )]))
        .expect("valid preparation");
        prepared.reject_before_apply = true;

        let error = index.commit_prepared(prepared, true).expect_err("injected preflight");

        assert!(matches!(error, crate::Error::CommitRejected("injected reducer preflight")));
        assert_eq!(index.clock(), before_clock);
        assert_eq!(index.total(), before_total);
        assert_eq!(index.len(), before_len);
        assert_eq!(index.since(Clock::ZERO), before_history);
        assert!(index.lookup(Path::new("new")).is_none());
    }

    #[test]
    fn reconciliation_state_moves_through_exact_commits() {
        let mut index = Index::new("/root");
        let (started, start) = index.begin_reconcile(Path::new("src")).expect("begin");
        let start = start.expect("start commit");
        assert!(start.changes.is_empty());
        assert_eq!(
            start.state,
            vec![
                StateTransition::Freshness {
                    path: "src".into(),
                    previous: Freshness::Fresh,
                    current: Freshness::Reconciling,
                },
                StateTransition::IndexState {
                    previous: IndexState::default(),
                    current: IndexState {
                        freshness: Freshness::Reconciling,
                        ..IndexState::default()
                    },
                },
            ]
        );

        let finish = index
            .finish_reconcile(Path::new("src"), started, true)
            .expect("finish")
            .expect("finish commit");
        assert!(finish.changes.is_empty());
        assert_eq!(
            finish.state,
            vec![
                StateTransition::Verified { path: "src".into() },
                StateTransition::Freshness {
                    path: "src".into(),
                    previous: Freshness::Reconciling,
                    current: Freshness::Fresh,
                },
                StateTransition::IndexState {
                    previous: IndexState {
                        freshness: Freshness::Reconciling,
                        ..IndexState::default()
                    },
                    current: IndexState::default(),
                },
            ]
        );
    }

    #[cfg(feature = "watch")]
    #[test]
    fn observation_failure_cannot_overwrite_a_terminal_resource_stop() {
        let handle = IndexHandle::new(Index::new("/root"));
        handle
            .transition_discovery(DiscoveryTransition::BudgetRefused(Issue::resource_budget(0)))
            .expect("stop for budget");
        let stopped = handle.state().expect("stopped state");
        let clock = handle.clock().expect("stopped clock");

        let outcome = handle
            .transition_observation(ObservationTransition::Failed(Issue::from_error(
                &crate::Error::WatchStopped,
            )))
            .expect("late observer failure is ignored");

        assert_eq!(outcome.commit, None);
        assert_eq!(handle.state().expect("terminal state"), stopped);
        assert_eq!(handle.clock().expect("terminal clock"), clock);
    }

    #[test]
    fn replaying_the_same_delta_twice_changes_nothing() {
        let mut index = Index::new("/root");
        let delta = Observation::new(vec![
            upsert("a", EntryKind::Dir, Attrs::default()),
            upsert("a/f.txt", EntryKind::File, file_attrs(10, 1)),
        ]);
        index.apply_ok(&delta);
        let after_first = index.total();
        let stats = index.apply_ok(&delta);

        assert_eq!(stats.unchanged, 2);
        assert_eq!(index.total(), after_first);
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
        assert_eq!(index.total().by_ext[".rs"], ExtTally { files: 2, bytes: 350, allocated: 1024 });
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
        assert!(!total.by_ext.contains_key(".md"), "emptied tallies are dropped");
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
        assert!(!total.by_ext.contains_key(".rs"));
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
    fn parent_first_batch_establishes_exact_ancestry() {
        let mut index = Index::new("/root");
        let deep = file_attrs(0, 1);
        let nested = file_attrs(0, 2);
        let tree = file_attrs(0, 3);
        index.apply_ok(&Observation::new(vec![
            upsert("deep", EntryKind::Dir, deep),
            upsert("deep/nested", EntryKind::Dir, nested),
            upsert("deep/nested/tree", EntryKind::Dir, tree),
            upsert("deep/nested/tree/file.txt", EntryKind::File, file_attrs(42, 7)),
        ]));

        assert_eq!(index.total().files, 1);
        assert_eq!(index.total().dirs, 3);
        assert_eq!(index.total().bytes, 42);
        assert_eq!(index.rollup(Path::new("deep/nested")).expect("created").files, 1);
        assert_eq!(index.attrs(Path::new("deep")), Some(&deep));
        assert_eq!(index.attrs(Path::new("deep/nested")), Some(&nested));
        assert_eq!(index.attrs(Path::new("deep/nested/tree")), Some(&tree));
    }

    #[test]
    fn live_upsert_refuses_unknown_ancestry_without_mutation() {
        let mut index = Index::new("/root");
        let before = index.clock();

        let error = index
            .apply(&Observation::new(vec![upsert(
                "unknown/deep/file.txt",
                EntryKind::File,
                file_attrs(10, 1),
            )]))
            .expect_err("live input must not invent parent metadata");

        assert!(matches!(
            error,
            crate::Error::UnknownAncestry { path, reconcile_from }
                if path == Path::new("unknown/deep/file.txt")
                    && reconcile_from.as_os_str().is_empty()
        ));
        assert_eq!(index.clock(), before);
        assert_eq!(index.len(), 1);
        assert!(index.since(before).commits.is_empty());
    }

    #[test]
    fn explicit_kind_replacement_precedes_attaching_a_child() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert(
            "conflict",
            EntryKind::File,
            file_attrs(9, 1),
        )]));

        let outcome = index.apply_ok(&Observation::new(vec![
            upsert("conflict", EntryKind::Dir, file_attrs(0, 2)),
            upsert("conflict/child.txt", EntryKind::File, file_attrs(4, 2)),
        ]));

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
        let mut index = Index::with_journal_capacity("/root", 2);
        let outcome = index.apply_ok(&Observation::new(vec![
            upsert("a.txt", EntryKind::File, file_attrs(1, 1)),
            upsert("b.txt", EntryKind::File, file_attrs(2, 2)),
            upsert("c.txt", EntryKind::File, file_attrs(3, 3)),
        ]));

        assert_eq!(outcome.applied().as_ref().expect("committed").len(), 3);
        let since = index.since(Clock::ZERO);
        assert!(since.truncated);
        assert!(since.deltas.is_empty());
    }

    #[test]
    fn journal_eviction_charges_the_complete_retained_payload() {
        let mut index = Index::with_journal_capacity("/root", 6);
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
    fn impact_drops_an_overflowing_path_set_instead_of_truncating_it() {
        let mut index = Index::new("/root");
        let ops = (0..=MAX_DIRTY_PATHS)
            .map(|which| {
                upsert(
                    &format!("file-{which}.txt"),
                    EntryKind::File,
                    file_attrs(u64::try_from(which).expect("bounded"), 1),
                )
            })
            .collect();

        let commit =
            index.apply_ok(&Observation::new(ops)).commit.expect("overflowing impact commit");

        assert!(commit.impact.all_dirty);
        assert!(commit.impact.dirty_paths.is_empty(), "a partial path list must not escape");
        assert_eq!(commit.changes.len(), MAX_DIRTY_PATHS + 1);
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
        let mut index = Index::new_opened_with_scope_types_and_journal_capacity(
            "/root",
            ScanScope::default(),
            crate::classify::TypeRegistry::compiled_shared(),
            DEFAULT_JOURNAL_CAPACITY,
        );
        index.apply_ok(&Observation::new(vec![
            Op::Upsert { path: first.clone(), kind: EntryKind::File, attrs: file_attrs(10, 1) },
            Op::Upsert { path: second.clone(), kind: EntryKind::File, attrs: file_attrs(20, 2) },
        ]));

        assert_eq!(index.total().files, 2);
        assert_eq!(index.total().bytes, 30);
        assert!(index.lookup(&first).is_some());
        assert!(index.lookup(&second).is_some());
        assert_serving_indexes(&index);
    }

    #[cfg(unix)]
    #[test]
    fn a_non_utf8_parent_still_lists_its_children() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let directory = PathBuf::from(OsString::from_vec(vec![b'd', 0x80]));
        let child = directory.join("child");
        let mut index = Index::new_opened_with_scope_types_and_journal_capacity(
            "/root",
            ScanScope::default(),
            crate::classify::TypeRegistry::compiled_shared(),
            DEFAULT_JOURNAL_CAPACITY,
        );
        index.apply_ok(&Observation::new(vec![
            Op::Upsert { path: directory.clone(), kind: EntryKind::Dir, attrs: Attrs::default() },
            Op::Upsert { path: child, kind: EntryKind::File, attrs: file_attrs(1, 1) },
        ]));

        assert_serving_indexes(&index);
        // The directory's own name escapes to `d%80`, and its child is reachable beneath
        // it. While the encoding was partial this directory had no portable name, so its
        // whole subtree was unlistable and the assertion here counted the loss instead.
        assert_eq!(
            index.portable_children(&directory).map(|children| children.nondirectories.len()),
            Some(1)
        );
        assert!(
            index.portable_entries().keys().any(|portable| portable.as_str() == "d%80/child"),
            "a child under a non-utf8 directory is listed at its escaped path"
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_stem_keeps_ascii_extension_tally() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let path = PathBuf::from(OsString::from_vec(vec![b'n', 0x80, b'.', b'R', b'S']));
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![Op::Upsert {
            path,
            kind: EntryKind::File,
            attrs: file_attrs(10, 1),
        }]));

        let tallies = index.total().by_ext;
        assert_eq!(
            tallies.get(".rs"),
            Some(&ExtTally { files: 1, bytes: 10, allocated: file_attrs(10, 1).allocated })
        );
    }

    #[test]
    fn content_results_commit_conditionally_and_metadata_changes_invalidate_them() {
        use crate::content::{
            AnalysisApplyOutcome, AnalysisRequest, AnalysisSet, ContentProvenance, CoverageReason,
            FileAnalysis, MetricValues,
        };

        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("src", EntryKind::Dir, file_attrs(0, 1)),
            upsert("src/lib.rs", EntryKind::File, file_attrs(10, 1)),
        ]));
        let candidate = index
            .analysis_candidates(AnalysisSet::NONE.with_lines())
            .into_iter()
            .next()
            .expect("file candidate");
        let analysis = FileAnalysis {
            classification: candidate.classification.clone(),
            fingerprint: candidate.attrs.fingerprint(),
            bytes: candidate.attrs.size,
            profile: candidate.profile,
            provenance: ContentProvenance::for_request(
                AnalysisRequest { profile: candidate.profile, ..AnalysisRequest::default() },
                crate::classify::type_rule_fingerprint(),
            ),
            metrics: MetricValues {
                physical_lines: 2,
                nonblank_lines: 2,
                ..MetricValues::default()
            },
            coverage: CoverageReason::Analyzed,
            error: None,
        };
        assert_eq!(
            index.apply_analysis(AnalysisObservation {
                candidate: candidate.clone(),
                analysis: analysis.clone(),
            }),
            AnalysisApplyOutcome::Applied
        );
        assert_eq!(
            index.content_rollup(Path::new("")).expect("content root").total.metrics.physical_lines,
            2
        );

        index.apply_ok(&Observation::new(vec![upsert(
            "src/lib.rs",
            EntryKind::File,
            file_attrs(20, 2),
        )]));
        assert!(index.content_rollup(Path::new("")).is_none());
        assert_eq!(
            index.apply_analysis(AnalysisObservation { candidate, analysis }),
            AnalysisApplyOutcome::Stale
        );
    }

    #[cfg(feature = "gitignore")]
    #[test]
    fn control_changes_atomically_move_fixed_partitions_without_changing_all() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert(".gitignore", EntryKind::File, file_attrs(6, 1)),
            upsert("debug.log", EntryKind::File, file_attrs(10, 2)),
            upsert("keep.rs", EntryKind::File, file_attrs(20, 3)),
            upsert("docs", EntryKind::Dir, file_attrs(0, 4)),
            upsert("docs/other.log", EntryKind::File, file_attrs(30, 5)),
            upsert("docs/keep.log", EntryKind::File, file_attrs(40, 6)),
        ]));
        let before = index.partition_total();

        let outcome = index.apply_ok(&Observation::new(vec![Op::ControlUpsert {
            path: PathBuf::from(".gitignore"),
            source: b"*.log\n".to_vec(),
        }]));
        let partitions = index.partition_total();

        assert_eq!(partitions.all, before.all, "classification never changes all facts");
        assert_eq!(partitions.all.files, 5);
        assert_eq!(partitions.unignored.files, 2);
        assert_eq!(partitions.unignored.bytes, 26);
        assert_eq!(outcome.controls, 1);
        assert_eq!(outcome.reclassified, 3);
        assert_eq!(index.is_ignored(Path::new("debug.log")), Some(true));
        assert_eq!(index.is_ignored(Path::new("keep.rs")), Some(false));

        let commit = outcome.commit.expect("control and classification commit together");
        assert!(matches!(
            commit.changes.first(),
            Some(EffectiveChange::ControlUpdated { path, previous: None, current: Some(_) })
                if path == Path::new(".gitignore")
        ));
        assert_eq!(
            commit
                .changes
                .iter()
                .filter(|change| matches!(change, EffectiveChange::Reclassified { .. }))
                .count(),
            3
        );
    }

    #[cfg(feature = "gitignore")]
    #[test]
    fn serving_semantics_follow_ignore_reclassification_exactly() {
        let mut index = Index::new_opened_with_scope_types_and_journal_capacity(
            "/root",
            ScanScope::default(),
            crate::classify::TypeRegistry::compiled_shared(),
            DEFAULT_JOURNAL_CAPACITY,
        );
        index.apply_ok(&Observation::new(vec![
            upsert(".gitignore", EntryKind::File, file_attrs(6, 1)),
            upsert("debug.log", EntryKind::File, file_attrs(10, 2)),
            upsert("keep.rs", EntryKind::File, file_attrs(20, 3)),
            upsert("Makefile", EntryKind::File, file_attrs(30, 4)),
        ]));
        assert_serving_indexes(&index);

        index.apply_ok(&Observation::new(vec![Op::ControlUpsert {
            path: PathBuf::from(".gitignore"),
            source: b"*.log\nMakefile\n".to_vec(),
        }]));
        assert_serving_indexes(&index);

        index.apply_ok(&Observation::new(vec![Op::ControlRemove {
            path: PathBuf::from(".gitignore"),
        }]));
        assert_serving_indexes(&index);
    }

    #[cfg(feature = "gitignore")]
    #[test]
    fn nested_negation_edit_and_last_control_deletion_reclassify_exactly() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert(".gitignore", EntryKind::File, file_attrs(6, 1)),
            upsert("docs", EntryKind::Dir, file_attrs(0, 2)),
            upsert("docs/.gitignore", EntryKind::File, file_attrs(10, 3)),
            upsert("docs/keep.log", EntryKind::File, file_attrs(40, 4)),
            Op::ControlUpsert { path: PathBuf::from(".gitignore"), source: b"*.log\n".to_vec() },
            Op::ControlUpsert {
                path: PathBuf::from("docs/.gitignore"),
                source: b"!keep.log\n".to_vec(),
            },
        ]));
        assert_eq!(index.is_ignored(Path::new("docs/keep.log")), Some(false));

        let edited = index.apply_ok(&Observation::new(vec![Op::ControlUpsert {
            path: PathBuf::from("docs/.gitignore"),
            source: b"# no exception\n".to_vec(),
        }]));
        assert_eq!(edited.reclassified, 1);
        assert_eq!(index.is_ignored(Path::new("docs/keep.log")), Some(true));

        let removed = index
            .apply_ok(&Observation::new(vec![Op::Remove { path: PathBuf::from(".gitignore") }]));
        assert_eq!(removed.controls, 1, "removing the retained row removes its control state");
        assert_eq!(removed.reclassified, 1);
        assert_eq!(index.controls().len(), 1, "the nested control remains");
        assert_eq!(index.is_ignored(Path::new("docs/keep.log")), Some(false));

        index.apply_ok(&Observation::new(vec![Op::Remove {
            path: PathBuf::from("docs/.gitignore"),
        }]));
        assert!(index.controls().is_empty());
        assert_eq!(index.is_ignored(Path::new("docs/keep.log")), Some(false));
    }

    #[cfg(feature = "gitignore")]
    #[test]
    fn control_bound_failure_is_atomic_with_ordinary_entry_work() {
        let mut index = Index::new_opened_with_scope_types_and_journal_capacity(
            "/root",
            ScanScope::default(),
            crate::classify::TypeRegistry::compiled_shared(),
            DEFAULT_JOURNAL_CAPACITY,
        );
        let before = index.clone();
        let mut oversized = crate::control::source_at_test_limit();
        oversized.push(b'a');
        let error = index
            .apply(&Observation::new(vec![
                upsert("ordinary.txt", EntryKind::File, file_attrs(1, 1)),
                Op::ControlUpsert { path: PathBuf::from(".gitignore"), source: oversized },
            ]))
            .expect_err("the complete batch must fail before any mutation");

        assert!(matches!(error, crate::Error::ControlSourceLimit { .. }));
        assert_eq!(index.clock(), before.clock());
        assert_eq!(index.len(), before.len());
        assert_eq!(index.total(), before.total());
        assert_eq!(index.serving, before.serving);
        assert!(index.lookup(Path::new("ordinary.txt")).is_none());
        assert!(index.controls().is_empty());
    }

    #[test]
    fn one_sweep_reports_one_as_of_time_for_everything_it_verified() {
        // Found by reviewing the PR #6 provenance work against the composable-CLI
        // merge. A revalidation sweep elides entries whose attributes did not change,
        // so within one pass some paths are named by a delta and some are not. Both
        // were verified at the same moment and must say so identically: if the
        // delta-touched entry dates itself to index construction while its untouched
        // sibling dates itself to the sweep, a consumer sorting rows by age is
        // comparing two different clocks and cannot tell.
        let mut index = Index::new("/root");
        index.set_applying_source(Source::Cached, 1_000);
        index.apply_ok(&Observation::new(vec![
            Op::Upsert { path: "a".into(), kind: EntryKind::Dir, attrs: file_attrs(0, 1) },
            Op::Upsert {
                path: "a/kept.txt".into(),
                kind: EntryKind::File,
                attrs: file_attrs(1, 1),
            },
            Op::Upsert {
                path: "a/changed.txt".into(),
                kind: EntryKind::File,
                attrs: file_attrs(2, 2),
            },
        ]));

        // A sweep re-observes both: one is unchanged and elided, one is updated.
        index.set_applying_source(Source::Revalidated, 2_000);
        index.begin_reconcile(Path::new("")).expect("begin reconciliation");
        index.apply_ok(&Observation::new(vec![
            Op::Upsert {
                path: "a/kept.txt".into(),
                kind: EntryKind::File,
                attrs: file_attrs(1, 1),
            },
            Op::Upsert {
                path: "a/changed.txt".into(),
                kind: EntryKind::File,
                attrs: file_attrs(9, 2),
            },
        ]));
        index.finish_reconcile(Path::new(""), 0, true).expect("finish reconciliation");

        let kept = index.provenance(Path::new("a/kept.txt")).expect("present");
        let changed = index.provenance(Path::new("a/changed.txt")).expect("present");
        assert!(kept.is_verified() && changed.is_verified(), "the sweep covered both");
        assert_eq!(
            kept.observed_at_ns, changed.observed_at_ns,
            "one sweep, one as-of time: {kept:?} vs {changed:?}"
        );
    }

    #[test]
    fn withdrawn_trust_beats_a_verification_interval() {
        // A verification interval records that a sweep once covered a path. If the
        // index has since withdrawn trust — an InvalidateSubtree marking it Stale, or
        // a sweep in progress marking it Reconciling — the interval must not promote
        // it, or provenance answers "partial, and verified" in one breath.
        let mut index = Index::new("/root");
        // The entry arrives the way a snapshot load delivers it: unverified.
        index.set_applying_source(Source::Cached, 1_000);
        index.apply_baseline_ok(&Observation::new(vec![
            Op::Upsert { path: PathBuf::from("a"), kind: EntryKind::Dir, attrs: Attrs::default() },
            Op::Upsert {
                path: PathBuf::from("a/file.txt"),
                kind: EntryKind::File,
                attrs: Attrs { size: 1, ..Attrs::default() },
            },
        ]));
        assert_eq!(
            index.provenance(Path::new("a/file.txt")).expect("present").source,
            Source::Cached,
            "nothing has checked it yet"
        );
        // A completed sweep then covers the whole tree.
        index.finish_reconcile(Path::new(""), 0, true).expect("finish reconciliation");
        let path = Path::new("a/file.txt");
        assert_eq!(
            index.provenance(path).expect("present").source,
            Source::Revalidated,
            "a completed sweep covers this path"
        );

        // Now withdraw trust over the subtree.
        index.mark_unfresh(Path::new("a"), Freshness::Stale);
        let provenance = index.provenance(path).expect("present");
        assert!(
            !provenance.is_verified(),
            "an invalidated path must not read as verified: {provenance:?}"
        );
        assert_eq!(
            provenance.status,
            Status::Complete,
            "withdrawing trust changes how far to believe the value, not how much of \
             the subtree it covers: the cached total still accounts for every entry \
             beneath this path, and reporting it as Partial would tell a consumer the \
             number is still being built when it is merely unverified"
        );
    }

    #[test]
    fn verification_intervals_stay_bounded() {
        // Repeated scoped sweeps of sibling subtrees must not grow without bound;
        // dropping the oldest only ever under-claims trust.
        let mut index = Index::new("/root");
        for which in 0..(MAX_VERIFIED_INTERVALS * 2) {
            index
                .finish_reconcile(&PathBuf::from(format!("dir-{which}")), 0, true)
                .expect("finish reconciliation");
        }
        assert!(
            index.verified.len() <= MAX_VERIFIED_INTERVALS,
            "interval list grew to {}",
            index.verified.len()
        );
    }
}
