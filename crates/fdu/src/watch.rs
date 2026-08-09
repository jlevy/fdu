//! The OS-native watch layer: turning an unreliable event stream into trustworthy deltas.
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
//! scan layer resolves back into precise deltas.
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
use crate::types::{Delta, Error, InvalidateReason, Op, Result};

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

/// What a coalesced path still needs before it can become a delta.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pending {
    /// Stat it and decide. Covers creates, writes, removes, and every rename shape:
    /// letting the stat decide is what makes the same code correct on all backends.
    Verify,
    /// The producer already knows it cannot describe this precisely.
    Escalate(InvalidateReason),
}

/// A live watcher over one tree.
///
/// Dropping it stops the OS watch and shuts the worker thread down.
pub struct Watcher {
    /// `Option` only so [`Drop`] can release it before joining the worker.
    inner: Option<RecommendedWatcher>,
    deltas: Receiver<Delta>,
    worker: Option<JoinHandle<()>>,
}

impl Watcher {
    /// Start watching `root` recursively.
    pub fn new(root: &Path, config: WatchConfig) -> Result<Self> {
        let root = root.canonicalize().map_err(|e| Error::io(root, e))?;

        let (raw_tx, raw_rx) = channel::<notify::Result<notify::Event>>();
        let (delta_tx, delta_rx) = channel::<Delta>();

        let mut inner = notify::recommended_watcher(move |res| {
            // A send failure means the worker is gone, which happens during shutdown.
            let _ = raw_tx.send(res);
        })
        .map_err(|e| notify_error(&root, e))?;

        inner.watch(&root, RecursiveMode::Recursive).map_err(|e| notify_error(&root, e))?;

        let worker_root = root.clone();
        let worker = std::thread::Builder::new()
            .name("fdu-watch".into())
            .spawn(move || run_worker(&worker_root, config, &raw_rx, &delta_tx))
            .map_err(|e| Error::io(&root, e))?;

        Ok(Self { inner: Some(inner), deltas: delta_rx, worker: Some(worker) })
    }

    /// The stream of verified deltas.
    pub fn deltas(&self) -> &Receiver<Delta> {
        &self.deltas
    }

    /// Block for the next delta, up to `timeout`.
    pub fn next_delta(&self, timeout: Duration) -> Option<Delta> {
        self.deltas.recv_timeout(timeout).ok()
    }
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
    out: &Sender<Delta>,
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
        // the whole root when it names nothing.
        let target = event.paths.first().and_then(|p| relative_to(root, p)).unwrap_or_default();
        pending.insert(target, Pending::Escalate(InvalidateReason::WatchOverflow));
        return;
    }

    for path in &event.paths {
        let Some(rel) = relative_to(root, path) else {
            continue;
        };
        // An escalation already queued for this path outranks a plain verify: it
        // describes strictly less certainty, and downgrading it would lose information.
        if matches!(pending.get(&rel), Some(Pending::Escalate(_))) {
            continue;
        }
        match event.kind {
            EventKind::Access(_) => {} // Reads change nothing this engine records.
            _ => {
                pending.insert(rel, Pending::Verify);
            }
        }
    }
}

/// Turn the pending set into one delta: stat once per path, never once per event.
fn flush(
    root: &Path,
    config: WatchConfig,
    pending: &mut BTreeMap<PathBuf, Pending>,
    out: &Sender<Delta>,
) -> std::result::Result<(), ()> {
    let mut ops = Vec::with_capacity(pending.len());

    for (rel, state) in std::mem::take(pending) {
        match state {
            Pending::Escalate(reason) => ops.push(Op::InvalidateSubtree { path: rel, reason }),
            Pending::Verify => {
                let absolute = root.join(&rel);
                match std::fs::symlink_metadata(&absolute) {
                    Ok(meta) => {
                        let (kind, attrs) = scan::observe(&meta);
                        ops.push(Op::Upsert { path: rel.clone(), kind, attrs });
                        if kind.is_dir() && config.relist_new_dirs {
                            // The watch for this directory was installed after it was
                            // created, so anything already inside produced no event.
                            ops.push(Op::InvalidateSubtree {
                                path: rel,
                                reason: InvalidateReason::WatchSetupRace,
                            });
                        }
                    }
                    // Gone by the time we looked: that is the answer, not a failure.
                    Err(_) => ops.push(Op::Remove { path: rel }),
                }
            }
        }
    }

    if ops.is_empty() {
        return Ok(());
    }
    out.send(Delta::new(ops)).map_err(|_| ())
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
            if let Some(delta) = watcher.next_delta(Duration::from_millis(200)) {
                seen.extend(delta.ops);
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
}
