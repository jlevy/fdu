//! The observation and commit contract shared by every producer and consumer.
//!
//! The walker, revalidator, and watch layer produce [`Observation`] batches. The index
//! arbitrates their preconditions, removes no-ops, and stamps an [`AppliedDelta`] only
//! after a change has been accepted. The journal and consumer-facing change feed see
//! only those committed deltas. Nothing else mutates the index.
//!
//! Three properties are load-bearing, and the rest of the crate depends on them:
//!
//! - **Observations carry truth, not hints.** A producer stats before it emits. Filesystem
//!   events on most platforms carry no metadata, so a raw event is never a delta.
//! - **Conditional observations cannot overwrite newer state.** Revalidation attaches
//!   state plus generation and revision guards from the start of its check. If another
//!   producer commits a conflicting change first, arbitration rejects the delayed
//!   observation.
//! - **Applied deltas contain changes, not attempts.** No-ops and stale observations do
//!   not advance the public clock or consume journal space.

use std::path::{Path, PathBuf};

/// A monotonic logical clock, in the spirit of Watchman's clockspec but process-local.
///
/// Every [`AppliedDelta`] is stamped, so a consumer can ask "what changed since C?"
/// rather than having to hold a live subscription.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub struct Clock(pub u64);

impl Clock {
    /// The clock before anything has been applied.
    pub const ZERO: Clock = Clock(0);

    /// The next clock value, or `None` when the process-local clock is exhausted.
    #[inline]
    #[must_use]
    pub const fn checked_next(self) -> Option<Clock> {
        match self.0.checked_add(1) {
            Some(value) => Some(Clock(value)),
            None => None,
        }
    }
}

/// What kind of filesystem entry a record describes.
///
/// The numeric values reach the snapshot format, so they are pinned: never renumber a
/// variant, only append.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum EntryKind {
    /// Regular file.
    File = 0,
    /// Directory.
    Dir = 1,
    /// Symbolic link retained without following it.
    Symlink = 2,
    /// Other filesystem object such as a socket or device.
    Other = 3,
}

impl EntryKind {
    /// Recover a kind from its pinned on-disk value.
    pub const fn from_u8(raw: u8) -> Option<Self> {
        match raw {
            0 => Some(Self::File),
            1 => Some(Self::Dir),
            2 => Some(Self::Symlink),
            3 => Some(Self::Other),
            _ => None,
        }
    }

    #[inline]
    /// Whether this kind is a directory.
    pub const fn is_dir(self) -> bool {
        matches!(self, Self::Dir)
    }
}

/// The stat fields an entry contributes to roll-ups, plus the ones that identify it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Attrs {
    /// Apparent size in bytes.
    pub size: u64,
    /// Allocated size in bytes (block count x 512 on Unix). Falls back to `size` on
    /// platforms that do not report block counts.
    pub allocated: u64,
    /// Modification time, nanoseconds since the Unix epoch.
    pub mtime_ns: i64,
    /// Inode change time, nanoseconds since the Unix epoch. Zero where unavailable.
    pub ctime_ns: i64,
    /// Inode number. Zero where unavailable.
    pub inode: u64,
    /// Device number. Zero where unavailable.
    pub dev: u64,
}

impl Attrs {
    /// The change-detection fingerprint for these attributes.
    #[inline]
    pub const fn fingerprint(&self) -> Fingerprint {
        Fingerprint {
            size: self.size,
            mtime_ns: self.mtime_ns,
            ctime_ns: self.ctime_ns,
            inode: self.inode,
        }
    }
}

/// The fingerprint used to decide whether an entry really changed.
///
/// Size and mtime alone are not enough. mtime is user-settable, and some applications
/// roll it back after modifying a file; ctime is kernel-controlled and cannot be set
/// directly. Borg keys on ctime/size/inode and restic requires both mtime and ctime to
/// match, and this engine follows them: an index that keys purely on mtime is trusting a
/// value userspace can forge.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Fingerprint {
    /// Apparent file size.
    pub size: u64,
    /// Modification time in nanoseconds since the Unix epoch.
    pub mtime_ns: i64,
    /// Inode-change time in nanoseconds since the Unix epoch.
    pub ctime_ns: i64,
    /// Platform inode or file identity component.
    pub inode: u64,
}

/// Semantic inputs that decide which entries and derived values belong in an index.
///
/// Operational settings such as producer batch size are intentionally absent. A
/// snapshot may be reused only when this value matches exactly.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ScanScope {
    /// Maximum retained relative depth, or unlimited when absent.
    pub max_depth: Option<usize>,
    /// Whether directory symlinks are followed.
    pub follow_symlinks: bool,
    /// Whether traversal stays on the root filesystem.
    pub one_filesystem: bool,
    /// Identity of the compiled ignore policy.
    pub ignore_rules_fingerprint: u64,
    /// Identity of the compiled type-classification policy.
    pub type_rules_fingerprint: u64,
    /// Identity of the enabled reducer set.
    pub reducers_fingerprint: u64,
}

/// Where a value came from, so a consumer can trade speed for certainty knowingly.
///
/// Ordered weakest-last: comparing two sources yields the one to trust less, which is
/// what a roll-up needs when combining a subtree.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default)]
pub enum Source {
    /// Observed from the filesystem by this process.
    #[default]
    Scanned,
    /// Loaded from a snapshot and re-verified by a fresh stat this session.
    Revalidated,
    /// Loaded from a snapshot; a change journal reported nothing touching this subtree
    /// since the cursor, and nothing has re-checked it.
    ///
    /// Named for what actually happened — a journal *scoped* the work — rather than for
    /// what a reader might wish it meant. Nothing here was confirmed against the
    /// filesystem: a scoped revalidation stats the paths the journal names and does not
    /// stat the rest, so this value rests on the journal having been complete.
    ///
    /// It is deliberately weaker than [`Self::Revalidated`] because that assumption is
    /// known to fail. macOS `FSEvents` will report `HistoryDone` after silently dropping
    /// history, with no degradation flag, which means a journal answer can be wrong
    /// without announcing it. Journal-assisted revalidation therefore bounds exposure
    /// with a maximum age and a periodic full sweep; those are risk controls, not
    /// proofs, and they do not make any individual answer here verified.
    JournalScoped,
    /// Loaded from a snapshot and not re-checked since.
    Cached,
}

/// Whether a value covers everything beneath its path.
///
/// This is the **structural coverage** axis, and only that. How far to *trust* what is
/// covered is [`Source`], and the two are deliberately independent: a cached value
/// covers the whole subtree but may be out of date, while a half-built one covers less
/// than the subtree but every byte in it was just observed.
///
/// An enum rather than a boolean because coverage will gain more ways to be incomplete
/// — truncated by a cap, cancelled, failed — and ordered worst-last so roll-ups combine
/// by taking the maximum. Those variants are not here yet; see the progressive-results
/// plan for the lifecycle they belong to.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default)]
#[non_exhaustive]
pub enum Status {
    /// The value accounts for everything beneath this path that is in scope.
    #[default]
    Complete,
    /// The value does not account for everything beneath this path.
    ///
    /// **Not a promise of monotonicity.** A value being built by an additive walk only
    /// grows, but one left incomplete by reconciliation errors can move either way once
    /// the missing part is read. Monotonicity is a property of the *producer* that is
    /// running, not of this status, and a consumer that needs it must know a walk is in
    /// progress rather than infer it from here.
    Partial,
}

/// Everything a consumer needs to decide how far to trust one value.
///
/// A *view* type, built on demand rather than stored: the index keeps one [`Source`]
/// byte per entry and its observation timestamps once, because on a tree of millions
/// of entries the timestamps are shared by nearly all of them and a per-entry struct
/// would cost more memory than the information is worth.
///
/// The three facts are independent on purpose, because they answer different
/// questions. [`Status`] asks how much of the subtree the number covers;
/// [`Source`] asks how far to trust what it covers; `observed_at_ns` asks when.
/// A [`Status::Complete`] but [`Source::Cached`] value is a point estimate that may
/// move either way and reads as "about 3.2 GB, as of two minutes ago", while a
/// [`Status::Partial`] value is missing part of its subtree and reads as "3.2 GB so
/// far". Collapsing them would make a shrinking number look like a defect.
///
/// Note that "3.2 GB so far" is only a *lower bound that grows* while an additive walk
/// is running. See [`Status::Partial`]: the status records coverage, not direction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Provenance {
    /// Where the value came from.
    pub source: Source,
    /// When the underlying filesystem observation was made, in nanoseconds since the
    /// Unix epoch. For [`Source::Cached`] this is when the snapshot captured it — the
    /// "as of" a consumer displays. Zero when unknown.
    pub observed_at_ns: i64,
    /// How settled the value is.
    pub status: Status,
}

impl Source {
    /// Whether a value from this source was checked against the filesystem during
    /// this session.
    pub const fn is_verified(self) -> bool {
        matches!(self, Self::Scanned | Self::Revalidated)
    }
}

impl Provenance {
    /// Freshly observed by this process, complete.
    pub const fn scanned(observed_at_ns: i64) -> Self {
        Self { source: Source::Scanned, observed_at_ns, status: Status::Complete }
    }

    /// Combine with another value's provenance, taking the less trustworthy of each
    /// fact.
    ///
    /// This is what makes a directory only as trustworthy as its least trustworthy
    /// descendant: the weakest source, the oldest observation, and the worst status.
    ///
    /// Every fact fails closed, including time: an unknown `observed_at_ns` is
    /// absorbing rather than skipped, so a subtree with one contributor of unknown age
    /// reports an unknown age instead of a precise time it cannot prove.
    ///
    /// There is deliberately **no identity element**. Because unknown is absorbing, it
    /// cannot double as the seed of a fold, and a caller aggregating a possibly-empty
    /// set must represent emptiness separately (`Option<Provenance>`) rather than
    /// seeding with a zero timestamp — otherwise every roll-up would come out unknown.
    #[must_use]
    pub fn combine(self, other: Self) -> Self {
        Self {
            source: self.source.max(other.source),
            observed_at_ns: match (self.observed_at_ns, other.observed_at_ns) {
                // Unknown is contagious, not skipped. Zero means "we cannot say when",
                // and the honest combination of a known time with an unknown one is
                // still unknown: a parent that drops the unknown contributor would
                // advertise a precise "as of" it cannot prove for the whole subtree.
                // Unknown is not the identity for "oldest observation" — it is the
                // absorbing element.
                (0, _) | (_, 0) => 0,
                (mine, other) => mine.min(other),
            },
            status: self.status.max(other.status),
        }
    }

    /// Whether this value was checked against the filesystem during this session.
    pub const fn is_verified(self) -> bool {
        self.source.is_verified()
    }
}

/// Trust state for an index or queried subtree.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Freshness {
    /// Every path in scope has been reconciled successfully.
    Fresh,
    /// A reconciliation pass is currently checking this scope.
    Reconciling,
    /// A producer reported lost precision and reconciliation has not completed.
    Stale,
    /// Reconciliation encountered errors, so some state is unknown.
    Partial,
}

impl Freshness {
    pub(crate) const fn rank(self) -> u8 {
        match self {
            Self::Fresh => 0,
            Self::Reconciling => 1,
            Self::Stale => 2,
            Self::Partial => 3,
        }
    }
}

/// Why a producer had to escalate to [`Op::InvalidateSubtree`] instead of describing a
/// change precisely.
///
/// Each variant corresponds to a real, documented platform limitation rather than a
/// defensive catch-all, and the scan layer resolves every one of them the same way.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum InvalidateReason {
    /// The kernel dropped events: inotify `Q_OVERFLOW`, `FSEvents` `MustScanSubDirs`, or a
    /// Windows `ReadDirectoryChangesW` buffer overrun. All three surface through notify
    /// as `Flag::Rescan`, and swallowing that flag silently corrupts any index built on
    /// events.
    WatchOverflow,
    /// A rename whose two sides could not be paired: `FSEvents` reports one path with no
    /// mechanism to associate old and new, and file-id stitching did not resolve it.
    UnpairedRename,
    /// A directory was created and its watch registered a moment later; anything created
    /// inside that window produced no event at all.
    WatchSetupRace,
    /// A periodic reconciliation sweep, for backends that cannot signal drops at all
    /// (kqueue).
    PeriodicSweep,
    /// Stat verification failed without proving the path is gone. The known entry must
    /// remain until reconciliation can retry and report the underlying I/O error.
    VerificationFailed,
    /// Repeated concurrent commits prevented a watch sample from reaching a stable
    /// arbitration boundary. The root is reconciled instead of doing filesystem I/O
    /// under the index lock or allowing an old sample to win.
    WatchContention,
    /// Requested by the caller.
    Requested,
}

/// A single change to one path.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Op {
    /// The entry appeared or changed. Always carries a fresh stat, never a bare event.
    Upsert {
        /// Path relative to the index root.
        path: PathBuf,
        /// Newly observed entry kind.
        kind: EntryKind,
        /// Newly observed metadata.
        attrs: Attrs,
    },
    /// The entry is gone. Implies removal of every descendant.
    Remove {
        /// Path relative to the index root.
        path: PathBuf,
    },
    /// The producer could not describe the change precisely; the consumer must re-scan
    /// this subtree. The scan layer turns this back into precise ops, so escalation is
    /// closed-loop rather than a dead end.
    InvalidateSubtree {
        /// Relative root of the scope that must be reconciled.
        path: PathBuf,
        /// Why the producer could not report precise changes.
        reason: InvalidateReason,
    },
}

impl Op {
    /// The path this op applies to.
    pub fn path(&self) -> &Path {
        match self {
            Self::Upsert { path, .. }
            | Self::Remove { path }
            | Self::InvalidateSubtree { path, .. } => path,
        }
    }
}

/// The complete indexed state of one path at an observation boundary.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum PathState {
    /// The path was not indexed.
    Absent,
    /// The path was indexed with these observed fields.
    Present {
        /// Indexed entry kind.
        kind: EntryKind,
        /// Indexed metadata.
        attrs: Attrs,
    },
}

/// Generation- and revision-safe identity for one indexed entry.
///
/// The fields are intentionally opaque. Producers obtain identities through
/// [`crate::Index::expectation`] rather than manufacturing handles that could
/// accidentally alias a recycled arena slot or bypass ABA detection.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub(crate) struct EntryIdentity {
    slot: u32,
    generation: u64,
    revision: u64,
    children_revision: u64,
    directory: bool,
}

impl EntryIdentity {
    pub(crate) const fn new(
        slot: u32,
        generation: u64,
        revision: u64,
        children_revision: u64,
        directory: bool,
    ) -> Self {
        Self { slot, generation, revision, children_revision, directory }
    }

    pub(crate) const fn same_target(self, other: Self, require_structure: bool) -> bool {
        self.slot == other.slot
            && self.generation == other.generation
            && self.revision == other.revision
            && (!require_structure || self.children_revision == other.children_revision)
    }

    pub(crate) const fn same_absence_guard(self, other: Self) -> bool {
        self.slot == other.slot
            && self.generation == other.generation
            && self.children_revision == other.children_revision
            && (other.directory || self.revision == other.revision)
    }
}

/// State and entry revisions captured at one observation boundary.
///
/// Present paths carry a generation-safe target identity and direct revision so a
/// change-away-and-back cannot masquerade as the original state. Absent paths carry the
/// nearest existing ancestor's structural revision, closing create/remove and parent
/// replacement races without making unrelated subtrees conflict.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct PathExpectation {
    /// Visible state captured at the observation boundary.
    pub state: PathState,
    entry: Option<EntryIdentity>,
    absence_guard: Option<EntryIdentity>,
}

impl PathExpectation {
    pub(crate) const fn new(
        state: PathState,
        entry: Option<EntryIdentity>,
        absence_guard: Option<EntryIdentity>,
    ) -> Self {
        Self { state, entry, absence_guard }
    }

    pub(crate) const fn entry(self) -> Option<EntryIdentity> {
        self.entry
    }

    pub(crate) const fn absence_guard(self) -> Option<EntryIdentity> {
        self.absence_guard
    }
}

/// The condition under which an observation may be committed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Expectation {
    /// Commit according to arrival order. Used by freshly verified watch observations
    /// and cold-scan bootstrap data.
    Any,
    /// Commit only if the target state and relevant structural revisions still match the
    /// producer baseline.
    State(PathExpectation),
}

/// One observed operation together with its arbitration precondition.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ObservationOp {
    /// Proposed path mutation.
    pub op: Op,
    /// Arbitration precondition for that mutation.
    pub expectation: Expectation,
}

impl ObservationOp {
    /// An operation whose fresh verification makes arrival order authoritative.
    pub const fn unconditional(op: Op) -> Self {
        Self { op, expectation: Expectation::Any }
    }

    /// An operation valid only while `expected` still matches the index.
    pub const fn if_state(op: Op, expected: PathExpectation) -> Self {
        Self { op, expectation: Expectation::State(expected) }
    }
}

/// A producer batch awaiting arbitration by the index.
///
/// Batching is not just an efficiency detail: producers coalesce per path within a batch
/// and stat once per batch rather than once per event.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Observation {
    /// Ordered operations in this producer batch.
    pub ops: Vec<ObservationOp>,
}

impl Observation {
    /// Build an unconditional batch from freshly verified operations.
    pub fn new(ops: Vec<Op>) -> Self {
        Self { ops: ops.into_iter().map(ObservationOp::unconditional).collect() }
    }

    /// Build a batch whose operations already carry explicit expectations.
    pub const fn from_ops(ops: Vec<ObservationOp>) -> Self {
        Self { ops }
    }

    #[inline]
    /// Whether the batch contains no operations.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    #[inline]
    /// Number of operations in the batch.
    pub fn len(&self) -> usize {
        self.ops.len()
    }
}

/// A committed batch containing only effective mutations.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AppliedDelta {
    /// Logical commit clock minted for the whole batch.
    pub clock: Clock,
    /// Effective mutations committed at that clock.
    pub ops: Vec<Op>,
}

impl AppliedDelta {
    #[inline]
    /// Whether the committed batch contains no operations.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    #[inline]
    /// Number of effective operations in the committed batch.
    pub fn len(&self) -> usize {
        self.ops.len()
    }
}

/// Errors the engine can report.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A filesystem operation failed at a specific path.
    #[error("I/O error at {path}: {source}")]
    Io {
        /// Path whose operation failed.
        path: PathBuf,
        #[source]
        /// Underlying operating-system error.
        source: std::io::Error,
    },

    /// An observation or subtree path was not relative to the index root.
    #[error("path escapes the index root: {0}")]
    PathEscapesRoot(PathBuf),

    /// Snapshot persistence failed after a usable snapshot had been selected.
    #[error("snapshot is not usable: {0}")]
    Snapshot(String),

    /// A scan or watch setting has no supported safe semantics.
    #[error("unsupported scan configuration: {0}")]
    UnsupportedScanConfig(&'static str),

    /// Requested scan semantics differ from the index's immutable scope.
    #[error("scan scope mismatch: index has {indexed:?}, requested {requested:?}")]
    ScanScopeMismatch {
        /// Scope represented by the index.
        indexed: ScanScope,
        /// Scope requested by the operation.
        requested: ScanScope,
    },

    /// A requested relative subtree lies beyond the configured scan boundary.
    #[error("subtree {path:?} lies outside scan scope {scope:?}")]
    SubtreeOutsideScanScope {
        /// Rejected relative path.
        path: PathBuf,
        /// Scope that excludes the path.
        scope: ScanScope,
    },

    /// A writer panicked while owning the shared index lock.
    #[error("index lock was poisoned by a panicking writer")]
    IndexLockPoisoned,

    /// No further logical commit clock can be represented.
    #[error("the process-local index clock is exhausted")]
    ClockExhausted,

    /// The watch worker ended permanently without panicking.
    #[error("watch worker stopped before another observation was available")]
    WatchStopped,

    /// The watch worker panicked; its bounded channel is no longer live.
    #[error("watch worker panicked and stopped")]
    WatchWorkerPanicked,

    /// A watcher was paired with an index rooted at a different directory.
    #[error("watch root {watched:?} does not match index root {indexed:?}")]
    WatchRootMismatch {
        /// Canonical root owned by the watcher.
        watched: PathBuf,
        /// Canonical root owned by the index.
        indexed: PathBuf,
    },
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io { path: path.into(), source }
    }
}

/// Result alias for engine operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_advances_monotonically() {
        let c = Clock::ZERO;
        assert_eq!(c.checked_next(), Some(Clock(1)));
        assert!(c.checked_next().is_some_and(|next| next > c));
        assert_eq!(Clock(u64::MAX).checked_next(), None);
    }

    #[test]
    fn entry_kind_roundtrips_through_its_pinned_value() {
        for kind in [EntryKind::File, EntryKind::Dir, EntryKind::Symlink, EntryKind::Other] {
            assert_eq!(EntryKind::from_u8(kind as u8), Some(kind));
        }
        assert_eq!(EntryKind::from_u8(99), None);
    }

    #[test]
    fn fingerprint_ignores_allocated_size_but_tracks_ctime() {
        let base = Attrs { size: 10, allocated: 4096, mtime_ns: 5, ctime_ns: 7, inode: 42, dev: 1 };
        // Allocation changes alone are not a content change.
        let repacked = Attrs { allocated: 8192, ..base };
        assert_eq!(base.fingerprint(), repacked.fingerprint());

        // A ctime bump is, even when mtime was rolled back to look unchanged.
        let touched = Attrs { ctime_ns: 9, ..base };
        assert_ne!(base.fingerprint(), touched.fingerprint());
    }
}

#[cfg(test)]
mod provenance_tests {
    use super::*;

    #[test]
    fn sources_order_from_most_to_least_trustworthy() {
        assert!(Source::Scanned < Source::Revalidated);
        assert!(Source::Revalidated < Source::JournalScoped);
        assert!(Source::JournalScoped < Source::Cached);
    }

    #[test]
    fn combining_takes_the_least_trustworthy_of_each_fact() {
        let verified =
            Provenance { source: Source::Scanned, observed_at_ns: 900, status: Status::Complete };
        let stale =
            Provenance { source: Source::Cached, observed_at_ns: 100, status: Status::Partial };
        let combined = verified.combine(stale);
        assert_eq!(combined.source, Source::Cached, "weakest source wins");
        assert_eq!(combined.observed_at_ns, 100, "oldest observation wins");
        assert_eq!(combined.status, Status::Partial, "worst status wins");
        assert_eq!(combined, stale.combine(verified), "combination is commutative");
    }

    #[test]
    fn an_unknown_timestamp_makes_the_combination_unknown() {
        // Fail closed. Zero means "we cannot say when", so a subtree containing one
        // contributor of unknown age has an unknown age too. Returning the known
        // timestamp instead would let a directory advertise a precise "as of" that is
        // wrong for part of what it summarises — the silent lie the provenance model
        // exists to prevent.
        let known = Provenance::scanned(500);
        let unknown = Provenance { observed_at_ns: 0, ..Provenance::scanned(0) };
        assert_eq!(known.combine(unknown).observed_at_ns, 0);
        assert_eq!(unknown.combine(known).observed_at_ns, 0);
        assert_eq!(known.combine(unknown), unknown.combine(known), "combination stays commutative");
    }

    #[test]
    fn only_this_session_counts_as_verified() {
        assert!(Provenance::scanned(1).is_verified());
        assert!(Provenance { source: Source::Revalidated, ..Provenance::scanned(1) }.is_verified());
        // The journal can omit history without saying so, so its word is not a check.
        assert!(
            !Provenance { source: Source::JournalScoped, ..Provenance::scanned(1) }.is_verified()
        );
        assert!(!Provenance { source: Source::Cached, ..Provenance::scanned(1) }.is_verified());
    }
}
