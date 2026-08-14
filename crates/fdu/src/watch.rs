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

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, TrySendError, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};

use crate::engine_contract::{AppliedDelta, Error, InvalidateReason, Observation, Op, Result};
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
#[derive(Clone, Copy, Debug)]
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
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            // The 50 ms step / 1.6 s ceiling pairing is watchfiles' batching loop, which
            // is the part of that stack worth keeping.
            settle: Duration::from_millis(50),
            max_hold: Duration::from_millis(1600),
            relist_new_dirs: true,
            event_capacity: 4096,
            batch_path_capacity: 4096,
            intent_capacity: 16,
        }
    }
}

impl WatchConfig {
    fn validate(self) -> Result<()> {
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
    inner: Option<RecommendedWatcher>,
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

        let mut inner = notify::recommended_watcher(move |res| {
            enqueue_raw(&raw_tx, &callback_overflowed, res);
        })
        .map_err(|e| notify_error(&root, e))?;

        inner.watch(&root, RecursiveMode::Recursive).map_err(|e| notify_error(&root, e))?;

        let worker_root = root.clone();
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
                        config,
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
            inner: Some(inner),
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
        Ok(Some(verify_intent(&self.root, self.config, &intent)))
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
    /// Restricted `max_depth` and `one_filesystem` scopes are rejected until the watch
    /// adapter can filter raw backend events against those boundaries.
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
        apply_intent(index, &self.root, self.config, &intent, scan_config, sink).map(Some)
    }
}

fn apply_intent(
    index: &IndexHandle,
    root: &Path,
    watch_config: WatchConfig,
    intent: &CoalescedIntent,
    scan_config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<WatchApplyReport> {
    let mut verifier = |_: &Path, _: &Observation| Ok(verify_intent(root, watch_config, intent));
    let apply = apply_reverified_with(index, &Observation::default(), scan_config, &mut verifier)?;
    if let Some(applied) = &apply.applied {
        sink(applied);
    }
    let reconciliation = scan::reconcile_pending_handle(index, scan_config, sink)?;
    Ok(WatchApplyReport { apply, reconciliation })
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
    let apply = apply_reverified(index, observation, scan_config)?;
    if let Some(applied) = &apply.applied {
        sink(applied);
    }
    let reconciliation = scan::reconcile_pending_handle(index, scan_config, sink)?;
    Ok(WatchApplyReport { apply, reconciliation })
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
) -> Result<ApplyOutcome> {
    apply_reverified_with(index, observation, scan_config, &mut reverify_observation)
}

fn apply_reverified_with(
    index: &IndexHandle,
    observation: &Observation,
    scan_config: &ScanConfig,
    verifier: &mut impl FnMut(&Path, &Observation) -> Result<Observation>,
) -> Result<ApplyOutcome> {
    let (root, scope, _) = index.watch_boundary()?;
    scan_config.validate_for_watch_scope(scope)?;
    for _ in 0..MAX_OPTIMISTIC_APPLY_ATTEMPTS {
        let clock = index.clock()?;
        let candidate = verifier(&root, observation)?;
        if let Some(outcome) = index.apply_if_clock(clock, &candidate)? {
            return Ok(outcome);
        }
    }

    index.invalidate_root(InvalidateReason::WatchContention)
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
    config: WatchConfig,
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
fn verify_intent(root: &Path, config: WatchConfig, intent: &CoalescedIntent) -> Observation {
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
    use notify::event::{CreateKind, Flag, MetadataKind, ModifyKind, RenameMode};
    use std::fs;

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
        let observation = verify_intent(Path::new("/unused"), WatchConfig::default(), &intent);
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
        let worker = std::thread::spawn(move || {
            run_tracked_worker(&tracked_status, || {
                run_worker(
                    &worker_root,
                    config,
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
        let observation = verify_intent(dir.path(), WatchConfig::default(), &intent);

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

        let outcome =
            apply_reverified_with(&handle, &queued, &crate::ScanConfig::default(), &mut verifier)
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

    #[test]
    fn observation_driver_rejects_restricted_scopes_until_events_are_filtered() {
        let dir = tempfile::tempdir().expect("tempdir");
        let shallow = crate::ScanConfig { max_depth: Some(1), ..crate::ScanConfig::default() };
        let (index, _) = crate::scan::scan_into_index(dir.path(), &shallow).expect("scan");
        let handle = crate::IndexHandle::new(index);
        let observation = Observation::new(vec![Op::Upsert {
            path: PathBuf::from("deep/nested.txt"),
            kind: crate::EntryKind::File,
            attrs: crate::Attrs { size: 5, allocated: 5, ..crate::Attrs::default() },
        }]);

        let error = apply_observation(&handle, &observation, &shallow, &mut |_| {})
            .expect_err("unfiltered bounded watch scope must fail");

        assert!(matches!(error, Error::UnsupportedScanConfig(_)));
        assert!(handle.kind(Path::new("deep/nested.txt")).expect("query").is_none());
    }

    #[test]
    fn apply_next_rejects_restricted_scope_without_consuming_an_observation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let shallow = crate::ScanConfig { max_depth: Some(1), ..crate::ScanConfig::default() };
        let (index, _) = crate::scan::scan_into_index(dir.path(), &shallow).expect("scan");
        let handle = crate::IndexHandle::new(index);
        let (sender, watcher) =
            queued_test_watcher(dir.path().canonicalize().expect("canonical root"));
        sender.try_send(CoalescedIntent::default()).expect("queue intent");

        let error = watcher
            .apply_next(&handle, &shallow, Duration::ZERO, &mut |_| {})
            .expect_err("restricted scope must fail before receive");

        assert!(matches!(error, Error::UnsupportedScanConfig(_)));
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
