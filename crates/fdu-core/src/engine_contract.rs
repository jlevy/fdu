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

/// Identity of one opened index, minted when it is constructed.
///
/// A [`Clock`] alone cannot say *whose* clock it is. Two processes watching the same tree
/// both count from zero, and so does the same process after reopening -- so a resume token
/// from a prior run compares numerically against an unrelated sequence and looks perfectly
/// valid. `Index::since` would answer with an empty, untruncated set: "nothing changed",
/// about a position that never existed here. That is the failure a session identity
/// exists to make impossible, and it is the same shape `MetaBrowser`'s provider contract
/// already specifies for its own cursors.
///
/// Process-local and not persisted. A snapshot reload is a new session by definition:
/// the journal it would resume against was never written to disk.
/// `Default` is the zero identity, which no minted session ever takes. That is what
/// makes a default-constructed token safe: it matches nothing, so it is refused rather
/// than mistaken for a position in whatever index it reaches.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct SessionId(pub u64);

impl SessionId {
    /// Mint an identity for a newly constructed index.
    ///
    /// Wall-clock nanoseconds mixed with a process-global counter, folded through the same
    /// FNV-1a this crate uses elsewhere. The counter alone would collide across processes
    /// and the clock alone would collide within one -- two indexes opened in the same
    /// nanosecond are ordinary in a test suite. Neither is a cryptographic claim: this
    /// distinguishes sessions, it does not authenticate them.
    #[must_use]
    pub fn mint() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            // Truncating to the low 64 bits is exactly what is wanted: this feeds a hash,
            // so the high bits of a nanosecond count carry no information the mix needs.
            .map_or(0, |since| u64::try_from(since.as_nanos() & u128::from(u64::MAX)).unwrap_or(0));
        let ordinal = NEXT.fetch_add(1, Ordering::Relaxed);

        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in nanos.to_le_bytes().iter().chain(ordinal.to_le_bytes().iter()) {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
        // Zero is reserved so a default-constructed token cannot impersonate a session.
        Self(if hash == 0 { 1 } else { hash })
    }
}

/// A resume position: which session, and how far into it.
///
/// Both halves are load-bearing. The clock says where to resume; the session says whether
/// resuming means anything at all. A caller storing this and replaying it after a restart
/// gets a refusal it can act on rather than an empty answer it will believe.
///
/// Captured with the data it describes, never sampled afterwards. Reading a journal slice
/// and then asking for the clock is two operations, and a commit landing between them
/// yields a cursor one ahead of the deltas returned -- so resuming from it skips that
/// commit permanently, and nothing reports the loss.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Cursor {
    /// Which opened index this position belongs to.
    pub session: SessionId,
    /// How far into that session's commits it points.
    pub clock: Clock,
}

impl Cursor {
    /// The position at the start of a session, before anything is applied.
    #[must_use]
    pub const fn start(session: SessionId) -> Self {
        Self { session, clock: Clock::ZERO }
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
            dev: self.dev,
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
    /// Platform device or volume identity component.
    pub dev: u64,
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
    /// Identity of the enabled tag-rule set.
    ///
    /// Occupies the wire position that named a compiled ignore policy while nothing
    /// implemented one, and holds the same value — the empty set fingerprints to zero — so
    /// the rename costs no snapshot. Tags decide which entries carry which named facts, so
    /// an index built under one set cannot answer a question posed under another.
    pub tag_rules_fingerprint: u64,
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
    /// The value does not account for everything beneath this path, and why.
    ///
    /// The reason rides inside the variant rather than beside it so the two cannot
    /// disagree: there is no way to spell a complete value carrying a reason, or a
    /// partial one carrying none. It also keeps the derived `Ord` correct for
    /// [`Provenance::combine`] -- `Complete` sorts below every `Partial`, and two
    /// partials sort by [`CoverageReason`], so taking the maximum still yields the
    /// least trustworthy contributor.
    ///
    /// **Not a promise of monotonicity.** A value being built by an additive walk only
    /// grows, but one left incomplete by reconciliation errors can move either way once
    /// the missing part is read. Monotonicity is a property of the *producer* that is
    /// running, not of this status, and a consumer that needs it must know a walk is in
    /// progress rather than infer it from here.
    Partial(CoverageReason),
}

/// Why a [`Status::Partial`] value does not cover its whole subtree.
///
/// The vocabulary is the interactive-client contract's, declared whole so a consumer can
/// match exhaustively today and not have its match break when the engine learns to
/// produce a variant it currently cannot. Two of the six are reachable now; the other
/// four are declared and unreachable, each noted with what would make it real. That is
/// deliberate: a vocabulary that matches the contract with stated gaps is honest, where
/// one that quietly omits four names invites a consumer to assume the engine can never
/// mean them.
///
/// Ordered least to most alarming, so [`Provenance::combine`] surfaces the reason a
/// consumer most needs to act on when a subtree's contributors disagree. A walk still
/// running is the mildest -- it resolves itself -- and an outright failure is the worst.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[non_exhaustive]
pub enum CoverageReason {
    /// A walk is still adding to this subtree.
    ///
    /// **Not reachable yet.** An in-progress reconciliation marks [`Freshness::Reconciling`],
    /// which is deliberately coverage-*complete*: a cached subtree still accounts for every
    /// entry it knows about. Producing this needs a session that can publish totals
    /// mid-walk, which is the progressive-results work (`fdu-4o0m`).
    Building,
    /// A cap stopped the walk before it finished.
    ///
    /// **Not reachable yet**: fdu has no walk budget. Declared because the contract has
    /// it and a bounded walk is a plausible future scope knob.
    Budget,
    /// A caller stopped the walk.
    ///
    /// **Not reachable yet**: cancellation belongs to the session (`fdu-4o0m`).
    ///
    /// Observation loss is deliberately *not* a reason in this enum, and the omission is
    /// the point rather than an oversight. An [`Op::InvalidateSubtree`] marks
    /// [`Freshness::Stale`], which is a statement about *trust*: the totals still account
    /// for every entry, they may simply be wrong. Coverage becomes partial only when the
    /// answer actually omits scope -- if the re-read that follows cannot complete, and
    /// then the reason is `Inaccessible`. A `WatcherGap` variant was declared here and
    /// never produced; exporting a reason nothing can return invites a consumer to branch
    /// on it forever, so it is gone.
    Cancelled,
    /// Some of the subtree could not be read -- a permission error, a vanished directory,
    /// an I/O failure during enumeration.
    ///
    /// Reachable: this is what a scan or reconciliation with a non-empty error list means.
    Inaccessible,
    /// The operation building this value returned an error.
    ///
    /// Reachable: a reconciliation that failed outright, as distinct from one that
    /// completed while skipping unreadable parts.
    Failed,
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
    /// How much of the subtree the value covers, and why it does not cover all of it.
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

/// One answer-affecting change that is not a change to any path's values.
///
/// Coverage, trust, provenance, and the tag rules all decide what a projection *means*,
/// so a consumer caching an answer has to be told when they move for the same reason it
/// has to be told a file changed size. They ride in the committed delta beside the ops,
/// under the same clock, because a transition delivered through some other channel is a
/// transition a cursor cannot name: one position would identify two different answers,
/// and nothing in either would report which.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum StateChange {
    /// A reconciliation sweep completed over this subtree.
    ///
    /// Provenance beneath it moved from cached to revalidated. The sweep is the unit,
    /// not the entry: an unchanged upsert promotes one entry's source without changing
    /// any value it reports, and a sweep performs millions of those. The index stores
    /// the interval for exactly that reason, and the interval is what commits.
    Verified {
        /// Relative root of the subtree the sweep covered.
        path: PathBuf,
    },
    /// How far a subtree may be trusted changed.
    Freshness {
        /// Relative root of the subtree whose trust state moved.
        path: PathBuf,
        /// The state it moved to.
        freshness: Freshness,
        /// Why coverage is partial, when it is.
        reason: Option<CoverageReason>,
    },
    /// The facts describing the operation behind the current state were replaced.
    ///
    /// Carries no payload: the envelope is read from the index under the same guard as
    /// the rows, so a consumer learns *that* it moved here and reads *what it moved to*
    /// from the read it was going to make anyway. Copying it into the journal would
    /// retain a second copy that can only ever agree or be wrong.
    RunFacts,
    /// The provider's lifecycle phase moved.
    ///
    /// Only where the phase is a fact of its own. A sweep starting or ending already
    /// commits a [`StateChange::Freshness`], and the phase is derived from it -- emitting
    /// both would report one fact twice and let a consumer see them disagree.
    Phase {
        /// The phase it moved to.
        phase: Phase,
    },
    /// The tag rules were bound against their control files again.
    ///
    /// Entries beneath these directories may carry different tags without any of them
    /// having been touched -- the one way a cached row goes wrong with no path event
    /// ever naming it.
    Retagged {
        /// Directories the rebuilt rules govern, or empty when `all` is set.
        directories: Vec<PathBuf>,
        /// The governed set exceeded its bound; treat every cached row as re-tagged.
        ///
        /// The list is dropped rather than truncated, for the same reason a bounded dirty
        /// set is: a truncated list is indistinguishable from a complete one at the
        /// consumer, so a row it no longer names survives an invalidation that should have
        /// reached it. The bound also keeps this variant from being an unbounded payload
        /// inside a bounded journal.
        all: bool,
    },
}

/// Governed directories a single re-tag enumerates before it says "all of them".
///
/// A rule set governing more directories than this is one whose scope is better described
/// by a bit than by a list: a `PathBuf` each, retained in the journal for as long as the
/// commit is resumable, to tell a consumer something one flag says as well.
pub const MAX_RETAGGED_DIRECTORIES: usize = 1024;

impl StateChange {
    /// What this transition costs against the journal's retention budget.
    ///
    /// One for the transition, plus one for each path it embeds. A variant carrying a
    /// vector charged as a single item is a hole in a bound that exists to cap memory: the
    /// budget would count one where the journal retained a thousand.
    #[must_use]
    pub fn cost(&self) -> usize {
        1 + self.paths().len()
    }

    /// The subtree roots this transition applies to.
    ///
    /// Empty for a transition about the index as a whole rather than about any part of
    /// it, which is what makes `RunFacts` different in kind from the other three.
    pub fn paths(&self) -> &[PathBuf] {
        match self {
            Self::Verified { path } | Self::Freshness { path, .. } => std::slice::from_ref(path),
            Self::RunFacts | Self::Phase { .. } => &[],
            Self::Retagged { directories, .. } => directories,
        }
    }
}

/// One state transition together with the commit it landed at.
///
/// The pair, because a transition without its clock is not placeable: a consumer ordering
/// it against the rows, or resuming from a position just before or after it, needs to know
/// which commit it belongs to. Flattening a batch's transitions into one list and stamping
/// them all with the batch's terminal position loses exactly that -- every transition then
/// claims to have happened at the end, and the interleaving with operations is gone.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CommittedState {
    /// The commit this transition landed at.
    pub clock: Clock,
    /// What moved.
    pub change: StateChange,
}

/// A committed batch containing only effective changes.
///
/// Rows and state travel together. A delta carrying only [`StateChange`]s is ordinary and
/// is the whole point: coverage moving with nothing else moving is still an answer
/// changing, and a clock that did not advance for it would let one cursor name the answer
/// before and the answer after.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AppliedDelta {
    /// Logical commit clock minted for the whole batch.
    pub clock: Clock,
    /// Effective mutations committed at that clock.
    pub ops: Vec<Op>,
    /// Answer-affecting state transitions committed at that clock.
    pub state: Vec<StateChange>,
}

impl AppliedDelta {
    /// A delta carrying only path mutations.
    #[must_use]
    pub const fn of_ops(clock: Clock, ops: Vec<Op>) -> Self {
        Self { clock, ops, state: Vec::new() }
    }

    /// A delta carrying only state transitions.
    #[must_use]
    pub const fn of_state(clock: Clock, state: Vec<StateChange>) -> Self {
        Self { clock, ops: Vec::new(), state }
    }

    #[inline]
    /// Whether the committed batch carries neither a mutation nor a transition.
    ///
    /// Never true of a delta the index minted: a batch with nothing effective in it does
    /// not advance the clock and never becomes a delta at all.
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty() && self.state.is_empty()
    }

    #[inline]
    /// What this delta costs against the journal's retention budget.
    ///
    /// State transitions are charged like ops rather than being free. A free transition
    /// would let a pathological producer -- a reconciliation loop over sibling subtrees,
    /// say -- fill the journal with entries the budget never sees, and the bound exists
    /// precisely so a long-lived server's change history cannot grow without limit.
    pub fn len(&self) -> usize {
        self.ops.len() + self.state.iter().map(StateChange::cost).sum::<usize>()
    }
}

/// Where a provider stands in its own lifecycle.
///
/// Independent of coverage and freshness, which say how much of the tree an answer accounts
/// for and how far it may be trusted. This says what the provider is *doing*, which is a
/// different question: an index can be complete and fresh while a sweep runs, and partial
/// and stale while nothing does.
///
/// Declared whole, following [`CoverageReason`], so an adapter maps it once. Three members
/// are reachable from today's engine; the rest need the session that publishes mid-walk
/// results and owns a run's start and end, and each says so.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
#[non_exhaustive]
pub enum Phase {
    /// Open, answering, with nothing running against it.
    #[default]
    Ready,
    /// A sweep is checking known entries against the filesystem.
    Reconciling,
    /// A watch is attached, so changes arrive as events rather than by asking.
    Watching,
    /// A snapshot is being opened and nothing has been served yet.
    ///
    /// **Not reachable yet.** Opening is not observable: the index does not exist until it
    /// finishes, so there is nothing to ask.
    OpeningCache,
    /// A cold walk is still adding entries.
    ///
    /// **Not reachable yet.** Same reason, and the fix is the same: a session that
    /// publishes a partial index while the walk continues.
    Discovering,
    /// The provider will serve no further operations.
    ///
    /// **Not reachable yet.** An `Index` is dropped rather than stopped; nothing outlives
    /// it to report that it has.
    Stopped,
    /// The last operation failed and the provider cannot continue.
    ///
    /// **Not reachable yet.** A failed operation today leaves an index that still answers
    /// from what it has, which is a partial coverage rather than a lifecycle state.
    Failed,
}

impl Phase {
    /// The stable wire label, shared by every surface.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Reconciling => "reconciling",
            Self::Watching => "watching",
            Self::OpeningCache => "opening_cache",
            Self::Discovering => "discovering",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        }
    }
}

/// What kind of thing went wrong, from a vocabulary a consumer can branch on.
///
/// Rendered messages are for people. A consumer deciding whether to retry, to prompt for
/// access, or to drop a subtree from its view needs to make that decision from a value,
/// and matching on prose is how it ends up depending on the wording of an error.
///
/// Declared whole, including the members today's engine cannot produce, so an adapter maps
/// this once rather than growing a branch each time one becomes reachable. Each says so.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[non_exhaustive]
pub enum IssueKind {
    /// The operating system refused access to a path.
    Permission,
    /// The path was gone by the time it was read.
    ///
    /// Ordinary during a walk of a live tree, and not a failure of the walk: it is
    /// reported so a consumer can tell "this is not here" from "I could not look".
    Disappeared,
    /// Metadata could not be interpreted.
    InvalidMetadata,
    /// A filesystem boundary was not crossed, by request.
    ///
    /// **Not reachable yet.** `one_filesystem` prunes at the boundary without recording
    /// where, so nothing today can name the paths it declined to enter.
    FilesystemBoundary,
    /// A limit stopped the operation before it finished.
    ///
    /// **Not reachable yet.** fdu has no walk budget; the entry exists so the vocabulary
    /// is the consumer contract's rather than a subset of it.
    ResourceStop,
    /// The provider's own observation of the tree had a hole, which it then covered.
    ///
    /// The kernel dropped events, a rename could not be paired, or a directory's watch was
    /// registered after something had already been created inside it. The engine re-scans,
    /// so the index is right -- but the *stream* had a gap, and a consumer that was told
    /// only "here are some changes" would not know its own view had been incomplete in the
    /// meantime.
    ///
    /// Deliberately an issue rather than a coverage reason: coverage is how much of the
    /// tree an answer accounts for, and after the re-scan the answer accounts for all of
    /// it. What moved is how far the stream between then and now can be trusted.
    ObservationGap,
    /// The engine itself failed.
    ProviderFailure,
}

impl IssueKind {
    /// The stable wire label, shared by every surface.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Permission => "permission",
            Self::Disappeared => "disappeared",
            Self::InvalidMetadata => "invalid_metadata",
            Self::FilesystemBoundary => "filesystem_boundary",
            Self::ResourceStop => "resource_stop",
            Self::ObservationGap => "observation_gap",
            Self::ProviderFailure => "provider_failure",
        }
    }
}

/// One non-fatal condition that made a result partial.
///
/// Carries the classification, the path it happened at when it had one, and the rendered
/// message. All three: the kind is what a consumer branches on, the path is what it acts
/// on, and the message is what a person reads when neither is enough.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Issue {
    /// What kind of condition this is.
    pub kind: IssueKind,
    /// Where it happened, when it happened somewhere.
    pub path: Option<PathBuf>,
    /// The rendered message, for a person.
    pub message: String,
    /// The operating system's own error number, when there was one.
    pub os_error: Option<i32>,
}

impl Issue {
    /// Classify one engine error.
    ///
    /// I/O failures are classified by the operating system's own error kind, because that
    /// is the only place the distinction exists: a permission failure and a vanished path
    /// arrive through the same variant and read the same way to anything matching on the
    /// enum alone.
    #[must_use]
    pub fn from_error(error: &Error) -> Self {
        match error {
            Error::Io { path, source } => Self {
                kind: match source.kind() {
                    std::io::ErrorKind::PermissionDenied => IssueKind::Permission,
                    std::io::ErrorKind::NotFound => IssueKind::Disappeared,
                    std::io::ErrorKind::InvalidData | std::io::ErrorKind::InvalidInput => {
                        IssueKind::InvalidMetadata
                    }
                    _ => IssueKind::ProviderFailure,
                },
                path: Some(path.clone()),
                message: error.to_string(),
                os_error: source.raw_os_error(),
            },
            other => Self {
                kind: IssueKind::ProviderFailure,
                path: None,
                message: other.to_string(),
                os_error: None,
            },
        }
    }

    /// An issue the engine did not raise, reported by a layer above it.
    #[must_use]
    pub fn reported(kind: IssueKind, message: String) -> Self {
        Self { kind, path: None, message, os_error: None }
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

    /// A pinned read asked for a version this index is no longer at.
    ///
    /// Retaining only the current image is the policy, so this is the expected answer to a
    /// pin that has aged out rather than a failure of one. A consumer assembling a
    /// complete result from bounded pages restarts on it; the alternative -- silently
    /// continuing on a newer version -- produces one answer stitched from two trees, with
    /// nothing in it saying so.
    #[error("version {requested:?} is no longer available; this index is at {current:?}")]
    VersionUnavailable {
        /// The version the caller pinned to.
        requested: Cursor,
        /// The version it is at now.
        current: Cursor,
    },

    /// A resume token belongs to a different opened index, or to a position that has not
    /// happened yet.
    ///
    /// Refused rather than answered. Both shapes would otherwise return an empty,
    /// untruncated result -- indistinguishable from "you are up to date" -- so a consumer
    /// resuming from a stale or foreign token would silently believe it had missed
    /// nothing. Saying so is what lets it reset and reread instead.
    #[error(
        "cursor {requested:?} does not belong to this index (session {current:?}, clock {clock:?})"
    )]
    CursorNotOfThisSession {
        /// The token the caller presented.
        requested: Cursor,
        /// The session actually serving.
        current: SessionId,
        /// How far that session has advanced.
        clock: Clock,
    },

    /// The watch worker ended permanently without panicking.
    #[error("watch worker stopped before another observation was available")]
    WatchStopped,

    /// The watch worker panicked; its bounded channel is no longer live.
    #[error("watch worker panicked and stopped")]
    WatchWorkerPanicked,

    /// An argument value did not match its documented grammar.
    ///
    /// Carries a suggestion rather than only a rejection, because these values are typed
    /// by humans and agents at a prompt: the whole point of a closed grammar is that a
    /// near miss can say what the near-hit spelling would be.
    #[error("invalid {kind} {value:?}: {hint}")]
    InvalidValue {
        /// Which grammar was expected, for the message: `time` or `size`.
        kind: &'static str,
        /// The rejected input, as written.
        value: String,
        /// What to write instead.
        hint: String,
    },

    /// A scripted watch backend's event file could not be used.
    ///
    /// A test seam's own error, carried rather than flattened into a scan-config message,
    /// so a malformed script fails where it was written instead of going quiet.
    #[error("invalid watch script: {0}")]
    WatchScript(String),

    /// A supplied file-type rule manifest could not be used.
    ///
    /// Carries the parser's own line-numbered message: a manifest is something a person
    /// edits, and "rejected" without a location is a worse message than none.
    #[error("invalid type rules: {0}")]
    TypeRules(String),

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

/// A bound that may be unlimited.
///
/// One vocabulary for every "how deep", "how many", and "how many rows" the library
/// answers, so a bound reads the same wherever it appears -- a report's depth and row
/// limits, and the extension rows a roll-up carries. The command line spells the
/// unlimited case the same way as the numeric one (`--depth all`, `-n all`) rather than
/// as a separate flag, which is this type surfacing rather than a second grammar.
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
        let stale = Provenance {
            source: Source::Cached,
            observed_at_ns: 100,
            status: Status::Partial(CoverageReason::Inaccessible),
        };
        let combined = verified.combine(stale);
        assert_eq!(combined.source, Source::Cached, "weakest source wins");
        assert_eq!(combined.observed_at_ns, 100, "oldest observation wins");
        assert_eq!(
            combined.status,
            Status::Partial(CoverageReason::Inaccessible),
            "worst status wins, and carries its reason"
        );
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
