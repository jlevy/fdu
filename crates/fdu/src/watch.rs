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
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};

use crate::scan;
use crate::types::{AppliedDelta, Error, InvalidateReason, Observation, Op, Result};
use crate::{ApplyOutcome, IndexHandle, ScanConfig};

/// Optimistic re-verification retries before the applying driver guarantees progress
/// under the writer lock.
const MAX_OPTIMISTIC_APPLY_ATTEMPTS: usize = 3;

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
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            // The 50 ms step / 1.6 s ceiling pairing is watchfiles' batching loop, which
            // is the part of that stack worth keeping.
            settle: Duration::from_millis(50),
            max_hold: Duration::from_millis(1600),
            relist_new_dirs: true,
        }
    }
}

impl WatchConfig {
    fn validate(self) -> Result<()> {
        if self.settle.is_zero() {
            return Err(Error::UnsupportedScanConfig(
                "watch settle duration must be greater than zero",
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

/// A live watcher over one tree.
///
/// Dropping it stops the OS watch and shuts the worker thread down.
pub struct Watcher {
    root: PathBuf,
    /// `Option` only so [`Drop`] can release it before joining the worker.
    inner: Option<RecommendedWatcher>,
    observations: Receiver<Observation>,
    worker: Option<JoinHandle<()>>,
}

/// Effects of one watch observation and any reconciliation it requested.
#[derive(Debug)]
pub struct WatchApplyReport {
    pub apply: ApplyOutcome,
    pub reconciliation: scan::ReconcileReport,
}

impl Watcher {
    /// Start watching `root` recursively.
    pub fn new(root: &Path, config: WatchConfig) -> Result<Self> {
        config.validate()?;
        let root = root.canonicalize().map_err(|e| Error::io(root, e))?;

        let (raw_tx, raw_rx) = channel::<notify::Result<notify::Event>>();
        let (observation_tx, observation_rx) = channel::<Observation>();

        let mut inner = notify::recommended_watcher(move |res| {
            // A send failure means the worker is gone, which happens during shutdown.
            let _ = raw_tx.send(res);
        })
        .map_err(|e| notify_error(&root, e))?;

        inner.watch(&root, RecursiveMode::Recursive).map_err(|e| notify_error(&root, e))?;

        let worker_root = root.clone();
        let worker = std::thread::Builder::new()
            .name("fdu-watch".into())
            .spawn(move || run_worker(&worker_root, config, &raw_rx, &observation_tx))
            .map_err(|e| Error::io(&root, e))?;

        Ok(Self { root, inner: Some(inner), observations: observation_rx, worker: Some(worker) })
    }

    /// The stream of verified observations awaiting index arbitration.
    pub fn observations(&self) -> &Receiver<Observation> {
        &self.observations
    }

    /// Block for the next observation, up to `timeout`.
    pub fn next_observation(&self, timeout: Duration) -> Option<Observation> {
        self.observations.recv_timeout(timeout).ok()
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
        let current = index.read()?;
        scan_config.validate_for_watch_scope(current.scope())?;
        if current.root_path() != self.root {
            return Err(Error::WatchRootMismatch {
                watched: self.root.clone(),
                indexed: current.root_path().to_path_buf(),
            });
        }
        drop(current);
        match self.observations.recv_timeout(timeout) {
            Ok(observation) => apply_observation(index, &observation, scan_config, sink).map(Some),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => Err(Error::WatchStopped),
        }
    }
}

/// Apply a verified observation and reconcile all subtrees it invalidates.
///
/// Restricted `max_depth` and `one_filesystem` scopes are rejected until event-scope
/// filtering is implemented; accepting those events would add paths a cold scan excludes.
pub fn apply_observation(
    index: &IndexHandle,
    observation: &Observation,
    scan_config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<WatchApplyReport> {
    scan_config.validate_for_watch_scope(index.read()?.scope())?;
    let apply = apply_reverified(index, observation, scan_config)?;
    if let Some(applied) = &apply.applied {
        sink(applied);
    }
    let reconciliation = scan::reconcile_pending_handle(index, scan_config, sink)?;
    Ok(WatchApplyReport { apply, reconciliation })
}

/// Re-stat a queued watch sample against a clock-stable index boundary before applying
/// it. The common path stays optimistic; sustained competing writes fall back to
/// verification under the writer lock so an old queue entry can never win by arriving
/// late.
fn apply_reverified(
    index: &IndexHandle,
    observation: &Observation,
    scan_config: &ScanConfig,
) -> Result<ApplyOutcome> {
    for _ in 0..MAX_OPTIMISTIC_APPLY_ATTEMPTS {
        let current = index.read()?;
        scan_config.validate_for_watch_scope(current.scope())?;
        let root = current.root_path().to_path_buf();
        let clock = current.clock();
        drop(current);

        let verified = reverify_observation(&root, observation)?;
        let mut current = index.write()?;
        if current.clock() == clock {
            return Ok(current.apply(&verified));
        }
    }

    let mut current = index.write()?;
    scan_config.validate_for_watch_scope(current.scope())?;
    let root = current.root_path().to_path_buf();
    let verified = reverify_observation(&root, observation)?;
    Ok(current.apply(&verified))
}

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
        // Order matters and is the whole reason `inner` is an Option: dropping the notify
        // watcher releases the only sender on the raw event channel, and that
        // disconnection is what ends the worker's loop. Joining first would block
        // forever waiting for a thread whose exit condition had not been created yet.
        self.inner.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn notify_error(path: &Path, err: notify::Error) -> Error {
    Error::io(path, std::io::Error::other(err))
}

fn run_worker(
    root: &Path,
    config: WatchConfig,
    raw: &Receiver<notify::Result<notify::Event>>,
    out: &Sender<Observation>,
) {
    let mut pending: BTreeMap<PathBuf, Pending> = BTreeMap::new();
    let mut batch_started: Option<Instant> = None;

    loop {
        match raw.recv_timeout(config.settle) {
            Ok(Ok(event)) => {
                record(root, &event, &mut pending);
                batch_started.get_or_insert_with(Instant::now);
            }
            Ok(Err(err)) => {
                // notify reports a watch failure. It cannot say what was missed, so the
                // only honest response is to escalate the whole tree.
                let _ = err;
                pending.insert(PathBuf::new(), Pending::Escalate(InvalidateReason::WatchOverflow));
                batch_started.get_or_insert_with(Instant::now);
            }
            Err(RecvTimeoutError::Timeout) => {
                // Quiet for a full step: the batch has settled.
                if !pending.is_empty() && flush(root, config, &mut pending, out).is_err() {
                    return;
                }
                batch_started = None;
                continue;
            }
            Err(RecvTimeoutError::Disconnected) => {
                if !pending.is_empty() {
                    let _ = flush(root, config, &mut pending, out);
                }
                return;
            }
        }

        // A tree under continuous churn never goes quiet, so cap how long a batch waits.
        if batch_started.is_some_and(|start| start.elapsed() >= config.max_hold) {
            if flush(root, config, &mut pending, out).is_err() {
                return;
            }
            batch_started = None;
        }
    }
}

/// Fold one event into the pending set.
fn record(root: &Path, event: &notify::Event, pending: &mut BTreeMap<PathBuf, Pending>) {
    if event.need_rescan() {
        // The kernel dropped events. Escalate the narrowest subtree the event names, or
        // the whole root when it names zero/multiple paths or crosses the watch boundary.
        let target = if event.paths.len() == 1 {
            relative_to(root, &event.paths[0]).unwrap_or_default()
        } else {
            PathBuf::new()
        };
        pending.insert(target, Pending::Escalate(InvalidateReason::WatchOverflow));
        return;
    }

    let rename_mode = match event.kind {
        EventKind::Modify(notify::event::ModifyKind::Name(mode)) => Some(mode),
        _ => None,
    };
    let relative_paths: Vec<(usize, PathBuf)> = event
        .paths
        .iter()
        .enumerate()
        .filter_map(|(position, path)| relative_to(root, path).map(|relative| (position, relative)))
        .collect();
    let paired_rename = matches!(rename_mode, Some(notify::event::RenameMode::Both))
        && event.paths.len() == 2
        && relative_paths.len() == 2;
    if rename_mode.is_some() && !paired_rename {
        // A one-sided rename gives no safe bound on where its counterpart lives. A full
        // reconciliation is more expensive than guessing a parent, but it cannot leave
        // the old name behind or miss a moved-in subtree.
        pending.insert(PathBuf::new(), Pending::Escalate(InvalidateReason::UnpairedRename));
    }

    for (position, rel) in relative_paths {
        if matches!(event.kind, EventKind::Access(_)) {
            continue; // Reads change nothing this engine records.
        }
        let relist_if_dir = matches!(event.kind, EventKind::Create(_))
            || matches!(rename_mode, Some(notify::event::RenameMode::To))
            || (paired_rename && position == 1);
        match pending.get_mut(&rel) {
            // An escalation outranks verification: it describes strictly less
            // certainty, and downgrading it would lose information.
            Some(Pending::Escalate(_)) => {}
            Some(Pending::Verify { relist_if_dir: queued }) => {
                *queued |= relist_if_dir;
            }
            None => {
                pending.insert(rel, Pending::Verify { relist_if_dir });
            }
        }
    }
}

/// Turn the pending set into one observation: stat once per path, never once per event.
fn flush(
    root: &Path,
    config: WatchConfig,
    pending: &mut BTreeMap<PathBuf, Pending>,
    out: &Sender<Observation>,
) -> std::result::Result<(), ()> {
    let mut ops = Vec::with_capacity(pending.len());

    for (rel, state) in std::mem::take(pending) {
        match state {
            Pending::Escalate(reason) => ops.push(Op::InvalidateSubtree { path: rel, reason }),
            Pending::Verify { relist_if_dir } => {
                let absolute = root.join(&rel);
                match std::fs::symlink_metadata(&absolute) {
                    Ok(meta) => {
                        let (kind, attrs) = scan::observe(&meta);
                        ops.push(Op::Upsert { path: rel.clone(), kind, attrs });
                        if kind.is_dir() && relist_if_dir && config.relist_new_dirs {
                            // The watch for this directory was installed after it was
                            // created, so anything already inside produced no event.
                            ops.push(Op::InvalidateSubtree {
                                path: rel,
                                reason: InvalidateReason::WatchSetupRace,
                            });
                        }
                    }
                    Err(error) => ops.push(op_for_stat_error(rel, &error)),
                }
            }
        }
    }

    if ops.is_empty() {
        return Ok(());
    }
    out.send(Observation::new(ops)).map_err(|_| ())
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
            if let Some(observation) = watcher.next_observation(Duration::from_millis(200)) {
                seen.extend(observation.ops.into_iter().map(|observed| observed.op));
                if want(&seen) {
                    return seen;
                }
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
        );
        record(
            root,
            &notify::Event::new(EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)))
                .add_path(path),
            &mut pending,
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
        );

        assert!(!matches!(
            pending.get(Path::new("")),
            Some(Pending::Escalate(InvalidateReason::UnpairedRename))
        ));
        assert_eq!(pending.get(Path::new("old")), Some(&Pending::Verify { relist_if_dir: false }));
        assert_eq!(pending.get(Path::new("new")), Some(&Pending::Verify { relist_if_dir: true }));
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

        assert!(handle.read().expect("read").lookup(Path::new("raced.txt")).is_some());
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
            handle
                .read()
                .expect("read index")
                .attrs(Path::new("sample.txt"))
                .expect("sample remains")
                .size,
            current_size
        );
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
        assert!(handle.read().expect("read").lookup(Path::new("deep/nested.txt")).is_none());
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
        assert!(handle.read().expect("read").lookup(Path::new("deep/nested.txt")).is_none());
    }

    #[test]
    fn apply_next_rejects_restricted_scope_without_consuming_an_observation() {
        let dir = tempfile::tempdir().expect("tempdir");
        let shallow = crate::ScanConfig { max_depth: Some(1), ..crate::ScanConfig::default() };
        let (index, _) = crate::scan::scan_into_index(dir.path(), &shallow).expect("scan");
        let handle = crate::IndexHandle::new(index);
        let (sender, observations) = channel();
        sender.send(Observation::default()).expect("queue observation");
        let watcher = Watcher {
            root: dir.path().canonicalize().expect("canonical root"),
            inner: None,
            observations,
            worker: None,
        };

        let error = watcher
            .apply_next(&handle, &shallow, Duration::ZERO, &mut |_| {})
            .expect_err("restricted scope must fail before receive");

        assert!(matches!(error, Error::UnsupportedScanConfig(_)));
        assert!(watcher.next_observation(Duration::ZERO).is_some());
    }

    #[test]
    fn apply_next_rejects_a_watcher_for_another_root_without_consuming() {
        let indexed = tempfile::tempdir().expect("indexed root");
        let watched_root_dir = tempfile::tempdir().expect("watched root");
        let (index, _) =
            crate::scan::scan_into_index(indexed.path(), &crate::ScanConfig::default())
                .expect("scan indexed root");
        let handle = crate::IndexHandle::new(index);
        let (sender, observations) = channel();
        sender.send(Observation::default()).expect("queue observation");
        let watcher = Watcher {
            root: watched_root_dir.path().canonicalize().expect("canonical watched root"),
            inner: None,
            observations,
            worker: None,
        };

        let error = watcher
            .apply_next(&handle, &crate::ScanConfig::default(), Duration::ZERO, &mut |_| {})
            .expect_err("mismatched root must fail");

        assert!(matches!(error, Error::WatchRootMismatch { .. }));
        assert!(watcher.next_observation(Duration::ZERO).is_some());
    }

    #[test]
    fn zero_settle_is_rejected_before_starting_a_busy_worker() {
        let config = WatchConfig { settle: Duration::ZERO, ..WatchConfig::default() };

        assert!(matches!(config.validate(), Err(Error::UnsupportedScanConfig(_))));
    }
}
