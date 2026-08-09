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
//!   the state it observed at the start of its check. If another producer commits a
//!   change first, arbitration rejects the delayed observation.
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

    /// The next clock value.
    #[inline]
    #[must_use]
    pub const fn next(self) -> Clock {
        Clock(self.0 + 1)
    }
}

/// What kind of filesystem entry a record describes.
///
/// The numeric values reach the snapshot format, so they are pinned: never renumber a
/// variant, only append.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum EntryKind {
    File = 0,
    Dir = 1,
    Symlink = 2,
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
    pub size: u64,
    pub mtime_ns: i64,
    pub ctime_ns: i64,
    pub inode: u64,
}

/// Semantic inputs that decide which entries and derived values belong in an index.
///
/// Operational settings such as producer batch size are intentionally absent. A
/// snapshot may be reused only when this value matches exactly.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ScanScope {
    pub max_depth: Option<usize>,
    pub follow_symlinks: bool,
    pub one_filesystem: bool,
    pub ignore_rules_fingerprint: u64,
    pub type_rules_fingerprint: u64,
    pub reducers_fingerprint: u64,
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
        kind: EntryKind,
        attrs: Attrs,
    },
    /// The entry is gone. Implies removal of every descendant.
    Remove { path: PathBuf },
    /// The producer could not describe the change precisely; the consumer must re-scan
    /// this subtree. The scan layer turns this back into precise ops, so escalation is
    /// closed-loop rather than a dead end.
    InvalidateSubtree { path: PathBuf, reason: InvalidateReason },
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
    Present { kind: EntryKind, attrs: Attrs },
}

/// The condition under which an observation may be committed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Expectation {
    /// Commit according to arrival order. Used by freshly verified watch observations
    /// and cold-scan bootstrap data.
    Any,
    /// Commit only if the index still matches the producer baseline.
    State(PathState),
}

/// One observed operation together with its arbitration precondition.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ObservationOp {
    pub op: Op,
    pub expectation: Expectation,
}

impl ObservationOp {
    /// An operation whose fresh verification makes arrival order authoritative.
    pub const fn unconditional(op: Op) -> Self {
        Self { op, expectation: Expectation::Any }
    }

    /// An operation valid only while `state` still matches the index.
    pub const fn if_state(op: Op, state: PathState) -> Self {
        Self { op, expectation: Expectation::State(state) }
    }
}

/// A producer batch awaiting arbitration by the index.
///
/// Batching is not just an efficiency detail: producers coalesce per path within a batch
/// and stat once per batch rather than once per event.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Observation {
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
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.ops.len()
    }
}

/// A committed batch containing only effective mutations.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AppliedDelta {
    pub clock: Clock,
    pub ops: Vec<Op>,
}

impl AppliedDelta {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.ops.len()
    }
}

/// Errors the engine can report.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("path escapes the index root: {0}")]
    PathEscapesRoot(PathBuf),

    #[error("snapshot is not usable: {0}")]
    Snapshot(String),

    #[error("unsupported scan configuration: {0}")]
    UnsupportedScanConfig(&'static str),

    #[error("index lock was poisoned by a panicking writer")]
    IndexLockPoisoned,

    #[error("watch worker stopped before another observation was available")]
    WatchStopped,
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
        assert_eq!(c.next(), Clock(1));
        assert!(c.next() > c);
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
