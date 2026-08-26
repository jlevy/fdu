//! Ownership and joined shutdown for one long-lived opened root.
//!
//! [`OpenedIndex`] is the public behavior surface. Its private shared state contains
//! data and synchronization only; it is deliberately not a second API-shaped service.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

use crate::{Error, Index, IndexHandle, Result, ScanConfig};

/// First ordinal reserved for a minted session; zero never identifies a live owner.
const FIRST_SESSION_ORDINAL: u64 = 1;
/// FNV-1a offset used to mix the process and open instance into an opaque identity.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a prime used for the opened-root identity mix.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
/// Nonzero fallback for the reserved zero identity.
const FIRST_SESSION_ID: u64 = 1;
#[cfg(test)]
/// Deadline for a missing deterministic test barrier to fail instead of hanging.
const TEST_GATE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Identity of one live opened-root lifetime.
///
/// The value is process-local, opaque, and never persisted. It prevents a future cursor
/// or continuation from being accepted by another open whose sequence also began at
/// zero; it is not a credential.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct SessionId(u64);

impl SessionId {
    fn mint() -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(FIRST_SESSION_ORDINAL);

        let ordinal = NEXT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| current.checked_add(1))
            .map_err(|_| Error::OpenedIdentityExhausted)?;
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| {
                u64::try_from(elapsed.as_nanos() & u128::from(u64::MAX)).unwrap_or(0)
            });
        let process = u64::from(std::process::id());
        let mut hash = FNV_OFFSET_BASIS;
        for byte in nanos
            .to_le_bytes()
            .iter()
            .chain(process.to_le_bytes().iter())
            .chain(ordinal.to_le_bytes().iter())
        {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        Ok(Self(hash.max(FIRST_SESSION_ID)))
    }
}

/// Configuration for a long-lived [`OpenedIndex`].
///
/// Scope and execution settings are flat here because each is one independent decision;
/// display depth is absent by design and belongs to a read request. Progressive
/// discovery budgets, observation, and bounded history are added with their respective
/// capabilities.
#[derive(Clone, Debug)]
pub struct OpenOptions {
    /// Ops per committed discovery batch.
    pub batch_size: usize,
    /// Follow directory symlinks when the configured platform semantics support it.
    pub follow_symlinks: bool,
    /// Stay on the filesystem containing the opened root.
    pub one_filesystem: bool,
    /// Hidden-component admission, or `None` to retain every component.
    pub hidden: Option<Arc<crate::HiddenPolicy>>,
    /// Exclude objects other than files, directories, and symlinks.
    pub exclude_special: bool,
    /// Directory-reading worker threads, or a bounded engine-selected default.
    pub threads: Option<usize>,
    /// Preferred discovery order.
    pub order: crate::ScanOrder,
    /// File-type rules, or `None` for the rules compiled into fdu.
    pub types: Option<Arc<crate::classify::TypeRegistry>>,
}

impl Default for OpenOptions {
    fn default() -> Self {
        let scan = ScanConfig::default();
        Self {
            batch_size: scan.batch_size,
            follow_symlinks: scan.follow_symlinks,
            one_filesystem: scan.one_filesystem,
            hidden: scan.hidden,
            exclude_special: scan.exclude_special,
            threads: scan.threads,
            order: scan.order,
            types: scan.types,
        }
    }
}

impl OpenOptions {
    fn into_scan_config(self) -> ScanConfig {
        ScanConfig {
            max_depth: None,
            batch_size: self.batch_size,
            follow_symlinks: self.follow_symlinks,
            one_filesystem: self.one_filesystem,
            hidden: self.hidden,
            exclude_special: self.exclude_special,
            threads: self.threads,
            order: self.order,
            types: self.types,
        }
    }
}

/// A long-lived, synchronously controlled filesystem index.
///
/// Clones are cheap references to one authority. Calling [`Self::close`] through any
/// clone cancels and joins every worker owned by that authority, and every concurrent
/// caller receives the same stored terminal outcome.
#[derive(Clone)]
pub struct OpenedIndex {
    state: Arc<OpenedState>,
}

impl std::fmt::Debug for OpenedIndex {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpenedIndex")
            .field("session", &self.state.session)
            .field("root", &self.state.root)
            .finish_non_exhaustive()
    }
}

impl OpenedIndex {
    /// Open one live root without changing the existing blocking [`crate::open`] API.
    ///
    /// This constructor validates and binds the root and semantic configuration. Cold
    /// progressive discovery is attached to this owner by the next implementation
    /// checkpoint; no cache image or second mutable index is created here.
    pub fn open(root: &Path, options: OpenOptions) -> Result<Self> {
        #[cfg(test)]
        let opened = Self::open_inner(root, options, Arc::default());
        #[cfg(not(test))]
        let opened = Self::open_inner(root, options);
        opened
    }

    #[cfg(not(test))]
    fn open_inner(root: &Path, options: OpenOptions) -> Result<Self> {
        Self::build(root, options)
    }

    #[cfg(test)]
    fn open_inner(root: &Path, options: OpenOptions, controls: Arc<TestControls>) -> Result<Self> {
        Self::build(root, options, controls)
    }

    #[cfg(not(test))]
    fn build(root: &Path, options: OpenOptions) -> Result<Self> {
        let state = OpenedState::new(root, options)?;
        Ok(Self { state: Arc::new(state) })
    }

    #[cfg(test)]
    fn build(root: &Path, options: OpenOptions, controls: Arc<TestControls>) -> Result<Self> {
        let state = OpenedState::new(root, options, controls)?;
        Ok(Self { state: Arc::new(state) })
    }

    /// Cancel and join all work owned by this opened root.
    ///
    /// The first caller performs shutdown. Concurrent and repeated callers wait for or
    /// replay its stored terminal outcome; success is never reported while a worker is
    /// still live.
    pub fn close(&self) -> Result<()> {
        self.state.shutdown()
    }

    #[allow(dead_code)]
    fn ensure_open(&self) -> Result<()> {
        self.state.ensure_open()
    }

    /// Register one worker with the shared owner before it can race with close.
    ///
    /// The worker receives only cancellation, not a strong owner reference. Discovery
    /// and observation use this boundary as their implementations land; the lifecycle
    /// checkpoint exercises it directly through its deterministic tests.
    #[allow(dead_code)]
    fn spawn_worker<F>(&self, name: &'static str, run: F) -> Result<()>
    where
        F: FnOnce(Arc<Cancellation>) -> Result<()> + Send + 'static,
    {
        let locked = self.state.lock_lifecycle();
        if locked.poisoned {
            return Err(Error::OpenedLifecyclePoisoned);
        }
        let mut lifecycle = locked.guard;
        if lifecycle.phase != OwnerPhase::Open {
            return Err(Error::OpenedIndexClosed);
        }

        let cancellation = Arc::clone(&self.state.cancellation);
        #[cfg(test)]
        let controls = Arc::clone(&self.state.test_controls);
        let worker = thread::Builder::new()
            .name(format!("fdu-{name}"))
            .spawn(move || {
                #[cfg(not(test))]
                {
                    run(cancellation)
                }
                #[cfg(test)]
                {
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        run(cancellation)
                    }));
                    controls.reach(TestPoint::BeforeWorkerExit);
                    match outcome {
                        Ok(result) => result,
                        Err(payload) => std::panic::resume_unwind(payload),
                    }
                }
            })
            .map_err(|source| Error::OpenedWorkerSpawn { worker: name, source })?;
        lifecycle.workers.push(Worker { name, handle: worker });
        Ok(())
    }

    #[cfg(test)]
    fn open_for_test(
        root: &Path,
        options: OpenOptions,
        controls: Arc<TestControls>,
    ) -> Result<Self> {
        Self::open_inner(root, options, controls)
    }
}

struct OpenedState {
    session: SessionId,
    root: std::path::PathBuf,
    index: IndexHandle,
    /// Retained for discovery and later verified producers; semantic identity is also
    /// fixed in `index`, so this cannot reinterpret already-retained facts.
    #[allow(dead_code)]
    scan: ScanConfig,
    cancellation: Arc<Cancellation>,
    lifecycle: Mutex<Lifecycle>,
    lifecycle_changed: Condvar,
    #[cfg(test)]
    test_controls: Arc<TestControls>,
}

impl OpenedState {
    #[cfg(not(test))]
    fn new(root: &Path, options: OpenOptions) -> Result<Self> {
        Self::build(root, options)
    }

    #[cfg(test)]
    fn new(root: &Path, options: OpenOptions, controls: Arc<TestControls>) -> Result<Self> {
        Self::build(root, options, controls)
    }

    #[cfg(not(test))]
    fn build(root: &Path, options: OpenOptions) -> Result<Self> {
        let (root, index, scan) = bind_root(root, options)?;
        Ok(Self {
            session: SessionId::mint()?,
            root,
            index,
            scan,
            cancellation: Arc::new(Cancellation::default()),
            lifecycle: Mutex::new(Lifecycle::default()),
            lifecycle_changed: Condvar::new(),
        })
    }

    #[cfg(test)]
    fn build(root: &Path, options: OpenOptions, controls: Arc<TestControls>) -> Result<Self> {
        let (root, index, scan) = bind_root(root, options)?;
        Ok(Self {
            session: SessionId::mint()?,
            root,
            index,
            scan,
            cancellation: Arc::new(Cancellation::default()),
            lifecycle: Mutex::new(Lifecycle::default()),
            lifecycle_changed: Condvar::new(),
            test_controls: controls,
        })
    }

    #[allow(dead_code)]
    fn ensure_open(&self) -> Result<()> {
        let locked = self.lock_lifecycle();
        if locked.poisoned {
            return Err(Error::OpenedLifecyclePoisoned);
        }
        if locked.guard.phase != OwnerPhase::Open {
            return Err(Error::OpenedIndexClosed);
        }
        Ok(())
    }

    fn lock_lifecycle(&self) -> LockedLifecycle<'_> {
        match self.lifecycle.lock() {
            Ok(guard) => LockedLifecycle { guard, poisoned: false },
            Err(poisoned) => LockedLifecycle { guard: poisoned.into_inner(), poisoned: true },
        }
    }

    fn shutdown(&self) -> Result<()> {
        let mut saw_poison = false;
        let workers = loop {
            let locked = self.lock_lifecycle();
            saw_poison |= locked.poisoned;
            let mut lifecycle = locked.guard;
            match lifecycle.phase {
                OwnerPhase::Open => {
                    lifecycle.phase = OwnerPhase::Closing;
                    let workers = std::mem::take(&mut lifecycle.workers);
                    drop(lifecycle);
                    self.cancellation.cancel();
                    self.lifecycle_changed.notify_all();
                    break workers;
                }
                OwnerPhase::Closing => {
                    #[cfg(test)]
                    self.test_controls.reach(TestPoint::BeforeCloseWait);
                    let waited = self.lifecycle_changed.wait(lifecycle);
                    match waited {
                        Ok(_) => {}
                        Err(poisoned) => {
                            saw_poison = true;
                            drop(poisoned.into_inner());
                        }
                    }
                }
                OwnerPhase::Closed => {
                    return lifecycle
                        .terminal
                        .as_ref()
                        .expect("closed lifecycle stores one outcome")
                        .to_result();
                }
            }
        };

        let worker_outcome = join_workers(workers);
        let index_poisoned = self.index.clock().is_err();
        let mut outcome = if saw_poison {
            CloseOutcome::LifecyclePoisoned
        } else if let Some(outcome) = worker_outcome {
            outcome
        } else if index_poisoned {
            CloseOutcome::IndexPoisoned
        } else {
            CloseOutcome::Success
        };

        let locked = self.lock_lifecycle();
        if locked.poisoned {
            outcome = CloseOutcome::LifecyclePoisoned;
        }
        let mut lifecycle = locked.guard;
        lifecycle.phase = OwnerPhase::Closed;
        lifecycle.terminal = Some(outcome.clone());
        self.lifecycle_changed.notify_all();
        outcome.to_result()
    }
}

impl Drop for OpenedState {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn bind_root(
    root: &Path,
    options: OpenOptions,
) -> Result<(std::path::PathBuf, IndexHandle, ScanConfig)> {
    let scan = options.into_scan_config();
    scan.validate()?;
    let root = root.canonicalize().map_err(|source| Error::io(root, source))?;
    let metadata = std::fs::symlink_metadata(&root).map_err(|source| Error::io(&root, source))?;
    if !metadata.is_dir() {
        return Err(Error::io(
            &root,
            std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                "opened-index root is not a directory",
            ),
        ));
    }

    let scope = scan.scope();
    let types = scan.types_shared();
    let index = IndexHandle::new(Index::new_with_scope_and_types(&root, scope, types));
    Ok((root, index, scan))
}

struct LockedLifecycle<'a> {
    guard: MutexGuard<'a, Lifecycle>,
    poisoned: bool,
}

#[derive(Default)]
struct Lifecycle {
    phase: OwnerPhase,
    workers: Vec<Worker>,
    terminal: Option<CloseOutcome>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum OwnerPhase {
    #[default]
    Open,
    Closing,
    Closed,
}

struct Worker {
    name: &'static str,
    handle: JoinHandle<Result<()>>,
}

#[derive(Clone)]
enum CloseOutcome {
    Success,
    LifecyclePoisoned,
    IndexPoisoned,
    WorkerPanicked { worker: &'static str },
    WorkerFailed { worker: &'static str, source: Arc<Error> },
}

impl CloseOutcome {
    fn to_result(&self) -> Result<()> {
        match self {
            Self::Success => Ok(()),
            Self::LifecyclePoisoned => Err(Error::OpenedLifecyclePoisoned),
            Self::IndexPoisoned => Err(Error::IndexLockPoisoned),
            Self::WorkerPanicked { worker } => Err(Error::OpenedWorkerPanicked { worker }),
            Self::WorkerFailed { worker, source } => {
                Err(Error::OpenedWorkerFailed { worker, source: Arc::clone(source) })
            }
        }
    }
}

fn join_workers(workers: Vec<Worker>) -> Option<CloseOutcome> {
    let mut first_failure = None;
    for worker in workers {
        let outcome = match worker.handle.join() {
            Ok(Ok(())) => None,
            Ok(Err(error)) => {
                Some(CloseOutcome::WorkerFailed { worker: worker.name, source: Arc::new(error) })
            }
            Err(_) => Some(CloseOutcome::WorkerPanicked { worker: worker.name }),
        };
        if first_failure.is_none() {
            first_failure = outcome;
        }
    }
    first_failure
}

#[derive(Default)]
struct Cancellation {
    cancelled: AtomicBool,
    wait_lock: Mutex<()>,
    changed: Condvar,
}

impl Cancellation {
    fn cancel(&self) {
        let guard = self.wait_lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        self.cancelled.store(true, Ordering::Release);
        self.changed.notify_all();
        drop(guard);
    }

    #[allow(dead_code)]
    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    #[allow(dead_code)]
    fn wait_cancelled(&self) {
        let mut guard = self.wait_lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        while !self.cancelled.load(Ordering::Acquire) {
            guard = self.changed.wait(guard).unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy)]
enum TestPoint {
    BeforeWorkerExit,
    BeforeCloseWait,
}

#[cfg(test)]
#[derive(Default)]
struct TestControls {
    before_worker_exit: TestGate,
    before_close_wait: TestGate,
}

#[cfg(test)]
impl TestControls {
    fn gate(&self, point: TestPoint) -> &TestGate {
        match point {
            TestPoint::BeforeWorkerExit => &self.before_worker_exit,
            TestPoint::BeforeCloseWait => &self.before_close_wait,
        }
    }

    fn reach(&self, point: TestPoint) {
        self.gate(point).reach();
    }
}

#[cfg(test)]
#[derive(Default)]
struct TestGate {
    state: Mutex<TestGateState>,
    changed: Condvar,
}

#[cfg(test)]
#[derive(Default)]
struct TestGateState {
    armed: bool,
    reached: bool,
    released: bool,
}

#[cfg(test)]
impl TestGate {
    fn arm(&self) {
        let mut state = self.state.lock().expect("test gate lock");
        *state = TestGateState { armed: true, reached: false, released: false };
    }

    fn reach(&self) {
        let mut state = self.state.lock().expect("test gate lock");
        if !state.armed {
            return;
        }
        state.reached = true;
        self.changed.notify_all();
        while !state.released {
            state = self.changed.wait(state).expect("test gate wait");
        }
        state.armed = false;
    }

    fn wait_reached(&self) {
        let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
        let mut state = self.state.lock().expect("test gate lock");
        while !state.reached {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            assert!(!remaining.is_zero(), "test gate was not reached");
            let (next, timed_out) =
                self.changed.wait_timeout(state, remaining).expect("test gate wait");
            state = next;
            assert!(!timed_out.timed_out() || state.reached, "test gate was not reached");
        }
    }

    fn release(&self) {
        let mut state = self.state.lock().expect("test gate lock");
        state.released = true;
        self.changed.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn opened(controls: Arc<TestControls>) -> (tempfile::TempDir, OpenedIndex) {
        let root = tempfile::tempdir().expect("temp root");
        let opened = OpenedIndex::open_for_test(root.path(), OpenOptions::default(), controls)
            .expect("open live root");
        (root, opened)
    }

    #[test]
    fn associated_and_free_open_contracts_coexist() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::write(root.path().join("file.txt"), b"one").expect("fixture");

        let opened = OpenedIndex::open(root.path(), OpenOptions::default()).expect("opened root");
        let (detached, _) = crate::open(
            root.path(),
            &crate::OpenConfig {
                cache_path: None,
                policy: crate::CachePolicy::Off,
                ..crate::OpenConfig::default()
            },
        )
        .expect("blocking open");

        assert_eq!(detached.total().files, 1);
        opened.close().expect("close");
    }

    #[test]
    fn one_clone_closes_the_shared_authority_and_close_is_idempotent() {
        let (_root, opened) = opened(Arc::default());
        let clone = opened.clone();
        assert_eq!(opened.state.session, clone.state.session);
        assert!(Arc::ptr_eq(&opened.state, &clone.state));

        clone.close().expect("first close");
        assert!(matches!(opened.ensure_open(), Err(Error::OpenedIndexClosed)));
        opened.close().expect("repeated close");
    }

    #[test]
    fn shutdown_refuses_new_owned_work() {
        let (_root, opened) = opened(Arc::default());
        opened.close().expect("close");
        let ran = Arc::new(AtomicBool::new(false));
        let worker_ran = Arc::clone(&ran);

        assert!(matches!(
            opened.spawn_worker("late", move |_cancellation| {
                worker_ran.store(true, Ordering::SeqCst);
                Ok(())
            }),
            Err(Error::OpenedIndexClosed)
        ));
        assert!(!ran.load(Ordering::SeqCst));
    }

    #[test]
    fn concurrent_close_waits_for_one_stored_worker_failure() {
        let controls = Arc::new(TestControls::default());
        controls.gate(TestPoint::BeforeWorkerExit).arm();
        controls.gate(TestPoint::BeforeCloseWait).arm();
        let (_root, opened) = opened(Arc::clone(&controls));
        opened
            .spawn_worker("failure", |cancellation| {
                cancellation.wait_cancelled();
                Err(Error::CommitRejected("injected opened worker failure"))
            })
            .expect("spawn worker");

        let first = opened.clone();
        let first_close = thread::spawn(move || first.close());
        controls.gate(TestPoint::BeforeWorkerExit).wait_reached();
        let second = opened.clone();
        let second_close = thread::spawn(move || second.close());
        controls.gate(TestPoint::BeforeCloseWait).wait_reached();

        controls.gate(TestPoint::BeforeCloseWait).release();
        controls.gate(TestPoint::BeforeWorkerExit).release();
        let first_error = first_close.join().expect("first close thread").expect_err("failure");
        let second_error = second_close.join().expect("second close thread").expect_err("failure");
        assert_eq!(first_error.to_string(), second_error.to_string());
        assert!(matches!(first_error, Error::OpenedWorkerFailed { worker: "failure", .. }));
        assert_eq!(
            opened.close().expect_err("stored failure").to_string(),
            second_error.to_string()
        );
    }

    #[test]
    fn a_panicking_worker_is_joined_and_reported() {
        let (_root, opened) = opened(Arc::default());
        opened
            .spawn_worker("panic", |_cancellation| panic!("injected worker panic"))
            .expect("spawn worker");

        let error = opened.close().expect_err("panic is terminal");
        assert!(matches!(error, Error::OpenedWorkerPanicked { worker: "panic" }));
        assert!(matches!(opened.close(), Err(Error::OpenedWorkerPanicked { worker: "panic" })));
    }

    #[test]
    fn dropping_the_last_reference_cancels_and_joins() {
        let active = Arc::new(AtomicUsize::new(0));
        let (started_sender, started_receiver) = std::sync::mpsc::sync_channel(0);
        let (_root, opened) = opened(Arc::default());
        let worker_active = Arc::clone(&active);
        opened
            .spawn_worker("drop", move |cancellation| {
                worker_active.fetch_add(1, Ordering::SeqCst);
                started_sender.send(()).expect("report worker start");
                cancellation.wait_cancelled();
                worker_active.fetch_sub(1, Ordering::SeqCst);
                Ok(())
            })
            .expect("spawn worker");

        started_receiver.recv_timeout(std::time::Duration::from_secs(5)).expect("worker started");
        drop(opened);
        assert_eq!(active.load(Ordering::SeqCst), 0, "drop returned before worker join");
    }

    #[test]
    fn poisoned_lifecycle_still_joins_before_returning_its_typed_failure() {
        let active = Arc::new(AtomicUsize::new(0));
        let (started_sender, started_receiver) = std::sync::mpsc::sync_channel(0);
        let (_root, opened) = opened(Arc::default());
        let worker_active = Arc::clone(&active);
        opened
            .spawn_worker("poison", move |cancellation| {
                worker_active.fetch_add(1, Ordering::SeqCst);
                started_sender.send(()).expect("report worker start");
                cancellation.wait_cancelled();
                worker_active.fetch_sub(1, Ordering::SeqCst);
                Ok(())
            })
            .expect("spawn worker");
        started_receiver.recv_timeout(std::time::Duration::from_secs(5)).expect("worker started");

        let state = Arc::clone(&opened.state);
        thread::spawn(move || {
            let _guard = state.lifecycle.lock().expect("lifecycle lock");
            panic!("inject lifecycle poison");
        })
        .join()
        .expect_err("injected panic");

        assert!(matches!(opened.close(), Err(Error::OpenedLifecyclePoisoned)));
        assert_eq!(active.load(Ordering::SeqCst), 0, "poison bypassed worker join");
        assert!(matches!(opened.close(), Err(Error::OpenedLifecyclePoisoned)));
    }

    #[test]
    fn poisoned_index_still_joins_and_replays_the_typed_failure() {
        let (_root, opened) = opened(Arc::default());
        opened.state.index.poison_for_test();

        assert!(matches!(opened.close(), Err(Error::IndexLockPoisoned)));
        assert!(matches!(opened.close(), Err(Error::IndexLockPoisoned)));
    }

    #[test]
    fn opened_root_rejects_invalid_scan_policy_and_nondirectories() {
        let root = tempfile::tempdir().expect("temp root");
        let options = OpenOptions { batch_size: 0, ..OpenOptions::default() };
        assert!(matches!(
            OpenedIndex::open(root.path(), options),
            Err(Error::UnsupportedScanConfig(_))
        ));

        let file = root.path().join("file");
        std::fs::write(&file, b"x").expect("fixture");
        assert!(matches!(OpenedIndex::open(&file, OpenOptions::default()), Err(Error::Io { .. })));
    }

    #[test]
    fn distinct_opens_have_distinct_live_identity() {
        let root = tempfile::tempdir().expect("temp root");
        let first = OpenedIndex::open(root.path(), OpenOptions::default()).expect("first");
        let second = OpenedIndex::open(root.path(), OpenOptions::default()).expect("second");
        assert_ne!(first.state.session, second.state.session);
        first.close().expect("first close");
        second.close().expect("second close");
    }
}
