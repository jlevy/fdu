//! The OS-native watch layer: turning an unreliable event stream into verified observations.
//!
//! This module's whole job is that conversion. Filesystem events are **hints, not
//! truth**, and the ways they lie are documented per platform:
//!
//! - Most events carry no metadata at all, so a producer must stat before it can say
//!   what an entry now looks like.
//! - Only inotify pairs the two sides of a rename (via a kernel cookie). `FSEvents` emits
//!   one path with no mechanism to associate old and new; Windows delivers both sides
//!   with no cookie; poll-based watching cannot see renames at all.
//! - When a directory is created, backends that watch per directory register the new
//!   watch *after* the fact — anything created inside that window produces no event.
//! - Kernel queues overflow. inotify's `Q_OVERFLOW`, `FSEvents`' `MustScanSubDirs`, and
//!   Windows buffer overruns all mean "your view is now incomplete". They surface here
//!   as `Flag::Rescan`, and dropping that signal is precisely how an event-driven index
//!   silently diverges from the filesystem — which is what the `watchfiles` layer
//!   metabrowser runs today does, mapping notify's rich event model down to
//!   `(change, path)` and letting the rescan flag fall through a match arm.
//!
//! So this layer never forwards an event. It coalesces, then **verifies by stat**, and
//! emits only [`Op::Upsert`] with a fresh fingerprint, [`Op::Remove`], or —
//! when it genuinely cannot describe the change — [`Op::InvalidateSubtree`], which the
//! scan layer resolves back into precise committed changes.
//!
//! Building on notify rather than on raw platform APIs is deliberate: its six backends
//! and its overflow signaling are proven, and the information loss that motivates this
//! module all happens in layers *above* it.

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, TrySendError, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use notify::{EventKind, RecursiveMode, Watcher as NotifyWatcher};

mod scripted_events;

use crate::engine_contract::{
    AppliedDelta, Error, InvalidateReason, Observation, ObservationOp, Op, Result,
};
use crate::scan;
use crate::{ApplyOutcome, IndexHandle, ScanConfig};

/// Optimistic re-verification retries before the applying driver guarantees progress
/// through conservative root invalidation and reconciliation.
const MAX_OPTIMISTIC_APPLY_ATTEMPTS: usize = 3;

const WORKER_RUNNING: u8 = 0;
const WORKER_STOPPED: u8 = 1;
const WORKER_PANICKED: u8 = 2;
const MAX_EVENT_CAPACITY: usize = 64 * 1024;
const MAX_BATCH_PATH_CAPACITY: usize = 64 * 1024;
const MAX_BUFFERED_INTENT_PATHS: usize = 1024 * 1024;

/// Tuning for event coalescing.
// No longer `Copy`: a scripted backend names a file, and a config that holds a path is
// a config that allocates. Cloned in the one place that needs two -- the watcher keeps
// its own and the worker takes one.
#[derive(Clone, Debug)]
pub struct WatchConfig {
    /// How long the event stream must be quiet before a batch is emitted.
    pub settle: Duration,
    /// Longest a batch may be held open while events keep arriving. Without a ceiling, a
    /// continuously busy tree would never produce a delta at all.
    pub max_hold: Duration,
    /// Whether a newly created directory triggers a re-list of its contents.
    ///
    /// On by default, and it should stay on for inotify and kqueue: those backends
    /// install a directory's watch only after the create event arrives, so files created
    /// in between are never reported by anything.
    pub relist_new_dirs: bool,
    /// Maximum raw backend events queued before overload collapses to one root
    /// invalidation. Backend callback threads never block on this queue.
    pub event_capacity: usize,
    /// Maximum distinct paths retained in one coalesced intent.
    pub batch_path_capacity: usize,
    /// Maximum coalesced intents awaiting a consumer.
    pub intent_capacity: usize,
    /// Where raw events come from.
    pub backend: WatchBackend,
}

/// The source of a watch's raw filesystem events.
///
/// Explicit rather than detected. Auto-detection means reading the mount table and
/// deciding which filesystem types are trustworthy, and being wrong in the quiet
/// direction -- choosing native on a filesystem that drops events -- produces an index
/// that is silently stale rather than one that is visibly slow. A caller that already
/// knows what it mounted can say so; deciding for it is an open question, not a default.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum WatchBackend {
    /// The platform's native notification API: `FSEvents`, inotify, or `ReadDirectoryChangesW`.
    ///
    /// What a local filesystem should use. Events arrive as they happen and an idle tree
    /// costs nothing.
    #[default]
    Native,
    /// Periodic restat of the whole tree.
    ///
    /// Network and FUSE filesystems accept a native watch and then deliver nothing, which
    /// is the worst failure available: the watcher reports no error and the index quietly
    /// stops tracking. Polling trades a fixed cost per interval for the guarantee that a
    /// change is eventually seen.
    Poll {
        /// How often the tree is restatted. Latency is bounded by this, not by the
        /// settle window.
        interval: Duration,
    },
    /// Events read from a script instead of from the kernel.
    ///
    /// A test seam for the conditions a real filesystem cannot be asked for: every
    /// [`InvalidateReason`] except `Requested` exists for something the kernel does under
    /// pressure, and none of them can be provoked on demand. The script replaces the
    /// event source and nothing else -- the same coalescing, the same stat verification,
    /// the same delta path -- so a scripted event is still verified against the real
    /// filesystem before it becomes an `Op`. A script cannot state a fact about the tree,
    /// only claim that something there may have changed.
    ///
    /// See `watch/scripted_events.rs` for the format.
    #[doc(hidden)]
    Scripted {
        /// Path to the script, whose own paths are relative to the watch root.
        events: PathBuf,
    },
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            // The 50 ms step / 1.6 s ceiling pairing is watchfiles' batching loop, which
            // is the part of that stack worth keeping.
            settle: Duration::from_millis(50),
            max_hold: Duration::from_millis(1600),
            relist_new_dirs: true,
            backend: WatchBackend::Native,
            event_capacity: 4096,
            batch_path_capacity: 4096,
            intent_capacity: 16,
        }
    }
}

impl WatchConfig {
    fn validate(&self) -> Result<()> {
        let buffered_paths = self.batch_path_capacity.checked_mul(self.intent_capacity);
        if self.settle.is_zero()
            || self.max_hold.is_zero()
            || self.max_hold < self.settle
            || self.event_capacity == 0
            || self.batch_path_capacity == 0
            || self.intent_capacity == 0
            || self.event_capacity > MAX_EVENT_CAPACITY
            || self.batch_path_capacity > MAX_BATCH_PATH_CAPACITY
            || buffered_paths.is_none_or(|paths| paths > MAX_BUFFERED_INTENT_PATHS)
        {
            return Err(Error::UnsupportedScanConfig(
                "watch durations and capacities exceed the supported nonzero bounds, or max_hold is less than settle",
            ));
        }
        // A zero poll interval is a busy loop over the whole tree, which is not a faster
        // watch but a slower one: the restat never finishes before the next begins.
        if matches!(self.backend, WatchBackend::Poll { interval } if interval.is_zero()) {
            return Err(Error::UnsupportedScanConfig(
                "the poll backend needs a nonzero interval; polling continuously restats the tree without ever finishing",
            ));
        }
        Ok(())
    }
}

/// What a coalesced path still needs before it can become an observation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pending {
    /// Stat it and decide. Covers creates, writes, removes, and every rename shape:
    /// letting the stat decide is what makes the same code correct on all backends.
    Verify {
        /// Preserve whether a create event occurred while this path was coalesced. Only
        /// a newly created directory has the watch-registration race that needs a relist.
        relist_if_dir: bool,
    },
    /// The producer already knows it cannot describe this precisely.
    Escalate(InvalidateReason),
}

#[derive(Debug, Default)]
struct CoalescedIntent {
    pending: BTreeMap<PathBuf, Pending>,
}

enum RawMessage {
    Event(notify::Result<notify::Event>),
    Stop,
}

/// A live watcher over one tree.
///
/// The watcher is movable to one consuming thread. It is intentionally not shareable:
/// its private standard-library receiver enforces one ordered consumer for coalesced
/// intents. Use a separate [`IndexHandle`] to serve concurrent readers and writers.
/// Dropping the watcher stops the OS watch and shuts the worker thread down.
pub struct Watcher {
    root: PathBuf,
    config: WatchConfig,
    /// `Option` only so [`Drop`] can release it before joining the worker.
    /// Boxed because the backend is chosen at run time: `notify`'s recommended watcher
    /// and its poller are different types implementing one trait.
    inner: Option<Box<dyn NotifyWatcher + Send>>,
    intents: Receiver<CoalescedIntent>,
    control: Option<SyncSender<RawMessage>>,
    cancelled: Arc<AtomicBool>,
    worker_status: Arc<AtomicU8>,
    worker: Option<JoinHandle<()>>,
}

/// Effects of one watch observation and any reconciliation it requested.
#[derive(Debug)]
pub struct WatchApplyReport {
    /// Effect of the verified watch intent itself.
    pub apply: ApplyOutcome,
    /// Effect of closing any invalidation loop opened by the intent.
    pub reconciliation: scan::ReconcileReport,
    /// Nanoseconds spent applying, excluding the wait for an event to arrive.
    ///
    /// Separated because the wait is not work. An idle tree with a one-minute interval
    /// would otherwise report a minute of "cost" for a batch that did nothing, and the one
    /// number an embedder compares providers on would be measuring its own patience.
    pub applied_ns: u64,
    /// Control files this apply saw and did not admit, in path order.
    ///
    /// A tag rule reads its control file from disk by path; the index never holds a row
    /// for one that hidden pruning excluded. So an event on a pruned `.gitignore` leaves no
    /// trace in the delta -- the upsert becomes a removal of a path that was never there --
    /// and a session watching only the delta would never rebind. This is that trace.
    ///
    /// Bounded by the coalesced intent that produced it, and deduplicated: one edit to one
    /// control file is one path however many events the backend reported for it.
    pub pruned_control_files: Vec<PathBuf>,
}

impl Watcher {
    /// Start watching `root` recursively.
    pub fn new(root: &Path, config: WatchConfig) -> Result<Self> {
        config.validate()?;
        let root = root.canonicalize().map_err(|e| Error::io(root, e))?;

        let (raw_tx, raw_rx) = sync_channel::<RawMessage>(config.event_capacity);
        let (intent_tx, intent_rx) = sync_channel::<CoalescedIntent>(config.intent_capacity);
        let control_tx = raw_tx.clone();
        let overflowed = Arc::new(AtomicBool::new(false));
        let callback_overflowed = Arc::clone(&overflowed);

        let handler = move |res| enqueue_raw(&raw_tx, &callback_overflowed, res);
        let mut inner: Option<Box<dyn NotifyWatcher + Send>> = match &config.backend {
            WatchBackend::Native => Some(Box::new(
                notify::recommended_watcher(handler).map_err(|e| notify_error(&root, e))?,
            )),
            WatchBackend::Poll { interval } => Some(Box::new(
                notify::PollWatcher::new(
                    handler,
                    notify::Config::default().with_poll_interval(*interval),
                )
                .map_err(|e| notify_error(&root, e))?,
            )),
            WatchBackend::Scripted { events } => {
                // Read before anything is spawned, so a malformed script fails at
                // construction naming its line rather than starting a watch that goes
                // quiet. Fed through the same queue the kernel's callback uses, so the
                // overflow path a full queue takes is the scripted path too.
                let scripted =
                    scripted_events::read_script(events, &root).map_err(Error::WatchScript)?;
                for event in scripted {
                    handler(event);
                }
                None
            }
        };

        if let Some(inner) = inner.as_mut() {
            inner.watch(&root, RecursiveMode::Recursive).map_err(|e| notify_error(&root, e))?;
        }

        let worker_root = root.clone();
        // The watcher keeps the config it was given; the worker takes its own, because a
        // config that names a script file is no longer `Copy`.
        let worker_config = config.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let worker_overflowed = Arc::clone(&overflowed);
        let worker_status = Arc::new(AtomicU8::new(WORKER_RUNNING));
        let tracked_status = Arc::clone(&worker_status);
        let worker = std::thread::Builder::new()
            .name("fdu-watch".into())
            .spawn(move || {
                let _counter_guard = crate::counters::thread_flush_guard();
                run_tracked_worker(&tracked_status, || {
                    run_worker(
                        &worker_root,
                        &worker_config,
                        &raw_rx,
                        &intent_tx,
                        &worker_overflowed,
                        &worker_cancelled,
                    );
                });
            })
            .map_err(|e| Error::io(&root, e))?;

        Ok(Self {
            root,
            config,
            inner,
            intents: intent_rx,
            control: Some(control_tx),
            cancelled,
            worker_status,
            worker: Some(worker),
        })
    }

    /// Block for and verify the next coalesced intent, up to `timeout`.
    ///
    /// Filesystem calls happen synchronously on this consuming thread and may have
    /// ordinary filesystem latency. A timeout, a stopped worker, and a panicked worker
    /// are distinct outcomes. The returned observation contains relative paths but no
    /// root identity; use [`Self::apply_next`] for the supported root-checked applying
    /// driver rather than applying it to an arbitrary index.
    pub fn next_observation(&self, timeout: Duration) -> Result<Option<Observation>> {
        let Some(intent) = self.next_intent(timeout)? else {
            return Ok(None);
        };
        Ok(Some(verify_intent(&self.root, &self.config, &intent)))
    }

    fn next_intent(&self, timeout: Duration) -> Result<Option<CoalescedIntent>> {
        match self.intents.recv_timeout(timeout) {
            Ok(intent) => Ok(Some(intent)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => match self.worker_status.load(Ordering::Acquire)
            {
                WORKER_PANICKED => Err(Error::WatchWorkerPanicked),
                _ => Err(Error::WatchStopped),
            },
        }
    }

    /// Apply one verified watch observation and close any invalidation loop it opens.
    ///
    /// The scan's own boundary is redrawn around every event before it is applied, so a
    /// depth-bounded or filesystem-bounded scope is watchable. A file cap is not, and is
    /// rejected here before the queue is touched.
    pub fn apply_next(
        &self,
        index: &IndexHandle,
        scan_config: &ScanConfig,
        timeout: Duration,
        sink: &mut dyn FnMut(&AppliedDelta),
    ) -> Result<Option<WatchApplyReport>> {
        scan_config.validate_for_watch_scope(index.scope()?)?;
        let indexed_root = index.root_path()?;
        if indexed_root != self.root {
            return Err(Error::WatchRootMismatch {
                watched: self.root.clone(),
                indexed: indexed_root,
            });
        }
        let Some(intent) = self.next_intent(timeout)? else {
            return Ok(None);
        };
        apply_intent(index, &self.root, &self.config, &intent, scan_config, sink).map(Some)
    }
}

fn apply_intent(
    index: &IndexHandle,
    root: &Path,
    watch_config: &WatchConfig,
    intent: &CoalescedIntent,
    scan_config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<WatchApplyReport> {
    let started = std::time::Instant::now();
    let mut verifier = |_: &Path, _: &Observation| Ok(verify_intent(root, watch_config, intent));
    let mut pruned_control_files = BTreeSet::new();
    let apply = apply_reverified_with(
        index,
        &Observation::default(),
        scan_config,
        &mut verifier,
        &mut pruned_control_files,
    )?;
    if let Some(applied) = &apply.applied {
        sink(applied);
    }
    let reconciliation = scan::reconcile_pending_handle(index, scan_config, sink)?;
    let applied_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    Ok(WatchApplyReport {
        apply,
        reconciliation,
        applied_ns,
        pruned_control_files: pruned_control_files.into_iter().collect(),
    })
}

/// Test the unrooted observation driver without making it a public apply capability.
///
/// Production callers use [`Watcher::apply_next`], which proves that the watcher and
/// index have the same root before consuming an intent. An [`Observation`] intentionally
/// remains a generic producer batch and does not claim a filesystem root identity.
#[cfg(test)]
fn apply_observation(
    index: &IndexHandle,
    observation: &Observation,
    scan_config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<WatchApplyReport> {
    scan_config.validate_for_watch_scope(index.scope()?)?;
    let started = std::time::Instant::now();
    let mut pruned_control_files = BTreeSet::new();
    let apply = apply_reverified(index, observation, scan_config, &mut pruned_control_files)?;
    if let Some(applied) = &apply.applied {
        sink(applied);
    }
    let reconciliation = scan::reconcile_pending_handle(index, scan_config, sink)?;
    let applied_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    Ok(WatchApplyReport {
        apply,
        reconciliation,
        applied_ns,
        pruned_control_files: pruned_control_files.into_iter().collect(),
    })
}

/// Re-stat a queued watch sample against a clock-stable index boundary before applying
/// it. Filesystem verification always runs outside the index lock. The filesystem itself
/// cannot be locked by this process: a sample is valid at its `stat` linearization point,
/// and any mutation after that point remains a later backend event. Queue loss or
/// ambiguity becomes an invalidation and reconciliation, rather than a claim that the
/// disk stayed frozen between `stat` and the in-memory commit. Sustained competing index
/// writes conservatively invalidate the root without blocking readers on filesystem I/O.
#[cfg(test)]
fn apply_reverified(
    index: &IndexHandle,
    observation: &Observation,
    scan_config: &ScanConfig,
    pruned_control_files: &mut BTreeSet<PathBuf>,
) -> Result<ApplyOutcome> {
    apply_reverified_with(
        index,
        observation,
        scan_config,
        &mut reverify_observation,
        pruned_control_files,
    )
}

fn apply_reverified_with(
    index: &IndexHandle,
    observation: &Observation,
    scan_config: &ScanConfig,
    verifier: &mut impl FnMut(&Path, &Observation) -> Result<Observation>,
    pruned_control_files: &mut BTreeSet<PathBuf>,
) -> Result<ApplyOutcome> {
    let (root, scope, _) = index.watch_boundary()?;
    scan_config.validate_for_watch_scope(scope)?;
    for _ in 0..MAX_OPTIMISTIC_APPLY_ATTEMPTS {
        let clock = index.clock()?;
        // Accumulated across attempts rather than reset per attempt: a retry re-verifies
        // the same intent, so what it saw the first time is still true, and the set is
        // deduplicated anyway.
        let candidate =
            admitted(verifier(&root, observation)?, &root, scan_config, pruned_control_files);
        if let Some(outcome) = index.apply_if_clock(clock, &candidate)? {
            return Ok(outcome);
        }
    }

    index.invalidate_root(InvalidateReason::WatchContention)
}

/// Hold a verified observation to the scan's own boundary, after the `stat`.
///
/// The watcher is the third producer of upserts, beside the walk and reconciliation, and
/// the only one that learns what an entry is from an event rather than from a listing it
/// controls. Without this it would be the hole in every scope rule at once: a bounded scan
/// would hold its boundary at boot and at refresh, and then one backend event would put
/// something outside it back -- an index whose contents depend on whether anyone was
/// watching.
///
/// Four axes, and they are not all the same shape. The kind comes with the event. The
/// hidden-path rule and the depth are properties of the path and need no I/O. The
/// filesystem boundary needs a `stat` -- of the entry's *parent*, because the walk retains
/// a mountpoint and stops below it, so the question is whether the walk descended into the
/// parent rather than what device the entry sits on.
///
/// Not the file cap: whether *this* file is inside a cap depends on every other file,
/// including the ones the capped walk never read, which is why the index keeps that one
/// instead, where the count it already maintains answers the question.
///
/// The fast path asks `ScanConfig::narrows_entries` rather than naming the axes, because a
/// fast path that names them is one that forgets the next one: this listed three while the
/// hidden rule was the fourth, and a hidden file created under a live watch entered a
/// pruned index and stayed until the next reconciliation.
///
/// An out-of-scope upsert becomes a removal rather than a dropped op, for the reason the
/// single-path reconcile does the same: the path may already hold a row. A file replaced in
/// place by a socket, or a filesystem mounted over a directory, is exactly one event, and
/// ignoring it would leave the old row standing over the new object forever. Beyond the
/// depth bound the removal is a no-op, since nothing there was ever recorded -- correct at
/// no cost, and cheaper than a second rule saying which of the two it is.
///
/// An invalidation outside the boundary is dropped instead. It asks for a subtree to be
/// reconciled, and there is no subtree: nothing at or below that path is in scope.
///
/// Applied to the candidate rather than inside each verifier, because both verifiers ---
/// the intent path and the test one --- would otherwise carry the rule separately, which
/// is the divergence these predicates exist to prevent.
fn admitted(
    observation: Observation,
    root: &Path,
    config: &ScanConfig,
    pruned_control_files: &mut BTreeSet<PathBuf>,
) -> Observation {
    if !config.narrows_entries() {
        return observation;
    }
    // Read once per apply rather than per op, and only for the axis that needs it. A root
    // that cannot be stat'ed leaves every entry admitted on this axis: the walk that built
    // the index already drew the boundary, and refusing everything here would empty the
    // index rather than bound it.
    //
    // Carried as `None` rather than flattened to `0`, which is the same sentence and the
    // opposite behaviour: no real device is zero, so every successfully `stat`ed parent
    // failed the comparison and a momentarily unreadable root turned into a stream of
    // removals.
    let root_dev = config
        .one_filesystem
        .then(|| std::fs::symlink_metadata(root).ok().map(|meta| scan::observe(&meta).1.dev))
        .flatten();
    // One `stat` per distinct parent rather than per op. A coalesced intent is usually one
    // directory's worth of events, so this is one call for the batch in the common case,
    // and never more than one per directory it touched.
    let mut parents: BTreeMap<PathBuf, bool> = BTreeMap::new();
    let mut inside = |path: &Path| -> bool {
        if !config.one_filesystem {
            return true;
        }
        let parent = path.parent().unwrap_or(Path::new("")).to_path_buf();
        *parents.entry(parent).or_insert_with(|| scan::on_root_filesystem(root, path, root_dev))
    };

    let ops = observation
        .ops
        .into_iter()
        .filter_map(|observed| {
            // Recorded for every op naming a control file this scope excludes, whatever the
            // op is and before anything rewrites it -- afterwards nothing names it.
            //
            // A tag rule reads its control file from disk by path, so a pruned `.gitignore`
            // never has a row: its upsert is rewritten to a removal of a path that was
            // never there, which commits nothing, and a session watching the delta alone
            // saw an idle tree while every tag it governs went stale. This is the live half
            // of what the walk records with `note_pruned_control_file`.
            //
            // Deliberately not folded into the `outside` decision below, which is about
            // whether an op may be applied: a *removal* is never outside -- removing what is
            // not there is harmless and always allowed -- yet deleting a pruned control file
            // is exactly as answer-affecting as creating one. Keying on the rewritten upsert
            // covers create and edit and misses delete.
            if !scan::within_scope(observed.op.path(), config)
                && config
                    .tags
                    .as_ref()
                    .is_some_and(|rules| rules.is_control_file(observed.op.path()))
            {
                pruned_control_files.insert(observed.op.path().to_path_buf());
            }
            let outside = match &observed.op {
                Op::Upsert { path, kind, .. } => {
                    !scan::retains(*kind, config)
                        || !scan::within_scope(path, config)
                        || !inside(path)
                }
                Op::InvalidateSubtree { path, .. } => {
                    !scan::within_scope(path, config) || !inside(path)
                }
                Op::Remove { .. } => false,
            };
            if !outside {
                return Some(observed);
            }
            match observed.op {
                // Rebuilt rather than rebatched through `Observation::new`, which would
                // flatten every expectation to `Any`: substituting one op must not also
                // decide the arbitration precondition its producer attached to the path.
                Op::Upsert { path, .. } => Some(ObservationOp {
                    op: Op::Remove { path },
                    expectation: observed.expectation,
                }),
                // An invalidation outside the boundary asks for a subtree that is not in
                // scope, so there is nothing to reconcile and nothing to remove either.
                _ => None,
            }
        })
        .collect();
    Observation::from_ops(ops)
}

#[cfg(test)]
fn reverify_observation(root: &Path, observation: &Observation) -> Result<Observation> {
    let mut ops = Vec::with_capacity(observation.len());
    for observed in &observation.ops {
        let relative = scan::normalize_subtree(observed.op.path())?;
        let op = match &observed.op {
            Op::InvalidateSubtree { reason, .. } => {
                Op::InvalidateSubtree { path: relative, reason: *reason }
            }
            Op::Upsert { .. } | Op::Remove { .. } => {
                let absolute = root.join(&relative);
                match std::fs::symlink_metadata(&absolute) {
                    Ok(metadata) => {
                        let (kind, attrs) = scan::observe(&metadata);
                        Op::Upsert { path: relative, kind, attrs }
                    }
                    Err(error) => op_for_stat_error(relative, &error),
                }
            }
        };
        ops.push(op);
    }
    Ok(Observation::new(ops))
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        if let Some(control) = self.control.take() {
            let _ = control.try_send(RawMessage::Stop);
        }
        // Stop the backend and release its callback sender before joining. Every worker
        // send is nonblocking, so a full consumer queue cannot deadlock teardown.
        self.inner.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn notify_error(path: &Path, err: notify::Error) -> Error {
    Error::io(path, std::io::Error::other(err))
}

fn enqueue_raw(
    sender: &SyncSender<RawMessage>,
    overflowed: &AtomicBool,
    event: notify::Result<notify::Event>,
) {
    match sender.try_send(RawMessage::Event(event)) {
        Ok(()) | Err(TrySendError::Disconnected(_)) => {}
        Err(TrySendError::Full(_)) => overflowed.store(true, Ordering::Release),
    }
}

fn run_tracked_worker(status: &AtomicU8, worker: impl FnOnce()) {
    let outcome = catch_unwind(AssertUnwindSafe(worker));
    status.store(if outcome.is_ok() { WORKER_STOPPED } else { WORKER_PANICKED }, Ordering::Release);
}

fn run_worker(
    root: &Path,
    config: &WatchConfig,
    raw: &Receiver<RawMessage>,
    out: &SyncSender<CoalescedIntent>,
    overflowed: &AtomicBool,
    cancelled: &AtomicBool,
) {
    let mut pending: BTreeMap<PathBuf, Pending> = BTreeMap::new();
    let mut batch_started: Option<Instant> = None;
    let mut sticky_overflow = false;

    loop {
        if cancelled.load(Ordering::Acquire) {
            return;
        }
        if overflowed.swap(false, Ordering::AcqRel) {
            collapse_to_overflow(&mut pending);
            batch_started.get_or_insert_with(Instant::now);
        }
        if sticky_overflow {
            match try_deliver_overflow(out) {
                Ok(true) => sticky_overflow = false,
                Ok(false) => {}
                Err(()) => return,
            }
        }

        match raw.recv_timeout(config.settle) {
            Ok(RawMessage::Event(Ok(event))) => {
                record(root, &event, &mut pending, config.batch_path_capacity);
                batch_started.get_or_insert_with(Instant::now);
            }
            Ok(RawMessage::Event(Err(err))) => {
                // notify reports a watch failure. It cannot say what was missed, so the
                // only honest response is to escalate the whole tree.
                let _ = err;
                collapse_to_overflow(&mut pending);
                batch_started.get_or_insert_with(Instant::now);
            }
            Ok(RawMessage::Stop) => return,
            Err(RecvTimeoutError::Timeout) => {
                // Quiet for a full step: the batch has settled.
                if !pending.is_empty()
                    && try_deliver_pending(&mut pending, out, &mut sticky_overflow).is_err()
                {
                    return;
                }
                batch_started = None;
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => {
                if !cancelled.load(Ordering::Acquire) && !pending.is_empty() {
                    let _ = try_deliver_pending(&mut pending, out, &mut sticky_overflow);
                }
                return;
            }
        }

        // A tree under continuous churn never goes quiet, so cap how long a batch waits.
        if max_hold_elapsed(batch_started, config.max_hold) {
            if try_deliver_pending(&mut pending, out, &mut sticky_overflow).is_err() {
                return;
            }
            batch_started = None;
        }
    }
}

fn max_hold_elapsed(started: Option<Instant>, max_hold: Duration) -> bool {
    started.is_some_and(|start| start.elapsed() >= max_hold)
}

/// Fold one event into the pending set.
fn record(
    root: &Path,
    event: &notify::Event,
    pending: &mut BTreeMap<PathBuf, Pending>,
    capacity: usize,
) {
    if event.need_rescan() {
        // The kernel dropped events. Escalate the narrowest subtree the event names, or
        // the whole root when it names zero/multiple paths or crosses the watch boundary.
        let target = if event.paths.len() == 1 {
            relative_to(root, &event.paths[0]).unwrap_or_default()
        } else {
            PathBuf::new()
        };
        queue_pending(
            pending,
            target,
            Pending::Escalate(InvalidateReason::WatchOverflow),
            capacity,
        );
        return;
    }

    let rename_mode = match event.kind {
        EventKind::Modify(notify::event::ModifyKind::Name(mode)) => Some(mode),
        _ => None,
    };
    let paired_rename = matches!(rename_mode, Some(notify::event::RenameMode::Both))
        && event.paths.len() == 2
        && event.paths.iter().all(|path| relative_to(root, path).is_some());
    if rename_mode.is_some() && !paired_rename {
        // A one-sided rename gives no safe bound on where its counterpart lives. A full
        // reconciliation is more expensive than guessing a parent, but it cannot leave
        // the old name behind or miss a moved-in subtree.
        queue_pending(
            pending,
            PathBuf::new(),
            Pending::Escalate(InvalidateReason::UnpairedRename),
            capacity,
        );
    }

    for (position, path) in event.paths.iter().enumerate() {
        let Some(rel) = relative_to(root, path) else {
            continue;
        };
        if matches!(event.kind, EventKind::Access(_)) {
            continue; // Reads change nothing this engine records.
        }
        let relist_if_dir = matches!(event.kind, EventKind::Create(_))
            || matches!(rename_mode, Some(notify::event::RenameMode::To))
            || (paired_rename && position == 1);
        queue_pending(pending, rel, Pending::Verify { relist_if_dir }, capacity);
    }
}

fn queue_pending(
    pending: &mut BTreeMap<PathBuf, Pending>,
    path: PathBuf,
    state: Pending,
    capacity: usize,
) {
    if matches!(pending.get(Path::new("")), Some(Pending::Escalate(_))) {
        return;
    }
    if path.as_os_str().is_empty() && matches!(state, Pending::Escalate(_)) {
        pending.clear();
        pending.insert(path, state);
        return;
    }
    if let Some(existing) = pending.get_mut(&path) {
        match (existing, state) {
            (Pending::Escalate(_), _) => {}
            (Pending::Verify { relist_if_dir }, Pending::Verify { relist_if_dir: additional }) => {
                *relist_if_dir |= additional;
            }
            (slot @ Pending::Verify { .. }, Pending::Escalate(reason)) => {
                *slot = Pending::Escalate(reason);
            }
        }
        return;
    }
    if pending.len() >= capacity {
        collapse_to_overflow(pending);
    } else {
        pending.insert(path, state);
    }
}

fn collapse_to_overflow(pending: &mut BTreeMap<PathBuf, Pending>) {
    pending.clear();
    pending.insert(PathBuf::new(), Pending::Escalate(InvalidateReason::WatchOverflow));
}

fn try_deliver_pending(
    pending: &mut BTreeMap<PathBuf, Pending>,
    out: &SyncSender<CoalescedIntent>,
    sticky_overflow: &mut bool,
) -> std::result::Result<(), ()> {
    if pending.is_empty() {
        return Ok(());
    }
    let intent = CoalescedIntent { pending: std::mem::take(pending) };
    match out.try_send(intent) {
        Ok(()) => Ok(()),
        Err(TrySendError::Full(_)) => {
            *sticky_overflow = true;
            Ok(())
        }
        Err(TrySendError::Disconnected(_)) => Err(()),
    }
}

fn try_deliver_overflow(out: &SyncSender<CoalescedIntent>) -> std::result::Result<bool, ()> {
    let mut pending = BTreeMap::new();
    collapse_to_overflow(&mut pending);
    match out.try_send(CoalescedIntent { pending }) {
        Ok(()) => Ok(true),
        Err(TrySendError::Full(_)) => Ok(false),
        Err(TrySendError::Disconnected(_)) => Err(()),
    }
}

/// Verify one bounded intent: stat once per path, never once per backend event.
fn verify_intent(root: &Path, config: &WatchConfig, intent: &CoalescedIntent) -> Observation {
    let mut ops = Vec::with_capacity(intent.pending.len());

    for (rel, state) in &intent.pending {
        match state {
            Pending::Escalate(reason) => {
                ops.push(Op::InvalidateSubtree { path: rel.clone(), reason: *reason });
            }
            Pending::Verify { relist_if_dir } => {
                let absolute = root.join(rel);
                match std::fs::symlink_metadata(&absolute) {
                    Ok(meta) => {
                        let (kind, attrs) = scan::observe(&meta);
                        ops.push(Op::Upsert { path: rel.clone(), kind, attrs });
                        if kind.is_dir() && *relist_if_dir && config.relist_new_dirs {
                            // The watch for this directory was installed after it was
                            // created, so anything already inside produced no event.
                            ops.push(Op::InvalidateSubtree {
                                path: rel.clone(),
                                reason: InvalidateReason::WatchSetupRace,
                            });
                        }
                    }
                    Err(error) => ops.push(op_for_stat_error(rel.clone(), &error)),
                }
            }
        }
    }
    Observation::new(ops)
}

fn op_for_stat_error(path: PathBuf, error: &std::io::Error) -> Op {
    match error.kind() {
        std::io::ErrorKind::NotFound if path.as_os_str().is_empty() => {
            Op::InvalidateSubtree { path, reason: InvalidateReason::VerificationFailed }
        }
        std::io::ErrorKind::NotFound => Op::Remove { path },
        std::io::ErrorKind::NotADirectory => Op::InvalidateSubtree {
            path: path.parent().map_or_else(PathBuf::new, Path::to_path_buf),
            reason: InvalidateReason::VerificationFailed,
        },
        _ => Op::InvalidateSubtree { path, reason: InvalidateReason::VerificationFailed },
    }
}

/// Express an absolute path relative to the watch root.
///
/// Returns `None` for anything outside the root, which should not happen but is not
/// worth trusting a backend about.
fn relative_to(root: &Path, path: &Path) -> Option<PathBuf> {
    path.strip_prefix(root).ok().map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_contract::{Expectation, PathExpectation, PathState};
    use crate::{Attrs, EntryKind};
    use notify::event::{CreateKind, Flag, MetadataKind, ModifyKind, RenameMode};
    use std::fs;

    /// An excluded kind becomes a removal of the path, not a dropped operation.
    ///
    /// Asserted here rather than through a live session, and the reason is worth writing
    /// down: on Linux a rename onto a watched path escalates to a root invalidation, so
    /// reconciliation sweeps the stale row away and both implementations look alike from
    /// outside. The integration tests prove the *rule* holds end to end; only this one
    /// separates excluding the object from ignoring its event, which is the difference
    /// that matters on any backend that reports the file without invalidating its parent.
    #[test]
    fn an_excluded_kind_becomes_a_removal_rather_than_a_dropped_operation() {
        let config = ScanConfig { exclude_special: true, ..ScanConfig::default() };
        let observation = Observation::new(vec![
            Op::Upsert {
                path: PathBuf::from("sock"),
                kind: EntryKind::Other,
                attrs: Attrs::default(),
            },
            Op::Upsert {
                path: PathBuf::from("file.txt"),
                kind: EntryKind::File,
                attrs: Attrs::default(),
            },
        ]);

        let held = admitted(observation, Path::new("/"), &config, &mut BTreeSet::new());
        assert_eq!(held.len(), 2, "the batch keeps its length: one op in, one op out");
        assert!(
            matches!(&held.ops[0].op, Op::Remove { path } if path == Path::new("sock")),
            "the socket's upsert became a removal of the path it named: {:?}",
            held.ops[0].op
        );
        assert!(
            matches!(&held.ops[1].op, Op::Upsert { kind: EntryKind::File, .. }),
            "and nothing else was touched: {:?}",
            held.ops[1].op
        );
    }

    /// Beyond the depth bound an upsert becomes a removal, and an invalidation is dropped.
    ///
    /// Asserted here rather than through a live session for the reason the special-object
    /// case gives: on Linux a rename onto or out of a watched path escalates to a root
    /// invalidation, so reconciliation sweeps the stale row away and dropping the event
    /// looks identical from outside. The two ops differ in what they *ask for* -- one
    /// takes a row away, the other asks for a subtree that is not in scope to be walked --
    /// so they are checked where that difference is visible.
    #[test]
    fn beyond_the_depth_bound_an_upsert_removes_and_an_invalidation_is_dropped() {
        let config = ScanConfig { max_depth: Some(1), ..ScanConfig::default() };
        let observation = Observation::new(vec![
            Op::Upsert {
                path: PathBuf::from("sub/deep.txt"),
                kind: EntryKind::File,
                attrs: Attrs::default(),
            },
            Op::InvalidateSubtree {
                path: PathBuf::from("sub/deeper"),
                reason: InvalidateReason::WatchSetupRace,
            },
            Op::Upsert {
                path: PathBuf::from("shallow.txt"),
                kind: EntryKind::File,
                attrs: Attrs::default(),
            },
        ]);

        let held = admitted(observation, Path::new("/"), &config, &mut BTreeSet::new());
        assert_eq!(held.len(), 2, "the invalidation is gone and the upsert is not: {held:?}");
        assert!(
            matches!(&held.ops[0].op, Op::Remove { path } if path == Path::new("sub/deep.txt")),
            "a row at a path outside the bound is taken away, not left standing: {:?}",
            held.ops[0].op
        );
        assert!(
            matches!(&held.ops[1].op, Op::Upsert { path, .. } if path == Path::new("shallow.txt")),
            "and the entry inside the bound is untouched: {:?}",
            held.ops[1].op
        );
    }

    /// Depth counts the same way the walk does: `max` components are inside the bound.
    ///
    /// The off-by-one that would otherwise be invisible, because both readings admit the
    /// root's own children and differ only one level down.
    #[test]
    fn the_depth_bound_admits_exactly_the_depth_the_walk_records() {
        let config = ScanConfig { max_depth: Some(2), ..ScanConfig::default() };
        for (path, inside) in [("a", true), ("a/b", true), ("a/b/c", false)] {
            assert_eq!(
                scan::within_scope(Path::new(path), &config),
                inside,
                "{path:?} at a bound of two"
            );
        }
    }

    /// Keeping is keeping: the default scope hands the batch back unchanged.
    #[test]
    fn a_default_scope_leaves_a_special_object_alone() {
        let observation = Observation::new(vec![Op::Upsert {
            path: PathBuf::from("sock"),
            kind: EntryKind::Other,
            attrs: Attrs::default(),
        }]);
        let held = admitted(
            observation.clone(),
            Path::new("/"),
            &ScanConfig::default(),
            &mut BTreeSet::new(),
        );
        assert_eq!(held, observation, "nothing excludes it, so nothing rewrites it");
    }

    /// An unreadable root leaves the filesystem boundary undrawn, not drawn at zero.
    ///
    /// This is the production half of the `Option`. The value used to be a `u64` with a
    /// failed root `stat` flattened to `0`, so a root that momentarily could not be read
    /// turned every successfully `stat`ed parent into a mismatch -- and an out-of-scope
    /// upsert becomes a *removal*, so the index would have been emptied one event at a
    /// time by a transient error whose comment said it fails open.
    ///
    /// Two absences meet on this path and only one of them is the subject, so the test has
    /// to separate them: a root whose `stat` fails, and a *parent* whose `stat` fails.
    /// Both admit, so a nonexistent root with an ordinary relative path passes whatever the
    /// missing device is flattened to. The op therefore carries an absolute path, which
    /// `Path::join` resolves to itself -- the parent is a real directory with a real
    /// nonzero device while the root is not readable at all, which is the one arrangement
    /// where the root's absence is the only thing left to decide the answer.
    #[test]
    fn an_unreadable_root_admits_rather_than_removing_everything() {
        let dir = tempfile::tempdir().expect("temp");
        fs::create_dir(dir.path().join("src")).expect("mkdir");
        let live = dir.path().join("src").join("main.rs");
        fs::write(&live, b"fn main() {}").expect("write");

        let observation = Observation::new(vec![Op::Upsert {
            path: live,
            kind: EntryKind::File,
            attrs: Attrs::default(),
        }]);
        let config = ScanConfig { one_filesystem: true, ..ScanConfig::default() };

        let held = admitted(
            observation.clone(),
            Path::new("/fdu-no-such-root-3f1c9e2a"),
            &config,
            &mut BTreeSet::new(),
        );
        assert_eq!(
            held, observation,
            "with no root device there is no boundary, so the upsert must survive it"
        );
    }

    /// Substituting an operation must not also rewrite its arbitration precondition.
    ///
    /// The batch a producer hands over carries, per path, the state it believed the index
    /// was in. Rebatching through `Observation::new` would flatten every one of those to
    /// `Any` -- turning a conditional removal into an unconditional one, and silently
    /// widening what an excluded kind is allowed to overwrite.
    #[test]
    fn a_substituted_removal_keeps_the_expectation_its_producer_attached() {
        // A concrete precondition rather than a blank one, so a rebuild that discarded it
        // would be visible: `Any` is what the flattening bug produces.
        let expectation = PathExpectation::new(
            PathState::Present { kind: EntryKind::File, attrs: Attrs::default() },
            None,
            None,
        );
        let observation = Observation::from_ops(vec![ObservationOp::if_state(
            Op::Upsert {
                path: PathBuf::from("sock"),
                kind: EntryKind::Other,
                attrs: Attrs::default(),
            },
            expectation,
        )]);

        let config = ScanConfig { exclude_special: true, ..ScanConfig::default() };
        let held = admitted(observation, Path::new("/"), &config, &mut BTreeSet::new());
        assert_eq!(
            held.ops[0].expectation,
            Expectation::State(expectation),
            "the precondition belongs to the path, not to the operation that was replaced"
        );
    }

    fn queued_test_watcher(root: PathBuf) -> (SyncSender<CoalescedIntent>, Watcher) {
        let (sender, intents) = sync_channel(1);
        let watcher = Watcher {
            root,
            config: WatchConfig::default(),
            inner: None,
            intents,
            control: None,
            cancelled: Arc::new(AtomicBool::new(false)),
            worker_status: Arc::new(AtomicU8::new(WORKER_RUNNING)),
            worker: None,
        };
        (sender, watcher)
    }

    #[test]
    fn watcher_can_move_to_its_single_consumer_thread() {
        fn assert_send<T: Send>() {}

        assert_send::<Watcher>();
    }

    /// Collect deltas until `want` is satisfied or the deadline passes.
    ///
    /// Event latency varies by orders of magnitude across backends (inotify is
    /// immediate, `FSEvents` batches), so the test waits on a condition rather than
    /// sleeping for a fixed guess.
    fn wait_for(
        watcher: &Watcher,
        deadline: Duration,
        mut want: impl FnMut(&[Op]) -> bool,
    ) -> Vec<Op> {
        let start = Instant::now();
        let mut seen: Vec<Op> = Vec::new();
        while start.elapsed() < deadline {
            match watcher.next_observation(Duration::from_millis(200)) {
                Ok(Some(observation)) => {
                    seen.extend(observation.ops.into_iter().map(|observed| observed.op));
                    if want(&seen) {
                        return seen;
                    }
                }
                Ok(None) => {}
                Err(error) => panic!("watcher stopped while waiting: {error}"),
            }
        }
        seen
    }

    #[test]
    fn created_files_arrive_as_verified_upserts() {
        let dir = tempfile::tempdir().expect("tempdir");
        let watcher = Watcher::new(dir.path(), WatchConfig::default()).expect("watcher");

        fs::write(dir.path().join("hello.txt"), b"hello world").expect("write");

        let ops = wait_for(&watcher, Duration::from_secs(20), |ops| {
            ops.iter().any(|op| op.path() == Path::new("hello.txt"))
        });

        let found = ops
            .iter()
            .find(|op| op.path() == Path::new("hello.txt"))
            .expect("an op for the new file");
        match found {
            Op::Upsert { attrs, kind, .. } => {
                assert!(!kind.is_dir());
                // The point of verify-then-emit: the delta carries real stat data, which
                // no backend put in the event.
                assert_eq!(attrs.size, 11);
                assert!(attrs.mtime_ns > 0);
            }
            other => panic!("expected an upsert, got {other:?}"),
        }
    }

    #[test]
    fn deleted_files_arrive_as_removes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("doomed.txt");
        fs::write(&path, b"x").expect("write");

        let watcher = Watcher::new(dir.path(), WatchConfig::default()).expect("watcher");
        fs::remove_file(&path).expect("remove");

        let ops = wait_for(&watcher, Duration::from_secs(20), |ops| {
            ops.iter()
                .any(|op| matches!(op, Op::Remove { path } if path == Path::new("doomed.txt")))
        });

        assert!(
            ops.iter()
                .any(|op| matches!(op, Op::Remove { path } if path == Path::new("doomed.txt"))),
            "expected a remove, saw {ops:?}"
        );
    }

    #[test]
    fn a_new_directory_also_escalates_for_a_relist() {
        let dir = tempfile::tempdir().expect("tempdir");
        let watcher = Watcher::new(dir.path(), WatchConfig::default()).expect("watcher");

        // Populate the directory immediately after creating it — the window in which a
        // per-directory backend has not yet installed its watch.
        let sub = dir.path().join("fresh");
        fs::create_dir(&sub).expect("mkdir");
        fs::write(sub.join("inside.txt"), b"raced").expect("write");

        let ops = wait_for(&watcher, Duration::from_secs(20), |ops| {
            ops.iter().any(|op| {
                matches!(
                    op,
                    Op::InvalidateSubtree { path, reason: InvalidateReason::WatchSetupRace }
                        if path == Path::new("fresh")
                )
            })
        });

        assert!(
            ops.iter().any(|op| matches!(
                op,
                Op::InvalidateSubtree { path, reason: InvalidateReason::WatchSetupRace }
                    if path == Path::new("fresh")
            )),
            "a created directory must escalate so its contents get listed, saw {ops:?}"
        );
    }

    /// The poll backend produces the same verified ops the native one does.
    ///
    /// The point of the seam: only the source of raw events changes. Coalescing, the stat
    /// verification, and the delta path are the same code, so a caller on a filesystem
    /// that drops native events gets the same answers more slowly rather than different
    /// answers.
    #[test]
    fn the_poll_backend_reports_the_same_verified_ops() {
        let dir = tempfile::tempdir().expect("tempdir");
        let watcher = Watcher::new(
            dir.path(),
            WatchConfig {
                backend: WatchBackend::Poll { interval: Duration::from_millis(50) },
                ..WatchConfig::default()
            },
        )
        .expect("poll watcher");

        let created = dir.path().join("polled.txt");
        fs::write(&created, b"hello").expect("write");

        // Polling latency is the interval plus a restat of the tree, so the wait is
        // generous: what is asserted is the op, not how fast it arrived.
        let ops = wait_for(&watcher, Duration::from_secs(10), |ops| {
            ops.iter().any(|op| op.path() == Path::new("polled.txt"))
        });
        let upsert = ops
            .iter()
            .find(|op| op.path() == Path::new("polled.txt"))
            .expect("the poll backend must report the created file");
        assert!(
            matches!(upsert, Op::Upsert { kind, attrs, .. }
                if *kind == EntryKind::File && attrs.size == 5),
            "a polled event is still verified by stat before becoming an op: {upsert:?}"
        );
    }

    /// A zero poll interval is a busy restat loop, not a faster watch.
    #[test]
    fn a_zero_poll_interval_is_rejected() {
        let dir = tempfile::tempdir().expect("tempdir");
        let Err(error) = Watcher::new(
            dir.path(),
            WatchConfig {
                backend: WatchBackend::Poll { interval: Duration::ZERO },
                ..WatchConfig::default()
            },
        ) else {
            panic!("a zero interval must be rejected");
        };
        assert!(error.to_string().contains("nonzero interval"), "{error}");
    }

    /// Write a script beside the watched tree and open a watcher driven by it.
    ///
    /// The script lives outside the root so it cannot itself be an event, and its own
    /// paths stay relative so the file says nothing about this machine.
    fn scripted_watcher(root: &Path, script_dir: &Path, script: &str) -> Result<Watcher> {
        let path = script_dir.join("events.script");
        fs::write(&path, script).expect("write script");
        Watcher::new(
            root,
            WatchConfig {
                backend: WatchBackend::Scripted { events: path },
                settle: Duration::from_millis(10),
                max_hold: Duration::from_millis(100),
                ..WatchConfig::default()
            },
        )
    }

    /// A dropped-event queue escalates, scoped to the path the backend named.
    ///
    /// The condition every backend signals and no test can provoke: `Q_OVERFLOW`,
    /// `MustScanSubDirs`, a `ReadDirectoryChangesW` overrun. Swallowing the flag silently
    /// corrupts any index built on events, so the escalation is the contract -- and until
    /// there was a seam, nothing exercised it end to end.
    ///
    /// Scoped rather than root-wide: the backend told us *where* it lost events, and
    /// reconciling the whole tree for a subtree's worth of loss is a cost with nothing
    /// behind it.
    #[test]
    fn a_scripted_overflow_escalates_the_path_it_names() {
        let dir = tempfile::tempdir().expect("tempdir");
        let scripts = tempfile::tempdir().expect("script dir");
        fs::create_dir(dir.path().join("src")).expect("mkdir");
        fs::write(dir.path().join("src/present.txt"), b"x").expect("write");

        let watcher = scripted_watcher(dir.path(), scripts.path(), "rescan\tsrc\n")
            .expect("scripted watcher");
        let ops = wait_for(&watcher, Duration::from_secs(5), |ops| !ops.is_empty());
        assert!(
            ops.iter().any(|op| matches!(
                op,
                Op::InvalidateSubtree { path, reason: InvalidateReason::WatchOverflow }
                    if path == Path::new("src")
            )),
            "a dropped-event flag must escalate the subtree it named: {ops:?}"
        );
    }

    /// A rename with no partner reconciles the whole tree rather than guessing.
    ///
    /// `FSEvents` reports one side of a rename with no way to associate the other. A lone
    /// `From` must not be read as a delete -- the file may have moved anywhere in the tree
    /// -- and there is no safe bound on where its counterpart landed, so the escalation is
    /// root-wide even though the event named one path. That is the expensive answer, and
    /// it is the only one that can neither leave the old name behind nor miss a moved-in
    /// subtree.
    #[test]
    fn a_scripted_unpaired_rename_reconciles_the_whole_tree() {
        let dir = tempfile::tempdir().expect("tempdir");
        let scripts = tempfile::tempdir().expect("script dir");
        fs::create_dir(dir.path().join("src")).expect("mkdir");
        fs::write(dir.path().join("src/moved.txt"), b"x").expect("write");

        let watcher = scripted_watcher(dir.path(), scripts.path(), "rename-from\tsrc/moved.txt\n")
            .expect("scripted watcher");
        let ops = wait_for(&watcher, Duration::from_secs(5), |ops| !ops.is_empty());
        assert!(
            ops.iter().any(|op| matches!(
                op,
                Op::InvalidateSubtree { path, reason: InvalidateReason::UnpairedRename }
                    if path.as_os_str().is_empty()
            )),
            "an unpaired rename has no safe bound and must escalate to the root: {ops:?}"
        );
    }

    /// A scripted event is still verified by stat before it becomes an op.
    ///
    /// The property that keeps this a test seam rather than a back door: a script claims
    /// something *may* have changed, and the filesystem decides what actually did. Here
    /// the script names a file that was never created, and the engine reports a removal
    /// rather than inventing the file the script implied.
    #[test]
    fn a_scripted_event_cannot_state_a_fact_the_filesystem_denies() {
        let dir = tempfile::tempdir().expect("tempdir");
        let scripts = tempfile::tempdir().expect("script dir");

        let watcher = scripted_watcher(dir.path(), scripts.path(), "create\tnever-existed.txt\n")
            .expect("scripted watcher");
        let ops = wait_for(&watcher, Duration::from_secs(5), |ops| !ops.is_empty());
        let reported = ops
            .iter()
            .find(|op| op.path() == Path::new("never-existed.txt"))
            .expect("the script's path must be reported one way or the other");
        assert!(
            matches!(reported, Op::Remove { .. }),
            "a create the filesystem denies must not become an upsert: {reported:?}"
        );
    }

    /// A malformed script fails at construction, naming its line.
    #[test]
    fn a_malformed_script_is_rejected_before_the_watch_starts() {
        let dir = tempfile::tempdir().expect("tempdir");
        let scripts = tempfile::tempdir().expect("script dir");
        let Err(error) = scripted_watcher(dir.path(), scripts.path(), "teleport\ta.txt\n") else {
            panic!("a malformed script must be rejected");
        };
        assert!(error.to_string().contains("line 1"), "{error}");
        assert!(error.to_string().contains("unknown directive"), "{error}");
    }

    #[test]
    fn watching_a_missing_path_is_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("not-there");
        assert!(Watcher::new(&missing, WatchConfig::default()).is_err());
    }

    #[test]
    fn paths_outside_the_root_are_ignored() {
        let root = Path::new("/a/b");
        assert_eq!(relative_to(root, Path::new("/a/b/c/d")), Some(PathBuf::from("c/d")));
        assert_eq!(relative_to(root, Path::new("/elsewhere")), None);
    }

    #[test]
    fn verification_errors_distinguish_absence_from_an_invalid_ancestor() {
        let path = PathBuf::from("parent/known.txt");
        let missing = op_for_stat_error(
            path.clone(),
            &std::io::Error::new(std::io::ErrorKind::NotFound, "gone"),
        );
        assert!(matches!(missing, Op::Remove { path: removed } if removed == path));

        let not_a_directory = op_for_stat_error(
            path.clone(),
            &std::io::Error::new(std::io::ErrorKind::NotADirectory, "ancestor is a file"),
        );
        assert!(matches!(
            not_a_directory,
            Op::InvalidateSubtree {
                path: invalidated,
                reason: InvalidateReason::VerificationFailed,
            } if invalidated == Path::new("parent")
        ));

        let denied = op_for_stat_error(
            path.clone(),
            &std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        );
        assert!(matches!(
            denied,
            Op::InvalidateSubtree {
                path: invalidated,
                reason: InvalidateReason::VerificationFailed,
            } if invalidated == path
        ));
    }

    #[test]
    fn create_intent_survives_coalescing_but_metadata_only_does_not_relist() {
        let root = Path::new("/watch-root");
        let path = root.join("directory");
        let mut pending = BTreeMap::new();

        record(
            root,
            &notify::Event::new(EventKind::Create(CreateKind::Folder)).add_path(path.clone()),
            &mut pending,
            16,
        );
        record(
            root,
            &notify::Event::new(EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)))
                .add_path(path),
            &mut pending,
            16,
        );
        assert_eq!(
            pending.get(Path::new("directory")),
            Some(&Pending::Verify { relist_if_dir: true })
        );

        let mut metadata_only = BTreeMap::new();
        record(
            root,
            &notify::Event::new(EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)))
                .add_path(root.join("existing")),
            &mut metadata_only,
            16,
        );
        assert_eq!(
            metadata_only.get(Path::new("existing")),
            Some(&Pending::Verify { relist_if_dir: false })
        );
    }

    #[test]
    fn unpaired_renames_and_ambiguous_rescans_escalate_the_root() {
        let root = Path::new("/watch-root");
        let mut rename_pending = BTreeMap::new();
        record(
            root,
            &notify::Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::From)))
                .add_path(root.join("old")),
            &mut rename_pending,
            16,
        );
        assert_eq!(
            rename_pending.get(Path::new("")),
            Some(&Pending::Escalate(InvalidateReason::UnpairedRename))
        );

        let mut rescan_pending = BTreeMap::new();
        record(
            root,
            &notify::Event::new(EventKind::Any)
                .add_path(root.join("a"))
                .add_path(root.join("b"))
                .set_flag(Flag::Rescan),
            &mut rescan_pending,
            16,
        );
        assert_eq!(
            rescan_pending.get(Path::new("")),
            Some(&Pending::Escalate(InvalidateReason::WatchOverflow))
        );
    }

    #[test]
    fn paired_rename_preserves_the_new_directory_relist_intent() {
        let root = Path::new("/watch-root");
        let mut pending = BTreeMap::new();
        record(
            root,
            &notify::Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
                .add_path(root.join("old"))
                .add_path(root.join("new")),
            &mut pending,
            16,
        );

        assert!(!matches!(
            pending.get(Path::new("")),
            Some(Pending::Escalate(InvalidateReason::UnpairedRename))
        ));
        assert_eq!(pending.get(Path::new("old")), Some(&Pending::Verify { relist_if_dir: false }));
        assert_eq!(pending.get(Path::new("new")), Some(&Pending::Verify { relist_if_dir: true }));
    }

    #[test]
    fn pending_path_overload_collapses_to_one_root_invalidation() {
        let root = Path::new("/watch-root");
        let mut pending = BTreeMap::new();
        for name in ["one", "two", "three"] {
            record(
                root,
                &notify::Event::new(EventKind::Any).add_path(root.join(name)),
                &mut pending,
                2,
            );
        }

        assert_eq!(pending.len(), 1);
        assert_eq!(
            pending.get(Path::new("")),
            Some(&Pending::Escalate(InvalidateReason::WatchOverflow))
        );
    }

    #[test]
    fn continuous_churn_has_a_deterministic_max_hold_ceiling() {
        let past = Instant::now()
            .checked_sub(Duration::from_secs(2))
            .expect("representable earlier instant");
        assert!(max_hold_elapsed(Some(past), Duration::from_secs(1)));
        assert!(!max_hold_elapsed(None, Duration::from_secs(1)));
    }

    #[test]
    fn backend_enqueue_is_nonblocking_and_marks_overflow() {
        let (sender, receiver) = sync_channel(1);
        let overflowed = AtomicBool::new(false);
        enqueue_raw(&sender, &overflowed, Ok(notify::Event::new(EventKind::Any)));
        enqueue_raw(&sender, &overflowed, Ok(notify::Event::new(EventKind::Any)));

        assert!(overflowed.load(Ordering::Acquire));
        assert!(matches!(receiver.try_recv(), Ok(RawMessage::Event(Ok(_)))));
    }

    #[test]
    fn full_intent_queue_retains_a_sticky_root_invalidation() {
        let (sender, receiver) = sync_channel(1);
        sender.try_send(CoalescedIntent::default()).expect("fill output");
        let mut pending =
            BTreeMap::from([(PathBuf::from("lost.txt"), Pending::Verify { relist_if_dir: false })]);
        let mut sticky_overflow = false;

        try_deliver_pending(&mut pending, &sender, &mut sticky_overflow).expect("connected");
        assert!(pending.is_empty());
        assert!(sticky_overflow);

        receiver.try_recv().expect("make output capacity");
        assert!(try_deliver_overflow(&sender).expect("connected"));
        let intent = receiver.try_recv().expect("sticky overflow intent");
        let observation = verify_intent(Path::new("/unused"), &WatchConfig::default(), &intent);
        assert!(matches!(
            &observation.ops[0].op,
            Op::InvalidateSubtree {
                path,
                reason: InvalidateReason::WatchOverflow,
            } if path.as_os_str().is_empty()
        ));
    }

    #[test]
    fn cancellation_wakes_and_joins_with_a_full_intent_queue() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonical root");
        let config = WatchConfig {
            settle: Duration::from_secs(30),
            max_hold: Duration::from_secs(30),
            event_capacity: 1,
            batch_path_capacity: 1,
            intent_capacity: 1,
            ..WatchConfig::default()
        };
        let (control, raw) = sync_channel(1);
        let (output, intents) = sync_channel(1);
        output.try_send(CoalescedIntent::default()).expect("fill intent queue");
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = Arc::clone(&cancelled);
        let status = Arc::new(AtomicU8::new(WORKER_RUNNING));
        let tracked_status = Arc::clone(&status);
        let overflowed = Arc::new(AtomicBool::new(false));
        let worker_overflowed = Arc::clone(&overflowed);
        let worker_root = root.clone();
        let worker_config = config.clone();
        let worker = std::thread::spawn(move || {
            run_tracked_worker(&tracked_status, || {
                run_worker(
                    &worker_root,
                    &worker_config,
                    &raw,
                    &output,
                    &worker_overflowed,
                    &worker_cancelled,
                );
            });
        });
        let watcher = Watcher {
            root,
            config,
            inner: None,
            intents,
            control: Some(control),
            cancelled,
            worker_status: status,
            worker: Some(worker),
        };
        let (done_tx, done_rx) = sync_channel(1);

        std::thread::spawn(move || {
            drop(watcher);
            done_tx.send(()).expect("report drop");
        });

        done_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("watcher drop must wake and join promptly");
    }

    #[test]
    fn coalescing_defers_filesystem_verification_to_the_consumer() {
        let dir = tempfile::tempdir().expect("tempdir");
        let relative = PathBuf::from("appeared.txt");
        let intent = CoalescedIntent {
            pending: BTreeMap::from([(relative.clone(), Pending::Verify { relist_if_dir: false })]),
        };

        fs::write(dir.path().join(&relative), b"current").expect("create after coalescing");
        let observation = verify_intent(dir.path(), &WatchConfig::default(), &intent);

        assert!(matches!(
            &observation.ops[0].op,
            Op::Upsert { path, attrs, .. } if path == &relative && attrs.size == 7
        ));
    }

    #[test]
    fn timeout_stop_and_worker_panic_are_distinct() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (live_sender, live) =
            queued_test_watcher(dir.path().canonicalize().expect("canonical root"));
        assert!(live.next_observation(Duration::ZERO).expect("timeout").is_none());
        drop(live_sender);

        let (stopped_sender, stopped) =
            queued_test_watcher(dir.path().canonicalize().expect("canonical root"));
        stopped.worker_status.store(WORKER_STOPPED, Ordering::Release);
        drop(stopped_sender);
        assert!(matches!(stopped.next_observation(Duration::ZERO), Err(Error::WatchStopped)));

        let (panicked_sender, panicked) =
            queued_test_watcher(dir.path().canonicalize().expect("canonical root"));
        panicked.worker_status.store(WORKER_PANICKED, Ordering::Release);
        drop(panicked_sender);
        assert!(matches!(
            panicked.next_observation(Duration::ZERO),
            Err(Error::WatchWorkerPanicked)
        ));
    }

    #[test]
    fn tracked_worker_records_a_panic() {
        let status = AtomicU8::new(WORKER_RUNNING);
        run_tracked_worker(&status, || panic!("injected worker panic"));
        assert_eq!(status.load(Ordering::Acquire), WORKER_PANICKED);
    }

    #[test]
    fn observation_driver_closes_the_invalidation_loop() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (index, _) =
            crate::scan::scan_into_index(dir.path(), &crate::ScanConfig::default()).expect("scan");
        let handle = crate::IndexHandle::new(index);
        fs::write(dir.path().join("raced.txt"), b"raced").expect("write");
        let observation = Observation::new(vec![Op::InvalidateSubtree {
            path: PathBuf::new(),
            reason: InvalidateReason::WatchSetupRace,
        }]);

        apply_observation(&handle, &observation, &crate::ScanConfig::default(), &mut |_| {})
            .expect("apply and reconcile");

        assert!(handle.kind(Path::new("raced.txt")).expect("query").is_some());
    }

    #[test]
    fn applying_driver_reverifies_a_queued_sample_after_reconciliation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("sample.txt");
        fs::write(&path, b"old").expect("write old sample");
        let (index, _) =
            crate::scan::scan_into_index(dir.path(), &crate::ScanConfig::default()).expect("scan");
        let old_attrs = *index.attrs(Path::new("sample.txt")).expect("sample attributes");
        let delayed = Observation::new(vec![Op::Upsert {
            path: PathBuf::from("sample.txt"),
            kind: crate::EntryKind::File,
            attrs: old_attrs,
        }]);
        let handle = crate::IndexHandle::new(index);

        fs::write(&path, b"new contents").expect("write current sample");
        crate::scan::reconcile_handle(&handle, &crate::ScanConfig::default(), &mut |_| {})
            .expect("reconcile newer sample");
        let current_size = fs::metadata(&path).expect("sample metadata").len();

        apply_observation(&handle, &delayed, &crate::ScanConfig::default(), &mut |_| {})
            .expect("apply delayed watch sample");

        assert_eq!(
            handle.attrs(Path::new("sample.txt")).expect("query").expect("sample remains").size,
            current_size
        );
    }

    #[test]
    fn blocked_verifier_holds_no_index_lock_and_commits_only_at_current_clock() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (index, _) =
            crate::scan::scan_into_index(dir.path(), &crate::ScanConfig::default()).expect("scan");
        let handle = crate::IndexHandle::new(index);
        let queued = Observation::new(vec![Op::Upsert {
            path: PathBuf::from("queued.txt"),
            kind: crate::EntryKind::File,
            attrs: crate::Attrs { size: 5, allocated: 5, ..crate::Attrs::default() },
        }]);
        let applying = handle.clone();
        let (entered_tx, entered_rx) = sync_channel(1);
        let (release_tx, release_rx) = sync_channel(1);
        let (done_tx, done_rx) = sync_channel(1);
        let apply_thread = std::thread::spawn(move || {
            let mut first = true;
            let mut verifier = |_: &Path, observation: &Observation| {
                if first {
                    first = false;
                    entered_tx.send(()).expect("signal blocked verifier");
                    release_rx.recv().expect("release blocked verifier");
                }
                Ok(observation.clone())
            };
            let result = apply_reverified_with(
                &applying,
                &queued,
                &crate::ScanConfig::default(),
                &mut verifier,
                &mut BTreeSet::new(),
            );
            done_tx.send(result).expect("report apply result");
        });

        entered_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("verifier must reach the injected block");
        let progressing = handle.clone();
        let (progress_tx, progress_rx) = sync_channel(1);
        let progress_thread = std::thread::spawn(move || {
            let total = progressing.total().expect("reader progresses");
            let write = progressing.apply(&Observation::new(vec![Op::Upsert {
                path: PathBuf::from("competitor.txt"),
                kind: crate::EntryKind::File,
                attrs: crate::Attrs { size: 3, allocated: 3, ..crate::Attrs::default() },
            }]));
            progress_tx.send((total, write)).expect("report progress");
        });
        let (_, competing_write) = progress_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("reader and writer must progress while verification is blocked");
        competing_write.expect("competing write");
        release_tx.send(()).expect("release verifier");

        let outcome = done_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("applying driver completes")
            .expect("applying driver succeeds");
        apply_thread.join().expect("apply thread");
        progress_thread.join().expect("progress thread");
        assert_eq!(outcome.inserted, 1);
        assert!(handle.kind(Path::new("competitor.txt")).expect("query").is_some());
        assert!(handle.kind(Path::new("queued.txt")).expect("query").is_some());
        assert_eq!(handle.clock().expect("clock"), crate::Clock(2));
    }

    #[test]
    fn exhausted_watch_contention_stays_unfresh_until_reconciliation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (index, _) =
            crate::scan::scan_into_index(dir.path(), &crate::ScanConfig::default()).expect("scan");
        let handle = crate::IndexHandle::new(index);
        let queued = Observation::new(vec![Op::Upsert {
            path: PathBuf::from("never-committed.txt"),
            kind: crate::EntryKind::File,
            attrs: crate::Attrs { size: 1, allocated: 1, ..crate::Attrs::default() },
        }]);
        let mut attempts = 0_usize;
        let mut verifier = |_: &Path, observation: &Observation| {
            attempts += 1;
            handle
                .apply(&Observation::new(vec![Op::Upsert {
                    path: PathBuf::from(format!("competitor-{attempts}.txt")),
                    kind: crate::EntryKind::File,
                    attrs: crate::Attrs {
                        size: attempts as u64,
                        allocated: attempts as u64,
                        ..crate::Attrs::default()
                    },
                }]))
                .expect("force a clock conflict");
            Ok(observation.clone())
        };

        let outcome = apply_reverified_with(
            &handle,
            &queued,
            &crate::ScanConfig::default(),
            &mut verifier,
            &mut BTreeSet::new(),
        )
        .expect("contention escalates");

        assert_eq!(attempts, MAX_OPTIMISTIC_APPLY_ATTEMPTS);
        assert_eq!(outcome.invalidated, 1);
        assert_eq!(handle.freshness().expect("freshness"), crate::Freshness::Stale);
        assert!(handle.kind(Path::new("never-committed.txt")).expect("query").is_none());
        let pending = handle.take_pending_invalidations().expect("pending invalidation");
        assert_eq!(pending, vec![(PathBuf::new(), InvalidateReason::WatchContention)]);
        handle.restore_pending_invalidations(pending).expect("restore invalidation");

        crate::scan::reconcile_pending_handle(&handle, &crate::ScanConfig::default(), &mut |_| {})
            .expect("reconcile contention");
        assert_eq!(handle.freshness().expect("freshness"), crate::Freshness::Fresh);
    }

    #[test]
    fn verifier_error_mutates_no_shared_state() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (index, _) =
            crate::scan::scan_into_index(dir.path(), &crate::ScanConfig::default()).expect("scan");
        let handle = crate::IndexHandle::new(index);
        let before_clock = handle.clock().expect("clock");
        let before_total = handle.total().expect("total");
        let mut verifier = |_: &Path, _: &Observation| {
            Err(Error::io(
                PathBuf::from("blocked"),
                std::io::Error::new(std::io::ErrorKind::PermissionDenied, "injected"),
            ))
        };

        let error = apply_reverified_with(
            &handle,
            &Observation::default(),
            &crate::ScanConfig::default(),
            &mut verifier,
            &mut BTreeSet::new(),
        )
        .expect_err("verification error");

        assert!(matches!(error, Error::Io { .. }));
        assert_eq!(handle.clock().expect("clock"), before_clock);
        assert_eq!(handle.total().expect("total"), before_total);
        assert_eq!(handle.freshness().expect("freshness"), crate::Freshness::Fresh);
        assert!(handle.take_pending_invalidations().expect("pending").is_empty());
    }

    #[test]
    fn stable_watch_arbitration_verifies_exactly_once() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (index, _) =
            crate::scan::scan_into_index(dir.path(), &crate::ScanConfig::default()).expect("scan");
        let handle = crate::IndexHandle::new(index);
        let mut calls = 0_u8;
        let mut verifier = |_: &Path, observation: &Observation| {
            calls += 1;
            Ok(observation.clone())
        };

        apply_reverified_with(
            &handle,
            &Observation::new(vec![Op::InvalidateSubtree {
                path: PathBuf::new(),
                reason: InvalidateReason::Requested,
            }]),
            &crate::ScanConfig::default(),
            &mut verifier,
            &mut BTreeSet::new(),
        )
        .expect("stable apply");

        assert_eq!(calls, 1);
    }

    #[test]
    fn disappearing_watch_root_escalates_instead_of_removing_the_index_root() {
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "root disappeared");

        assert!(matches!(
            op_for_stat_error(PathBuf::new(), &error),
            Op::InvalidateSubtree {
                path,
                reason: InvalidateReason::VerificationFailed,
            } if path.as_os_str().is_empty()
        ));
    }

    #[test]
    fn observation_driver_rejects_scope_mismatch_before_apply() {
        let dir = tempfile::tempdir().expect("tempdir");
        let shallow = crate::ScanConfig { max_depth: Some(1), ..crate::ScanConfig::default() };
        let (index, _) = crate::scan::scan_into_index(dir.path(), &shallow).expect("scan");
        let handle = crate::IndexHandle::new(index);
        let observation = Observation::new(vec![Op::Upsert {
            path: PathBuf::from("deep/nested.txt"),
            kind: crate::EntryKind::File,
            attrs: crate::Attrs { size: 5, allocated: 5, ..crate::Attrs::default() },
        }]);

        let error =
            apply_observation(&handle, &observation, &crate::ScanConfig::default(), &mut |_| {})
                .expect_err("mismatched scope must fail");

        assert!(matches!(error, Error::ScanScopeMismatch { .. }));
        assert!(handle.kind(Path::new("deep/nested.txt")).expect("query").is_none());
    }

    /// A depth-bounded watch runs, and the event below the bound does not enter the index.
    ///
    /// This used to refuse the whole apply, which got the same final state for a weaker
    /// reason: nothing entered the index because nothing was applied. The property worth
    /// having is that the *shallow* half of a batch lands and the deep half does not, so
    /// the assertion below is paired with one that something did.
    #[test]
    fn a_depth_bounded_watch_admits_what_is_inside_the_bound_and_nothing_below_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let shallow = crate::ScanConfig { max_depth: Some(1), ..crate::ScanConfig::default() };
        fs::create_dir(dir.path().join("deep")).expect("mkdir");
        let (index, _) = crate::scan::scan_into_index(dir.path(), &shallow).expect("scan");
        let handle = crate::IndexHandle::new(index);

        // Written after the scan, so neither is in the index yet, and written to disk
        // because the driver re-stats every path it is handed: a path that is not there
        // verifies as a removal, which would make both halves of this pass for one reason.
        fs::write(dir.path().join("deep/nested.txt"), b"hello").expect("write");
        fs::write(dir.path().join("shallow.txt"), b"hello").expect("write");
        let attrs = crate::Attrs { size: 5, allocated: 5, ..crate::Attrs::default() };
        let observation = Observation::new(vec![
            Op::Upsert { path: PathBuf::from("deep/nested.txt"), kind: EntryKind::File, attrs },
            Op::Upsert { path: PathBuf::from("shallow.txt"), kind: EntryKind::File, attrs },
        ]);

        apply_observation(&handle, &observation, &shallow, &mut |_| {})
            .expect("a bounded scope is watchable");

        assert!(
            handle.kind(Path::new("deep/nested.txt")).expect("query").is_none(),
            "an event below the depth bound is outside the scope the index was built under"
        );
        assert_eq!(
            handle.kind(Path::new("shallow.txt")).expect("query"),
            Some(EntryKind::File),
            "and one inside it is applied, or the assertion above is about a refused batch"
        );
    }

    /// A capped scan is watchable, and the cap survives the watch.
    ///
    /// This used to be a refusal, on the argument that a cap is not a property of the
    /// entry an event names. That is true and is why `within_scope` does not carry it --
    /// but it does not follow that nothing can: the *index* knows how many files it holds,
    /// so the cap is kept where the previous state of a path is already in hand.
    #[test]
    fn a_capped_index_refuses_a_watched_file_past_its_cap() {
        let dir = tempfile::tempdir().expect("tempdir");
        for index in 0..4 {
            fs::write(dir.path().join(format!("f{index}.txt")), b"seed").expect("seed");
        }
        let capped = crate::ScanConfig { max_files: Some(4), ..crate::ScanConfig::default() };
        let (index, _) = crate::scan::scan_into_index(dir.path(), &capped).expect("scan");
        let handle = crate::IndexHandle::new(index);
        assert_eq!(handle.total().expect("total").files, 4, "the walk filled the cap");

        fs::write(dir.path().join("late.txt"), b"one too many").expect("write");
        let observation = Observation::new(vec![Op::Upsert {
            path: PathBuf::from("late.txt"),
            kind: EntryKind::File,
            attrs: Attrs::default(),
        }]);
        apply_observation(&handle, &observation, &capped, &mut |_| {})
            .expect("a capped scope is watchable");

        assert_eq!(handle.total().expect("total").files, 4, "and the cap is still the cap");
        assert!(handle.kind(Path::new("late.txt")).expect("query").is_none());
        assert_eq!(
            handle.with_index(|index| index.coverage_at(Path::new(""))).expect("coverage"),
            crate::Status::Partial(crate::CoverageReason::Budget),
            "a refused row is a coverage fact, not a silent drop"
        );
    }

    /// A slot freed by a deletion is available to the next arrival.
    ///
    /// The consequence worth stating rather than discovering: the cap bounds the retained
    /// set, so which files a long-lived capped index holds depends on the order events
    /// arrived -- as which files a capped *walk* holds depends on the order it reached
    /// them. Coverage says the set is short either way.
    #[test]
    fn a_deletion_frees_a_slot_the_next_arrival_can_take() {
        let dir = tempfile::tempdir().expect("tempdir");
        for index in 0..4 {
            fs::write(dir.path().join(format!("f{index}.txt")), b"seed").expect("seed");
        }
        let capped = crate::ScanConfig { max_files: Some(4), ..crate::ScanConfig::default() };
        let (index, _) = crate::scan::scan_into_index(dir.path(), &capped).expect("scan");
        let handle = crate::IndexHandle::new(index);

        fs::remove_file(dir.path().join("f0.txt")).expect("unlink");
        fs::write(dir.path().join("late.txt"), b"takes the slot").expect("write");
        let observation = Observation::new(vec![
            Op::Remove { path: PathBuf::from("f0.txt") },
            Op::Upsert {
                path: PathBuf::from("late.txt"),
                kind: EntryKind::File,
                attrs: Attrs::default(),
            },
        ]);
        apply_observation(&handle, &observation, &capped, &mut |_| {})
            .expect("a capped scope is watchable");

        assert_eq!(handle.total().expect("total").files, 4, "still exactly the cap");
        assert_eq!(handle.kind(Path::new("late.txt")).expect("query"), Some(EntryKind::File));
    }

    /// A scope that does not match the index still fails before the queue is touched.
    ///
    /// The remaining refusal, and the one that has to keep this shape: applying an
    /// observation gathered under one scope to an index built under another would mix two
    /// inventories, and the intent has to survive so it can be applied under the right
    /// config rather than being lost to a configuration mistake.
    #[test]
    fn apply_next_rejects_a_mismatched_scope_without_consuming_an_observation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let capped = crate::ScanConfig { max_files: Some(4), ..crate::ScanConfig::default() };
        let (index, _) = crate::scan::scan_into_index(dir.path(), &capped).expect("scan");
        let handle = crate::IndexHandle::new(index);
        let (sender, watcher) =
            queued_test_watcher(dir.path().canonicalize().expect("canonical root"));
        sender.try_send(CoalescedIntent::default()).expect("queue intent");

        let error = watcher
            .apply_next(&handle, &crate::ScanConfig::default(), Duration::ZERO, &mut |_| {})
            .expect_err("a scope the index was not built under must fail before receive");

        assert!(matches!(error, Error::ScanScopeMismatch { .. }));
        assert!(watcher.next_observation(Duration::ZERO).expect("receive").is_some());
    }

    #[test]
    fn apply_next_rejects_a_watcher_for_another_root_without_consuming() {
        let indexed = tempfile::tempdir().expect("indexed root");
        let watched_root_dir = tempfile::tempdir().expect("watched root");
        let (index, _) =
            crate::scan::scan_into_index(indexed.path(), &crate::ScanConfig::default())
                .expect("scan indexed root");
        let handle = crate::IndexHandle::new(index);
        let (sender, watcher) = queued_test_watcher(
            watched_root_dir.path().canonicalize().expect("canonical watched root"),
        );
        sender.try_send(CoalescedIntent::default()).expect("queue intent");

        let error = watcher
            .apply_next(&handle, &crate::ScanConfig::default(), Duration::ZERO, &mut |_| {})
            .expect_err("mismatched root must fail");

        assert!(matches!(error, Error::WatchRootMismatch { .. }));
        assert!(watcher.next_observation(Duration::ZERO).expect("receive").is_some());
    }

    #[test]
    fn zero_settle_is_rejected_before_starting_a_busy_worker() {
        let config = WatchConfig { settle: Duration::ZERO, ..WatchConfig::default() };

        assert!(matches!(config.validate(), Err(Error::UnsupportedScanConfig(_))));

        let config = WatchConfig { intent_capacity: 0, ..WatchConfig::default() };
        assert!(matches!(config.validate(), Err(Error::UnsupportedScanConfig(_))));

        let config = WatchConfig {
            batch_path_capacity: MAX_BATCH_PATH_CAPACITY,
            intent_capacity: MAX_BUFFERED_INTENT_PATHS / MAX_BATCH_PATH_CAPACITY + 1,
            ..WatchConfig::default()
        };
        assert!(matches!(config.validate(), Err(Error::UnsupportedScanConfig(_))));
    }
}
