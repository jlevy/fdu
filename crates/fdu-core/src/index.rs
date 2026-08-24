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

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::classify::ext_bucket;
use crate::content::{
    AnalysisApplyOutcome, AnalysisCandidate, AnalysisObservation, AnalysisSet, ContentIndex,
    ContentRollUp,
};
use crate::engine_contract::{
    AppliedDelta, Attrs, Bound, Clock, CoverageReason, EntryIdentity, EntryKind, Expectation,
    Freshness, InvalidateReason, Observation, Op, PathExpectation, PathState, Provenance,
    ScanScope, Source, Status,
};

/// Verification intervals kept before the oldest are dropped.
///
/// Bounds the memory a long-lived session can accumulate through repeated scoped
/// reconciliation. Dropping an interval only ever moves a path back to reporting
/// `Cached`, so the bound costs precision, never correctness.
const MAX_VERIFIED_INTERVALS: usize = 256;

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
    /// Descendant entries that are neither files nor directories: symlinks and the rest.
    ///
    /// Zero bytes each, and counted anyway, because otherwise a subtree holding a hundred
    /// symlinks and one holding nothing at all are the same arithmetic and a listing
    /// cannot tell them apart. See [`RollUp::is_empty`].
    pub others: u64,
    /// Apparent bytes across descendant files.
    pub bytes: u64,
    /// Allocated bytes across descendant files.
    pub allocated: u64,
    /// Newest mtime among descendant files, or 0 when there are none.
    pub newest_mtime_ns: i64,
    /// Per-extension file and byte tallies across the subtree.
    ///
    /// Complete unless `ext_remainder` says otherwise.
    pub by_ext: BTreeMap<String, ExtTally>,
    /// Per-group file and byte tallies across the subtree, by group id.
    ///
    /// A browsing axis maintained beside `by_ext` rather than derived from it. Deriving
    /// would be wrong twice over: an exact-filename rule (`Makefile`, `Dockerfile`) has
    /// no extension bucket to derive from, and a registry may map two extensions of one
    /// group to different types.
    ///
    /// Empty when the active rule registry declares no groups.
    pub by_group: BTreeMap<String, ExtTally>,
    /// Extension tallies a bound withheld from `by_ext`, or `None` when it holds them all.
    ///
    /// Same contract as a tree node's remainder: presence is the signal, and the listed
    /// rows plus this account for every file in the subtree. A caller rendering a handful
    /// of rows can label the rest instead of appearing to have shown everything.
    pub ext_remainder: Option<ExtRemainder>,
}

impl RollUp {
    /// Descendant entries of every kind.
    ///
    /// The sum the emptiness question is really about: `bytes` cannot answer it, because
    /// an empty file, a symlink and nothing at all all weigh nothing.
    pub fn entries(&self) -> u64 {
        self.files + self.dirs + self.others
    }

    /// Whether this subtree holds no entries at all.
    ///
    /// An exact fact only about a **complete** value. A [`Status::Partial`] roll-up has
    /// not accounted for its whole subtree, so zero here means "nothing found yet", and a
    /// caller holding one must consult its provenance before believing this -- see
    /// [`ChildSnapshot::is_empty_subtree`], which does that consulting.
    pub fn is_empty(&self) -> bool {
        self.entries() == 0
    }
}

impl RollUpScalars {
    /// Descendant entries of every kind.
    pub fn entries(&self) -> u64 {
        self.files + self.dirs + self.others
    }

    /// Whether this subtree holds no entries at all, with the same caveat as
    /// [`RollUp::is_empty`].
    pub fn is_empty(&self) -> bool {
        self.entries() == 0
    }
}

/// Extension tallies a bound withheld from a roll-up.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ExtRemainder {
    /// Distinct extensions not listed.
    pub extensions: u64,
    /// Files carrying them.
    pub files: u64,
    /// Apparent bytes across those files.
    pub bytes: u64,
    /// Allocated bytes across those files.
    pub allocated: u64,
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
    others: u64,
    bytes: u64,
    allocated: u64,
    newest_mtime_ns: i64,
    by_ext: BTreeMap<ExtId, ExtTally>,
    /// Per-group tallies, as a sorted association list.
    ///
    /// A `Vec` rather than a map because a registry declares a handful of groups -- six
    /// in the one fdu ships and the one metabrowser publishes -- and a linear scan over
    /// six beats a tree node per key. Empty, and therefore unallocated, when the registry
    /// declares no groups or the subtree holds nothing classified.
    by_group: Vec<(crate::classify::GroupId, ExtTally)>,
}

/// The scalar half of a roll-up: subtree totals with no per-extension breakdown.
///
/// [`RollUp`] answers two different questions in one value -- "how big is this subtree"
/// and "what is it made of" -- and the second one costs a `BTreeMap` clone per copy. A
/// listing asks only the first, once per row, so a directory of a thousand children was
/// cloning a thousand extension maps to render a thousand size columns. This is the part
/// a listing needs, `Copy`, with no allocation anywhere in it.
///
/// The breakdown is still available, as its own projection, for the one directory a
/// consumer is actually inspecting: [`IndexHandle::rollup_bounded`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct RollUpScalars {
    /// Descendant files.
    pub files: u64,
    /// Descendant directories, not counting the directory that owns this roll-up.
    pub dirs: u64,
    /// Descendant entries that are neither files nor directories.
    pub others: u64,
    /// Apparent bytes across descendant files.
    pub bytes: u64,
    /// Allocated bytes across descendant files.
    pub allocated: u64,
    /// Newest mtime among descendant files, or 0 when there are none.
    pub newest_mtime_ns: i64,
}

impl From<&InternedRollUp> for RollUpScalars {
    fn from(rollup: &InternedRollUp) -> Self {
        Self {
            files: rollup.files,
            dirs: rollup.dirs,
            others: rollup.others,
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
        self.others += other.others;
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
        for (group, tally) in &other.by_group {
            match self.by_group.binary_search_by_key(group, |(id, _)| *id) {
                Ok(position) => {
                    let slot = &mut self.by_group[position].1;
                    slot.files += tally.files;
                    slot.bytes += tally.bytes;
                    slot.allocated += tally.allocated;
                }
                Err(position) => self.by_group.insert(position, (*group, *tally)),
            }
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
        self.others = self.others.saturating_sub(other.others);
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
        for (group, tally) in &other.by_group {
            if let Ok(position) = self.by_group.binary_search_by_key(group, |(id, _)| *id) {
                let slot = &mut self.by_group[position].1;
                slot.files = slot.files.saturating_sub(tally.files);
                slot.bytes = slot.bytes.saturating_sub(tally.bytes);
                slot.allocated = slot.allocated.saturating_sub(tally.allocated);
                if slot.files == 0 && slot.bytes == 0 && slot.allocated == 0 {
                    self.by_group.remove(position);
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
    /// Browsing group, resolved once at insert from the index's rule registry. Files
    /// only, and `None` for a file no rule names or when the registry declares no
    /// groups. Precomputed here for the same reason `ext_id` is: the reducer that
    /// maintains group totals must not classify.
    group_id: Option<crate::classify::GroupId>,
    /// Named boolean facts about this entry, one bit per enabled tag rule.
    ///
    /// Computed at insert for the same reason `ext_id` and `group_id` are: the reducer
    /// and every projection must be able to read a fact without re-deriving it, and a
    /// watch upsert must reach the same answer as a scan upsert because it ran the same
    /// line of code rather than a matching one. Zero when no rule is enabled, which is
    /// the default and costs nothing.
    tag_bits: crate::tags::TagBits,
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
    rollup: InternedRollUp,
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

impl ApplyStats {
    /// True when any operation changed indexed state.
    ///
    /// `unchanged` and `stale` are decisions not to mutate, so a pass reporting only
    /// those left the index exactly as it was loaded.  Callers use this to tell a
    /// reconciliation that found real changes from one that confirmed a tree is still
    /// what the snapshot already says it is.
    pub const fn mutated(&self) -> bool {
        self.inserted > 0 || self.updated > 0 || self.removed > 0 || self.invalidated > 0
    }
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
    /// Pre-computed subtree totals for a directory, without the extension breakdown.
    ///
    /// Scalars on purpose. A listing wants a size column per row; the breakdown belongs
    /// to the one directory being inspected, and asking for it per row cloned a
    /// `BTreeMap` per child to render a number. Ask [`IndexHandle::rollup_bounded`] for
    /// the breakdown of the directory a consumer actually opened.
    pub totals: Option<RollUpScalars>,
    /// Origin, observation time, and coverage for this child.
    ///
    /// Captured here rather than looked up per child by path, because a consumer
    /// rendering a listing wants provenance for every row and resolving each one
    /// separately would take the read lock once per child and re-walk the path.
    pub provenance: Provenance,
    /// What the index's rule registry makes of this child, for files.
    ///
    /// Metadata-only: the name decides it, no file is opened, and the shebang and
    /// content-probe tiers -- which need bytes -- are not consulted. `None` for anything
    /// that is not a regular file.
    ///
    /// Here so a consumer can stop carrying a classifier of its own. Resolving it per row
    /// afterwards would mean re-deriving from the name what the engine already knows, in
    /// a second language, against a rule set that has no way to stay in step with this
    /// one.
    pub classification: Option<crate::classify::Classification>,
    /// The *logical* extension of this child's name: its final two eligible components.
    ///
    /// The raw level of the shared format's two, which is the one a person reads off the
    /// name: `release.v2.zip` is `.v2.zip` here while its roll-up bucket and its type are
    /// both `.zip`/`archive`. A consumer filtering on a literal extension or labelling a
    /// row wants this; one summing bytes per pile wants the breakdown's key.
    pub extension: Option<String>,
    /// Tags this child carries, in the enabled set's bit order.
    ///
    /// Names rather than bits, and for the same reason `group` is a `String`: a mask means
    /// nothing without the rule set that issued it, and a captured row outlives the read
    /// guard it came from. Empty when no rule is enabled, which is the default and costs
    /// no allocation.
    pub tags: Vec<String>,
    /// The browsing group this child falls in, resolved to its registry id.
    ///
    /// Resolved rather than left as the index `classification` carries, because a snapshot
    /// travels: a `GroupId` is meaningful only alongside the registry that issued it.
    pub group: Option<String>,
}

impl std::ops::Deref for ApplyOutcome {
    type Target = ApplyStats;

    fn deref(&self) -> &Self::Target {
        &self.stats
    }
}

impl ChildSnapshot {
    /// Whether this child is a directory whose subtree is provably empty.
    ///
    /// `None` rather than `false` for anything that cannot be decided: a non-directory,
    /// which has no subtree, and a directory whose roll-up is [`Status::Partial`], which
    /// has not accounted for one. A partial subtree reporting zero entries means "nothing
    /// found yet", and a listing that greyed out such a row would be greying out a
    /// directory it had not finished reading.
    ///
    /// Decidable at all only because a roll-up counts symlinks and other non-file entries
    /// as well as files and directories. Before that a subtree of a hundred symlinks was
    /// zero files, zero directories and zero bytes -- the same arithmetic as nothing.
    pub fn is_empty_subtree(&self) -> Option<bool> {
        let totals = self.totals?;
        if self.provenance.status != Status::Complete {
            return None;
        }
        Some(totals.is_empty())
    }
}

/// Which slice of a directory's children one listing call should return.
///
/// A directory is unbounded in a way a screen is not, and a listing that always returns
/// every child makes the caller pay for the whole directory to draw the top of it. The
/// bound is stated here rather than applied by the caller after the fact, because after
/// the fact is one snapshot per child too late.
#[derive(Clone, Debug, Default)]
pub struct ChildPageRequest {
    /// Resume strictly after this child name; `None` starts at the first child.
    ///
    /// A name rather than an offset. Children are ordered by name, and a directory that
    /// gains or loses an entry between two pages shifts every offset after it: an offset
    /// cursor silently repeats or skips rows, where a name resumes at the right place
    /// whatever happened in between. Resuming is a range seek, not a scan.
    pub after: Option<OsString>,
    /// Most rows to return.
    pub limit: Bound,
}

/// One page of a directory's children, with what the bound withheld stated on it.
#[derive(Clone, Debug)]
pub struct ChildPage {
    /// The rows this page carries, in name order.
    pub rows: Vec<ChildSnapshot>,
    /// The children this page does not carry, or `None` when it carries the whole
    /// directory.
    ///
    /// Presence is the signal, as everywhere else a bound applies: a consumer branches on
    /// having been given a remainder rather than comparing counts it would have to
    /// reconstruct.
    pub remainder: Option<ChildRemainder>,
    /// Cursor to pass as [`ChildPageRequest::after`] for the next page.
    ///
    /// `None` at the end of the directory. This, not `remainder`, is what says whether
    /// paging continues: a later page's remainder counts earlier pages' rows too, so it
    /// stays `Some` on the last page.
    pub next: Option<OsString>,
}

/// The children a page does not carry, as their share of the directory's own totals.
///
/// This page's rows plus this remainder account for the directory exactly -- the
/// partition property the index maintains, read backwards -- so a consumer showing fifty
/// of eight hundred children can still say honestly what the other seven hundred and
/// fifty come to. Derived by subtracting the emitted rows from the directory's roll-up,
/// which is work proportional to what was shown rather than to what was hidden.
///
/// It is the complement of *this page*, not of everything delivered so far: on page two
/// it counts page one's rows as well. Stating it against a fixed denominator is what
/// keeps it exact on every page without a cursor that has to carry a running total, and
/// "showing 50 of 812" is the sentence a listing wants anyway. [`ChildPage::next`], not
/// this, says whether more pages remain.
///
/// `dirs` counts a withheld directory row itself, unlike a tree node's
/// [`Remainder`](crate::query::Remainder), where every row is a directory and the row is
/// counted separately. Here a row may be a file, so the useful number is what the rows
/// account for.
///
/// No newest-mtime field: a maximum cannot be subtracted back out, and a figure that is
/// sometimes wrong is worse than one that is absent.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ChildRemainder {
    /// Child rows this page does not carry.
    pub rows: u64,
    /// Files those rows account for.
    pub files: u64,
    /// Directories those rows account for.
    pub dirs: u64,
    /// Symlinks and other non-file, non-directory entries those rows account for.
    pub others: u64,
    /// Apparent bytes those rows account for.
    pub bytes: u64,
    /// Allocated bytes those rows account for.
    pub allocated: u64,
}

impl ChildPage {
    /// Whether this page carries fewer than the directory's children.
    ///
    /// Derived rather than stored, so it cannot disagree with the remainder.
    pub fn truncated(&self) -> bool {
        self.remainder.is_some()
    }

    /// Whether another page follows this one.
    ///
    /// Derived from the cursor, for the same reason.
    pub fn has_next(&self) -> bool {
        self.next.is_some()
    }
}

/// What one read actually did, beside the answer rather than inside it.
///
/// Execution telemetry, not a fact about the tree: two reads that answer identically can
/// do very different amounts of work, and the difference is what a serving loop needs to
/// see. It also turns "no hidden `O(index)` pass" from a review principle into something
/// a test can assert -- a frequent read must be proportional to its own output or to
/// maintained state, and `entries_visited` is where a regression shows up.
///
/// # What is deliberately not here
///
/// **CPU time.** A read on a maintained index performs no I/O, so its wall time is CPU
/// time plus whatever it spent waiting for the guard, and `lock_wait_ns` already
/// separates those. Sampling a thread clock would add a platform-gated syscall per read
/// to restate a number these two already carry.
///
/// **Bytes copied across a language binding.** The engine cannot see a binding, and a
/// binding can only estimate what its own serialisation costs. `name_bytes` is the one
/// term that grows without bound, and it is exact; the rest is a fixed per-row schema
/// that `rows` and `tally_rows` multiply.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Work {
    /// Index entries this read examined, including those it walked past to find a path.
    ///
    /// The load-bearing number. A read of maintained state visits its path's depth plus
    /// the rows it returns; one that visits a subtree is doing an aggregate pass, and
    /// says so here whatever its result looks like.
    pub entries_visited: u64,
    /// Directories among them.
    pub dirs_visited: u64,
    /// Rows the result carries.
    pub rows: u64,
    /// Extension and group tallies the result *examined*, which a bound may exceed.
    ///
    /// Examined rather than returned on purpose: bounding a roll-up's extension rows
    /// still ranks every tally to decide which ones survive, and a counter that reported
    /// the bound would hide exactly that.
    pub tally_rows: u64,
    /// Bytes of entry and extension names the result carries.
    ///
    /// The only unbounded term in what a consumer copies out of a result; everything else
    /// is a fixed-width row.
    pub name_bytes: u64,
    /// Nanoseconds spent waiting for the read guard.
    ///
    /// Separated from `wall_ns` because a slow read and a read behind a long write are
    /// different problems with different fixes, and a single duration cannot tell a
    /// serving loop which one it has.
    pub lock_wait_ns: u64,
    /// Nanoseconds from entering the call to returning, guard wait included.
    pub wall_ns: u64,
}

/// A sink for the entries a lookup walks past.
///
/// Generic rather than an `Option<&mut Work>` so the uncounted case compiles to nothing:
/// [`Index::lookup`] is on the apply path, where an unconditional counter would be a
/// throughput change to justify rather than an instrument. One body serves both, so the
/// counted and uncounted walks cannot drift apart.
trait Visits {
    fn visit(&mut self, kind: EntryKind);
}

/// The uncounted case. Every method is empty, so a monomorphised walk keeps no counter.
struct Uncounted;

impl Visits for Uncounted {
    #[inline]
    fn visit(&mut self, _kind: EntryKind) {}
}

impl Visits for Work {
    #[inline]
    fn visit(&mut self, kind: EntryKind) {
        self.entries_visited += 1;
        if kind.is_dir() {
            self.dirs_visited += 1;
        }
    }
}

/// What one bundled read should evaluate.
///
/// Everything is optional because a caller composes the page it is drawing: a directory
/// listing wants children and its own totals, a breadcrumb wants several roll-ups and no
/// children at all.
#[derive(Clone, Debug, Default)]
pub struct ReadRequest {
    /// Directory whose children to list, when any.
    pub children_of: Option<PathBuf>,
    /// Which page of that directory's children to return.
    pub children_page: ChildPageRequest,
    /// Relative paths whose roll-ups to return, in the order given.
    pub rollups: Vec<PathBuf>,
    /// Whether to include the whole-tree totals.
    pub total: bool,
    /// Bound on the extension rows every roll-up in this bundle carries.
    pub extensions: Bound,
}

/// Everything one bundled read saw, at one boundary.
///
/// The parts cannot disagree, because there was no moment between them at which a write
/// could land. `clock` is the version all of them saw and the cursor to resume from.
#[derive(Clone, Debug)]
pub struct ReadBundle {
    /// The version every part of this bundle saw, and the cursor to pass to
    /// [`IndexHandle::since`] next.
    pub clock: Clock,
    /// The scan scope this index represents, carrying the ignore, type-rule, and reducer
    /// fingerprints a consumer cache key should derive from.
    pub scope: ScanScope,
    /// Whether the index is fully verified, or holds anything stale or unverified.
    pub freshness: Freshness,
    /// Live entries, including the root.
    pub entries: u64,
    /// Absolute filesystem root.
    pub root: PathBuf,
    /// Whole-tree totals, when requested.
    pub total: Option<RollUp>,
    /// One entry per requested path, `None` where the path is absent or not a directory.
    pub rollups: Vec<Option<RollUp>>,
    /// The requested directory's children, or `None` when it is absent or not a directory.
    ///
    /// Distinct from a page with no rows, which means a directory with no children.
    pub children: Option<ChildPage>,
    /// What producing this bundle cost.
    ///
    /// Here rather than on each part because the parts shared one guard and one wall
    /// clock: attributing a lock wait to one of three projections that waited together
    /// would be inventing a number. This is also why measurement rides with the bundled
    /// read rather than with every accessor -- the bundled read is what an interactive
    /// client serves from.
    pub work: Work,
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
    tag_rules: Arc<crate::tags::TagRules>,
    freshness_epoch: u64,
    freshness_marks: BTreeMap<PathBuf, FreshnessMark>,
}

#[derive(Clone, Copy, Debug)]
struct FreshnessMark {
    state: Freshness,
    epoch: u64,
    /// Why coverage was lost, for the marks that lose it.
    ///
    /// `None` for every mark whose state is about trust rather than coverage --
    /// `Reconciling` and `Stale` both leave a subtree fully accounted for. Carried here
    /// because the site that marks a subtree is the only one that still knows why, and
    /// re-deriving a reason at read time from a state that has forgotten it is how a
    /// plausible-but-wrong reason gets reported.
    reason: Option<CoverageReason>,
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
        self.total_bounded(Bound::All)
    }

    /// [`IndexHandle::total`] carrying at most `extensions` extension rows.
    pub fn total_bounded(&self, extensions: Bound) -> crate::Result<RollUp> {
        Ok(self.read_index()?.total_bounded(extensions))
    }

    /// Owned roll-up state for a relative directory path.
    pub fn rollup(&self, path: &Path) -> crate::Result<Option<RollUp>> {
        self.rollup_bounded(path, Bound::All)
    }

    /// [`IndexHandle::rollup`] carrying at most `extensions` extension rows.
    pub fn rollup_bounded(&self, path: &Path, extensions: Bound) -> crate::Result<Option<RollUp>> {
        Ok(self.read_index()?.rollup_bounded(path, extensions))
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

    /// Several projections evaluated under one read guard.
    ///
    /// A composed response must not straddle a commit. Answering a directory listing and
    /// its parent's totals with two calls lets a write land between them, and the
    /// resulting page is internally inconsistent in a way nothing in it reports -- the
    /// rows say one thing, the header another, and both are individually true. One guard
    /// makes that impossible rather than unlikely.
    ///
    /// The returned `clock` is the version every part of this bundle saw, so it is also
    /// the cursor to pass to [`Self::since`] next time: a consumer's cache key derives
    /// from what was actually read rather than from a version sampled before dispatch.
    ///
    /// It also collapses the per-call cost. Across a language boundary each of these is a
    /// crossing and a lock acquisition; bundled, they are one of each.
    pub fn read(&self, request: &ReadRequest) -> crate::Result<ReadBundle> {
        let started = std::time::Instant::now();
        let index = self.read_index()?;
        // Each projection counts the entries it walks, so a bundle's visits are the sum
        // of its parts -- including the root, counted once per projection that reads it,
        // because that is what a projection actually does.
        let mut work = Work { lock_wait_ns: nanos(started.elapsed()), ..Work::default() };
        let children = request
            .children_of
            .as_deref()
            .and_then(|path| child_page(&index, path, &request.children_page, &mut work));
        let total = request.total.then(|| {
            let root = &index.entry(EntryId::ROOT).rollup;
            work.visit(EntryKind::Dir);
            work.tally_rows += (root.by_ext.len() + root.by_group.len()) as u64;
            let roll = index.total_bounded(request.extensions);
            work.name_bytes += roll.by_ext.keys().map(|name| name.len() as u64).sum::<u64>();
            work.name_bytes += roll.by_group.keys().map(|name| name.len() as u64).sum::<u64>();
            roll
        });
        let rollups: Vec<Option<RollUp>> = request
            .rollups
            .iter()
            .map(|path| index.rollup_measured(path, request.extensions, &mut work))
            .collect();
        let bundle = ReadBundle {
            clock: index.clock(),
            scope: index.scope(),
            freshness: index.freshness(),
            entries: index.len(),
            root: index.root_path().to_path_buf(),
            total,
            rollups,
            children,
            work,
        };
        // Stamped last, and after the guard is still held, so the figure covers the whole
        // call rather than the part that happened to be convenient to time.
        Ok(ReadBundle { work: Work { wall_ns: nanos(started.elapsed()), ..bundle.work }, ..bundle })
    }

    /// Every direct child, captured coherently at one read boundary.
    ///
    /// Unbounded, so its cost is the directory's width. Prefer
    /// [`children_page`](Self::children_page) anywhere the caller is drawing a screen.
    pub fn children(&self, path: &Path) -> crate::Result<Option<Vec<ChildSnapshot>>> {
        Ok(self.children_page(path, &ChildPageRequest::default())?.map(|page| page.rows))
    }

    /// One page of a directory's children, with the rest accounted for.
    ///
    /// The listing and the breakdown are separate questions and now cost separately. A
    /// row carries scalar subtree totals, its classification, and its provenance; the
    /// per-extension breakdown belongs to [`rollup_bounded`](Self::rollup_bounded) for
    /// the single directory a consumer opened. Returning it per row meant a wide
    /// directory cloned one `BTreeMap` per child to render one number per child.
    ///
    /// Work is proportional to the rows returned, not to the directory's width: the page
    /// is a range seek from the cursor, and the remainder is the directory's own roll-up
    /// minus what was emitted.
    pub fn children_page(
        &self,
        path: &Path,
        request: &ChildPageRequest,
    ) -> crate::Result<Option<ChildPage>> {
        let index = self.read_index()?;
        Ok(child_page(&index, path, request, &mut Work::default()))
    }

    /// Origin, observation time, and coverage for one retained path.
    pub fn provenance(&self, path: &Path) -> crate::Result<Option<Provenance>> {
        Ok(self.read_index()?.provenance(path))
    }

    /// Run content analysis against the held index.
    ///
    /// Unlike metadata reconciliation, this holds the write lock for its whole run: the
    /// analyzers read file bodies and commit per-file records, so there is no wave
    /// boundary to publish between. Readers are served throughout a metadata refresh and
    /// are not served throughout an analysis pass, which is why the content tier stays
    /// opt-in.
    pub fn analyze(
        &self,
        request: crate::content::AnalysisRequest,
    ) -> crate::Result<crate::content::AnalysisReport> {
        let mut index = self.write_index()?;
        Ok(crate::content::analyze_index(&mut index, request))
    }

    /// Read the held index in place, without copying it.
    ///
    /// For a pure reader such as `report`, which needs `&Index` but neither retains it
    /// nor blocks inside it. [`snapshot`](Self::snapshot) would answer the same question
    /// by cloning every entry, which is O(entries) per call and turned a millisecond
    /// report into seconds on a large tree; this holds the read lock for the duration
    /// instead, which other readers share and only a writer waits behind.
    ///
    /// Do not do filesystem or network work inside `read`: a writer is blocked until it
    /// returns.
    pub fn with_index<R>(&self, read: impl FnOnce(&Index) -> R) -> crate::Result<R> {
        let index = self.read_index()?;
        Ok(read(&index))
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
        coverage: Status,
    ) -> crate::Result<()> {
        self.write_index()?.finish_reconcile(path, started_at, coverage);
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
            group_id: None,
            tag_bits: 0,
            source: Source::Scanned,
            kind: EntryKind::Dir,
            attrs: Attrs::default(),
            children: BTreeMap::new(),
            rollup: InternedRollUp::default(),
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
            verified: Vec::new(),
            ext_names: Vec::new(),
            ext_ids: BTreeMap::new(),
            ext_refcounts: Vec::new(),
            free_ext_ids: Vec::new(),
            content: None,
            types: crate::classify::TypeRegistry::compiled().clone(),
            tag_rules: std::sync::Arc::new(crate::tags::TagRules::none().clone()),
        }
    }

    /// Adopt the file-type rules this index is being built under.
    ///
    /// Taken from the scan config so the rules and the scope's fingerprint of them are
    /// established together; defaulting one and setting the other is how they come apart.
    #[must_use]
    pub fn with_types(mut self, types: std::sync::Arc<crate::classify::TypeRegistry>) -> Self {
        self.types = types;
        self
    }

    /// The file-type rules this index classifies against.
    pub fn types(&self) -> &std::sync::Arc<crate::classify::TypeRegistry> {
        &self.types
    }

    /// Adopt the tag rules this index is being built under.
    ///
    /// Taken from the scan config for the same reason the type registry is: the rules and
    /// the scope's fingerprint of them are established together, and defaulting one while
    /// setting the other is how they come apart.
    #[must_use]
    pub fn with_tag_rules(mut self, tag_rules: Arc<crate::tags::TagRules>) -> Self {
        self.tag_rules = tag_rules;
        self.retag();
        self
    }

    /// Recompute every entry's tag bits under the current rules.
    ///
    /// Called from [`Index::with_tag_rules`], which on the scan path runs against an empty
    /// index and does nothing. It exists for the *load* path, where it is the whole reason
    /// tag bits are not in the snapshot format: a snapshot is adopted into a caller's rules
    /// after it is read, so entries arrive untagged, and a warm start would otherwise
    /// answer every tag question with "no" -- a bug that reads as a cache fault rather than
    /// a tagging one. Re-deriving is also the cheaper contract: bits are a pure function of
    /// facts the index already holds, so storing them would widen the format to cache
    /// something a traversal reproduces.
    fn retag(&mut self) {
        if self.tag_rules.is_empty() {
            // Nothing to compute, and nothing to clear: a snapshot written under a
            // non-empty set never reaches an index with an empty one, because the scope
            // fingerprints differ and the loader rejects it before this runs.
            return;
        }
        let rules = Arc::clone(&self.tag_rules);
        // Walked from the root with the path carried down rather than reconstructed per
        // entry, so a Path-tier rule costs one join per entry instead of one ancestor
        // chain. Collected first because evaluating borrows the entry the assignment
        // writes to.
        let mut computed: Vec<(EntryId, crate::tags::TagBits)> = Vec::new();
        let mut stack: Vec<(EntryId, PathBuf)> = vec![(EntryId::ROOT, PathBuf::new())];
        while let Some((id, path)) = stack.pop() {
            let Some(entry) = self.try_entry(id) else {
                continue;
            };
            // The root has no name and cannot be tagged; every other entry can.
            if entry.parent.is_some() {
                computed.push((id, rules.evaluate(&entry.name, || Cow::Borrowed(path.as_path()))));
            }
            for (name, child) in &entry.children {
                stack.push((*child, path.join(name)));
            }
        }
        for (id, bits) in computed {
            self.entry_mut(id).tag_bits = bits;
        }
    }

    /// The tag rules this index evaluates.
    pub fn tag_rules(&self) -> &Arc<crate::tags::TagRules> {
        &self.tag_rules
    }

    /// Raw tag bits for an entry id, or zero when the handle is stale.
    ///
    /// The filtering path reads bits rather than names: selection compares masks, and
    /// resolving to strings per entry to compare strings would be the expensive way to ask
    /// the same question.
    pub fn tag_bits_of(&self, id: EntryId) -> crate::tags::TagBits {
        self.try_entry(id).map_or(0, |entry| entry.tag_bits)
    }

    /// Tags carried by one relative path, resolved to names.
    ///
    /// Empty for an absent path and for one no enabled rule matches; a caller
    /// distinguishing those wants [`Index::path_state`].
    pub fn tags_of(&self, path: &Path) -> Vec<&str> {
        self.lookup(path)
            .map(|id| self.tag_rules.names_of(self.entry(id).tag_bits))
            .unwrap_or_default()
    }

    /// Classify one relative path under this index's rules, without opening the file.
    pub fn classify(&self, relative_path: &Path) -> crate::classify::Classification {
        crate::classify::classify_with(&self.types, relative_path, None)
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

    /// How much of one subtree the index accounts for, and why not all of it.
    ///
    /// Resolved from the same overlapping marks as [`Index::freshness_at`] and by the
    /// same rule -- worst wins -- but over a different axis. Only [`Freshness::Partial`]
    /// marks contribute: `Reconciling` and `Stale` describe values whose *trust* is in
    /// doubt while their coverage is not.
    pub fn coverage_at(&self, path: &Path) -> Status {
        self.freshness_marks
            .iter()
            .filter(|(marked, _)| path.starts_with(marked) || marked.starts_with(path))
            .filter_map(|(_, mark)| mark.reason)
            .max()
            .map_or(Status::Complete, Status::Partial)
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

    /// Owned, self-describing roll-up state for the whole tree.
    pub fn total(&self) -> RollUp {
        self.total_bounded(Bound::All)
    }

    /// [`Index::total`] carrying at most `extensions` extension rows.
    pub fn total_bounded(&self, extensions: Bound) -> RollUp {
        self.named_rollup_bounded(&self.entry(EntryId::ROOT).rollup, extensions)
    }

    /// Map-free whole-tree totals for in-crate reporting paths.
    pub(crate) fn total_scalars(&self) -> RollUpScalars {
        RollUpScalars::from(&self.entry(EntryId::ROOT).rollup)
    }

    /// Arbitrate a producer observation and commit its effective mutations.
    ///
    /// Conditional operations are accepted only while their baseline still matches.
    /// No-ops and stale operations do not advance the clock or enter the journal.
    pub fn apply(&mut self, observation: &Observation) -> crate::Result<ApplyOutcome> {
        self.apply_with(observation, true)
    }

    /// [`Self::apply`] with the change-history capture optional.
    ///
    /// `journal: false` exists for the bootstrap path, whose history
    /// [`Self::establish_baseline`] clears after every batch: capturing it first
    /// cost one op clone per changed entry plus one delta clone per batch, all
    /// freed unread. Arbitration, validation, guards, and stats are identical in
    /// both modes; only what is retained afterwards differs.
    fn apply_with(
        &mut self,
        observation: &Observation,
        journal: bool,
    ) -> crate::Result<ApplyOutcome> {
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

        Ok(self.apply_validated_with(observation, next_clock, journal))
    }

    /// Apply an already validated observation with a clock known to be available.
    fn apply_validated(&mut self, observation: &Observation, next_clock: Clock) -> ApplyOutcome {
        self.apply_validated_with(observation, next_clock, true)
    }

    fn apply_validated_with(
        &mut self,
        observation: &Observation,
        next_clock: Clock,
        journal: bool,
    ) -> ApplyOutcome {
        let mut stats = ApplyStats::default();
        let mut effective = Vec::new();
        let mut changed_ops = 0usize;
        let mut parent_memo = ParentMemo::default();
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
                    self.apply_upsert(path, *kind, *attrs, &mut stats, &mut parent_memo)
                }
                Op::Remove { path } => {
                    // A removal takes a subtree with it, so a remembered id inside that
                    // subtree would dangle. Both of the non-upsert arms drop the memo
                    // rather than reason about whether this particular path could be an
                    // ancestor of it: the memo is refilled by the next upsert, so the
                    // cost of being conservative is one path resolution.
                    parent_memo.clear();
                    self.apply_remove(path, &mut stats)
                }
                Op::InvalidateSubtree { path, reason } => {
                    parent_memo.clear();
                    self.pending_invalidations.push((path.clone(), *reason));
                    self.mark_unfresh(path, Freshness::Stale);
                    stats.invalidated += 1;
                    true
                }
            };
            if changed {
                changed_ops += 1;
                if journal {
                    effective.push(op.clone());
                }
            }
        }

        if changed_ops == 0 {
            return ApplyOutcome { stats, applied: None };
        }

        self.clock = next_clock;
        if !journal {
            // The caller declared this history unread: no delta is minted and the
            // journal keeps whatever it held, which `establish_baseline` clears.
            return ApplyOutcome { stats, applied: None };
        }
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
        self.journal_ops = 0;
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

    pub(crate) fn set_initial_coverage(&mut self, coverage: Status) {
        self.freshness_marks.clear();
        if let Status::Partial(reason) = coverage {
            self.mark_unfresh_because(Path::new(""), Freshness::Partial, Some(reason));
        }
    }

    pub(crate) fn begin_reconcile(&mut self, path: &Path) -> u64 {
        self.mark_unfresh(path, Freshness::Reconciling)
    }

    pub(crate) fn finish_reconcile(&mut self, path: &Path, started_at: u64, coverage: Status) {
        self.freshness_marks
            .retain(|marked, mark| !marked.starts_with(path) || mark.epoch > started_at);
        if let Status::Partial(reason) = coverage {
            self.mark_unfresh_because(path, Freshness::Partial, Some(reason));
            return;
        }
        // A completed sweep stat'd every entry beneath `path`, including the ones the
        // producer elided as no-ops before they ever reached a delta. Per-entry
        // stamping cannot see those, so verification is recorded here as an interval
        // instead: one record per reconciled subtree rather than a write to each of
        // millions of entries. This is the same "store where it varies, derive where
        // it does not" choice as the timestamps, and it keeps the elision — a measured
        // 18% win on the warm path — intact.
        let now = Self::now_unix_nanos();
        self.verified.retain(|(verified_path, _)| !verified_path.starts_with(path));
        self.verified.push((path.to_path_buf(), now));
        // Repeated scoped sweeps of sibling subtrees — what a consumer revalidating
        // per navigation produces — would otherwise grow this list without bound,
        // since only records *under* the swept path are collapsed. Dropping the oldest
        // is fail-safe: a path that loses its interval reports `Cached` rather than
        // `Revalidated`, which under-claims trust rather than over-claiming it.
        if self.verified.len() > MAX_VERIFIED_INTERVALS {
            let excess = self.verified.len() - MAX_VERIFIED_INTERVALS;
            self.verified.sort_by_key(|(_, at)| *at);
            self.verified.drain(..excess);
        }
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
        self.mark_unfresh_because(path, state, None)
    }

    fn mark_unfresh_because(
        &mut self,
        path: &Path,
        state: Freshness,
        reason: Option<CoverageReason>,
    ) -> u64 {
        debug_assert_eq!(
            reason.is_some(),
            state == Freshness::Partial,
            "a coverage reason belongs to Partial and to nothing else"
        );
        self.freshness_epoch =
            self.freshness_epoch.checked_add(1).expect("freshness epoch exhausted");
        let epoch = self.freshness_epoch;
        self.freshness_marks.insert(path.to_path_buf(), FreshnessMark { state, epoch, reason });
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
        self.lookup_visiting(path, &mut Uncounted)
    }

    /// [`Index::lookup`], reporting each entry it walks past to a sink.
    ///
    /// The one body both cases share; see [`Visits`] for why the sink is a type parameter
    /// rather than an option.
    fn lookup_visiting<V: Visits>(&self, path: &Path, visits: &mut V) -> Option<EntryId> {
        let mut current = EntryId::ROOT;
        visits.visit(EntryKind::Dir);
        for part in normalize(path)? {
            current = *self.entry(current).children.get(part)?;
            visits.visit(self.entry(current).kind);
        }
        Some(current)
    }

    /// Owned, self-describing roll-up state for a directory by relative path.
    /// The empty path is the root.
    pub fn rollup(&self, path: &Path) -> Option<RollUp> {
        self.rollup_bounded(path, Bound::All)
    }

    /// [`Index::rollup`] carrying at most `extensions` extension rows.
    ///
    /// A wide subtree can hold hundreds of distinct extensions while a listing shows a
    /// handful. Bounding here rather than at the caller keeps the rows that were dropped
    /// accounted for -- see [`RollUp::ext_remainder`] -- and avoids materialising names
    /// nobody reads.
    pub fn rollup_bounded(&self, path: &Path, extensions: Bound) -> Option<RollUp> {
        self.rollup_measured(path, extensions, &mut Work::default())
    }

    /// [`Index::rollup_bounded`], folding what it cost into a running record.
    fn rollup_measured(&self, path: &Path, extensions: Bound, work: &mut Work) -> Option<RollUp> {
        let id = self.lookup_visiting(path, work)?;
        let entry = self.entry(id);
        entry.kind.is_dir().then(|| {
            work.tally_rows += (entry.rollup.by_ext.len() + entry.rollup.by_group.len()) as u64;
            let roll = self.named_rollup_bounded(&entry.rollup, extensions);
            work.name_bytes += roll.by_ext.keys().map(|name| name.len() as u64).sum::<u64>();
            work.name_bytes += roll.by_group.keys().map(|name| name.len() as u64).sum::<u64>();
            roll
        })
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

    /// Owned, self-describing roll-up state for an entry id, if it is a directory.
    pub fn rollup_of(&self, id: EntryId) -> Option<RollUp> {
        let entry = self.try_entry(id)?;
        entry.kind.is_dir().then(|| self.named_rollup(&entry.rollup))
    }

    /// Map-free directory totals for in-crate reporting paths.
    pub(crate) fn rollup_scalars_of(&self, id: EntryId) -> Option<RollUpScalars> {
        let entry = self.try_entry(id)?;
        entry.kind.is_dir().then(|| RollUpScalars::from(&entry.rollup))
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
    /// **Provenance transitions are not clocked.** A path moving `Cached` ->
    /// `Revalidated` is visible by polling here, but it does not advance [`Clock`],
    /// does not appear in `since()`, and does not reach the `AppliedDelta` sink,
    /// because the sweep that caused it committed no change. A consumer following the
    /// change feed therefore sees nothing while a consumer polling this sees a
    /// transition. Making the two agree needs provenance to travel *on* the committed
    /// operation, which is a delta-format change and a state machine rather than a
    /// patch (`fdu-jxs0`, `fdu-livs`). Until then, treat this as a poll-only view.
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
        self.coverage_at(&path)
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
        self.named_rollup_bounded(rollup, Bound::All)
    }

    /// Resolve interned extension ids to names, keeping at most what `extensions` admits.
    ///
    /// Under a limit the kept rows are the largest by apparent bytes, ties broken by name
    /// so the same subtree always yields the same rows -- an order that varied with
    /// hashing would make a golden pass on one run and fail on the next. What the limit
    /// drops is aggregated rather than discarded, so a caller showing five rows can still
    /// say what the sixth through five-hundredth were worth.
    ///
    /// The limit is applied before the names are cloned: a wide subtree's cost is then
    /// one pass for the selection plus `n` clones, rather than one clone per distinct
    /// extension for a map the caller is about to throw most of away.
    fn named_rollup_bounded(&self, rollup: &InternedRollUp, extensions: Bound) -> RollUp {
        let name_of = |id: &ExtId| -> &String {
            self.ext_names
                .get(*id as usize)
                .and_then(Option::as_ref)
                .expect("a live roll-up's extension id has a name")
        };

        let (by_ext, ext_remainder) = match extensions.limit() {
            Some(limit) if limit < rollup.by_ext.len() => {
                let mut ranked: Vec<(&ExtId, &ExtTally)> = rollup.by_ext.iter().collect();
                // `sort_unstable_by` is fine because the key is total: no two entries
                // share both a byte count and a name.
                ranked.sort_unstable_by(|(left_id, left), (right_id, right)| {
                    right
                        .bytes
                        .cmp(&left.bytes)
                        .then_with(|| name_of(left_id).cmp(name_of(right_id)))
                });
                let mut remainder = ExtRemainder::default();
                for (_, tally) in &ranked[limit..] {
                    remainder.extensions += 1;
                    remainder.files += tally.files;
                    remainder.bytes += tally.bytes;
                    remainder.allocated += tally.allocated;
                }
                let kept = ranked[..limit]
                    .iter()
                    .map(|(id, tally)| (name_of(id).clone(), **tally))
                    .collect();
                (kept, Some(remainder))
            }
            _ => (
                rollup.by_ext.iter().map(|(id, tally)| (name_of(id).clone(), *tally)).collect(),
                None,
            ),
        };

        // Group ids are indexes into this index's registry, so they are resolved to
        // names here for the same reason extension ids are: a retained roll-up must not
        // be relabelled by a later registry.
        let by_group = rollup
            .by_group
            .iter()
            .filter_map(|(id, tally)| {
                self.types.group(*id).map(|group| (group.id.to_string(), *tally))
            })
            .collect();

        RollUp {
            files: rollup.files,
            dirs: rollup.dirs,
            others: rollup.others,
            bytes: rollup.bytes,
            allocated: rollup.allocated,
            newest_mtime_ns: rollup.newest_mtime_ns,
            by_ext,
            by_group,
            ext_remainder,
        }
    }

    /// What an entry contributes to each of its ancestors.
    fn contribution(&self, id: EntryId) -> InternedRollUp {
        let entry = self.entry(id);
        match entry.kind {
            EntryKind::Dir => {
                let mut roll = entry.rollup.clone();
                roll.dirs += 1;
                roll
            }
            EntryKind::File => {
                let mut roll = InternedRollUp {
                    files: 1,
                    dirs: 0,
                    others: 0,
                    bytes: entry.attrs.size,
                    allocated: entry.attrs.allocated,
                    newest_mtime_ns: entry.attrs.mtime_ns,
                    by_ext: BTreeMap::new(),
                    by_group: Vec::new(),
                };
                if let Some(group_id) = entry.group_id {
                    roll.by_group.push((
                        group_id,
                        ExtTally {
                            files: 1,
                            bytes: entry.attrs.size,
                            allocated: entry.attrs.allocated,
                        },
                    ));
                }
                if let Some(ext_id) = entry.ext_id {
                    roll.by_ext.insert(
                        ext_id,
                        ExtTally {
                            files: 1,
                            bytes: entry.attrs.size,
                            allocated: entry.attrs.allocated,
                        },
                    );
                }
                roll
            }
            // Zero bytes, and one entry. A default here made a subtree of symlinks
            // arithmetically identical to an empty one, so nothing downstream could tell
            // "nothing is here" from "nothing here has a size".
            EntryKind::Symlink | EntryKind::Other => {
                InternedRollUp { others: 1, ..InternedRollUp::default() }
            }
        }
    }

    fn merge_upward(&mut self, from_parent: Option<EntryId>, contribution: &InternedRollUp) {
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

    fn unmerge_upward(&mut self, from_parent: Option<EntryId>, contribution: &InternedRollUp) {
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
        let rules = Arc::clone(&self.tag_rules);
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
            // Tagged here rather than left at zero for a later observation to fill in.
            // `apply_upsert` on an existing entry of the same kind rewrites attributes and
            // source only, so a placeholder that entered untagged would stay untagged for
            // the life of the index — and every ancestor of a deep first observation
            // enters through this line.
            let tag_bits = rules.evaluate(part, || {
                Cow::Owned(self.path_of(current).unwrap_or_default().join(part))
            });
            let child = self.alloc(Entry {
                parent: Some(current),
                name: (*part).to_os_string(),
                ext_id: None,
                group_id: None,
                tag_bits,
                source: self.applying_source,
                kind: EntryKind::Dir,
                attrs: Attrs::default(),
                children: BTreeMap::new(),
                rollup: InternedRollUp::default(),
                revision: 0,
                children_revision: 0,
            });
            self.insert_child(current, (*part).to_os_string(), child);
            // A new empty directory contributes one to `dirs` all the way up.
            let contribution = InternedRollUp { dirs: 1, ..InternedRollUp::default() };
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
                return self.upsert_beneath(parent, name, path, kind, attrs, stats);
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
            root.attrs = attrs;
            root.source = source;
            Self::bump_revision(root);
            stats.updated += 1;
            return true;
        };
        let parent = self.ensure_dir_chain(ancestors, stats);
        if let Some(dir) = path.parent() {
            parent_memo.set(dir, parent);
        }
        self.upsert_beneath(parent, name, path, kind, attrs, stats)
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
                    entry.attrs = attrs;
                    entry.source = source;
                    Self::bump_revision(entry);
                    stats.updated += 1;
                    return true;
                }
                self.invalidate_content(path);
                let old = self.contribution(id);
                self.unmerge_upward(Some(parent), &old);
                let entry = self.entry_mut(id);
                entry.attrs = attrs;
                entry.source = source;
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
            //
            // This drops a subtree but cannot invalidate the memo: the memo holds this
            // entry's *parent*, and the subtree removed is rooted at the entry itself.
            // Clearing here would be untestable defensive code, which reads as a hazard
            // that does not exist.
            self.remove_entry(id, stats);
        }

        let ext_id = (kind == EntryKind::File)
            .then(|| self.intern_ext(&ext_bucket(&self.types.clone(), name)));
        let group_id = (kind == EntryKind::File).then(|| self.types.group_of_name(name)).flatten();
        let tag_bits = Arc::clone(&self.tag_rules).evaluate(name, || Cow::Borrowed(path));
        let id = self.alloc(Entry {
            parent: Some(parent),
            name: name.to_os_string(),
            tag_bits,
            ext_id,
            group_id,
            source,
            kind,
            attrs,
            children: BTreeMap::new(),
            rollup: InternedRollUp::default(),
            revision: 0,
            children_revision: 0,
        });
        self.insert_child(parent, name.to_os_string(), id);
        let contribution = self.contribution(id);
        self.merge_upward(Some(parent), &contribution);
        stats.inserted += 1;
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
        let ext_id = (kind == EntryKind::File)
            .then(|| self.intern_ext(&ext_bucket(&self.types.clone(), &name)));
        let group_id = (kind == EntryKind::File).then(|| self.types.group_of_name(&name)).flatten();
        // The path is a closure, not a value: this is the loader, which holds a parent id
        // and a basename precisely so that it never joins one per record.  No Name-tier
        // rule calls it.
        let tag_bits = Arc::clone(&self.tag_rules)
            .evaluate(&name, || Cow::Owned(self.path_of(parent).unwrap_or_default().join(&name)));
        let id = self.alloc(Entry {
            parent: Some(parent),
            name: name.clone(),
            tag_bits,
            ext_id,
            group_id,
            source,
            kind,
            attrs,
            children: BTreeMap::new(),
            rollup: InternedRollUp::default(),
            revision: 0,
            children_revision: 0,
        });
        self.insert_child(parent, name, id);
        // Roll-ups stay eager. The same profile put `merge_upward` at about 3.5%, so
        // deferring it to a bottom-up pass would buy little and would introduce a window
        // in which the index is structurally complete but numerically wrong.
        let contribution = self.contribution(id);
        self.merge_upward(Some(parent), &contribution);
        Some(id)
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
        if let Some(path) = self.path_of(id) {
            self.invalidate_content(&path);
        }
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
            let ext_id = self.entry(node).ext_id;
            queue.extend(children);
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

/// One page of a directory's children, built the same way for a listing and for a
/// bundled read.
///
/// Shared so the two cannot describe the same directory differently: a bundle whose rows
/// disagreed with the listing API's would be a second definition of what a child is.
///
/// Returns `None` for a path that is absent or is not a directory, which is distinct from
/// a page with no rows.
fn child_page(
    index: &Index,
    path: &Path,
    request: &ChildPageRequest,
    work: &mut Work,
) -> Option<ChildPage> {
    let id = index.lookup_visiting(path, work)?;
    let entry = index.entry(id);
    if !entry.kind.is_dir() {
        return None;
    }

    // Seek rather than scan. `after` is exclusive, and `OsString`'s ordering is the one
    // the map is keyed by, so the excluded bound lands on exactly the cursor's own row.
    let remaining: &mut dyn Iterator<Item = (&OsString, &EntryId)> =
        &mut match &request.after {
            Some(after) => Either::Right(entry.children.range::<OsString, _>((
                std::ops::Bound::Excluded(after),
                std::ops::Bound::Unbounded,
            ))),
            None => Either::Left(entry.children.iter()),
        };

    let mut emitted = ChildRemainder::default();
    let mut rows = Vec::new();
    let mut last = None;
    let mut more = false;
    for (name, child) in remaining {
        if !request.limit.admits(rows.len()) {
            // The loop stops on the first child past the bound, so reaching here is the
            // O(1) proof that another page exists -- no second pass, no tail count.
            more = true;
            break;
        }
        emitted.absorb(index, *child);
        last = Some(name.clone());
        let row = child_snapshot(index, name.as_os_str(), *child);
        work.visit(row.kind);
        work.rows += 1;
        work.name_bytes += name.as_encoded_bytes().len() as u64;
        rows.push(row);
    }

    // The remainder is this page's complement within the whole directory, so it is the
    // directory's width less the rows returned and its roll-up less what they accounted
    // for. Both read off state the index already maintains: no withheld child is touched,
    // on any page.
    let withheld = (entry.children.len() - rows.len()) as u64;
    let remainder = (withheld > 0).then(|| ChildRemainder {
        rows: withheld,
        files: entry.rollup.files - emitted.files,
        dirs: entry.rollup.dirs - emitted.dirs,
        others: entry.rollup.others - emitted.others,
        bytes: entry.rollup.bytes - emitted.bytes,
        allocated: entry.rollup.allocated - emitted.allocated,
    });

    Some(ChildPage { rows, remainder, next: more.then_some(last).flatten() })
}

impl ChildRemainder {
    /// Fold one emitted row's contribution to its parent's roll-up into the running sum.
    ///
    /// The same arithmetic [`Index::contribution`] performs when maintaining the parent,
    /// which is what makes emitted-plus-withheld exact rather than approximately right.
    fn absorb(&mut self, index: &Index, id: EntryId) {
        let entry = index.entry(id);
        match entry.kind {
            EntryKind::Dir => {
                self.files += entry.rollup.files;
                self.dirs += entry.rollup.dirs + 1;
                self.bytes += entry.rollup.bytes;
                self.allocated += entry.rollup.allocated;
            }
            EntryKind::File => {
                self.files += 1;
                self.bytes += entry.attrs.size;
                self.allocated += entry.attrs.allocated;
            }
            // No bytes, but one entry: a page that withheld only symlinks still says so.
            EntryKind::Symlink | EntryKind::Other => self.others += 1,
        }
    }
}

/// Two iterator shapes behind one name, so the cursor branch does not box its iterator.
enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<T, L: Iterator<Item = T>, R: Iterator<Item = T>> Iterator for Either<L, R> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self {
            Self::Left(left) => left.next(),
            Self::Right(right) => right.next(),
        }
    }
}

/// A duration as nanoseconds, saturating rather than wrapping.
///
/// A read that took more than 584 years is a broken clock, not a number worth
/// representing; saturating says so without making every caller handle a `u128`.
fn nanos(elapsed: std::time::Duration) -> u64 {
    u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX)
}

/// One child's captured row.
fn child_snapshot(index: &Index, name: &std::ffi::OsStr, id: EntryId) -> ChildSnapshot {
    let entry = index.entry(id);
    let is_file = entry.kind == EntryKind::File;
    let classification =
        is_file.then(|| crate::classify::classify_with(index.types(), Path::new(name), None));
    ChildSnapshot {
        id,
        name: name.to_os_string(),
        kind: entry.kind,
        attrs: entry.attrs,
        totals: entry.kind.is_dir().then(|| RollUpScalars::from(&entry.rollup)),
        provenance: index.provenance_of(id),
        tags: index
            .tag_rules()
            .names_of(entry.tag_bits)
            .iter()
            .map(|tag| (*tag).to_string())
            .collect(),
        classification: classification.clone(),
        extension: is_file.then(|| crate::classify::logical_ext(name)).flatten(),
        group: classification
            .and_then(|verdict| verdict.group)
            .and_then(|group| index.types().group(group))
            .map(|group| group.id.to_string()),
    }
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
///
/// Shared rather than re-derived because the obvious spelling of this check is
/// `!path.is_absolute()`, and that is wrong on Windows: `/escape.txt` is rooted but
/// carries no drive prefix, so `is_absolute` answers `false` and the path escapes the
/// root anyway. `..` slips past the same check on every platform. Asking about
/// components answers both at once.
pub(crate) fn path_is_representable(path: &Path) -> bool {
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

        // The upsert after the removal must name a path *inside* the removed directory
        // and must follow it immediately. Rebuilding `dir` first would refill the memo
        // from the root and hide the bug — the window is exactly one op wide.
        index.apply_ok(&Observation::new(vec![
            upsert("dir/b.txt", EntryKind::File, file_attrs(20, 1)),
            Op::Remove { path: PathBuf::from("dir") },
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
        assert!(no_op.applied.is_none());
        assert_eq!(stale.stale, 1);
        assert!(stale.applied.is_none());
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
                let before_total = index.total();
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
                assert_eq!(index.total(), before_total);
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

    /// A bundle's parts cannot disagree, even while a writer commits between reads.
    ///
    /// The property the primitive exists for, and it fails silently without it: two
    /// separate reads can straddle a commit, and the page they compose says one thing in
    /// its rows and another in its header, with both individually true and nothing in the
    /// response reporting the split. Here the oracle is arithmetic -- the root's totals
    /// must equal the sum over its children -- and it is checked while a writer is adding
    /// files as fast as it can.
    ///
    /// The negative was checked by hand rather than asserted, because asserting that a
    /// race *occurs* is a flaky test: replacing the bundle with two separate calls under
    /// this same writer fails within a few iterations (6 files in the rows against 4 in
    /// the header). So this passes because the guard holds, not because nothing overlapped.
    #[test]
    fn a_bundled_read_cannot_straddle_a_commit() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let handle = IndexHandle::new(Index::new("/root"));
        handle
            .apply(&Observation::new(vec![
                upsert("src", EntryKind::Dir, Attrs::default()),
                upsert("docs", EntryKind::Dir, Attrs::default()),
            ]))
            .expect("seed");

        let stop = Arc::new(AtomicBool::new(false));
        let writer_stop = Arc::clone(&stop);
        let writer_handle = handle.clone();
        let writer = std::thread::spawn(move || {
            for round in 0..400i64 {
                if writer_stop.load(Ordering::Relaxed) {
                    break;
                }
                let parent = if round % 2 == 0 { "src" } else { "docs" };
                writer_handle
                    .apply(&Observation::new(vec![upsert(
                        &format!("{parent}/f{round}.txt"),
                        EntryKind::File,
                        file_attrs(100, round + 1),
                    )]))
                    .expect("apply");
            }
        });

        let request = ReadRequest {
            children_of: Some(PathBuf::new()),
            total: true,
            ..ReadRequest::default()
        };
        for _ in 0..500 {
            let bundle = handle.read(&request).expect("bundle");
            let total = bundle.total.as_ref().expect("totals were requested");
            let children = bundle.children.as_ref().expect("the root is a directory");
            let from_children: u64 = children
                .rows
                .iter()
                .map(|child| child.totals.map_or(0, |totals| totals.files))
                .sum();
            assert_eq!(
                from_children, total.files,
                "one bundle's rows and header must describe one instant"
            );
            assert!(bundle.clock.0 >= 1, "every bundle reports the version it read");
        }
        stop.store(true, Ordering::Relaxed);
        writer.join().expect("writer");
    }

    /// A directory of six children, three of them directories carrying subtrees, so a
    /// page's rows and its remainder each have something to be wrong about.
    fn paged_directory() -> IndexHandle {
        let handle = IndexHandle::new(Index::new("/root"));
        handle
            .apply(&Observation::new(vec![
                upsert("a", EntryKind::Dir, file_attrs(0, 1)),
                upsert("a/deep", EntryKind::Dir, file_attrs(0, 1)),
                upsert("a/deep/one.txt", EntryKind::File, file_attrs(100, 1)),
                upsert("b.txt", EntryKind::File, file_attrs(200, 2)),
                upsert("c", EntryKind::Dir, file_attrs(0, 1)),
                upsert("c/two.txt", EntryKind::File, file_attrs(300, 3)),
                upsert("d.txt", EntryKind::File, file_attrs(400, 4)),
                upsert("e", EntryKind::Dir, file_attrs(0, 1)),
                upsert("f.txt", EntryKind::File, file_attrs(500, 5)),
            ]))
            .expect("apply");
        handle
    }

    /// The property that makes a partial listing safe to render totals from: whatever the
    /// bound keeps, the rows and the remainder still add up to the directory itself.
    ///
    /// Checked at every bound rather than at one, because an off-by-one in the remainder
    /// hides at exactly the width where the page happens to hold everything.
    #[test]
    fn a_page_and_its_remainder_partition_the_directory() {
        let handle = paged_directory();
        let whole = handle.rollup(Path::new("")).expect("read").expect("the root");

        for limit in 0..=7 {
            let page = handle
                .children_page(
                    Path::new(""),
                    &ChildPageRequest { limit: Bound::Limit(limit), ..ChildPageRequest::default() },
                )
                .expect("read")
                .expect("the root is a directory");

            let mut shown = ChildRemainder::default();
            for row in &page.rows {
                shown.rows += 1;
                match row.kind {
                    EntryKind::Dir => {
                        let totals = row.totals.expect("a directory row carries totals");
                        shown.files += totals.files;
                        shown.dirs += totals.dirs + 1;
                        shown.bytes += totals.bytes;
                        shown.allocated += totals.allocated;
                    }
                    EntryKind::File => {
                        shown.files += 1;
                        shown.bytes += row.attrs.size;
                        shown.allocated += row.attrs.allocated;
                    }
                    EntryKind::Symlink | EntryKind::Other => shown.others += 1,
                }
            }
            let rest = page.remainder.unwrap_or_default();
            assert_eq!(shown.rows + rest.rows, 6, "limit {limit}: every child is on one side");
            assert_eq!(shown.files + rest.files, whole.files, "limit {limit}: files");
            assert_eq!(shown.dirs + rest.dirs, whole.dirs, "limit {limit}: dirs");
            assert_eq!(shown.others + rest.others, whole.others, "limit {limit}: others");
            assert_eq!(shown.bytes + rest.bytes, whole.bytes, "limit {limit}: bytes");
            assert_eq!(
                shown.allocated + rest.allocated,
                whole.allocated,
                "limit {limit}: allocated"
            );
        }
    }

    /// Paging the whole directory two rows at a time visits every child exactly once, in
    /// name order, and stops.
    #[test]
    fn the_cursor_walks_the_directory_once_and_ends() {
        let handle = paged_directory();
        let mut seen = Vec::new();
        let mut after = None;
        loop {
            let page = handle
                .children_page(
                    Path::new(""),
                    &ChildPageRequest { after: after.clone(), limit: Bound::Limit(2) },
                )
                .expect("read")
                .expect("the root is a directory");
            seen.extend(page.rows.iter().map(|row| row.name.clone()));
            assert!(page.truncated(), "two rows never cover six children");
            match page.next {
                Some(cursor) => after = Some(cursor),
                None => break,
            }
            assert!(seen.len() <= 6, "the cursor must make progress");
        }
        assert_eq!(
            seen,
            ["a", "b.txt", "c", "d.txt", "e", "f.txt"].map(OsString::from),
            "every child once, in name order"
        );
    }

    /// `next` and `remainder` answer different questions, and the last page is where a
    /// consumer that conflated them would loop forever.
    ///
    /// The remainder is this page's complement in the whole directory, so on a later page
    /// it counts the earlier pages' rows and stays present; `next` is absent. Pinning
    /// this because the obvious implementation derives one from the other.
    #[test]
    fn the_last_page_still_reports_a_remainder_but_no_cursor() {
        let handle = paged_directory();
        let page = handle
            .children_page(
                Path::new(""),
                &ChildPageRequest { after: Some(OsString::from("d.txt")), limit: Bound::Limit(4) },
            )
            .expect("read")
            .expect("the root is a directory");

        assert_eq!(
            page.rows.iter().map(|row| row.name.clone()).collect::<Vec<_>>(),
            ["e", "f.txt"].map(OsString::from),
            "the cursor is exclusive"
        );
        assert!(!page.has_next(), "the directory ended");
        let rest = page.remainder.expect("four earlier children are not on this page");
        assert_eq!(rest.rows, 4, "a, b.txt, c and d.txt");
    }

    /// An unbounded page is the whole directory and says so by omission.
    #[test]
    fn an_unbounded_page_reports_neither_a_remainder_nor_a_cursor() {
        let handle = paged_directory();
        let page = handle
            .children_page(Path::new(""), &ChildPageRequest::default())
            .expect("read")
            .expect("the root is a directory");

        assert_eq!(page.rows.len(), 6);
        assert!(!page.truncated(), "nothing was withheld");
        assert!(!page.has_next(), "nothing follows");
    }

    /// A listing row carries the size column, not the breakdown behind it.
    ///
    /// The regression this exists for is silent: restoring a `RollUp` per row would keep
    /// every test above passing and cost one `BTreeMap` clone per child. The type is the
    /// assertion -- `RollUpScalars` is `Copy`, so a row physically cannot carry a map.
    #[test]
    fn a_child_row_carries_scalars_and_the_breakdown_stays_a_separate_projection() {
        let handle = paged_directory();
        let page = handle
            .children_page(Path::new(""), &ChildPageRequest::default())
            .expect("read")
            .expect("the root is a directory");

        let a = page.rows.iter().find(|row| row.name == "a").expect("a");
        let totals = a.totals.expect("a directory row carries totals");
        assert_eq!((totals.files, totals.dirs, totals.bytes), (1, 1, 100));

        let breakdown = handle.rollup(Path::new("a")).expect("read").expect("a");
        assert_eq!(breakdown.by_ext.len(), 1, "the breakdown is still available, on request");
        assert_eq!(breakdown.files, totals.files, "and it agrees with the row");

        assert!(
            page.rows.iter().find(|row| row.name == "b.txt").expect("b.txt").totals.is_none(),
            "a file has no subtree to total"
        );
    }

    /// Paging is not defined for something that is not a directory, and that is distinct
    /// from a directory with no children.
    #[test]
    fn a_page_distinguishes_absent_from_empty() {
        let handle = paged_directory();
        assert!(
            handle
                .children_page(Path::new("b.txt"), &ChildPageRequest::default())
                .expect("read")
                .is_none(),
            "a file is not a directory"
        );
        assert!(
            handle
                .children_page(Path::new("nope"), &ChildPageRequest::default())
                .expect("read")
                .is_none(),
            "an absent path has no children"
        );
        let empty = handle
            .children_page(Path::new("e"), &ChildPageRequest::default())
            .expect("read")
            .expect("e is a directory");
        assert!(empty.rows.is_empty() && empty.remainder.is_none(), "empty is a page, not None");
    }

    /// A tree wide and deep enough that "proportional to output" and "proportional to
    /// the index" are different numbers by an order of magnitude.
    fn measured_tree() -> IndexHandle {
        let handle = IndexHandle::new(Index::new("/root"));
        let mut ops = vec![upsert("a", EntryKind::Dir, file_attrs(0, 1))];
        ops.push(upsert("a/b", EntryKind::Dir, file_attrs(0, 1)));
        ops.push(upsert("a/b/c", EntryKind::Dir, file_attrs(0, 1)));
        for index_of in 0..200 {
            ops.push(upsert(&format!("a/b/c/f{index_of}.txt"), EntryKind::File, file_attrs(10, 1)));
        }
        handle.apply(&Observation::new(ops)).expect("apply");
        handle
    }

    /// The contract the counter exists to make assertable: a roll-up read is proportional
    /// to its path's depth, not to the subtree it summarises.
    ///
    /// Two hundred files sit under the directory being asked about. A read that visited
    /// them -- an aggregate pass where a maintained value was expected -- reports it here
    /// even though its answer is identical, which is the regression no assertion on the
    /// result itself can catch.
    #[test]
    fn a_rollup_read_visits_its_path_and_not_its_subtree() {
        let handle = measured_tree();
        let bundle = handle
            .read(&ReadRequest {
                rollups: vec![PathBuf::from("a/b/c")],
                total: true,
                ..ReadRequest::default()
            })
            .expect("bundle");

        assert_eq!(bundle.rollups[0].as_ref().expect("a/b/c").files, 200);
        assert_eq!(
            bundle.work.entries_visited, 5,
            "the root, then a, b and c, then the root again for the totals"
        );
        assert!(
            bundle.work.entries_visited < bundle.entries / 10,
            "203 entries, {} visited",
            bundle.work.entries_visited
        );
    }

    /// A listing is proportional to the rows it returns, at every bound.
    #[test]
    fn a_listing_visits_the_rows_it_returns() {
        let handle = measured_tree();
        for limit in [1_usize, 5, 50] {
            let bundle = handle
                .read(&ReadRequest {
                    children_of: Some(PathBuf::from("a/b/c")),
                    children_page: ChildPageRequest {
                        limit: Bound::Limit(limit),
                        ..ChildPageRequest::default()
                    },
                    ..ReadRequest::default()
                })
                .expect("bundle");

            let work = bundle.work;
            assert_eq!(work.rows, limit as u64, "limit {limit}: rows");
            // The root, plus a, b and c on the way down, plus one per row returned.
            assert_eq!(work.entries_visited, 4 + limit as u64, "limit {limit}: entries");
            assert_eq!(work.dirs_visited, 4, "limit {limit}: only the path is directories");
        }
    }

    /// A bound on the extension rows does not bound the tallies a roll-up ranks, and the
    /// counter says so rather than reporting the bound back.
    ///
    /// This is the counter earning its keep on the surface it was added for: the result
    /// looks perfectly bounded, and the work behind it is not.
    #[test]
    fn a_bounded_rollup_still_reports_every_tally_it_ranked() {
        let handle = IndexHandle::new(Index::new("/root"));
        let ops: Vec<Op> = (0..12)
            .map(|index_of| {
                upsert(&format!("f{index_of}.e{index_of}"), EntryKind::File, file_attrs(10, 1))
            })
            .collect();
        handle.apply(&Observation::new(ops)).expect("apply");

        let bundle = handle
            .read(&ReadRequest {
                rollups: vec![PathBuf::new()],
                extensions: Bound::Limit(3),
                ..ReadRequest::default()
            })
            .expect("bundle");

        assert_eq!(bundle.rollups[0].as_ref().expect("the root").by_ext.len(), 3, "rows are bound");
        assert_eq!(bundle.work.tally_rows, 12, "the ranking was not");
    }

    /// Both clocks are stamped, and the guard wait is separated from the whole call.
    ///
    /// Times are not asserted against thresholds -- a shared runner measures the runner --
    /// but a field nobody fills is worse than one that is absent, so presence and their
    /// one structural relation are pinned.
    #[test]
    fn a_bundle_states_what_it_spent_and_what_it_waited_for() {
        let handle = measured_tree();
        let bundle = handle
            .read(&ReadRequest {
                children_of: Some(PathBuf::from("a/b/c")),
                total: true,
                ..ReadRequest::default()
            })
            .expect("bundle");

        assert!(bundle.work.wall_ns > 0, "a read that took no measurable time did not happen");
        assert!(
            bundle.work.lock_wait_ns <= bundle.work.wall_ns,
            "waiting for the guard is part of the call, not beside it"
        );
        assert!(bundle.work.name_bytes >= bundle.work.rows, "each row carries at least a name");
    }

    /// A subtree of symlinks weighs nothing and is not nothing, and the roll-up now says
    /// which.
    ///
    /// Before non-file leaves were counted, `links` below was zero files, zero
    /// directories and zero bytes -- arithmetically identical to `hollow`, which really
    /// is empty. A listing had no way to tell them apart, so it either greyed out a
    /// directory with contents or greyed out nothing.
    #[test]
    fn a_symlink_only_subtree_is_not_an_empty_one() {
        let handle = IndexHandle::new(Index::new("/root"));
        handle
            .apply(&Observation::new(vec![
                upsert("links", EntryKind::Dir, file_attrs(0, 1)),
                upsert("links/to-a", EntryKind::Symlink, file_attrs(0, 1)),
                upsert("links/to-b", EntryKind::Symlink, file_attrs(0, 1)),
                upsert("hollow", EntryKind::Dir, file_attrs(0, 1)),
            ]))
            .expect("apply");

        let links = handle.rollup(Path::new("links")).expect("read").expect("links");
        let hollow = handle.rollup(Path::new("hollow")).expect("read").expect("hollow");
        assert_eq!((links.files, links.dirs, links.bytes), (0, 0, 0));
        assert_eq!((hollow.files, hollow.dirs, hollow.bytes), (0, 0, 0));
        assert_eq!(links.others, 2, "the symlinks are counted even though they weigh nothing");
        assert_eq!(hollow.others, 0);
        assert!(!links.is_empty(), "two entries is not empty");
        assert!(hollow.is_empty());

        // And it reaches a listing row, decided rather than left to the consumer.
        let page = handle
            .children_page(Path::new(""), &ChildPageRequest::default())
            .expect("read")
            .expect("the root");
        let row = |name: &str| {
            page.rows.iter().find(|row| row.name == name).expect("row").is_empty_subtree()
        };
        assert_eq!(row("links"), Some(false));
        assert_eq!(row("hollow"), Some(true));
    }

    /// A partial subtree says why, and the reason survives the read path a consumer uses.
    #[test]
    fn coverage_carries_the_reason_it_was_lost_for() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert("part", EntryKind::Dir, file_attrs(0, 1))]));
        assert_eq!(index.coverage_at(Path::new("")), Status::Complete, "nothing lost yet");

        let started = index.begin_reconcile(Path::new("part"));
        index.finish_reconcile(
            Path::new("part"),
            started,
            Status::Partial(CoverageReason::Inaccessible),
        );

        assert_eq!(
            index.coverage_at(Path::new("part")),
            Status::Partial(CoverageReason::Inaccessible)
        );
        // Coverage propagates the way freshness does: a parent is only as covered as its
        // least-covered descendant.
        assert_eq!(
            index.coverage_at(Path::new("")),
            Status::Partial(CoverageReason::Inaccessible),
            "an uncovered subtree makes its ancestors uncovered"
        );
        assert_eq!(
            IndexHandle::new(index)
                .provenance(Path::new("part"))
                .expect("read")
                .expect("part")
                .status,
            Status::Partial(CoverageReason::Inaccessible),
            "and the reason reaches the provenance a consumer actually reads"
        );
    }

    /// Trust and coverage are different axes, and an invalidated subtree is the case that
    /// proves it: its totals still account for every entry, they may simply be wrong.
    ///
    /// This is why `CoverageReason::WatcherGap` is declared and unreachable. The obvious
    /// implementation reports it here, and would be saying "part of this subtree is
    /// missing" about a subtree that is entirely present.
    #[test]
    fn a_dropped_watch_queue_costs_trust_and_not_coverage() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("watched", EntryKind::Dir, file_attrs(0, 1)),
            upsert("watched/a.txt", EntryKind::File, file_attrs(10, 1)),
        ]));
        index.apply_ok(&Observation::new(vec![Op::InvalidateSubtree {
            path: PathBuf::from("watched"),
            reason: crate::engine_contract::InvalidateReason::WatchOverflow,
        }]));

        assert_eq!(index.freshness_at(Path::new("watched")), Freshness::Stale, "trust is gone");
        assert_eq!(
            index.coverage_at(Path::new("watched")),
            Status::Complete,
            "coverage is not: every entry is still accounted for"
        );
        assert_eq!(index.total().files, 1, "and the totals still include it");
    }

    /// The worst reason wins when contributors disagree, so a consumer sees the one it
    /// most needs to act on rather than whichever subtree happened to be visited last.
    #[test]
    fn combining_coverage_surfaces_the_most_alarming_reason() {
        let mild = Provenance {
            source: Source::Scanned,
            observed_at_ns: 10,
            status: Status::Partial(CoverageReason::Inaccessible),
        };
        let severe = Provenance {
            source: Source::Scanned,
            observed_at_ns: 10,
            status: Status::Partial(CoverageReason::Failed),
        };
        assert_eq!(mild.combine(severe).status, Status::Partial(CoverageReason::Failed));
        assert_eq!(severe.combine(mild).status, Status::Partial(CoverageReason::Failed));

        let whole = Provenance { status: Status::Complete, ..mild };
        assert_eq!(
            whole.combine(mild).status,
            Status::Partial(CoverageReason::Inaccessible),
            "any partial contributor makes the combination partial"
        );
    }

    /// A partial subtree can never claim emptiness, however its counts read.
    ///
    /// Zero entries under a partial roll-up means "nothing found yet", and a listing that
    /// greyed the row out would be greying out a directory it had not finished reading.
    #[test]
    fn a_partial_subtree_declines_to_answer_rather_than_claiming_empty() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![upsert("part", EntryKind::Dir, file_attrs(0, 1))]));
        // A sweep that ended without reading everything -- which is what a reconciliation
        // error looks like from here.
        let started = index.begin_reconcile(Path::new("part"));
        index.finish_reconcile(
            Path::new("part"),
            started,
            Status::Partial(CoverageReason::Inaccessible),
        );
        let handle = IndexHandle::new(index);

        let page = handle
            .children_page(Path::new(""), &ChildPageRequest::default())
            .expect("read")
            .expect("the root");
        let row = page.rows.iter().find(|row| row.name == "part").expect("part");
        assert_ne!(row.provenance.status, Status::Complete, "the fixture must be partial");
        assert_eq!(
            row.is_empty_subtree(),
            None,
            "an unfinished directory reporting zero has not proved anything"
        );
    }

    /// A file row has no subtree, so emptiness is not a question it can answer.
    #[test]
    fn a_file_row_has_no_emptiness_to_report() {
        let handle = paged_directory();
        let page = handle
            .children_page(Path::new(""), &ChildPageRequest::default())
            .expect("read")
            .expect("the root");
        let file = page.rows.iter().find(|row| row.name == "b.txt").expect("b.txt");
        assert_eq!(file.is_empty_subtree(), None);
    }

    /// A bundle carries the identity a consumer's cache key derives from.
    #[test]
    fn a_bundle_reports_the_scope_and_rules_it_was_read_under() {
        let handle = IndexHandle::new(Index::new("/root"));
        let bundle = handle
            .read(&ReadRequest {
                rollups: vec![PathBuf::from("missing")],
                ..ReadRequest::default()
            })
            .expect("bundle");

        assert_eq!(bundle.rollups.len(), 1);
        assert!(bundle.rollups[0].is_none(), "an absent path is None, not an empty roll-up");
        assert!(bundle.total.is_none(), "totals were not requested");
        assert!(bundle.children.is_none(), "no directory was named");
        assert_eq!(
            bundle.scope.type_rules_fingerprint,
            crate::classify::type_rule_fingerprint(),
            "the registry a consumer cache key must key on travels with the read"
        );
        assert_eq!(bundle.entries, 1, "an empty index is its root");
    }

    /// Group tallies are maintained by the reducer, and survive removal.
    ///
    /// The property that makes a groups view a read rather than a walk: every file's
    /// group is folded into every ancestor as it arrives, and unfolded when it leaves.
    /// The removal half is the one that fails silently -- a merge with no matching
    /// unmerge leaves totals that only grow, and every individual number still looks
    /// plausible.
    #[test]
    fn group_tallies_roll_up_and_back_down() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("src", EntryKind::Dir, Attrs::default()),
            upsert("src/main.rs", EntryKind::File, file_attrs(100, 1)),
            upsert("src/lib.rs", EntryKind::File, file_attrs(200, 2)),
            upsert("docs", EntryKind::Dir, Attrs::default()),
            upsert("docs/guide.md", EntryKind::File, file_attrs(300, 3)),
            // An exact-filename rule, which has no extension bucket to derive a group
            // from: this is why the tally is maintained rather than derived from by_ext.
            upsert("Makefile", EntryKind::File, file_attrs(50, 4)),
        ]));

        let total = index.total();
        assert_eq!(total.by_group["code"].files, 3, "two sources and a Makefile");
        assert_eq!(total.by_group["code"].bytes, 350);
        assert_eq!(total.by_group["docs"].files, 1);
        assert_eq!(total.by_group["docs"].bytes, 300);
        assert_eq!(
            total.by_group.values().map(|tally| tally.files).sum::<u64>(),
            total.files,
            "every file lands in exactly one group"
        );

        let src = index.rollup(Path::new("src")).expect("src is a directory");
        assert_eq!(src.by_group["code"].files, 2, "a subtree carries its own tallies");
        assert!(!src.by_group.contains_key("docs"));

        // Removing the markdown file takes its group row with it rather than leaving a
        // zero, so a consumer rendering the map sees the groups that exist.
        index
            .apply_ok(&Observation::new(vec![Op::Remove { path: PathBuf::from("docs/guide.md") }]));
        let total = index.total();
        assert!(!total.by_group.contains_key("docs"), "{:?}", total.by_group);
        assert_eq!(total.by_group["code"].files, 3);
        assert_eq!(total.by_group.values().map(|tally| tally.files).sum::<u64>(), total.files);
    }

    /// Both insert paths tag, and a directory is as taggable as a file.
    ///
    /// The observation path and the snapshot loader are separate bodies of code, and the
    /// loader is deliberately the one that never reconstructs a path -- so a tag that only
    /// the walk applied would survive a scan and vanish on the next warm start, which is
    /// the failure that looks like a cache bug rather than a tagging one.
    #[test]
    fn every_insert_path_tags_an_entry_the_same_way() {
        let rules = std::sync::Arc::new(
            crate::tags::TagRules::from_names(["dotfile"]).expect("dotfile is a real rule"),
        );
        let mut index = Index::new("/root").with_tag_rules(std::sync::Arc::clone(&rules));
        index.apply_ok(&Observation::new(vec![
            upsert(".env", EntryKind::File, file_attrs(10, 1)),
            upsert("README.md", EntryKind::File, file_attrs(20, 2)),
            // Never observed directly: `.git` enters through `ensure_dir_chain` as an
            // ancestor of the file below it, which is the path a placeholder takes.
            upsert(".git/HEAD", EntryKind::File, file_attrs(30, 3)),
        ]));

        assert_eq!(index.tags_of(Path::new(".env")), vec!["dotfile"]);
        assert!(index.tags_of(Path::new("README.md")).is_empty());
        assert_eq!(
            index.tags_of(Path::new(".git")),
            vec!["dotfile"],
            "a directory that entered as a placeholder is tagged, not left at zero"
        );
        assert!(index.tags_of(Path::new(".git/HEAD")).is_empty(), "HEAD is not a dotfile");
        assert!(index.tags_of(Path::new("absent")).is_empty(), "an absent path carries no tags");
    }

    /// Adopting rules re-tags what is already there, which is what a warm start needs.
    ///
    /// A snapshot carries no tag bits -- they are derived, and the loader restores entries
    /// before the caller's rules are known -- so an index that only tagged at insert would
    /// answer "no tags" for every entry after a warm start, while a cold scan of the same
    /// tree answered correctly. Two surfaces, one tree, two answers.
    #[test]
    fn adopting_rules_tags_entries_that_are_already_in_the_index() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert(".env", EntryKind::File, file_attrs(10, 1)),
            upsert("src/main.rs", EntryKind::File, file_attrs(20, 2)),
            upsert("src/.keep", EntryKind::File, file_attrs(0, 3)),
        ]));
        assert!(index.tags_of(Path::new(".env")).is_empty(), "no rules, no tags");

        let index = index.with_tag_rules(std::sync::Arc::new(
            crate::tags::TagRules::from_names(["dotfile"]).expect("enables"),
        ));
        assert_eq!(index.tags_of(Path::new(".env")), vec!["dotfile"]);
        assert_eq!(
            index.tags_of(Path::new("src/.keep")),
            vec!["dotfile"],
            "the walk reaches nested entries, not only the root's children"
        );
        assert!(index.tags_of(Path::new("src/main.rs")).is_empty());
        assert!(index.tags_of(Path::new("src")).is_empty());
    }

    /// Enabling a rule changes the scan scope, and the empty set changes nothing.
    ///
    /// The second half is the load-bearing one: the fingerprint slot this occupies held a
    /// constant zero for an ignore policy nobody implemented, so every snapshot in
    /// existence recorded zero there. A non-zero empty set would discard all of them to
    /// express "still no rules".
    #[test]
    fn enabling_a_tag_rule_invalidates_a_snapshot_and_enabling_none_does_not() {
        let none = crate::scan::ScanConfig::default();
        assert_eq!(none.scope().tag_rules_fingerprint, 0);

        let empty = crate::scan::ScanConfig {
            tags: Some(std::sync::Arc::new(crate::tags::TagRules::none().clone())),
            ..crate::scan::ScanConfig::default()
        };
        assert_eq!(empty.scope(), none.scope(), "asking for no rules is not asking for anything");

        let tagged = crate::scan::ScanConfig {
            tags: Some(std::sync::Arc::new(
                crate::tags::TagRules::from_names(["dotfile"]).expect("enables"),
            )),
            ..crate::scan::ScanConfig::default()
        };
        assert_ne!(tagged.scope(), none.scope(), "an index without the bit cannot answer for it");
    }

    /// A registry with no groups leaves the axis empty rather than inventing one.
    #[test]
    fn a_registry_without_groups_reports_none() {
        let registry = std::sync::Arc::new(
            crate::classify::TypeRegistry::from_manifest(
                "[[kind]]\nid = \"notes\"\nfamily = \"prose\"\nextensions = [\"rs\"]\n",
            )
            .expect("a minimal manifest"),
        );
        let mut index = Index::new("/root").with_types(registry);
        index.apply_ok(&Observation::new(vec![upsert(
            "main.rs",
            EntryKind::File,
            file_attrs(100, 1),
        )]));
        let total = index.total();
        assert_eq!(total.files, 1);
        assert!(total.by_group.is_empty(), "no groups declared, so no group rows");
    }

    /// The kept rows are the largest, and what is dropped is still accounted for.
    ///
    /// Checked against the unbounded roll-up rather than against literals: a bound that
    /// kept the wrong end, or a remainder summed over the wrong slice, agrees with a
    /// hand-written number as readily as the right one.
    #[test]
    fn a_bounded_rollup_keeps_the_largest_rows_and_accounts_for_the_rest() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            // `file_attrs` derives allocated from size as whole 512-byte blocks, so
            // every file here allocates one block whatever its apparent size.
            upsert("big.rs", EntryKind::File, file_attrs(500, 1)),
            upsert("mid.md", EntryKind::File, file_attrs(300, 2)),
            upsert("small.txt", EntryKind::File, file_attrs(100, 3)),
            upsert("tiny.toml", EntryKind::File, file_attrs(10, 4)),
        ]));

        let all = index.total();
        assert_eq!(all.by_ext.len(), 4);
        assert!(all.ext_remainder.is_none(), "an unbounded roll-up withheld nothing");

        let bounded = index.total_bounded(Bound::Limit(2));
        assert_eq!(
            bounded.by_ext.keys().collect::<Vec<_>>(),
            vec![".md", ".rs"],
            "the two largest by bytes, still keyed by name"
        );
        let remainder = bounded.ext_remainder.expect("two of four rows were withheld");
        assert_eq!(remainder.extensions, 2);
        assert_eq!(remainder.files, 2);
        assert_eq!(remainder.bytes, 110);
        assert_eq!(remainder.allocated, 1024, "one 512-byte block each");

        let kept: u64 = bounded.by_ext.values().map(|tally| tally.bytes).sum();
        assert_eq!(kept + remainder.bytes, all.by_ext.values().map(|t| t.bytes).sum::<u64>());
        let kept_files: u64 = bounded.by_ext.values().map(|tally| tally.files).sum();
        assert_eq!(kept_files + remainder.files, all.files);
    }

    /// A limit at or above what is present is not a truncation.
    #[test]
    fn a_bound_wider_than_the_map_withholds_nothing() {
        let index = index_with_sample_tree();
        let all = index.total();
        let bounded = index.total_bounded(Bound::Limit(all.by_ext.len()));
        assert_eq!(bounded.by_ext, all.by_ext);
        assert!(bounded.ext_remainder.is_none());
    }

    /// Ties break by name, so the same subtree always yields the same rows.
    #[test]
    fn equal_sized_extensions_are_ordered_by_name() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("a.zzz", EntryKind::File, file_attrs(100, 1)),
            upsert("b.aaa", EntryKind::File, file_attrs(100, 2)),
        ]));
        let bounded = index.total_bounded(Bound::Limit(1));
        assert_eq!(bounded.by_ext.keys().collect::<Vec<_>>(), vec![".aaa"]);
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
        index.apply_ok(&Observation::new(vec![upsert(
            "src/lib.rs",
            EntryKind::File,
            file_attrs(10, 1),
        )]));
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
        index.begin_reconcile(Path::new(""));
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
        index.finish_reconcile(Path::new(""), 0, Status::Complete);

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
        index.apply_baseline_ok(&Observation::new(vec![Op::Upsert {
            path: PathBuf::from("a/file.txt"),
            kind: EntryKind::File,
            attrs: Attrs { size: 1, ..Attrs::default() },
        }]));
        assert_eq!(
            index.provenance(Path::new("a/file.txt")).expect("present").source,
            Source::Cached,
            "nothing has checked it yet"
        );
        // A completed sweep then covers the whole tree.
        index.finish_reconcile(Path::new(""), 0, Status::Complete);
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
            index.finish_reconcile(&PathBuf::from(format!("dir-{which}")), 0, Status::Complete);
        }
        assert!(
            index.verified.len() <= MAX_VERIFIED_INTERVALS,
            "interval list grew to {}",
            index.verified.len()
        );
    }
}
