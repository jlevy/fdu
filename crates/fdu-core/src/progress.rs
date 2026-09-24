//! A polled view of how much work a route has done so far.
//!
//! A [`Progress`] handle is created by the caller and passed to a route that does long
//! work: [`prepare_report_with_progress`](crate::prepare_report_with_progress), a
//! [`ScanConfig`](crate::ScanConfig) with its `progress` field set, and, with the `watch`
//! build feature, `Session::start_with_progress`. The route adds to the handle as it
//! goes; the caller polls [`Progress::snapshot`] from any thread, typically a ticker that
//! redraws a wait indicator. Nothing calls back into the caller, so there is no
//! re-entrancy and no callback cost on worker threads.
//!
//! **It counts work done, never index state.** The counters say how much the walk has
//! read, not what the answer is: an in-progress cold build stays unobservable, and a
//! directory a retry rereads is counted each time it is read. The display should call
//! these entries walked, not found.
//!
//! **What holds at completion.** When a walking route returns, `files` and `bytes` equal
//! the walked totals its own report exposes, and `directories` equals the directories it
//! read, except where a documented retry reread part of the tree, in which case the
//! handle is larger by exactly the rereads. Content analysis leaves `analysis` at
//! `Some((candidates, candidates))`.
//!
//! **Cost.** Walker workers already keep local counts; they add the difference since
//! their last addition to the shared cells once per chunk of directories they hand over,
//! never per entry. Without a handle attached, a walk pays one `Option` check per chunk
//! (per directory on the revalidation and reconcile walks, which fill no batch for an
//! unchanged tree).

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};

/// Which kind of work a route is doing.
///
/// A snapshot carries the phase most recently entered. Routes enter phases in this
/// order, skipping the ones they do not do: a cold report over a full index goes
/// `Scanning`, `Indexing` once the walk is over, then `Analyzing` if content was
/// requested, `Saving` if a snapshot is written, and `Summarizing` while the answer is
/// built; a warm one goes `Loading`, `Revalidating`, then the same without `Indexing`;
/// a cache-only one walks nothing and goes `Loading`, then `Summarizing`.
///
/// A watch start builds no answer through these phases and ends at its save. It then
/// runs a second pass: it verifies the tree once more while it binds observation, and
/// that pass begins again at `Revalidating` with the walk counters restarted, so the
/// line shows the second walk's own progress rather than a sum.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum ProgressPhase {
    /// No route has begun work on this handle.
    ///
    /// What a fresh handle reports until the route enters its first phase. A caller
    /// polling before then shows a wait, not a phase it has not been told about.
    #[default]
    Starting,
    /// A metadata snapshot is being read from the cache.
    Loading,
    /// The tree is being walked cold.
    Scanning,
    /// A loaded snapshot is being verified against the tree.
    Revalidating,
    /// The walk is over and the index is being assembled from the listings it read.
    ///
    /// The walkers count as they read, and the thread that assembles the index can
    /// still be working through their listings when the last one finishes, so the
    /// counters stop moving before the route returns. This phase is what tells a person
    /// the walk has finished rather than stalled.
    Indexing,
    /// File contents are being read and analyzed.
    Analyzing,
    /// A snapshot or content sidecar is being written.
    Saving,
    /// The answer is being built from the index.
    ///
    /// The last phase of a one-shot report over a full index. A save entered before it
    /// continues in the background, so the phase names the work in the foreground:
    /// building a heavy view (`full`, or a deep tree with no limit) over a large index
    /// takes seconds, which `Saving` would misdescribe.
    Summarizing,
}

impl ProgressPhase {
    /// Every phase, indexed by the code a cell stores.
    const ALL: [Self; 8] = [
        Self::Starting,
        Self::Loading,
        Self::Scanning,
        Self::Revalidating,
        Self::Indexing,
        Self::Analyzing,
        Self::Saving,
        Self::Summarizing,
    ];

    const fn code(self) -> u8 {
        match self {
            Self::Starting => 0,
            Self::Loading => 1,
            Self::Scanning => 2,
            Self::Revalidating => 3,
            Self::Indexing => 4,
            Self::Analyzing => 5,
            Self::Saving => 6,
            Self::Summarizing => 7,
        }
    }

    /// The cell only ever stores a value [`Self::code`] produced, so a code outside the
    /// table is unreachable; reading it as `Starting` keeps a poller from panicking on
    /// a state no route can have written.
    fn from_code(code: u8) -> Self {
        Self::ALL.get(usize::from(code)).copied().unwrap_or_default()
    }
}

/// What a route has done so far, as read at one moment.
///
/// A view for display, not a consistent cut: each counter is monotonic within a pass
/// (every route is one pass, except a watch start, whose closing verification begins a
/// second), but the counters are read one at a time, so a snapshot taken while workers
/// run can pair a `files` value with a `bytes` value from a moment later. The phase is
/// the one most recently entered.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ProgressSnapshot {
    /// The kind of work the route is doing.
    pub phase: ProgressPhase,
    /// Directories whose listing was read.
    pub directories: u64,
    /// Regular files whose metadata was read.
    pub files: u64,
    /// Apparent bytes of those files.
    pub bytes: u64,
    /// Content files analyzed so far and the candidates known when analysis began,
    /// or `None` until a route has begun content analysis.
    ///
    /// The one counter with an exact denominator: the candidate set is fixed before the
    /// first file is read, and every candidate produces one result, so the pair reaches
    /// `(n, n)` when analysis ends. A result the index discards as stale still counts as
    /// analyzed, because the file was read.
    pub analysis: Option<(u64, u64)>,
}

/// One cache line, wide enough for the 128-byte lines of the two shipped architectures
/// (Apple Silicon's L2, and the adjacent-line prefetch pairing on x86-64).
///
/// The counters a walker adds to together share one line and share it with nothing
/// else, so a worker's addition costs one line transfer rather than three, and a poller
/// reading the walk counters never invalidates the line the analysis loop writes.
#[repr(align(128))]
#[derive(Default)]
struct WalkCells {
    directories: AtomicU64,
    files: AtomicU64,
    bytes: AtomicU64,
}

/// Written by the content-analysis result loop, on the caller's thread.
#[repr(align(128))]
#[derive(Default)]
struct AnalysisCells {
    /// Whether `total` has been set.
    ///
    /// The one place ordering matters: `total` is stored before this is released, and a
    /// poller acquires this before reading `total`, so a snapshot never pairs a known
    /// analysis with a denominator it has not seen. Once per run, so it costs nothing.
    known: AtomicBool,
    done: AtomicU64,
    total: AtomicU64,
}

/// Written rarely, at phase boundaries.
#[repr(align(128))]
#[derive(Default)]
struct PhaseCell(AtomicU8);

#[derive(Default)]
struct Cells {
    walk: WalkCells,
    analysis: AnalysisCells,
    phase: PhaseCell,
}

/// A handle a route reports its progress through.
///
/// Cheap to clone: clones share one set of counters, so the caller keeps one clone to
/// poll and hands another to the route. A handle is for one run; within a pass the
/// counters only grow, and a second run on the same handle would start from the first
/// run's totals. A watch start is the one run with two passes: its closing verification
/// restarts the walk counters.
///
/// Relaxed atomics, except the once-per-run flag that publishes the analysis denominator
/// (Release and Acquire) and the phase, which a new pass stores with Release after
/// zeroing the counters and a snapshot reads with Acquire before them. Each counter is
/// read on its own, so no other ordering between them is promised; see
/// [`ProgressSnapshot`].
#[derive(Clone, Default)]
pub struct Progress {
    cells: Arc<Cells>,
}

impl Progress {
    /// A fresh handle: phase [`ProgressPhase::Starting`], every counter zero, no analysis.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Read every counter and the current phase.
    ///
    /// Safe to call from any thread at any rate; each call is a handful of atomic loads.
    #[must_use]
    pub fn snapshot(&self) -> ProgressSnapshot {
        let cells = &*self.cells;
        let analysis = cells.analysis.known.load(Ordering::Acquire).then(|| {
            (
                cells.analysis.done.load(Ordering::Relaxed),
                cells.analysis.total.load(Ordering::Relaxed),
            )
        });
        ProgressSnapshot {
            // Acquire, read before the counters: a new pass stores its phase after zeroing them.
            phase: ProgressPhase::from_code(cells.phase.0.load(Ordering::Acquire)),
            directories: cells.walk.directories.load(Ordering::Relaxed),
            files: cells.walk.files.load(Ordering::Relaxed),
            bytes: cells.walk.bytes.load(Ordering::Relaxed),
            analysis,
        }
    }

    /// Record that the route has begun `phase`.
    pub(crate) fn enter(&self, phase: ProgressPhase) {
        self.cells.phase.0.store(phase.code(), Ordering::Relaxed);
    }

    /// Begin a second pass at `phase`, with the walk counters back at zero.
    ///
    /// Only a watch start runs two passes: its closing verification walks the tree again,
    /// and counting that walk on top of the first would show about twice the tree. The
    /// phase is stored after the counters, so a poller that sees the new phase never
    /// pairs it with the first pass's totals.
    #[cfg(feature = "watch")]
    pub(crate) fn begin_pass(&self, phase: ProgressPhase) {
        let walk = &self.cells.walk;
        walk.directories.store(0, Ordering::Relaxed);
        walk.files.store(0, Ordering::Relaxed);
        walk.bytes.store(0, Ordering::Relaxed);
        self.cells.phase.0.store(phase.code(), Ordering::Release);
    }

    /// Add one worker's share of the walk since it last added.
    pub(crate) fn add_walked(&self, directories: u64, files: u64, bytes: u64) {
        let walk = &self.cells.walk;
        walk.directories.fetch_add(directories, Ordering::Relaxed);
        walk.files.fetch_add(files, Ordering::Relaxed);
        walk.bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record the candidate total content analysis will work through.
    pub(crate) fn begin_analysis(&self, total: u64) {
        let analysis = &self.cells.analysis;
        analysis.total.store(total, Ordering::Relaxed);
        analysis.known.store(true, Ordering::Release);
    }

    /// Record one analyzed candidate.
    pub(crate) fn add_analyzed(&self, files: u64) {
        self.cells.analysis.done.fetch_add(files, Ordering::Relaxed);
    }
}

impl fmt::Debug for Progress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ProgressSnapshot { phase, directories, files, bytes, analysis } = self.snapshot();
        f.debug_struct("Progress")
            .field("phase", &phase)
            .field("directories", &directories)
            .field("files", &files)
            .field("bytes", &bytes)
            .field("analysis", &analysis)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_handle_is_starting_with_nothing_counted() {
        assert_eq!(
            Progress::new().snapshot(),
            ProgressSnapshot {
                phase: ProgressPhase::Starting,
                directories: 0,
                files: 0,
                bytes: 0,
                analysis: None,
            }
        );
    }

    #[test]
    fn clones_share_one_set_of_counters() {
        let polled = Progress::new();
        let handed_to_route = polled.clone();
        handed_to_route.enter(ProgressPhase::Scanning);
        handed_to_route.add_walked(2, 5, 700);
        handed_to_route.add_walked(1, 0, 0);
        assert_eq!(
            polled.snapshot(),
            ProgressSnapshot {
                phase: ProgressPhase::Scanning,
                directories: 3,
                files: 5,
                bytes: 700,
                analysis: None,
            }
        );
    }

    #[test]
    fn analysis_is_unknown_until_a_total_is_recorded() {
        let progress = Progress::new();
        progress.add_analyzed(1);
        assert_eq!(progress.snapshot().analysis, None, "a count without a denominator");
        progress.begin_analysis(4);
        assert_eq!(progress.snapshot().analysis, Some((1, 4)));
        progress.add_analyzed(3);
        assert_eq!(progress.snapshot().analysis, Some((4, 4)));
    }

    #[test]
    fn every_phase_survives_the_cell_round_trip() {
        let progress = Progress::new();
        for phase in ProgressPhase::ALL {
            progress.enter(phase);
            assert_eq!(progress.snapshot().phase, phase);
            assert_eq!(ProgressPhase::from_code(phase.code()), phase);
        }
        assert_eq!(ProgressPhase::from_code(u8::MAX), ProgressPhase::Starting);
    }

    #[test]
    fn debug_shows_the_snapshot_rather_than_the_cells() {
        let progress = Progress::new();
        progress.enter(ProgressPhase::Analyzing);
        progress.begin_analysis(2);
        assert_eq!(
            format!("{progress:?}"),
            "Progress { phase: Analyzing, directories: 0, files: 0, bytes: 0, analysis: Some((0, 2)) }"
        );
    }

    #[test]
    fn the_shared_cells_keep_each_writer_on_its_own_line() {
        assert_eq!(std::mem::align_of::<WalkCells>(), 128);
        assert_eq!(std::mem::align_of::<AnalysisCells>(), 128);
        assert_eq!(std::mem::align_of::<PhaseCell>(), 128);
        assert!(std::mem::size_of::<WalkCells>() <= 128, "three counters fit one line");
    }
}
