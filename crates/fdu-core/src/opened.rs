//! Ownership and joined shutdown for one long-lived opened root.
//!
//! [`OpenedIndex`] is the public behavior surface. Its private shared state contains
//! data and synchronization only; it is deliberately not a second API-shaped service.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

#[cfg(test)]
use crate::EntryKind;
use crate::index::{DiscoveryCommit, DiscoveryTransition};
use crate::scan::ReconcileControl;
use crate::{Error, Index, IndexHandle, Observation, Op, Result, ScanConfig, SessionId};

mod continuation;
#[cfg(all(test, feature = "watch", feature = "gitignore"))]
mod golden_support;
#[cfg(all(test, feature = "watch", feature = "gitignore"))]
mod golden_tests;
mod journal;
pub(crate) mod read;

/// First ordinal reserved for a minted session; zero never identifies a live owner.
const FIRST_SESSION_ORDINAL: u64 = 1;
/// FNV-1a offset used to mix the process and open instance into an opaque identity.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a prime used for the opened-root identity mix.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
/// Nonzero fallback for the reserved zero identity.
const FIRST_SESSION_ID: u64 = 1;
/// Maximum paths accepted by one best-effort priority request.
pub const MAX_PRIORITY_PATHS: usize = 64;
/// Maximum paths accepted by one refresh operation.
pub const MAX_REFRESH_PATHS: usize = 1_024;
#[cfg(test)]
/// Deadline for a missing deterministic test barrier to fail instead of hanging.
const TEST_GATE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Identity of one live opened-root lifetime.
///
/// The value is process-local, opaque, and never persisted. It prevents a future cursor
/// or continuation from being accepted by another open whose sequence also began at
/// zero; it is not a credential.
impl crate::SessionId {
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

/// Resource bounds applied to progressive discovery.
///
/// The first version deliberately has one measured resource: retained regular files.
/// The value is execution policy, not semantic scan scope, and therefore is absent from
/// [`crate::ScanScope`] and snapshot identity.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DiscoveryBudget {
    /// Maximum regular files retained, or no limit when absent.
    pub max_files: Option<u64>,
}

/// Configuration for a long-lived [`OpenedIndex`].
///
/// Scope and execution settings are flat here because each is one independent decision;
/// display depth is absent by design and belongs to a read request. The options bind
/// progressive discovery, exact-history bounds, and optional live observation without
/// changing the one-shot API.
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
    /// File-type rules, or `None` for the rules compiled into fdu.
    pub types: Option<Arc<crate::classify::TypeRegistry>>,
    /// Resource policy for the cold progressive walk.
    pub budget: DiscoveryBudget,
    /// Optional filesystem observation captured before cold discovery begins.
    #[cfg(feature = "watch")]
    pub observation: Option<crate::watch::WatchConfig>,
    /// Test-only replacement for the native event source.
    #[cfg(all(feature = "watch", test))]
    #[doc(hidden)]
    pub observation_script: Option<PathBuf>,
    /// Maximum retained-cost units in the exact commit journal.
    pub journal_capacity: usize,
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
            types: scan.types,
            budget: DiscoveryBudget::default(),
            #[cfg(feature = "watch")]
            observation: None,
            #[cfg(all(feature = "watch", test))]
            observation_script: None,
            journal_capacity: crate::DEFAULT_JOURNAL_CAPACITY,
        }
    }
}

impl OpenOptions {
    fn into_parts(self) -> (ScanConfig, DiscoveryBudget, usize) {
        let scan = ScanConfig {
            max_depth: None,
            batch_size: self.batch_size,
            follow_symlinks: self.follow_symlinks,
            one_filesystem: self.one_filesystem,
            hidden: self.hidden,
            exclude_special: self.exclude_special,
            // The first opened-root scheduler is intentionally one parent-first
            // producer. Parallel I/O is an internal optimization, not a public
            // semantic or tuning promise for this new API.
            threads: Some(1),
            order: crate::ScanOrder::BreadthFirst,
            types: self.types,
        };
        (scan, self.budget, self.journal_capacity)
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
    /// This constructor validates and binds the root and semantic configuration, starts
    /// cold progressive discovery, and captures optional observation before that
    /// baseline begins. No cache image or second mutable index is created here.
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
        let opened = Self { state: Arc::new(state) };
        opened.start_discovery()?;
        #[cfg(feature = "watch")]
        if let Err(error) = opened.start_observation() {
            let _ = opened.close();
            return Err(error);
        }
        Ok(opened)
    }

    #[cfg(test)]
    fn build(root: &Path, options: OpenOptions, controls: Arc<TestControls>) -> Result<Self> {
        let state = OpenedState::new(root, options, controls)?;
        let opened = Self { state: Arc::new(state) };
        if !opened.state.test_controls.discovery_disabled.load(Ordering::Acquire) {
            opened.start_discovery()?;
        }
        #[cfg(feature = "watch")]
        if let Err(error) = opened.start_observation() {
            let _ = opened.close();
            return Err(error);
        }
        Ok(opened)
    }

    /// Cancel and join all work owned by this opened root.
    ///
    /// The first caller performs shutdown. Concurrent and repeated callers wait for or
    /// replay its stored terminal outcome; success is never reported while a worker is
    /// still live.
    pub fn close(&self) -> Result<()> {
        self.state.shutdown()
    }

    /// Reorder pending discovery toward the supplied relative paths.
    ///
    /// This is a bounded best-effort scheduling hint. It does not change scope, facts,
    /// lifecycle state, or the index clock, and paths that are already complete simply
    /// have no effect.
    pub fn prioritize(&self, paths: &[PathBuf]) -> Result<()> {
        self.ensure_open()?;
        if paths.len() > MAX_PRIORITY_PATHS {
            return Err(Error::PriorityPathLimit {
                attempted: paths.len(),
                limit: MAX_PRIORITY_PATHS,
            });
        }
        if self.state.index.state()?.phase == crate::LifecyclePhase::Stopped {
            return Err(Error::OpenedIndexStopped);
        }
        let mut normalized = Vec::with_capacity(paths.len());
        for path in paths {
            normalized.push(crate::scan::normalize_subtree(path)?);
        }
        normalized.sort();
        normalized.dedup();
        self.state.frontier.prioritize(normalized);
        Ok(())
    }

    /// Return requested projections from one committed version and state boundary.
    pub fn read(&self, request: crate::ReadRequest) -> Result<crate::ReadResponse> {
        let locked = self.state.lock_lifecycle();
        if locked.poisoned {
            return Err(Error::OpenedLifecyclePoisoned);
        }
        if locked.guard.phase != OwnerPhase::Open {
            return Err(Error::OpenedIndexClosed);
        }
        read::read(self, request)
    }

    /// Return exact commits after one version, waiting up to the supplied timeout.
    pub fn changes(&self, request: crate::ChangeRequest) -> Result<crate::ChangePoll> {
        self.ensure_open()?;
        journal::poll(self, request)
    }

    /// Verify a bounded set of relative paths and conditionally commit exact changes.
    ///
    /// Inputs are classified as one set: duplicates are removed, descendants covered
    /// by an accepted ancestor cost no second walk, and an empty set is a no-op. The
    /// returned `(after, version]` interval is safe as the next journal boundary even
    /// when another producer committed concurrently.
    pub fn refresh(&self, paths: &[PathBuf]) -> Result<crate::RefreshResult> {
        let _active = self.state.begin_refresh()?;
        if paths.len() > MAX_REFRESH_PATHS {
            return Err(Error::RefreshPathLimit {
                attempted: paths.len(),
                limit: MAX_REFRESH_PATHS,
            });
        }
        let control = OpenedReconcileControl { state: &self.state };
        let (after, initial_state) = self.version_and_state()?;
        // A stopped partial root remains readable and may verify work that proves it
        // cannot expand retained truth. Before that terminal state, the exact commit
        // boundary arbitrates every file upsert against the shared budget.
        let forbid_expansion = initial_state.phase == crate::LifecyclePhase::Stopped
            && initial_state.coverage == crate::Coverage::Partial(crate::CoverageReason::Budget);
        let report = crate::scan::reconcile_paths_handle_controlled(
            &self.state.index,
            paths,
            &self.state.scan,
            forbid_expansion,
            &control,
            &mut |_commit| self.state.journal.notify_commit(),
        )?;
        control.check_active()?;
        let (version, state, impact) = self.state.index.read_with(|index| {
            let scope = index.scope();
            let since = index.since(after.sequence);
            let version = crate::EngineVersion {
                session: self.state.session,
                sequence: since.clock,
                scope: scope.scope_identity(),
                semantics: scope.semantic_identity(),
            };
            let impact = journal::interval_impact(&since);
            (version, since.state, impact)
        })?;
        let mut issues = Vec::new();
        let mut omitted_issues = 0_u64;
        for error in &report.reconciliation.scan.errors {
            let issue = crate::Issue::from_error(error);
            if issues.len() < crate::MAX_RETAINED_ISSUES {
                issues.push(issue);
            } else {
                omitted_issues = omitted_issues.saturating_add(1);
            }
        }
        if report.reconciliation.apply.resource_refused > 0 {
            let issue = crate::Issue::resource_budget(
                self.state
                    .budget
                    .max_files
                    .expect("resource refusal requires a configured file limit"),
            );
            if issues.len() < crate::MAX_RETAINED_ISSUES {
                issues.push(issue);
            } else {
                omitted_issues = omitted_issues.saturating_add(1);
            }
        }
        let work = crate::Work {
            observations: report.reconciliation.observations,
            unchanged: report.reconciliation.apply.unchanged,
            stale: report.reconciliation.apply.stale,
            resource_refused: report.reconciliation.apply.resource_refused,
            directories_read: report.reconciliation.scan.dirs_read,
            entries_visited: report.reconciliation.scan.entries,
            files_visited: report.reconciliation.scan.files_walked,
            bytes_visited: report.reconciliation.scan.bytes_walked,
            ..crate::Work::default()
        };
        Ok(crate::RefreshResult {
            after,
            version,
            state,
            accepted: report.accepted,
            rejected: report.rejected,
            impact,
            work,
            issues,
            omitted_issues,
        })
    }

    fn version_and_state(&self) -> Result<(crate::EngineVersion, crate::IndexState)> {
        self.state.index.read_with(|index| {
            let scope = index.scope();
            (
                crate::EngineVersion {
                    session: self.state.session,
                    sequence: index.clock(),
                    scope: scope.scope_identity(),
                    semantics: scope.semantic_identity(),
                },
                index.state(),
            )
        })
    }

    fn start_discovery(&self) -> Result<()> {
        publish_discovery_transition(
            &self.state.index,
            &self.state.journal,
            DiscoveryTransition::Begin,
        )?;
        let root = self.state.root.clone();
        let index = self.state.index.clone();
        let journal = Arc::clone(&self.state.journal);
        let scan = self.state.scan.clone();
        let budget = self.state.budget;
        let frontier = Arc::clone(&self.state.frontier);
        #[cfg(feature = "watch")]
        let baseline = Arc::clone(&self.state.baseline);
        #[cfg(test)]
        let controls = Arc::clone(&self.state.test_controls);
        self.spawn_worker("discovery", move |cancellation| {
            #[cfg(feature = "watch")]
            let _baseline_finished = BaselineCompletion(baseline);
            #[cfg(test)]
            controls.reach(TestPoint::BeforeDiscovery);
            #[cfg(not(test))]
            let outcome =
                { run_discovery(&root, &index, &journal, &scan, budget, &frontier, &cancellation) };
            #[cfg(test)]
            let outcome = {
                run_discovery(
                    &root,
                    &index,
                    &journal,
                    &scan,
                    budget,
                    &frontier,
                    &cancellation,
                    &controls,
                )
            };
            if let Err(error) = outcome {
                let state = index.state()?;
                if state.phase == crate::LifecyclePhase::Discovering {
                    publish_discovery_transition(
                        &index,
                        &journal,
                        DiscoveryTransition::Failed(crate::Issue::from_error(&error)),
                    )?;
                }
                return Err(error);
            }
            Ok(())
        })
    }

    #[cfg(feature = "watch")]
    fn start_observation(&self) -> Result<()> {
        let watcher =
            self.state.observer.lock().map_err(|_| Error::OpenedLifecyclePoisoned)?.take();
        let Some(watcher) = watcher else {
            return Ok(());
        };
        let index = self.state.index.clone();
        let journal = Arc::clone(&self.state.journal);
        let scan = self.state.scan.clone();
        let budget = self.state.budget;
        let baseline = Arc::clone(&self.state.baseline);
        #[cfg(test)]
        let controls = Arc::clone(&self.state.test_controls);
        self.spawn_worker("observation", move |cancellation| {
            let outcome = run_observation(
                watcher,
                &index,
                &journal,
                &scan,
                budget,
                &baseline,
                &cancellation,
                #[cfg(test)]
                &controls,
            );
            if matches!(outcome, Err(Error::OpenedIndexClosed)) && cancellation.is_cancelled() {
                return Ok(());
            }
            if let Err(error) = &outcome {
                if !cancellation.is_cancelled() {
                    publish_observation_transition(
                        &index,
                        &journal,
                        crate::index::ObservationTransition::Failed(crate::Issue::from_error(
                            error,
                        )),
                    )?;
                }
            }
            outcome
        })
    }

    #[allow(dead_code)]
    fn ensure_open(&self) -> Result<()> {
        self.state.ensure_open()
    }

    /// Register one worker with the shared owner before it can race with close.
    ///
    /// The worker receives only cancellation, not a strong owner reference. Discovery
    /// and observation both use this boundary, which deterministic lifecycle tests
    /// exercise directly.
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
                    if name != "discovery" {
                        controls.reach(TestPoint::BeforeWorkerExit);
                    }
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
    /// Retained for discovery and verified producers; semantic identity is also
    /// fixed in `index`, so this cannot reinterpret already-retained facts.
    #[allow(dead_code)]
    scan: ScanConfig,
    budget: DiscoveryBudget,
    frontier: Arc<DiscoveryFrontier>,
    continuations: Mutex<continuation::ContinuationTable>,
    journal: Arc<journal::JournalWait>,
    cancellation: Arc<Cancellation>,
    #[cfg(feature = "watch")]
    baseline: Arc<BaselineLatch>,
    #[cfg(feature = "watch")]
    observer: Mutex<Option<crate::watch::Watcher>>,
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
        #[cfg(feature = "watch")]
        let observation = options.observation;
        let (root, index, scan, budget) = bind_root(root, options)?;
        #[cfg(feature = "watch")]
        let observer = if let Some(config) = observation {
            scan.validate_for_watch_scope(index.scope()?)?;
            Some(crate::watch::Watcher::new(&root, config)?)
        } else {
            None
        };
        Ok(Self {
            session: SessionId::mint()?,
            root,
            index,
            scan,
            budget,
            frontier: Arc::new(DiscoveryFrontier::new()),
            continuations: Mutex::new(continuation::ContinuationTable::default()),
            journal: Arc::new(journal::JournalWait::new()),
            cancellation: Arc::new(Cancellation::default()),
            #[cfg(feature = "watch")]
            baseline: Arc::new(BaselineLatch::default()),
            #[cfg(feature = "watch")]
            observer: Mutex::new(observer),
            lifecycle: Mutex::new(Lifecycle::default()),
            lifecycle_changed: Condvar::new(),
        })
    }

    #[cfg(test)]
    fn build(root: &Path, options: OpenOptions, controls: Arc<TestControls>) -> Result<Self> {
        #[cfg(feature = "watch")]
        let observation = options.observation;
        #[cfg(feature = "watch")]
        let observation_script = options.observation_script.clone();
        let (root, index, scan, budget) = bind_root(root, options)?;
        #[cfg(feature = "watch")]
        if observation.is_some() {
            scan.validate_for_watch_scope(index.scope()?)?;
        }
        #[cfg(feature = "watch")]
        let (observer, scripted_sender) = match (observation, observation_script) {
            (Some(config), Some(events)) => {
                let (watcher, sender) = crate::watch::Watcher::scripted(&root, config, &events)?;
                (Some(watcher), Some(sender))
            }
            (Some(config), None) => (Some(crate::watch::Watcher::new(&root, config)?), None),
            (None, _) => (None, None),
        };
        #[cfg(feature = "watch")]
        {
            *controls.scripted_observer.lock().unwrap_or_else(std::sync::PoisonError::into_inner) =
                scripted_sender;
        }
        Ok(Self {
            session: SessionId::mint()?,
            root,
            index,
            scan,
            budget,
            frontier: Arc::new(DiscoveryFrontier::new()),
            continuations: Mutex::new(continuation::ContinuationTable::default()),
            journal: Arc::new(journal::JournalWait::new()),
            cancellation: Arc::new(Cancellation::default()),
            #[cfg(feature = "watch")]
            baseline: Arc::new(BaselineLatch::default()),
            #[cfg(feature = "watch")]
            observer: Mutex::new(observer),
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

    fn begin_refresh(&self) -> Result<ActiveRefresh<'_>> {
        let locked = self.lock_lifecycle();
        if locked.poisoned {
            return Err(Error::OpenedLifecyclePoisoned);
        }
        let mut lifecycle = locked.guard;
        if lifecycle.phase != OwnerPhase::Open {
            return Err(Error::OpenedIndexClosed);
        }
        lifecycle.active_refreshes = lifecycle.active_refreshes.saturating_add(1);
        Ok(ActiveRefresh { state: self })
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
                    self.cancellation.cancel();
                    #[cfg(feature = "watch")]
                    self.baseline.wake();
                    drop(lifecycle);
                    self.journal.close();
                    match self.continuations.lock() {
                        Ok(mut continuations) => continuations.clear(),
                        Err(poisoned) => poisoned.into_inner().clear(),
                    }
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
        let locked = self.lock_lifecycle();
        saw_poison |= locked.poisoned;
        let mut lifecycle = locked.guard;
        while lifecycle.active_refreshes > 0 {
            match self.lifecycle_changed.wait(lifecycle) {
                Ok(next) => lifecycle = next,
                Err(poisoned) => {
                    saw_poison = true;
                    lifecycle = poisoned.into_inner();
                }
            }
        }
        drop(lifecycle);
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
) -> Result<(std::path::PathBuf, IndexHandle, ScanConfig, DiscoveryBudget)> {
    let (scan, budget, journal_capacity) = options.into_parts();
    scan.validate()?;
    if budget.max_files == Some(0) {
        return Err(Error::UnsupportedScanConfig(
            "max_files must be nonzero; omit it for an unlimited discovery",
        ));
    }
    if journal_capacity == 0 {
        return Err(Error::UnsupportedScanConfig("journal_capacity must be nonzero"));
    }
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
    let index = IndexHandle::new(Index::new_opened_with_scope_types_and_journal_capacity(
        &root,
        scope,
        types,
        journal_capacity,
    ));
    Ok((root, index, scan, budget))
}

#[derive(Clone, Debug)]
struct PendingDirectory {
    path: PathBuf,
    depth: usize,
}

struct DiscoveryFrontier {
    state: Mutex<FrontierState>,
}

struct FrontierState {
    pending: VecDeque<PendingDirectory>,
    priorities: Vec<PathBuf>,
    stopped: bool,
}

impl DiscoveryFrontier {
    fn new() -> Self {
        Self {
            state: Mutex::new(FrontierState {
                pending: VecDeque::from([PendingDirectory { path: PathBuf::new(), depth: 0 }]),
                priorities: Vec::new(),
                stopped: false,
            }),
        }
    }

    fn pop(&self) -> Option<PendingDirectory> {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.stopped {
            return None;
        }
        let selected = state
            .pending
            .iter()
            .enumerate()
            .filter_map(|(position, pending)| {
                state
                    .priorities
                    .iter()
                    .position(|priority| {
                        priority.starts_with(&pending.path) || pending.path.starts_with(priority)
                    })
                    .map(|priority| (priority, position))
            })
            .min()
            .map_or(0, |(_, position)| position);
        state.pending.remove(selected)
    }

    fn extend(&self, directories: impl IntoIterator<Item = PendingDirectory>) {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.stopped {
            state.pending.extend(directories);
        }
    }

    fn prioritize(&self, priorities: Vec<PathBuf>) {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.stopped {
            state.priorities = priorities;
        }
    }

    fn stop(&self) {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        state.stopped = true;
        state.pending.clear();
        state.priorities.clear();
    }
}

#[allow(clippy::too_many_arguments)]
fn run_discovery(
    root: &Path,
    index: &IndexHandle,
    journal: &journal::JournalWait,
    scan: &ScanConfig,
    budget: DiscoveryBudget,
    frontier: &DiscoveryFrontier,
    cancellation: &Cancellation,
    #[cfg(test)] controls: &TestControls,
) -> Result<()> {
    let root_metadata =
        std::fs::symlink_metadata(root).map_err(|source| Error::io(root, source))?;
    let root_dev = crate::scan::attrs_from(&root_metadata).dev;

    while let Some(directory) = frontier.pop() {
        if cancellation.is_cancelled() {
            frontier.stop();
            publish_discovery_transition(index, journal, DiscoveryTransition::Cancelled)?;
            return Ok(());
        }
        match discover_directory(
            root,
            index,
            journal,
            scan,
            budget,
            root_dev,
            &directory,
            frontier,
            cancellation,
            #[cfg(test)]
            controls.deterministic_discovery_order.load(Ordering::Acquire),
        )? {
            DiscoveryStep::Continue => {}
            DiscoveryStep::Stopped => return Ok(()),
        }
        #[cfg(test)]
        if directory.path.as_os_str().is_empty() {
            controls.reach(TestPoint::AfterRootDirectory);
        }
    }
    publish_discovery_transition(index, journal, DiscoveryTransition::Finish)?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DiscoveryStep {
    Continue,
    Stopped,
}

#[allow(clippy::too_many_arguments)]
fn discover_directory(
    root: &Path,
    index: &IndexHandle,
    journal: &journal::JournalWait,
    scan: &ScanConfig,
    budget: DiscoveryBudget,
    root_dev: u64,
    directory: &PendingDirectory,
    frontier: &DiscoveryFrontier,
    cancellation: &Cancellation,
    #[cfg(test)] deterministic_discovery_order: bool,
) -> Result<DiscoveryStep> {
    let absolute = root.join(&directory.path);
    crate::counters::bump(|c| c.dir_opens += 1);
    let listing = match std::fs::read_dir(&absolute) {
        Ok(listing) => listing,
        Err(source) => {
            let error = Error::io(&absolute, source);
            publish_discovery_transition(
                index,
                journal,
                DiscoveryTransition::Inaccessible {
                    issues: vec![crate::Issue::from_error(&error)],
                    omitted: 0,
                },
            )?;
            return Ok(DiscoveryStep::Continue);
        }
    };
    #[cfg(test)]
    let listing = test_directory_listing(listing, deterministic_discovery_order);
    let mut batch = Vec::with_capacity(scan.batch_size);
    let mut discovered = Vec::new();
    let mut issues = Vec::new();
    let mut omitted_issues = 0_u64;

    for item in listing {
        if cancellation.is_cancelled() {
            commit_discovery_batch(
                index,
                journal,
                &mut batch,
                None,
                Some(DiscoveryTransition::Cancelled),
                budget.max_files,
            )?;
            frontier.stop();
            return Ok(DiscoveryStep::Stopped);
        }
        let item = item
            .inspect_err(|source| {
                retain_local_issue(
                    &mut issues,
                    &mut omitted_issues,
                    crate::Issue::from_io(&absolute, source),
                );
            })
            .ok();
        let Some(item) = item else {
            continue;
        };
        crate::counters::bump(|c| c.dir_entries += 1);
        let metadata = match crate::scan::metadata_for_fingerprint(&item) {
            Ok(metadata) => metadata,
            Err(source) => {
                retain_local_issue(
                    &mut issues,
                    &mut omitted_issues,
                    crate::Issue::from_io(&item.path(), &source),
                );
                continue;
            }
        };
        let name = item.file_name();
        let (kind, attrs) = crate::scan::observe(&metadata);
        let Some(prepared) = crate::scan::prepare_walk_entry(
            root,
            &directory.path,
            directory.depth,
            &name,
            kind,
            attrs,
            root_dev,
            scan,
        ) else {
            continue;
        };
        let crate::scan::PreparedWalkEntry {
            path,
            kind,
            attrs,
            retained,
            control,
            descend,
            control_error,
        } = prepared;
        if let Some(error) = control_error {
            retain_local_issue(&mut issues, &mut omitted_issues, crate::Issue::from_error(&error));
        }
        if !retained {
            if let Some(control) = control {
                if push_discovery_op(
                    index,
                    journal,
                    scan.batch_size,
                    &mut batch,
                    control,
                    budget.max_files,
                )? {
                    frontier.stop();
                    return Ok(DiscoveryStep::Stopped);
                }
            }
            continue;
        }

        if push_discovery_op(
            index,
            journal,
            scan.batch_size,
            &mut batch,
            Op::Upsert { path: path.clone(), kind, attrs },
            budget.max_files,
        )? {
            frontier.stop();
            return Ok(DiscoveryStep::Stopped);
        }
        if let Some(control) = control {
            if push_discovery_op(
                index,
                journal,
                scan.batch_size,
                &mut batch,
                control,
                budget.max_files,
            )? {
                frontier.stop();
                return Ok(DiscoveryStep::Stopped);
            }
        }
        if descend {
            discovered.push(PendingDirectory { path, depth: directory.depth.saturating_add(1) });
        }
    }

    let incomplete = !issues.is_empty() || omitted_issues > 0;
    let transition =
        incomplete.then_some(DiscoveryTransition::Inaccessible { issues, omitted: omitted_issues });
    let complete = (!incomplete).then(|| directory.path.clone());
    if commit_discovery_batch(index, journal, &mut batch, complete, transition, budget.max_files)? {
        frontier.stop();
        return Ok(DiscoveryStep::Stopped);
    }
    frontier.extend(discovered);
    Ok(DiscoveryStep::Continue)
}

#[cfg(test)]
fn test_directory_listing(
    listing: std::fs::ReadDir,
    deterministic: bool,
) -> Box<dyn Iterator<Item = std::io::Result<std::fs::DirEntry>>> {
    if !deterministic {
        return Box::new(listing);
    }

    // Schedule real filesystem inputs before they enter production admission; do not
    // normalize or reorder the commits whose exact sequence the golden records.
    let mut entries = listing.collect::<Vec<_>>();
    entries.sort_by(|left, right| match (left, right) {
        (Ok(left), Ok(right)) => left.file_name().cmp(&right.file_name()),
        (Ok(_), Err(_)) => std::cmp::Ordering::Less,
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
        (Err(_), Err(_)) => std::cmp::Ordering::Equal,
    });
    Box::new(entries.into_iter())
}

fn retain_local_issue(issues: &mut Vec<crate::Issue>, omitted: &mut u64, issue: crate::Issue) {
    if issues.len() < crate::MAX_RETAINED_ISSUES {
        issues.push(issue);
    } else {
        *omitted = omitted.saturating_add(1);
    }
}

fn push_discovery_op(
    index: &IndexHandle,
    journal: &journal::JournalWait,
    batch_size: usize,
    batch: &mut Vec<Op>,
    op: Op,
    max_files: Option<u64>,
) -> Result<bool> {
    batch.push(op);
    if batch.len() >= batch_size {
        return commit_discovery_batch(index, journal, batch, None, None, max_files);
    }
    Ok(false)
}

fn commit_discovery_batch(
    index: &IndexHandle,
    journal: &journal::JournalWait,
    batch: &mut Vec<Op>,
    directory_complete: Option<PathBuf>,
    transition: Option<DiscoveryTransition>,
    max_files: Option<u64>,
) -> Result<bool> {
    let observation = Observation::new(std::mem::take(batch));
    let outcome = index.apply_discovery_bounded(
        &observation,
        DiscoveryCommit { directory_complete, transition },
        max_files,
    )?;
    if outcome.commit.is_some() {
        journal.notify_commit();
    }
    Ok(outcome.stats.resource_refused > 0)
}

fn publish_discovery_transition(
    index: &IndexHandle,
    journal: &journal::JournalWait,
    transition: DiscoveryTransition,
) -> Result<()> {
    let outcome = index.transition_discovery(transition)?;
    if outcome.commit.is_some() {
        journal.notify_commit();
    }
    Ok(())
}

#[cfg(feature = "watch")]
/// Maximum time the owner waits for an idle observation before checking cancellation.
const OBSERVATION_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
#[cfg(feature = "watch")]
/// Full-root handoff retries allowed after benign conditional conflicts with a producer.
const MAX_HANDOFF_RECONCILIATION_ATTEMPTS: usize = 3;

#[cfg(feature = "watch")]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::needless_pass_by_value)] // Ownership keeps the backend alive for this worker.
fn run_observation(
    watcher: crate::watch::Watcher,
    index: &IndexHandle,
    journal: &journal::JournalWait,
    scan: &ScanConfig,
    budget: DiscoveryBudget,
    baseline: &BaselineLatch,
    cancellation: &Cancellation,
    #[cfg(test)] controls: &TestControls,
) -> Result<()> {
    if !baseline.wait(cancellation) {
        return Ok(());
    }
    let state = index.state()?;
    if state.phase != crate::LifecyclePhase::Ready {
        return Ok(());
    }

    publish_observation_transition(
        index,
        journal,
        crate::index::ObservationTransition::Reconciling,
    )?;
    let state = index.state()?;
    if state.phase == crate::LifecyclePhase::Stopped {
        return Ok(());
    }
    if state.phase != crate::LifecyclePhase::Reconciling {
        return Err(Error::ObservationHandoffIncomplete);
    }
    #[cfg(test)]
    controls.reach(TestPoint::BeforeObservationHandoff);
    let control = ObservationReconcileControl {
        cancellation,
        max_files: budget.max_files,
        #[cfg(test)]
        controls,
    };
    let handoff_intents = watcher.capture_backlog_bound();
    let mut handoff_evidence = HandoffEvidence::default();

    watcher.flush_capture()?;
    let _ = drain_observation_hints(
        &watcher,
        index,
        journal,
        scan,
        &control,
        handoff_intents,
        &mut handoff_evidence,
    )?;
    if index.state()?.phase == crate::LifecyclePhase::Stopped {
        return Ok(());
    }
    watcher.flush_capture()?;
    let mut handoff_complete = false;
    for _ in 0..MAX_HANDOFF_RECONCILIATION_ATTEMPTS {
        let final_pass = crate::scan::reconcile_paths_handle_controlled(
            index,
            &[PathBuf::new()],
            scan,
            false,
            &control,
            &mut |_commit| journal.notify_commit(),
        )?;
        handoff_evidence.retain(&final_pass.reconciliation);
        watcher.flush_capture()?;
        let drained = drain_observation_hints(
            &watcher,
            index,
            journal,
            scan,
            &control,
            handoff_intents,
            &mut handoff_evidence,
        )?;
        let final_settled = final_pass.reconciliation.apply.resource_refused == 0
            && final_pass.reconciliation.apply.stale == 0;
        if final_settled && drained.complete {
            handoff_complete = true;
            break;
        }
        if index.state()?.phase == crate::LifecyclePhase::Stopped {
            return Ok(());
        }
        let final_retryable =
            final_settled || reconcile_conflict_is_retryable(&final_pass.reconciliation);
        let retryable_conflict =
            final_retryable && (drained.complete || drained.retryable_conflict);
        if !retryable_conflict {
            break;
        }
    }

    control.check_active()?;
    let state = index.state()?;
    if state.phase == crate::LifecyclePhase::Stopped {
        return Ok(());
    }
    if !handoff_complete {
        return Err(Error::ObservationHandoffIncomplete);
    }
    #[cfg(test)]
    controls.reach(TestPoint::BeforeObservationWatching);
    publish_observation_transition(
        index,
        journal,
        crate::index::ObservationTransition::Watching {
            issues: handoff_evidence.issues,
            omitted: handoff_evidence.omitted,
        },
    )?;
    let state = index.state()?;
    if state.phase == crate::LifecyclePhase::Stopped {
        return Ok(());
    }
    if state.phase != crate::LifecyclePhase::Watching {
        return Err(Error::ObservationHandoffIncomplete);
    }

    loop {
        control.check_active()?;
        #[cfg(test)]
        controls.reach(TestPoint::BeforeObservationPoll);
        match watcher.apply_next_controlled(
            index,
            scan,
            OBSERVATION_POLL_INTERVAL,
            &control,
            &mut |_commit| journal.notify_commit(),
        ) {
            Ok(_) => {}
            Err(Error::OpenedIndexClosed) if cancellation.is_cancelled() => return Ok(()),
            Err(error) => return Err(error),
        }
        if index.state()?.phase == crate::LifecyclePhase::Stopped {
            return Ok(());
        }
    }
}

#[cfg(feature = "watch")]
fn drain_observation_hints(
    watcher: &crate::watch::Watcher,
    index: &IndexHandle,
    journal: &journal::JournalWait,
    scan: &ScanConfig,
    control: &dyn ReconcileControl,
    limit: usize,
    evidence: &mut HandoffEvidence,
) -> Result<HandoffDrain> {
    let mut drained = HandoffDrain { complete: true, retryable_conflict: true };
    for _ in 0..limit {
        let Some(report) = watcher.apply_next_controlled(
            index,
            scan,
            std::time::Duration::ZERO,
            control,
            &mut |_commit| journal.notify_commit(),
        )?
        else {
            break;
        };
        evidence.retain(&report.reconciliation);
        let report_complete = report.apply.resource_refused == 0
            && report.apply.stale == 0
            && report.reconciliation.apply.resource_refused == 0
            && report.reconciliation.apply.stale == 0;
        if !report_complete {
            drained.complete = false;
            drained.retryable_conflict &= report.apply.resource_refused == 0
                && report.reconciliation.apply.resource_refused == 0
                && (report.apply.stale > 0 || report.reconciliation.apply.stale > 0);
        }
    }
    Ok(drained)
}

#[cfg(feature = "watch")]
struct HandoffDrain {
    complete: bool,
    retryable_conflict: bool,
}

#[cfg(feature = "watch")]
fn reconcile_conflict_is_retryable(report: &crate::scan::ReconcileReport) -> bool {
    report.apply.resource_refused == 0 && report.apply.stale > 0
}

#[cfg(feature = "watch")]
#[derive(Default)]
struct HandoffEvidence {
    issues: Vec<crate::Issue>,
    omitted: u64,
}

#[cfg(feature = "watch")]
impl HandoffEvidence {
    fn retain(&mut self, report: &crate::scan::ReconcileReport) {
        for error in &report.scan.errors {
            retain_local_issue(
                &mut self.issues,
                &mut self.omitted,
                crate::Issue::from_error(error),
            );
        }
    }
}

#[cfg(feature = "watch")]
fn publish_observation_transition(
    index: &IndexHandle,
    journal: &journal::JournalWait,
    transition: crate::index::ObservationTransition,
) -> Result<()> {
    let outcome = index.transition_observation(transition)?;
    if outcome.commit.is_some() {
        journal.notify_commit();
    }
    Ok(())
}

#[cfg(feature = "watch")]
struct ObservationReconcileControl<'a> {
    cancellation: &'a Cancellation,
    max_files: Option<u64>,
    #[cfg(test)]
    controls: &'a TestControls,
}

#[cfg(feature = "watch")]
impl ReconcileControl for ObservationReconcileControl<'_> {
    fn check_active(&self) -> Result<()> {
        if self.cancellation.is_cancelled() { Err(Error::OpenedIndexClosed) } else { Ok(()) }
    }

    fn before_conditional_commit(&self) -> Result<()> {
        self.check_active()?;
        #[cfg(test)]
        self.controls.reach(TestPoint::AfterObservationVerification);
        self.check_active()
    }

    fn max_files(&self) -> Option<u64> {
        self.max_files
    }
}

struct LockedLifecycle<'a> {
    guard: MutexGuard<'a, Lifecycle>,
    poisoned: bool,
}

#[derive(Default)]
struct Lifecycle {
    phase: OwnerPhase,
    workers: Vec<Worker>,
    active_refreshes: usize,
    terminal: Option<CloseOutcome>,
}

struct ActiveRefresh<'a> {
    state: &'a OpenedState,
}

impl Drop for ActiveRefresh<'_> {
    fn drop(&mut self) {
        let mut lifecycle = self.state.lock_lifecycle().guard;
        lifecycle.active_refreshes = lifecycle.active_refreshes.saturating_sub(1);
        self.state.lifecycle_changed.notify_all();
    }
}

struct OpenedReconcileControl<'a> {
    state: &'a OpenedState,
}

impl crate::scan::ReconcileControl for OpenedReconcileControl<'_> {
    fn check_active(&self) -> Result<()> {
        if self.state.cancellation.is_cancelled() { Err(Error::OpenedIndexClosed) } else { Ok(()) }
    }

    fn before_conditional_commit(&self) -> Result<()> {
        self.check_active()?;
        #[cfg(test)]
        self.state.test_controls.reach(TestPoint::AfterRefreshVerification);
        self.check_active()
    }

    fn max_files(&self) -> Option<u64> {
        self.state.budget.max_files
    }
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

#[cfg(feature = "watch")]
#[derive(Default)]
struct BaselineLatch {
    finished: Mutex<bool>,
    changed: Condvar,
}

#[cfg(feature = "watch")]
impl BaselineLatch {
    fn finish(&self) {
        let mut finished = self.finished.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        *finished = true;
        self.changed.notify_all();
    }

    fn wake(&self) {
        let guard = self.finished.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        self.changed.notify_all();
        drop(guard);
    }

    fn wait(&self, cancellation: &Cancellation) -> bool {
        let mut finished = self.finished.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        while !*finished && !cancellation.is_cancelled() {
            finished =
                self.changed.wait(finished).unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        *finished && !cancellation.is_cancelled()
    }
}

#[cfg(feature = "watch")]
struct BaselineCompletion(Arc<BaselineLatch>);

#[cfg(feature = "watch")]
impl Drop for BaselineCompletion {
    fn drop(&mut self) {
        self.0.finish();
    }
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
    BeforeDiscovery,
    AfterRootDirectory,
    BeforeJournalWait,
    AfterRefreshVerification,
    #[cfg(feature = "watch")]
    BeforeObservationHandoff,
    #[cfg(feature = "watch")]
    BeforeObservationWatching,
    #[cfg(feature = "watch")]
    BeforeObservationPoll,
    #[cfg(feature = "watch")]
    AfterObservationVerification,
}

#[cfg(test)]
#[derive(Default)]
struct TestControls {
    before_worker_exit: TestGate,
    before_close_wait: TestGate,
    before_discovery: TestGate,
    after_root_directory: TestGate,
    before_journal_wait: TestGate,
    after_refresh_verification: TestGate,
    #[cfg(feature = "watch")]
    before_observation_handoff: TestGate,
    #[cfg(feature = "watch")]
    before_observation_watching: TestGate,
    #[cfg(feature = "watch")]
    before_observation_poll: TestGate,
    #[cfg(feature = "watch")]
    after_observation_verification: TestGate,
    #[cfg(feature = "watch")]
    scripted_observer: Mutex<Option<crate::watch::ScriptedSender>>,
    discovery_disabled: AtomicBool,
    deterministic_discovery_order: AtomicBool,
}

#[cfg(test)]
impl TestControls {
    #[cfg(all(feature = "watch", feature = "gitignore"))]
    fn use_deterministic_discovery_order(&self) {
        self.deterministic_discovery_order.store(true, Ordering::Release);
    }

    fn gate(&self, point: TestPoint) -> &TestGate {
        match point {
            TestPoint::BeforeWorkerExit => &self.before_worker_exit,
            TestPoint::BeforeCloseWait => &self.before_close_wait,
            TestPoint::BeforeDiscovery => &self.before_discovery,
            TestPoint::AfterRootDirectory => &self.after_root_directory,
            TestPoint::BeforeJournalWait => &self.before_journal_wait,
            TestPoint::AfterRefreshVerification => &self.after_refresh_verification,
            #[cfg(feature = "watch")]
            TestPoint::BeforeObservationHandoff => &self.before_observation_handoff,
            #[cfg(feature = "watch")]
            TestPoint::BeforeObservationWatching => &self.before_observation_watching,
            #[cfg(feature = "watch")]
            TestPoint::BeforeObservationPoll => &self.before_observation_poll,
            #[cfg(feature = "watch")]
            TestPoint::AfterObservationVerification => &self.after_observation_verification,
        }
    }

    fn reach(&self, point: TestPoint) {
        self.gate(point).reach();
    }

    #[cfg(feature = "watch")]
    fn send_observation_hints(&self, source: &str) {
        self.scripted_observer
            .lock()
            .expect("scripted observer lock")
            .as_ref()
            .expect("scripted observer installed")
            .send(source)
            .expect("valid scripted hints");
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
    use std::ffi::OsString;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn wait_until_settled(opened: &OpenedIndex) -> crate::IndexState {
        let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
        loop {
            let state = opened.state.index.state().expect("read state");
            if state.phase != crate::LifecyclePhase::Discovering {
                return state;
            }
            assert!(std::time::Instant::now() < deadline, "discovery did not settle");
            std::thread::yield_now();
        }
    }

    #[cfg(feature = "watch")]
    fn wait_until_phase(
        opened: &OpenedIndex,
        expected: crate::LifecyclePhase,
    ) -> crate::IndexState {
        let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
        loop {
            let state = opened.state.index.state().expect("read state");
            if state.phase == expected {
                return state;
            }
            assert!(std::time::Instant::now() < deadline, "phase did not become {expected:?}");
            std::thread::yield_now();
        }
    }

    #[cfg(feature = "watch")]
    fn scripted_options(script: &Path) -> OpenOptions {
        OpenOptions {
            observation: Some(crate::watch::WatchConfig {
                settle: std::time::Duration::from_millis(1),
                max_hold: std::time::Duration::from_millis(10),
                ..crate::watch::WatchConfig::default()
            }),
            observation_script: Some(script.to_path_buf()),
            ..OpenOptions::default()
        }
    }

    fn child_facts(index: &Index, path: &Path) -> Vec<(OsString, EntryKind, crate::Attrs)> {
        index
            .children(path)
            .expect("known directory")
            .map(|(name, id)| {
                (
                    name.to_os_string(),
                    index.kind(&index.path_of(id).expect("path")).expect("kind"),
                    *index.attrs(&index.path_of(id).expect("path")).expect("attrs"),
                )
            })
            .collect()
    }

    fn opened(controls: Arc<TestControls>) -> (tempfile::TempDir, OpenedIndex) {
        controls.discovery_disabled.store(true, Ordering::Release);
        let root = tempfile::tempdir().expect("temp root");
        let opened = OpenedIndex::open_for_test(root.path(), OpenOptions::default(), controls)
            .expect("open live root");
        (root, opened)
    }

    fn current_version(opened: &OpenedIndex) -> crate::EngineVersion {
        opened.read(crate::ReadRequest::default()).expect("read version").version
    }

    fn apply_and_notify(opened: &OpenedIndex, observation: &Observation) -> crate::ApplyOutcome {
        let outcome = opened.state.index.apply(observation).expect("apply observation");
        if outcome.commit.is_some() {
            opened.state.journal.notify_commit();
        }
        outcome
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

        let zero_budget = OpenOptions {
            budget: DiscoveryBudget { max_files: Some(0) },
            ..OpenOptions::default()
        };
        assert!(matches!(
            OpenedIndex::open(root.path(), zero_budget),
            Err(Error::UnsupportedScanConfig(_))
        ));

        let zero_journal = OpenOptions { journal_capacity: 0, ..OpenOptions::default() };
        assert!(matches!(
            OpenedIndex::open(root.path(), zero_journal),
            Err(Error::UnsupportedScanConfig(_))
        ));
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

    #[test]
    fn terminal_discovery_failure_retains_bounded_typed_evidence() {
        let root = tempfile::tempdir().expect("temp root");
        let controls = Arc::new(TestControls::default());
        controls.gate(TestPoint::BeforeDiscovery).arm();
        let opened =
            OpenedIndex::open_for_test(root.path(), OpenOptions::default(), Arc::clone(&controls))
                .expect("opened root");
        std::fs::remove_dir(root.path()).expect("remove empty fixture root");
        controls.gate(TestPoint::BeforeDiscovery).release();

        let state = wait_until_settled(&opened);
        assert_eq!(state.phase, crate::LifecyclePhase::Failed);
        assert_eq!(state.coverage, crate::Coverage::Partial(crate::CoverageReason::Failed));
        assert_eq!(state.issues.retained, 1);
        let issues = opened.state.index.issues().expect("issues");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, crate::IssueKind::Disappeared);
        assert!(matches!(opened.close(), Err(Error::OpenedWorkerFailed { .. })));
    }

    #[test]
    fn progressive_discovery_settles_to_the_one_shot_tree() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::create_dir_all(root.path().join("alpha/deep")).expect("fixture directories");
        std::fs::write(root.path().join("root.txt"), b"root").expect("root fixture");
        std::fs::write(root.path().join("alpha/child.bin"), b"child").expect("child fixture");
        std::fs::write(root.path().join("alpha/deep/leaf.rs"), b"leaf").expect("leaf fixture");
        let options = OpenOptions { batch_size: 2, ..OpenOptions::default() };

        let opened = OpenedIndex::open(root.path(), options.clone()).expect("opened root");
        let state = wait_until_settled(&opened);
        assert_eq!(state.phase, crate::LifecyclePhase::Ready);
        assert_eq!(state.coverage, crate::Coverage::Complete);
        assert_eq!(state.progress.files_retained, 3);

        let live = opened.state.index.snapshot().expect("live snapshot");
        let (one_shot, report) =
            crate::scan::scan_into_index(root.path(), &options.clone().into_parts().0)
                .expect("one-shot scan");
        assert!(report.is_complete());
        assert_eq!(live.total(), one_shot.total());
        assert_eq!(live.len(), one_shot.len());
        for path in [Path::new(""), Path::new("alpha"), Path::new("alpha/deep")] {
            assert_eq!(child_facts(&live, path), child_facts(&one_shot, path));
            assert_eq!(live.directory_complete(path), Some(true));
        }
        opened.close().expect("close");
    }

    #[test]
    fn parent_listing_commits_before_prioritized_child_work_without_a_clock_change() {
        let root = tempfile::tempdir().expect("temp root");
        for directory in ["alpha", "target"] {
            std::fs::create_dir(root.path().join(directory)).expect("fixture directory");
            std::fs::write(root.path().join(directory).join("leaf"), directory)
                .expect("fixture file");
        }
        let controls = Arc::new(TestControls::default());
        controls.gate(TestPoint::AfterRootDirectory).arm();
        let opened = OpenedIndex::open_for_test(
            root.path(),
            OpenOptions { batch_size: 64, ..OpenOptions::default() },
            Arc::clone(&controls),
        )
        .expect("opened root");
        controls.gate(TestPoint::AfterRootDirectory).wait_reached();

        let before_priority = opened.state.index.clock().expect("clock");
        assert_eq!(
            opened.state.index.directory_complete(Path::new("")).expect("root completeness"),
            Some(true)
        );
        assert_eq!(
            opened
                .state
                .index
                .directory_complete(Path::new("target"))
                .expect("target completeness"),
            Some(false)
        );
        opened.prioritize(&[PathBuf::from("target")]).expect("prioritize");
        assert_eq!(opened.state.index.clock().expect("clock"), before_priority);
        controls.gate(TestPoint::AfterRootDirectory).release();
        let state = wait_until_settled(&opened);
        assert_eq!(state.coverage, crate::Coverage::Complete);

        let since = opened.state.index.since(before_priority).expect("commits after root");
        let first_file = since
            .commits
            .iter()
            .flat_map(|commit| &commit.changes)
            .find_map(|change| match change {
                crate::EffectiveChange::Inserted { path, kind: EntryKind::File, .. } => {
                    Some(path.clone())
                }
                _ => None,
            })
            .expect("child file commit");
        assert_eq!(first_file, PathBuf::from("target/leaf"));
        opened.close().expect("close");
    }

    #[test]
    fn reaching_a_file_limit_without_refusal_remains_complete() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::write(root.path().join("one"), b"1").expect("fixture");
        std::fs::write(root.path().join("two"), b"2").expect("fixture");
        let opened = OpenedIndex::open(
            root.path(),
            OpenOptions {
                budget: DiscoveryBudget { max_files: Some(2) },
                ..OpenOptions::default()
            },
        )
        .expect("opened root");

        let state = wait_until_settled(&opened);
        assert_eq!(state.coverage, crate::Coverage::Complete);
        assert_eq!(state.progress.files_retained, 2);
        assert_eq!(
            opened.state.index.directory_complete(Path::new("")).expect("root completeness"),
            Some(true)
        );
        opened.close().expect("close");
    }

    #[test]
    fn one_read_returns_lookup_state_and_version_from_one_boundary() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![Op::Upsert {
                path: PathBuf::from("note.txt"),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 7, ..crate::Attrs::default() },
            }]))
            .expect("seed entry");

        let response = opened
            .read(crate::ReadRequest {
                projections: vec![
                    crate::ReadProjection::Lookup { path: PathBuf::from("note.txt") },
                    crate::ReadProjection::Lookup { path: PathBuf::from("missing.txt") },
                ],
                ..crate::ReadRequest::default()
            })
            .expect("coherent read");

        assert_eq!(response.version.sequence, opened.state.index.clock().expect("clock"));
        assert_eq!(response.state, opened.state.index.state().expect("state"));
        assert_eq!(response.results.len(), 2);
        assert!(matches!(
            &response.results[0],
            crate::ProjectionResult::Lookup(crate::Knowledge::Present(entry))
                if entry.path == Path::new("note.txt") && entry.attrs.size == 7
        ));
        assert!(matches!(
            response.results[1],
            crate::ProjectionResult::Lookup(crate::Knowledge::Absent)
        ));
        opened.close().expect("close");
    }

    #[test]
    fn change_poll_returns_the_detached_exact_range_and_terminal_state() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        let after = current_version(&opened);
        apply_and_notify(
            &opened,
            &Observation::new(vec![Op::Upsert {
                path: PathBuf::from("note.txt"),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 7, ..crate::Attrs::default() },
            }]),
        );
        let state_only = opened
            .state
            .index
            .transition_discovery(DiscoveryTransition::Begin)
            .expect("state-only commit");
        assert!(state_only.commit.as_ref().is_some_and(|commit| commit.changes.is_empty()));
        opened.state.journal.notify_commit();

        let poll = opened
            .changes(crate::ChangeRequest { after, timeout: std::time::Duration::ZERO })
            .expect("immediate changes");
        let crate::ChangeOutcome::Changes { commits, impact } = &poll.outcome else {
            panic!("expected changes");
        };
        let detached = opened.state.index.since(after.sequence).expect("detached range");
        assert_eq!(commits, &detached.commits);
        assert_eq!(commits.len(), 2);
        assert!(commits[1].changes.is_empty(), "state-only commit remains observable");
        assert!(impact.domains.contains(&crate::ImpactDomain::State));
        assert_eq!(poll.cursor, poll.version);
        assert_eq!(poll.version.sequence, detached.clock);
        assert_eq!(poll.state, detached.state);
        assert_eq!(poll.work.commits_visited, 2);
        assert_eq!(poll.work.commits_returned, 2);
        opened.close().expect("close");
    }

    #[test]
    fn idle_change_poll_waits_without_advancing_its_cursor() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        let after = current_version(&opened);
        let poll = opened
            .changes(crate::ChangeRequest { after, timeout: std::time::Duration::from_millis(5) })
            .expect("idle poll");

        assert!(matches!(poll.outcome, crate::ChangeOutcome::Idle));
        assert_eq!(poll.cursor, after);
        assert_eq!(poll.version, after);
        assert_eq!(poll.work, crate::Work::default());
        opened.close().expect("close");
    }

    #[test]
    fn a_commit_at_the_wait_boundary_cannot_lose_its_wakeup() {
        let controls = Arc::new(TestControls::default());
        controls.gate(TestPoint::BeforeJournalWait).arm();
        let (_root, opened) = opened(Arc::clone(&controls));
        let after = current_version(&opened);
        let poller = opened.clone();
        let poll = thread::spawn(move || {
            poller
                .changes(crate::ChangeRequest { after, timeout: std::time::Duration::from_secs(5) })
        });
        controls.gate(TestPoint::BeforeJournalWait).wait_reached();

        let (applied_sender, applied_receiver) = std::sync::mpsc::sync_channel(0);
        let committer = opened.clone();
        let commit = thread::spawn(move || {
            let outcome = committer
                .state
                .index
                .apply(&Observation::new(vec![Op::Upsert {
                    path: PathBuf::from("arrived"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs::default(),
                }]))
                .expect("commit at wait boundary");
            applied_sender.send(()).expect("report applied commit");
            if outcome.commit.is_some() {
                committer.state.journal.notify_commit();
            }
        });
        applied_receiver.recv_timeout(TEST_GATE_TIMEOUT).expect("commit applied");
        controls.gate(TestPoint::BeforeJournalWait).release();

        commit.join().expect("committer");
        let poll = poll.join().expect("poller").expect("change poll");
        assert!(matches!(
            poll.outcome,
            crate::ChangeOutcome::Changes { ref commits, .. } if commits.len() == 1
        ));
        opened.close().expect("close");
    }

    #[test]
    fn progressive_discovery_notifies_the_same_change_poll() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::write(root.path().join("discovered"), b"data").expect("fixture");
        let controls = Arc::new(TestControls::default());
        controls.gate(TestPoint::BeforeDiscovery).arm();
        let opened =
            OpenedIndex::open_for_test(root.path(), OpenOptions::default(), Arc::clone(&controls))
                .expect("opened root");
        let after = current_version(&opened);
        let poller = opened.clone();
        let poll = thread::spawn(move || {
            poller
                .changes(crate::ChangeRequest { after, timeout: std::time::Duration::from_secs(5) })
        });

        controls.gate(TestPoint::BeforeDiscovery).release();
        let poll = poll.join().expect("poller").expect("discovery changes");
        assert!(matches!(
            poll.outcome,
            crate::ChangeOutcome::Changes { ref commits, .. } if !commits.is_empty()
        ));
        assert!(poll.version.sequence > after.sequence);
        wait_until_settled(&opened);
        opened.close().expect("close");
    }

    #[test]
    fn terminal_state_only_discovery_commit_wakes_a_blocked_poll() {
        let root = tempfile::tempdir().expect("temp root");
        let controls = Arc::new(TestControls::default());
        controls.gate(TestPoint::AfterRootDirectory).arm();
        controls.gate(TestPoint::BeforeJournalWait).arm();
        let opened =
            OpenedIndex::open_for_test(root.path(), OpenOptions::default(), Arc::clone(&controls))
                .expect("opened root");
        controls.gate(TestPoint::AfterRootDirectory).wait_reached();
        let after = current_version(&opened);
        let poller = opened.clone();
        let poll = thread::spawn(move || {
            poller
                .changes(crate::ChangeRequest { after, timeout: std::time::Duration::from_secs(5) })
        });
        controls.gate(TestPoint::BeforeJournalWait).wait_reached();
        controls.gate(TestPoint::BeforeJournalWait).release();
        controls.gate(TestPoint::AfterRootDirectory).release();

        let poll = poll.join().expect("poller").expect("terminal change");
        let crate::ChangeOutcome::Changes { commits, .. } = poll.outcome else {
            panic!("expected terminal change");
        };
        assert_eq!(commits.len(), 1);
        assert!(commits[0].changes.is_empty());
        assert!(commits[0].state.iter().any(|transition| matches!(
            transition,
            crate::StateTransition::IndexState {
                current: crate::IndexState { phase: crate::LifecyclePhase::Ready, .. },
                ..
            }
        )));
        assert_eq!(poll.state.phase, crate::LifecyclePhase::Ready);
        opened.close().expect("close");
    }

    #[test]
    fn change_cursors_reject_foreign_identity_and_future_sequences() {
        let (_first_root, first) = opened(Arc::new(TestControls::default()));
        let (_second_root, second) = opened(Arc::new(TestControls::default()));
        let first_version = current_version(&first);
        let second_version = current_version(&second);

        assert!(matches!(
            first.changes(crate::ChangeRequest {
                after: second_version,
                timeout: std::time::Duration::ZERO,
            }),
            Err(Error::ChangeCursorUnavailable { .. })
        ));
        let future = crate::EngineVersion {
            sequence: first_version.sequence.checked_next().expect("future sequence"),
            ..first_version
        };
        assert!(matches!(
            first.changes(crate::ChangeRequest {
                after: future,
                timeout: std::time::Duration::ZERO,
            }),
            Err(Error::ChangeCursorUnavailable { .. })
        ));
        first.close().expect("first close");
        second.close().expect("second close");
    }

    #[test]
    fn a_slow_consumer_gets_one_coherent_all_dirty_reset() {
        let controls = Arc::new(TestControls::default());
        controls.discovery_disabled.store(true, Ordering::Release);
        let root = tempfile::tempdir().expect("temp root");
        let opened = OpenedIndex::open_for_test(
            root.path(),
            OpenOptions { journal_capacity: 1, ..OpenOptions::default() },
            controls,
        )
        .expect("opened root");
        let after = current_version(&opened);
        apply_and_notify(
            &opened,
            &Observation::new(vec![Op::Upsert {
                path: PathBuf::from("larger-than-history"),
                kind: EntryKind::File,
                attrs: crate::Attrs::default(),
            }]),
        );

        let poll = opened
            .changes(crate::ChangeRequest { after, timeout: std::time::Duration::ZERO })
            .expect("consumer reset");
        let crate::ChangeOutcome::Reset { impact } = &poll.outcome else {
            panic!("expected reset");
        };
        assert!(impact.all_dirty);
        assert!(impact.dirty_paths.is_empty());
        assert_eq!(impact.domains.len(), 6);
        assert_eq!(poll.cursor, poll.version);
        assert_eq!(poll.state, opened.state.index.state().expect("terminal state"));
        assert!(opened.state.index.since(after.sequence).expect("history").truncated);
        opened.close().expect("close");
    }

    #[test]
    fn close_wakes_a_blocked_change_poll() {
        let controls = Arc::new(TestControls::default());
        controls.gate(TestPoint::BeforeJournalWait).arm();
        let (_root, opened) = opened(Arc::clone(&controls));
        let after = current_version(&opened);
        let poller = opened.clone();
        let poll = thread::spawn(move || {
            poller.changes(crate::ChangeRequest {
                after,
                timeout: std::time::Duration::from_secs(60),
            })
        });
        controls.gate(TestPoint::BeforeJournalWait).wait_reached();
        controls.gate(TestPoint::BeforeJournalWait).release();
        opened.close().expect("close");

        assert!(matches!(poll.join().expect("poller"), Err(Error::OpenedIndexClosed)));
    }

    #[test]
    fn change_invalidations_fail_closed_at_the_existing_path_bound() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        let after = current_version(&opened);
        let ops = (0..=crate::MAX_DIRTY_PATHS)
            .map(|index| Op::Upsert {
                path: PathBuf::from(format!("entry-{index}")),
                kind: EntryKind::File,
                attrs: crate::Attrs::default(),
            })
            .collect();
        apply_and_notify(&opened, &Observation::new(ops));

        let poll = opened
            .changes(crate::ChangeRequest { after, timeout: std::time::Duration::ZERO })
            .expect("bounded invalidation");
        assert!(matches!(
            poll.outcome,
            crate::ChangeOutcome::Changes {
                ref commits,
                impact: crate::Impact { all_dirty: true, ref dirty_paths, .. },
            } if commits.len() == 1 && dirty_paths.is_empty()
        ));
        opened.close().expect("close");
    }

    #[test]
    fn lookup_uses_directory_completeness_before_global_discovery_settles() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .transition_discovery(DiscoveryTransition::Begin)
            .expect("begin discovery");
        opened
            .state
            .index
            .apply(&Observation::new(vec![Op::Upsert {
                path: PathBuf::from("known"),
                kind: EntryKind::Dir,
                attrs: crate::Attrs::default(),
            }]))
            .expect("seed directory");

        let lookup = || {
            opened
                .read(crate::ReadRequest {
                    projections: vec![crate::ReadProjection::Lookup {
                        path: PathBuf::from("known/missing"),
                    }],
                    ..crate::ReadRequest::default()
                })
                .expect("lookup")
                .results
                .into_iter()
                .next()
                .expect("lookup result")
        };
        assert!(matches!(
            lookup(),
            crate::ProjectionResult::Lookup(crate::Knowledge::Unknown {
                reason: crate::CoverageReason::Building
            })
        ));

        opened
            .state
            .index
            .apply_discovery(
                &Observation::new(Vec::new()),
                DiscoveryCommit {
                    directory_complete: Some(PathBuf::from("known")),
                    transition: None,
                },
            )
            .expect("complete directory");
        assert!(matches!(lookup(), crate::ProjectionResult::Lookup(crate::Knowledge::Absent)));
        assert!(matches!(
            opened.state.index.state().expect("state").coverage,
            crate::Coverage::Partial(_)
        ));
        opened.close().expect("close");
    }

    #[test]
    fn mixed_read_preserves_projection_order_and_uses_maintained_rollups() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![Op::Upsert {
                path: PathBuf::from("note.txt"),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 7, ..crate::Attrs::default() },
            }]))
            .expect("seed entry");

        let response = opened
            .read(crate::ReadRequest {
                projections: vec![
                    crate::ReadProjection::Diagnostics,
                    crate::ReadProjection::RollUp { path: PathBuf::new() },
                ],
                ..crate::ReadRequest::default()
            })
            .expect("coherent read");

        assert!(matches!(
            &response.results[0],
            crate::ProjectionResult::Diagnostics(diagnostics)
                if diagnostics.root == opened.state.root && diagnostics.entries == 2
        ));
        assert!(matches!(
            &response.results[1],
            crate::ProjectionResult::RollUp(crate::Knowledge::Present(rollup))
                if rollup.all.files == 1 && rollup.all.bytes == 7
        ));
        assert_eq!(response.work.maintained_index_work, 1);
        opened.close().expect("close");
    }

    #[test]
    fn tree_pages_are_directory_first_and_resume_at_the_same_version() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("z-dir"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("a.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 1, ..crate::Attrs::default() },
                },
                Op::Upsert {
                    path: PathBuf::from("b.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 2, ..crate::Attrs::default() },
                },
            ]))
            .expect("seed entries");

        let first = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Tree {
                    path: PathBuf::new(),
                    depth: crate::query::Bound::Limit(1),
                    include_ignored: true,
                    page: crate::PageRequest { limit: 2, max_work: 4 },
                }],
                ..crate::ReadRequest::default()
            })
            .expect("first page");
        let crate::ProjectionResult::Tree(crate::Knowledge::Present(first_page)) =
            &first.results[0]
        else {
            panic!("tree page");
        };
        assert_eq!(
            first_page.rows.iter().map(|row| row.path.as_path()).collect::<Vec<_>>(),
            vec![Path::new("z-dir"), Path::new("a.txt")]
        );
        let continuation = first_page.next.expect("more rows");

        let second = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Continue {
                    continuation,
                    page: crate::PageRequest { limit: 2, max_work: 2 },
                }],
                expected: Some(first.version),
            })
            .expect("second page");
        let crate::ProjectionResult::Tree(crate::Knowledge::Present(second_page)) =
            &second.results[0]
        else {
            panic!("continued tree page");
        };
        assert_eq!(
            second_page.rows.iter().map(|row| row.path.as_path()).collect::<Vec<_>>(),
            vec![Path::new("b.txt")]
        );
        assert!(second_page.next.is_none());
        assert_eq!(first.version, second.version);
        opened.close().expect("close");
    }

    #[test]
    fn flat_pages_follow_complete_portable_path_order_without_rescanning() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("c.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("a.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("b.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs::default(),
                },
            ]))
            .expect("seed entries");

        let first = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Flat {
                    selection: crate::query::EntrySelection::default(),
                    shape: crate::RowShape::Compact,
                    page: crate::PageRequest { limit: 2, max_work: 3 },
                }],
                ..crate::ReadRequest::default()
            })
            .expect("first page");
        let crate::ProjectionResult::Flat(first_page) = &first.results[0] else {
            panic!("flat page");
        };
        assert_eq!(
            first_page.rows.iter().map(|row| row.portable_path.as_str()).collect::<Vec<_>>(),
            vec!["a.txt", "b.txt"]
        );
        let continuation = first_page.next.expect("more rows");

        let second = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Continue {
                    continuation,
                    page: crate::PageRequest { limit: 2, max_work: 2 },
                }],
                expected: Some(first.version),
            })
            .expect("second page");
        let crate::ProjectionResult::Flat(second_page) = &second.results[0] else {
            panic!("continued flat page");
        };
        assert_eq!(
            second_page.rows.iter().map(|row| row.portable_path.as_str()).collect::<Vec<_>>(),
            vec!["c.txt"]
        );
        assert!(second_page.next.is_none());
        assert!(second.work.rows_visited <= 2, "continuation resumed from retained position");
        opened.close().expect("close");
    }

    #[test]
    fn flat_continuation_retains_its_normalized_native_query() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("a.rs"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("b.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("c.rs"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs::default(),
                },
            ]))
            .expect("seed entries");
        let selection = crate::query::EntrySelection {
            query: crate::query::Selection {
                include: vec![crate::query::Pattern::parse("*.rs").expect("pattern")],
                ..crate::query::Selection::default()
            },
            ..crate::query::EntrySelection::default()
        };

        let first = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Flat {
                    selection,
                    shape: crate::RowShape::Compact,
                    page: crate::PageRequest { limit: 1, max_work: 3 },
                }],
                ..crate::ReadRequest::default()
            })
            .expect("first page");
        let crate::ProjectionResult::Flat(first_page) = &first.results[0] else {
            panic!("flat page");
        };
        assert_eq!(first_page.rows[0].portable_path.as_str(), "a.rs");

        let second = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Continue {
                    continuation: first_page.next.expect("continuation"),
                    page: crate::PageRequest { limit: 1, max_work: 1 },
                }],
                expected: Some(first.version),
            })
            .expect("continued page");
        let crate::ProjectionResult::Flat(second_page) = &second.results[0] else {
            panic!("continued flat page");
        };
        assert_eq!(second_page.rows[0].portable_path.as_str(), "c.rs");
        assert!(second_page.next.is_none());
        assert_eq!(second.work.rows_visited, 1);
        opened.close().expect("close");
    }

    #[test]
    fn continuations_are_single_use_version_pinned_handle_local_and_bounded() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("a"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("b"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs::default(),
                },
            ]))
            .expect("seed entries");
        let page = crate::PageRequest { limit: 1, max_work: 2 };
        let new_token = || {
            let response = opened
                .read(crate::ReadRequest {
                    projections: vec![crate::ReadProjection::Flat {
                        selection: crate::query::EntrySelection::default(),
                        shape: crate::RowShape::Compact,
                        page,
                    }],
                    ..crate::ReadRequest::default()
                })
                .expect("first page");
            let crate::ProjectionResult::Flat(result) = &response.results[0] else {
                panic!("flat page");
            };
            result.next.expect("continuation")
        };

        let replay = new_token();
        opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Continue { continuation: replay, page }],
                ..crate::ReadRequest::default()
            })
            .expect("first continuation use");
        assert!(matches!(
            opened.read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Continue { continuation: replay, page }],
                ..crate::ReadRequest::default()
            }),
            Err(Error::ContinuationUnavailable)
        ));

        let stale = new_token();
        opened
            .state
            .index
            .apply(&Observation::new(vec![Op::Upsert {
                path: PathBuf::from("c"),
                kind: EntryKind::File,
                attrs: crate::Attrs::default(),
            }]))
            .expect("advance version");
        assert!(matches!(
            opened.read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Continue { continuation: stale, page }],
                ..crate::ReadRequest::default()
            }),
            Err(Error::ContinuationStale { .. })
        ));

        let retryable = new_token();
        let limited = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Continue {
                    continuation: retryable,
                    page: crate::PageRequest { limit: 1, max_work: 1 },
                }],
                ..crate::ReadRequest::default()
            })
            .expect("bounded continuation");
        assert!(matches!(limited.results[0], crate::ProjectionResult::Limit(_)));
        assert!(matches!(
            opened
                .read(crate::ReadRequest {
                    projections: vec![crate::ReadProjection::Continue {
                        continuation: retryable,
                        page: crate::PageRequest { limit: 1, max_work: 2 },
                    }],
                    ..crate::ReadRequest::default()
                })
                .expect("retry continuation")
                .results[0],
            crate::ProjectionResult::Flat(_)
        ));

        let foreign = new_token();
        let (_other_root, other) = self::opened(Arc::new(TestControls::default()));
        assert!(matches!(
            other.read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Continue { continuation: foreign, page }],
                ..crate::ReadRequest::default()
            }),
            Err(Error::ContinuationUnavailable)
        ));

        let oldest = new_token();
        for _ in 0..super::continuation::MAX_CONTINUATIONS {
            let _ = new_token();
        }
        assert!(matches!(
            opened.read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Continue { continuation: oldest, page }],
                ..crate::ReadRequest::default()
            }),
            Err(Error::ContinuationUnavailable)
        ));
        other.close().expect("close other");
        opened.close().expect("close");
    }

    /// Level order, proved against the sequence pre-order would have produced.
    ///
    /// An order is only proved by a fixture whose answer differs between the plausible
    /// readings, and "parent-first" admits both. This tree is three levels deep and wide
    /// at the top, so the two disagree:
    ///
    /// ```text
    /// a/  a/a1/  a/a1/deep.txt  b/  b/b1/  z.txt
    /// ```
    ///
    /// Level order returns `a`, `b`, `z.txt`, then `a/a1`, `b/b1`, then `a/a1/deep.txt` —
    /// every level whole before descending. Pre-order would return `a`, `a/a1`,
    /// `a/a1/deep.txt`, `b`, `b/b1`, `z.txt`, burying `b` behind the whole of `a`'s
    /// subtree. A page bound cutting the pre-order sequence at three rows would hide the
    /// existence of `b` and `z.txt` entirely, which is what level order prevents.
    #[test]
    fn tree_pages_are_breadth_first_across_levels() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("a"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("b"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("z.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 1, ..crate::Attrs::default() },
                },
                Op::Upsert {
                    path: PathBuf::from("a/a1"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("b/b1"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("a/a1/deep.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 2, ..crate::Attrs::default() },
                },
            ]))
            .expect("seed tree");

        let rows = |depth: crate::query::Bound| -> Vec<String> {
            let response = opened
                .read(crate::ReadRequest {
                    projections: vec![crate::ReadProjection::Tree {
                        path: PathBuf::new(),
                        depth,
                        include_ignored: true,
                        page: crate::PageRequest {
                            limit: crate::MAX_PAGE_ROWS,
                            max_work: crate::MAX_PAGE_WORK,
                        },
                    }],
                    ..crate::ReadRequest::default()
                })
                .expect("tree read");
            let crate::ProjectionResult::Tree(crate::Knowledge::Present(page)) =
                &response.results[0]
            else {
                panic!("tree page");
            };
            page.rows.iter().map(|row| row.portable_path.as_str().to_owned()).collect()
        };

        // One level is this directory's own children: directories first, then files, each
        // partition in canonical byte order.
        assert_eq!(rows(crate::query::Bound::Limit(1)), vec!["a", "b", "z.txt"]);

        // Two levels adds the next level whole, never a subtree at a time.
        assert_eq!(rows(crate::query::Bound::Limit(2)), vec!["a", "b", "z.txt", "a/a1", "b/b1"]);

        // Unbounded reaches the leaf, still level by level. Pre-order would have placed
        // `a/a1` and `a/a1/deep.txt` before `b`.
        assert_eq!(
            rows(crate::query::Bound::All),
            vec!["a", "b", "z.txt", "a/a1", "b/b1", "a/a1/deep.txt"]
        );

        opened.close().expect("close");
    }

    /// Paging a multi-level tree one row at a time reassembles the same sequence.
    ///
    /// Resumption is where a level-order traversal can go wrong invisibly: the cursor
    /// holds one frame, and crossing a level boundary means re-deriving the position from
    /// the ancestor chain. A page bound that lands exactly on such a boundary is the case
    /// that would duplicate or drop a row, so this walks every boundary in the fixture.
    #[test]
    fn tree_pages_resume_across_level_boundaries() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("a"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("b"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("z.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 1, ..crate::Attrs::default() },
                },
                Op::Upsert {
                    path: PathBuf::from("a/a1"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("b/b1"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("a/a1/deep.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 2, ..crate::Attrs::default() },
                },
            ]))
            .expect("seed tree");

        let whole = vec!["a", "b", "z.txt", "a/a1", "b/b1", "a/a1/deep.txt"];
        let page = crate::PageRequest { limit: 1, max_work: crate::MAX_PAGE_WORK };

        let mut seen: Vec<String> = Vec::new();
        let response = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Tree {
                    path: PathBuf::new(),
                    depth: crate::query::Bound::All,
                    include_ignored: true,
                    page,
                }],
                ..crate::ReadRequest::default()
            })
            .expect("first page");
        let crate::ProjectionResult::Tree(crate::Knowledge::Present(first)) = &response.results[0]
        else {
            panic!("tree page");
        };
        seen.extend(first.rows.iter().map(|row| row.portable_path.as_str().to_owned()));
        let mut continuation = first.next;

        while let Some(token) = continuation {
            let response = opened
                .read(crate::ReadRequest {
                    projections: vec![crate::ReadProjection::Continue {
                        continuation: token,
                        page,
                    }],
                    ..crate::ReadRequest::default()
                })
                .expect("resumed page");
            let crate::ProjectionResult::Tree(crate::Knowledge::Present(next)) =
                &response.results[0]
            else {
                panic!("tree page");
            };
            seen.extend(next.rows.iter().map(|row| row.portable_path.as_str().to_owned()));
            continuation = next.next;
            assert!(seen.len() <= whole.len(), "paging must terminate, saw {seen:?}");
        }

        assert_eq!(seen, whole, "one row at a time reassembles the single-page order");
        opened.close().expect("close");
    }

    /// A page the work budget stops must hand back a continuation.
    ///
    /// The row limit and the work budget are different stopping conditions, and only the
    /// row limit is reached inside `collect_children`, which knows the exact child it
    /// stopped at. The budget can also run out while *advancing* between parents, where
    /// no row has been reached to point at. Breaking there returns `next: None`, which a
    /// caller cannot tell apart from a traversal that finished — the tree simply comes
    /// back missing every level below the one that fit.
    ///
    /// Sweeping the budget rather than naming one keeps this from testing an arithmetic
    /// coincidence: every budget large enough to make progress must reassemble the whole
    /// tree, whichever of the two conditions happens to stop each page.
    #[test]
    fn a_tree_page_stopped_by_the_work_budget_is_resumable() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("a"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("b"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("z.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 1, ..crate::Attrs::default() },
                },
                Op::Upsert {
                    path: PathBuf::from("a/a1"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("b/b1"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("a/a1/deep.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 2, ..crate::Attrs::default() },
                },
            ]))
            .expect("seed tree");

        let whole = ["a", "b", "z.txt", "a/a1", "b/b1", "a/a1/deep.txt"];

        // Two is the smallest budget that can still afford a row after the path walk, so
        // it is the smallest at which paging is obliged to make progress at all.
        for max_work in 2..=14_u64 {
            let page = crate::PageRequest { limit: crate::MAX_PAGE_ROWS, max_work };
            let mut seen: Vec<String> = Vec::new();
            let mut continuation = None;
            let mut pages = 0;

            loop {
                let projection = match continuation {
                    None => crate::ReadProjection::Tree {
                        path: PathBuf::new(),
                        depth: crate::query::Bound::All,
                        include_ignored: true,
                        page,
                    },
                    Some(token) => crate::ReadProjection::Continue { continuation: token, page },
                };
                let response = opened
                    .read(crate::ReadRequest {
                        projections: vec![projection],
                        ..crate::ReadRequest::default()
                    })
                    .expect("tree read");
                let current = match &response.results[0] {
                    crate::ProjectionResult::Tree(crate::Knowledge::Present(page)) => page,
                    // A limit here would mean a budget the tree cannot be read at, and
                    // re-asking cannot help: the search that overran would restart and
                    // overrun again. Every budget that can hold a row must finish.
                    other => panic!("unexpected result at budget {max_work}: {other:?}"),
                };
                seen.extend(current.rows.iter().map(|row| row.portable_path.as_str().to_owned()));
                continuation = current.next;
                pages += 1;
                assert!(
                    pages <= whole.len() * 4 + 16,
                    "budget {max_work} never finished paging, saw {seen:?}"
                );
                if continuation.is_none() {
                    break;
                }
            }

            let mut sorted = seen.clone();
            sorted.sort();
            let mut expected: Vec<String> = whole.iter().map(|row| (*row).to_owned()).collect();
            expected.sort();
            assert_eq!(
                sorted, expected,
                "budget {max_work} finished with next: None while missing rows; saw {seen:?}"
            );
        }

        opened.close().expect("close");
    }

    /// A page must move, even when the path walk has already spent the budget.
    ///
    /// `spent` starts at the cost of walking to the requested directory, and only a walk
    /// strictly longer than the budget is refused outright. At exactly the budget the walk
    /// is allowed, and then the first child pushes `spent` over before any row is emitted:
    /// the page returns no rows and a cursor pointing at that same child, and resuming
    /// reproduces it exactly. The bound stops being "how much work per page" and becomes
    /// "no page ever finishes".
    ///
    /// A budget says where to stop, not whether to start. Every page therefore emits at
    /// least one row or ends the traversal, and this reads a nested directory so the path
    /// walk is expensive enough to collide with the budget at all.
    #[test]
    fn a_page_moves_even_when_the_path_walk_spends_the_budget() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("a"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("a/b"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("a/b/x.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 1, ..crate::Attrs::default() },
                },
                Op::Upsert {
                    path: PathBuf::from("a/b/y.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 2, ..crate::Attrs::default() },
                },
            ]))
            .expect("seed tree");

        // Walking to `a/b` costs three, so three is the budget that is spent on arrival.
        // Sweeping upward from it keeps this a property rather than one arithmetic
        // coincidence: every budget the request is allowed to make must terminate.
        for max_work in 3..=12_u64 {
            let page = crate::PageRequest { limit: crate::MAX_PAGE_ROWS, max_work };
            let mut seen: Vec<String> = Vec::new();
            let mut continuation = None;
            let mut pages = 0;

            loop {
                let projection = match continuation {
                    None => crate::ReadProjection::Tree {
                        path: PathBuf::from("a/b"),
                        depth: crate::query::Bound::All,
                        include_ignored: true,
                        page,
                    },
                    Some(token) => crate::ReadProjection::Continue { continuation: token, page },
                };
                let response = opened
                    .read(crate::ReadRequest {
                        projections: vec![projection],
                        ..crate::ReadRequest::default()
                    })
                    .expect("tree read");
                let crate::ProjectionResult::Tree(crate::Knowledge::Present(current)) =
                    &response.results[0]
                else {
                    panic!("tree page");
                };
                seen.extend(current.rows.iter().map(|row| row.portable_path.as_str().to_owned()));
                continuation = current.next;
                pages += 1;
                assert!(
                    pages <= 16,
                    "budget {max_work} never terminated; after {pages} pages saw {seen:?}"
                );
                if continuation.is_none() {
                    break;
                }
            }

            assert_eq!(seen, vec!["a/b/x.txt", "a/b/y.txt"], "budget {max_work} lost rows");
        }
        opened.close().expect("close");
    }

    /// Descending must not scan the level it is leaving.
    ///
    /// A level of leaf directories is the shape of every tree's last level, and searching
    /// it for a directory child asks every parent in order to conclude there is nothing
    /// below. Noticing the first directory while the level is emitted answers the same
    /// question for free, so the work a page reports has to stay proportional to the rows
    /// it returns rather than to the width of the level under it.
    #[test]
    fn descending_costs_nothing_on_a_level_of_leaves() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        let mut ops = Vec::new();
        for index in 0..60 {
            ops.push(Op::Upsert {
                path: PathBuf::from(format!("d{index:03}")),
                kind: EntryKind::Dir,
                attrs: crate::Attrs::default(),
            });
            ops.push(Op::Upsert {
                path: PathBuf::from(format!("d{index:03}/leaf.txt")),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 1, ..crate::Attrs::default() },
            });
        }
        opened.state.index.apply(&Observation::new(ops)).expect("seed wide leaf level");

        let response = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Tree {
                    path: PathBuf::new(),
                    depth: crate::query::Bound::All,
                    include_ignored: true,
                    page: crate::PageRequest {
                        limit: crate::MAX_PAGE_ROWS,
                        max_work: crate::MAX_PAGE_WORK,
                    },
                }],
                ..crate::ReadRequest::default()
            })
            .expect("tree read");
        let crate::ProjectionResult::Tree(crate::Knowledge::Present(page)) = &response.results[0]
        else {
            panic!("tree page");
        };
        assert_eq!(page.rows.len(), 120, "60 directories and their 60 files");
        assert!(page.next.is_none(), "one page holds the whole tree");
        // Three steps per directory and no more: emit the directory as a row at level
        // one, emit its file as a row at level two, and step past it to its sibling. The
        // slack covers the path walk and the two advances that end each level.
        //
        // Searching for the descent instead of remembering it adds a fourth step per
        // directory, because it asks every one of them for a directory child before
        // concluding there is no level below. That is what this bound rejects: measured,
        // it is 180 steps with the memo and 241 without, for the same 121 rows.
        let width = 60;
        assert!(
            response.work.rows_visited <= 3 * width + 10,
            "descent scanned the level it was leaving: {} steps for {} rows",
            response.work.rows_visited,
            response.work.rows_returned
        );
        opened.close().expect("close");
    }

    /// Excluding ignored entries prunes the subtree, not merely the row.
    ///
    /// Filtering the row and descending anyway is an equally reasonable reading of an
    /// unstated rule, and it is observably different: it would still return
    /// `vendor/keep.txt` while hiding the directory that explains where it came from.
    // The ignore partition is only populated when the feature that reads control files is
    // compiled in; without it nothing is ignored and the fixture cannot express the case.
    #[cfg(feature = "gitignore")]
    #[test]
    fn excluding_ignored_prunes_the_subtree() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("src"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("src/main.rs"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 1, ..crate::Attrs::default() },
                },
                Op::ControlUpsert {
                    path: PathBuf::from(".gitignore"),
                    source: b"vendor/\n".to_vec(),
                },
                Op::Upsert {
                    path: PathBuf::from("vendor"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("vendor/keep.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 2, ..crate::Attrs::default() },
                },
            ]))
            .expect("seed tree");

        let rows = |include_ignored: bool| -> Vec<String> {
            let response = opened
                .read(crate::ReadRequest {
                    projections: vec![crate::ReadProjection::Tree {
                        path: PathBuf::new(),
                        depth: crate::query::Bound::All,
                        include_ignored,
                        page: crate::PageRequest {
                            limit: crate::MAX_PAGE_ROWS,
                            max_work: crate::MAX_PAGE_WORK,
                        },
                    }],
                    ..crate::ReadRequest::default()
                })
                .expect("tree read");
            let crate::ProjectionResult::Tree(crate::Knowledge::Present(page)) =
                &response.results[0]
            else {
                panic!("tree page");
            };
            page.rows.iter().map(|row| row.portable_path.as_str().to_owned()).collect()
        };

        let included = rows(true);
        assert!(included.iter().any(|row| row == "vendor"));
        assert!(included.iter().any(|row| row == "vendor/keep.txt"));

        let excluded = rows(false);
        assert!(!excluded.iter().any(|row| row == "vendor"), "the row is gone");
        assert!(
            !excluded.iter().any(|row| row == "vendor/keep.txt"),
            "and so is everything beneath it, which is what pruning means"
        );
        assert!(excluded.iter().any(|row| row == "src/main.rs"), "unignored work is untouched");

        opened.close().expect("close");
    }

    /// The remembered descent must be the first *unpruned* directory, not the first one.
    ///
    /// Noticing the next level's first parent while emitting is only equivalent to
    /// searching for it if both apply the same pruning rule. `first_directory_child`
    /// skips ignored directories, so remembering one before the ignore check makes the
    /// two disagree and hands the traversal a parent the search would never have chosen.
    ///
    /// No row leaks when that happens — every child of a pruned directory is itself
    /// ignored, so the row filter catches them a second time. Which is the point: the
    /// mistake is invisible in the output and visible only in the work, and a fixture
    /// that checked rows alone would pass either way. Pruning means an excluded
    /// directory is never expanded; being saved by a second filter is not pruning.
    ///
    /// So the ignored directory sorts first and is given enough children that expanding
    /// it cannot hide in the noise.
    #[cfg(feature = "gitignore")]
    #[test]
    fn the_remembered_descent_skips_a_pruned_first_child() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::ControlUpsert {
                    path: PathBuf::from(".gitignore"),
                    source: b"aaa_vendor/\n".to_vec(),
                },
                Op::Upsert {
                    path: PathBuf::from("src"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("src/aaa_vendor"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("src/bbb_keep"),
                    kind: EntryKind::Dir,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("src/bbb_keep/kept.txt"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 2, ..crate::Attrs::default() },
                },
            ]))
            .expect("seed tree");
        let buried: Vec<Op> = (0..40)
            .map(|index| Op::Upsert {
                path: PathBuf::from(format!("src/aaa_vendor/hidden{index:03}.txt")),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 1, ..crate::Attrs::default() },
            })
            .collect();
        opened.state.index.apply(&Observation::new(buried)).expect("seed the pruned subtree");

        let response = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Tree {
                    path: PathBuf::new(),
                    depth: crate::query::Bound::All,
                    include_ignored: false,
                    page: crate::PageRequest {
                        limit: crate::MAX_PAGE_ROWS,
                        max_work: crate::MAX_PAGE_WORK,
                    },
                }],
                ..crate::ReadRequest::default()
            })
            .expect("tree read");
        let crate::ProjectionResult::Tree(crate::Knowledge::Present(page)) = &response.results[0]
        else {
            panic!("tree page");
        };
        let rows: Vec<String> =
            page.rows.iter().map(|row| row.portable_path.as_str().to_owned()).collect();

        assert!(rows.iter().any(|row| row == "src/bbb_keep"), "the kept directory is listed");
        assert!(
            rows.iter().any(|row| row == "src/bbb_keep/kept.txt"),
            "and the descent reached the level below it"
        );
        assert!(
            !rows.iter().any(|row| row == "src/aaa_vendor"),
            "the pruned directory is not a row"
        );
        assert!(
            !rows.iter().any(|row| row.starts_with("src/aaa_vendor/")),
            "and nothing beneath it is listed"
        );

        // The load-bearing assertion. Expanding the pruned directory charges a step for
        // each of its forty children before discarding every one of them, so the work
        // separates a remembered descent that prunes from one that does not, where the
        // rows above cannot.
        assert!(
            response.work.rows_visited < 40,
            "the pruned subtree was expanded: {} steps for {} rows",
            response.work.rows_visited,
            response.work.rows_returned
        );

        opened.close().expect("close");
    }

    /// A name whose bytes are not UTF-8 is escaped and listed, not omitted.
    ///
    /// This fixture used to prove the opposite. While a portable name was optional these
    /// two entries were retained, counted in roll-ups, and absent from every page, and
    /// the page reported an omission count with escaped examples so the loss was at least
    /// visible. It also meant a lookup below such a directory had to answer `unknown`
    /// rather than `absent`, because the name asked for might have been in the invisible
    /// set.
    ///
    /// The encoding is total now, so the same fixture must show the opposite: both rows
    /// appear, ordered pages and roll-ups agree on the population, and absence is
    /// answerable. `x\xff` becomes `x%FF`; the valid prefix survives as text and only the
    /// undecodable byte is escaped.
    #[cfg(unix)]
    #[test]
    fn non_utf8_names_are_escaped_into_pages_rather_than_omitted() {
        use std::os::unix::ffi::OsStringExt;

        let (_root, opened) = opened(Arc::new(TestControls::default()));
        let invalid = PathBuf::from(OsString::from_vec(vec![b'x', 0xff]));
        // A literal `%` beside an escaped byte is the pair that proves injectivity: if
        // `%` were left alone, a file actually named `y%FE` and this one would collide.
        let literal_percent = PathBuf::from("y%FE");
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: invalid.clone(),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 9, ..crate::Attrs::default() },
                },
                Op::Upsert {
                    path: literal_percent.clone(),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 1, ..crate::Attrs::default() },
                },
            ]))
            .expect("seed non-utf8 and literal-percent entries");

        let response = opened
            .read(crate::ReadRequest {
                projections: vec![
                    crate::ReadProjection::Lookup { path: PathBuf::from("missing") },
                    crate::ReadProjection::Tree {
                        path: PathBuf::new(),
                        depth: crate::query::Bound::Limit(1),
                        include_ignored: true,
                        page: crate::PageRequest {
                            limit: crate::MAX_PAGE_ROWS,
                            max_work: crate::MAX_PAGE_WORK,
                        },
                    },
                    crate::ReadProjection::Flat {
                        selection: crate::query::EntrySelection::default(),
                        shape: crate::RowShape::Compact,
                        page: crate::PageRequest {
                            limit: crate::MAX_PAGE_ROWS,
                            max_work: crate::MAX_PAGE_WORK,
                        },
                    },
                    crate::ReadProjection::RollUp { path: PathBuf::new() },
                ],
                ..crate::ReadRequest::default()
            })
            .expect("portable read");

        // Absence is answerable: nothing can be hiding in an unlistable set any more.
        assert!(matches!(
            response.results[0],
            crate::ProjectionResult::Lookup(crate::Knowledge::Absent)
        ));

        let crate::ProjectionResult::Tree(crate::Knowledge::Present(tree)) = &response.results[1]
        else {
            panic!("tree page");
        };
        assert!(tree.complete);
        let names: Vec<_> =
            tree.rows.iter().map(|row| row.portable_path.as_str().to_owned()).collect();
        assert_eq!(names, vec!["x%FF".to_owned(), "y%25FE".to_owned()]);

        let crate::ProjectionResult::Flat(flat) = &response.results[2] else {
            panic!("flat page");
        };
        let flat_names: Vec<_> =
            flat.rows.iter().map(|row| row.portable_path.as_str().to_owned()).collect();
        assert_eq!(flat_names, names, "ordered pages agree on one population");

        // The population the pages return is the population the roll-up counts.
        assert!(matches!(
            &response.results[3],
            crate::ProjectionResult::RollUp(crate::Knowledge::Present(rollup))
                if rollup.all.files == 2
                    && rollup.all.files
                        == u64::try_from(flat.rows.len()).expect("row count fits u64")
        ));

        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Remove { path: invalid },
                Op::Remove { path: literal_percent },
            ]))
            .expect("remove escaped entries");
        let after = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Tree {
                    path: PathBuf::new(),
                    depth: crate::query::Bound::Limit(1),
                    include_ignored: true,
                    page: crate::PageRequest {
                        limit: crate::MAX_PAGE_ROWS,
                        max_work: crate::MAX_PAGE_WORK,
                    },
                }],
                ..crate::ReadRequest::default()
            })
            .expect("read after removal");
        let crate::ProjectionResult::Tree(crate::Knowledge::Present(tree)) = &after.results[0]
        else {
            panic!("tree page");
        };
        assert!(tree.rows.is_empty(), "removal is symmetric for escaped names too");
        opened.close().expect("close");
    }

    #[test]
    fn read_bounds_are_validated_before_any_continuation_is_consumed() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("a"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("b"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs::default(),
                },
            ]))
            .expect("seed entries");
        let first = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Flat {
                    selection: crate::query::EntrySelection::default(),
                    shape: crate::RowShape::Compact,
                    page: crate::PageRequest { limit: 1, max_work: 2 },
                }],
                ..crate::ReadRequest::default()
            })
            .expect("first page");
        let crate::ProjectionResult::Flat(page) = &first.results[0] else {
            panic!("flat page");
        };
        let continuation = page.next.expect("continuation");

        assert!(matches!(
            opened.read(crate::ReadRequest {
                projections: vec![
                    crate::ReadProjection::Continue {
                        continuation,
                        page: crate::PageRequest { limit: 1, max_work: 2 },
                    },
                    crate::ReadProjection::Tree {
                        path: PathBuf::new(),
                        depth: crate::query::Bound::Limit(1),
                        include_ignored: true,
                        page: crate::PageRequest { limit: 0, max_work: 1 },
                    },
                ],
                ..crate::ReadRequest::default()
            }),
            Err(Error::PageRowLimit { attempted: 0, .. })
        ));
        opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Continue {
                    continuation,
                    page: crate::PageRequest { limit: 1, max_work: 2 },
                }],
                ..crate::ReadRequest::default()
            })
            .expect("validation preserved continuation");

        assert!(matches!(
            opened.read(crate::ReadRequest {
                projections: vec![
                    crate::ReadProjection::Diagnostics;
                    crate::MAX_READ_PROJECTIONS + 1
                ],
                ..crate::ReadRequest::default()
            }),
            Err(Error::ReadProjectionLimit { .. })
        ));
        assert!(matches!(
            opened.read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Aggregate {
                    selection: crate::query::EntrySelection::default(),
                    count_cap: 0,
                    max_work: 1,
                }],
                ..crate::ReadRequest::default()
            }),
            Err(Error::CountCapLimit { attempted: 0, .. })
        ));
        // Zero levels is its own rejection, not a row-bound one. It once reported
        // `PageRowLimit { attempted: 0 }`, which named a bound the caller had not set and
        // sent them to inspect `page.limit` instead of `depth`.
        assert!(matches!(
            opened.read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Tree {
                    path: PathBuf::new(),
                    depth: crate::query::Bound::Limit(0),
                    include_ignored: true,
                    page: crate::PageRequest { limit: 1, max_work: 1 },
                }],
                ..crate::ReadRequest::default()
            }),
            Err(Error::TreeDepthZero)
        ));
        let bounded_path = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Tree {
                    path: PathBuf::from("missing/deep"),
                    depth: crate::query::Bound::Limit(1),
                    include_ignored: true,
                    page: crate::PageRequest { limit: 1, max_work: 1 },
                }],
                ..crate::ReadRequest::default()
            })
            .expect("bounded path traversal");
        assert!(matches!(
            bounded_path.results[0],
            crate::ProjectionResult::Limit(crate::QueryLimit {
                projection: crate::LimitedProjection::Tree,
                rows_visited: 1,
                ..
            })
        ));
        assert!(matches!(
            opened.read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Report(crate::ReportRequest {
                    query: crate::query::Query {
                        views: vec![crate::query::ViewSpec::Summary; crate::MAX_REPORT_VIEWS + 1],
                        ..crate::query::Query::default()
                    },
                    generated_at: std::time::UNIX_EPOCH,
                    max_work: crate::MAX_PAGE_WORK,
                })],
                ..crate::ReadRequest::default()
            }),
            Err(Error::ReportViewLimit { .. })
        ));
        opened.close().expect("close");
    }

    #[test]
    fn a_coherent_read_cannot_straddle_a_commit() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let writer_stop = Arc::clone(&stop);
        let writer_index = opened.state.index.clone();
        let writer = std::thread::spawn(move || {
            for round in 0..400_u64 {
                if writer_stop.load(Ordering::Relaxed) {
                    break;
                }
                writer_index
                    .apply(&Observation::new(vec![Op::Upsert {
                        path: PathBuf::from(format!("file-{round}")),
                        kind: EntryKind::File,
                        attrs: crate::Attrs { size: 1, ..crate::Attrs::default() },
                    }]))
                    .expect("writer commit");
            }
        });

        for _ in 0..500 {
            let response = opened
                .read(crate::ReadRequest {
                    projections: vec![
                        crate::ReadProjection::Tree {
                            path: PathBuf::new(),
                            depth: crate::query::Bound::Limit(1),
                            include_ignored: true,
                            page: crate::PageRequest {
                                limit: crate::MAX_PAGE_ROWS,
                                max_work: crate::MAX_PAGE_WORK,
                            },
                        },
                        crate::ReadProjection::RollUp { path: PathBuf::new() },
                    ],
                    ..crate::ReadRequest::default()
                })
                .expect("coherent read");
            let crate::ProjectionResult::Tree(crate::Knowledge::Present(tree)) =
                &response.results[0]
            else {
                panic!("tree page");
            };
            let crate::ProjectionResult::RollUp(crate::Knowledge::Present(rollup)) =
                &response.results[1]
            else {
                panic!("root roll-up");
            };
            assert!(tree.next.is_none());
            assert_eq!(
                tree.rows.iter().filter(|row| row.kind == EntryKind::File).count() as u64,
                rollup.all.files
            );
        }
        stop.store(true, Ordering::Relaxed);
        writer.join().expect("writer");
        opened.close().expect("close");
    }

    #[test]
    fn aggregates_distinguish_maintained_exact_totals_from_capped_counts() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(
                ["a", "b", "c"]
                    .into_iter()
                    .map(|path| Op::Upsert {
                        path: PathBuf::from(path),
                        kind: EntryKind::File,
                        attrs: crate::Attrs::default(),
                    })
                    .collect(),
            ))
            .expect("seed entries");

        let response = opened
            .read(crate::ReadRequest {
                projections: vec![
                    crate::ReadProjection::Aggregate {
                        selection: crate::query::EntrySelection::default(),
                        count_cap: 1,
                        max_work: 1,
                    },
                    crate::ReadProjection::Aggregate {
                        selection: crate::query::EntrySelection {
                            query: crate::query::Selection {
                                kinds: vec![EntryKind::File],
                                ..crate::query::Selection::default()
                            },
                            ..crate::query::EntrySelection::default()
                        },
                        count_cap: 2,
                        max_work: 3,
                    },
                ],
                ..crate::ReadRequest::default()
            })
            .expect("aggregate read");

        assert!(matches!(
            response.results[0],
            crate::ProjectionResult::Aggregate(crate::CountResult::Exact(3))
        ));
        assert!(matches!(
            response.results[1],
            crate::ProjectionResult::Aggregate(crate::CountResult::AtLeast(2))
        ));
        opened.close().expect("close");
    }

    #[test]
    fn report_projection_matches_the_existing_query_and_fails_closed_at_its_work_bound() {
        let (_root, opened) = opened(Arc::new(TestControls::default()));
        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Upsert {
                    path: PathBuf::from("a"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 3, ..crate::Attrs::default() },
                },
                Op::Upsert {
                    path: PathBuf::from("b"),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 5, ..crate::Attrs::default() },
                },
            ]))
            .expect("seed entries");
        let query = crate::query::Query {
            views: vec![crate::query::ViewSpec::Summary],
            ..crate::query::Query::default()
        };

        let response = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Report(crate::ReportRequest {
                    query: query.clone(),
                    generated_at: std::time::UNIX_EPOCH,
                    max_work: 1,
                })],
                ..crate::ReadRequest::default()
            })
            .expect("maintained report");
        let crate::ProjectionResult::Report(report) = &response.results[0] else {
            panic!("report projection");
        };
        let crate::query::Section::Summary(summary) = &report.sections[0] else {
            panic!("summary section");
        };
        assert_eq!((summary.files, summary.bytes), (2, 8));

        let limited = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Report(crate::ReportRequest {
                    query: crate::query::Query {
                        selection: crate::query::Selection {
                            kinds: vec![EntryKind::File],
                            ..crate::query::Selection::default()
                        },
                        views: vec![crate::query::ViewSpec::Summary],
                        ..crate::query::Query::default()
                    },
                    generated_at: std::time::UNIX_EPOCH,
                    max_work: 1,
                })],
                ..crate::ReadRequest::default()
            })
            .expect("bounded report");
        assert!(matches!(
            limited.results[0],
            crate::ProjectionResult::Limit(crate::QueryLimit {
                projection: crate::LimitedProjection::Report,
                ..
            })
        ));
        opened.close().expect("close");
    }

    #[test]
    fn first_refusal_stops_expansion_and_commits_the_budget_state_with_prior_facts() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::create_dir(root.path().join("nested")).expect("fixture directory");
        std::fs::write(root.path().join("one"), b"1").expect("fixture");
        std::fs::write(root.path().join("two"), b"2").expect("fixture");
        std::fs::write(root.path().join("nested/deep"), b"deep").expect("deep fixture");
        let opened = OpenedIndex::open(
            root.path(),
            OpenOptions {
                batch_size: 64,
                budget: DiscoveryBudget { max_files: Some(1) },
                ..OpenOptions::default()
            },
        )
        .expect("opened root");

        let state = wait_until_settled(&opened);
        assert_eq!(state.phase, crate::LifecyclePhase::Stopped);
        assert_eq!(state.coverage, crate::Coverage::Partial(crate::CoverageReason::Budget));
        assert_eq!(state.progress.files_retained, 1);
        assert_eq!(opened.state.index.total().expect("total").files, 1);
        assert_eq!(opened.state.index.kind(Path::new("nested/deep")).expect("deep lookup"), None);
        assert_eq!(
            opened.state.index.directory_complete(Path::new("")).expect("root completeness"),
            Some(false)
        );
        let partial = opened.state.index.snapshot().expect("partial snapshot image");
        assert!(matches!(
            crate::snapshot::save(&partial, &root.path().join("partial.fdu")),
            Err(Error::Snapshot(_))
        ));
        assert!(matches!(
            opened.prioritize(&[PathBuf::from("nested")]),
            Err(Error::OpenedIndexStopped)
        ));

        let terminal = opened
            .state
            .index
            .since(crate::Clock::ZERO)
            .expect("journal")
            .commits
            .into_iter()
            .find(|commit| {
                commit.state.iter().any(|transition| {
                    matches!(
                        transition,
                        crate::StateTransition::IndexState {
                            current: crate::IndexState {
                                coverage: crate::Coverage::Partial(crate::CoverageReason::Budget),
                                ..
                            },
                            ..
                        }
                    )
                })
            })
            .expect("budget commit");
        assert!(terminal.changes.iter().any(|change| matches!(
            change,
            crate::EffectiveChange::Inserted { kind: EntryKind::File, .. }
        )));
        opened.close().expect("close");
    }

    #[test]
    fn refresh_receipt_counts_verified_no_op_work_without_a_fact_commit() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::write(root.path().join("stable.txt"), b"stable").expect("fixture");
        let opened = OpenedIndex::open(root.path(), OpenOptions::default()).expect("opened root");
        assert_eq!(wait_until_settled(&opened).phase, crate::LifecyclePhase::Ready);

        let result = opened.refresh(&[PathBuf::from("stable.txt")]).expect("refresh");

        assert_eq!(result.accepted, vec![PathBuf::from("stable.txt")]);
        assert!(result.rejected.is_empty());
        assert_eq!(result.work.observations, 1, "the verified observation is still work");
        assert_eq!(result.work.unchanged, 1, "the matching fact is reported as unchanged");
        assert_eq!(result.work.stale, 0);
        opened.close().expect("close");
    }

    #[test]
    fn refresh_can_fill_remaining_file_budget_without_exceeding_it() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::write(root.path().join("one"), b"1").expect("fixture");
        let opened = OpenedIndex::open(
            root.path(),
            OpenOptions {
                budget: DiscoveryBudget { max_files: Some(2) },
                ..OpenOptions::default()
            },
        )
        .expect("opened root");
        assert_eq!(wait_until_settled(&opened).progress.files_retained, 1);
        std::fs::write(root.path().join("two"), b"2").expect("new file");

        let result = opened.refresh(&[PathBuf::from("two")]).expect("refresh");

        assert_eq!(result.accepted, vec![PathBuf::from("two")]);
        assert!(result.rejected.is_empty());
        assert_eq!(opened.state.index.total().expect("total").files, 2);
        assert_eq!(result.state.phase, crate::LifecyclePhase::Ready);
        assert_eq!(result.state.progress.files_retained, 2);
        opened.close().expect("close");
    }

    #[test]
    fn refresh_classifies_paths_and_collapses_overlapping_walks() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::create_dir_all(root.path().join("visible/nested")).expect("fixture directories");
        std::fs::write(root.path().join("visible/nested/leaf"), b"leaf").expect("fixture");
        std::fs::create_dir(root.path().join(".hidden")).expect("hidden directory");
        std::fs::write(root.path().join(".hidden/leaf"), b"hidden").expect("hidden fixture");
        let opened = OpenedIndex::open(
            root.path(),
            OpenOptions {
                hidden: Some(Arc::new(crate::HiddenPolicy::prune_hidden::<[&str; 0], &str>([]))),
                ..OpenOptions::default()
            },
        )
        .expect("opened root");
        assert_eq!(wait_until_settled(&opened).phase, crate::LifecyclePhase::Ready);

        let result = opened
            .refresh(&[
                PathBuf::from("visible/nested"),
                PathBuf::from("visible"),
                PathBuf::from("visible/nested"),
                PathBuf::from("../escape"),
                PathBuf::from(".hidden/leaf"),
            ])
            .expect("refresh");

        assert_eq!(
            result.accepted,
            vec![PathBuf::from("visible"), PathBuf::from("visible/nested")]
        );
        assert_eq!(
            result.rejected,
            vec![
                crate::RejectedRefreshPath {
                    path: PathBuf::from("../escape"),
                    reason: crate::RefreshRejection::OutsideRoot,
                },
                crate::RejectedRefreshPath {
                    path: PathBuf::from(".hidden/leaf"),
                    reason: crate::RefreshRejection::NotAdmitted,
                },
            ]
        );
        assert_eq!(result.work.directories_read, 2, "the descendant was not walked twice");
        opened.close().expect("close");
    }

    #[test]
    fn refresh_widens_through_a_replaced_ancestor_and_reports_exact_commits() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::create_dir_all(root.path().join("parent/child")).expect("fixture directories");
        std::fs::write(root.path().join("parent/child/leaf"), b"leaf").expect("fixture");
        let opened = OpenedIndex::open(root.path(), OpenOptions::default()).expect("opened root");
        assert_eq!(wait_until_settled(&opened).phase, crate::LifecyclePhase::Ready);
        std::fs::remove_dir_all(root.path().join("parent")).expect("remove old subtree");
        std::fs::write(root.path().join("parent"), b"replacement").expect("replacement file");
        let before = current_version(&opened);

        let result =
            opened.refresh(&[PathBuf::from("parent/child/leaf")]).expect("refresh widened path");
        let poll = opened
            .changes(crate::ChangeRequest { after: before, timeout: std::time::Duration::ZERO })
            .expect("refresh commits");

        assert_eq!(result.after, before);
        assert_eq!(result.version, poll.version);
        assert_eq!(result.accepted, vec![PathBuf::from("parent/child/leaf")]);
        assert_eq!(
            opened.state.index.kind(Path::new("parent")).expect("kind"),
            Some(EntryKind::File)
        );
        assert_eq!(
            opened.state.index.kind(Path::new("parent/child/leaf")).expect("removed child"),
            None
        );
        let crate::ChangeOutcome::Changes { commits, impact } = poll.outcome else {
            panic!("refresh must advance the journal");
        };
        assert!(!commits.is_empty());
        assert_eq!(impact, result.impact);
        assert!(commits.iter().all(|commit| {
            commit.clock.0 > result.after.sequence.0 && commit.clock.0 <= result.version.sequence.0
        }));
        opened.close().expect("close");
    }

    #[cfg(unix)]
    #[test]
    fn refresh_rejects_symlink_shadowed_ancestry_without_aborting_other_paths() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().expect("temp root");
        let outside = tempfile::tempdir().expect("outside root");
        std::fs::create_dir_all(root.path().join("shadow/child")).expect("baseline ancestry");
        std::fs::write(root.path().join("good.txt"), b"before").expect("baseline file");
        let opened = OpenedIndex::open(root.path(), OpenOptions::default()).expect("opened root");
        assert_eq!(wait_until_settled(&opened).phase, crate::LifecyclePhase::Ready);

        std::fs::remove_dir_all(root.path().join("shadow")).expect("remove ancestry");
        symlink(outside.path(), root.path().join("shadow")).expect("shadow with symlink");
        std::fs::write(root.path().join("good.txt"), b"after and larger").expect("mutate file");

        let result = opened
            .refresh(&[PathBuf::from("shadow/child/leaf"), PathBuf::from("good.txt")])
            .expect("one unsafe path is a rejection, not a batch error");

        assert_eq!(result.accepted, vec![PathBuf::from("good.txt")]);
        assert_eq!(result.rejected.len(), 1);
        assert_eq!(result.rejected[0].path, Path::new("shadow/child/leaf"));
        assert_eq!(result.rejected[0].reason, crate::RefreshRejection::UnsafeAncestry);
        assert_eq!(
            opened.state.index.attrs(Path::new("good.txt")).expect("attrs").expect("retained").size,
            16
        );
        opened.close().expect("close");
    }

    #[test]
    fn refresh_refusal_is_atomic_with_the_shared_file_budget() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::write(root.path().join("one"), b"1").expect("fixture");
        let opened = OpenedIndex::open(
            root.path(),
            OpenOptions {
                budget: DiscoveryBudget { max_files: Some(2) },
                ..OpenOptions::default()
            },
        )
        .expect("opened root");
        assert_eq!(wait_until_settled(&opened).progress.files_retained, 1);
        std::fs::write(root.path().join("two"), b"2").expect("new file");
        std::fs::write(root.path().join("three"), b"3").expect("new file");

        let result = opened
            .refresh(&[PathBuf::from("two"), PathBuf::from("three")])
            .expect("bounded refresh");

        assert_eq!(result.accepted.len(), 2);
        assert!(result.rejected.is_empty());
        assert_eq!(result.work.observations, 2);
        assert_eq!(result.work.resource_refused, 1);
        assert_eq!(opened.state.index.total().expect("total").files, 2);
        assert_eq!(result.state.progress.files_retained, 2);
        assert_eq!(result.state.phase, crate::LifecyclePhase::Stopped);
        assert_eq!(result.state.coverage, crate::Coverage::Partial(crate::CoverageReason::Budget));
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].kind, crate::IssueKind::ResourceBudget);

        std::fs::write(root.path().join("four"), b"4").expect("later file");
        let stopped = opened.refresh(&[PathBuf::from("four")]).expect("stopped refresh receipt");
        assert!(stopped.accepted.is_empty());
        assert_eq!(
            stopped.rejected,
            vec![crate::RejectedRefreshPath {
                path: PathBuf::from("four"),
                reason: crate::RefreshRejection::ResourceBudget,
            }]
        );
        assert_eq!(stopped.work.entries_visited, 1, "the refusal reports its probe");
        assert_eq!(stopped.work.files_visited, 1);
        assert_eq!(stopped.work.bytes_visited, 1);
        assert_eq!(opened.state.index.total().expect("bounded total").files, 2);
        opened.close().expect("close");
    }

    #[test]
    fn concurrent_discovery_and_refresh_share_one_atomic_file_budget() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::write(root.path().join("from-discovery"), b"discovery").expect("fixture");
        let controls = Arc::new(TestControls::default());
        controls.gate(TestPoint::BeforeDiscovery).arm();
        let opened = OpenedIndex::open_for_test(
            root.path(),
            OpenOptions {
                budget: DiscoveryBudget { max_files: Some(1) },
                ..OpenOptions::default()
            },
            Arc::clone(&controls),
        )
        .expect("opened root");
        controls.gate(TestPoint::BeforeDiscovery).wait_reached();
        std::fs::write(root.path().join("from-refresh"), b"refresh").expect("new file");

        let refreshed =
            opened.refresh(&[PathBuf::from("from-refresh")]).expect("refresh during discovery");
        assert_eq!(refreshed.accepted, vec![PathBuf::from("from-refresh")]);
        assert_eq!(opened.state.index.total().expect("after refresh").files, 1);
        controls.gate(TestPoint::BeforeDiscovery).release();

        let state = wait_until_settled(&opened);
        assert_eq!(opened.state.index.total().expect("bounded total").files, 1);
        assert_eq!(state.progress.files_retained, 1);
        assert_eq!(state.phase, crate::LifecyclePhase::Stopped);
        assert_eq!(state.coverage, crate::Coverage::Partial(crate::CoverageReason::Budget));
        opened.close().expect("close");
    }

    #[test]
    fn refresh_rejects_an_unbounded_input_before_filesystem_work() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::write(root.path().join("same"), b"same").expect("fixture");
        let opened = OpenedIndex::open(root.path(), OpenOptions::default()).expect("opened root");
        assert_eq!(wait_until_settled(&opened).phase, crate::LifecyclePhase::Ready);
        let paths = vec![PathBuf::from("same"); MAX_REFRESH_PATHS + 2];

        assert!(matches!(
            opened.refresh(&paths),
            Err(Error::RefreshPathLimit {
                attempted,
                limit: MAX_REFRESH_PATHS,
            }) if attempted == MAX_REFRESH_PATHS + 2
        ));
        opened.close().expect("close");
    }

    #[test]
    fn refresh_rejects_stale_preparation_and_counts_the_lost_race() {
        let controls = Arc::new(TestControls::default());
        let (root, opened) = opened(Arc::clone(&controls));
        std::fs::write(root.path().join("race"), b"filesystem").expect("fixture");
        controls.gate(TestPoint::AfterRefreshVerification).arm();
        let refresher = opened.clone();
        let refresh = thread::spawn(move || refresher.refresh(&[PathBuf::from("race")]));
        controls.gate(TestPoint::AfterRefreshVerification).wait_reached();
        let concurrent = Observation::new(vec![
            Op::Upsert {
                path: PathBuf::from("race"),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 99, ..crate::Attrs::default() },
            },
            Op::Upsert {
                path: PathBuf::from("other"),
                kind: EntryKind::File,
                attrs: crate::Attrs { size: 5, ..crate::Attrs::default() },
            },
        ]);
        apply_and_notify(&opened, &concurrent);
        controls.gate(TestPoint::AfterRefreshVerification).release();

        let result = refresh.join().expect("refresh thread").expect("refresh receipt");
        assert_eq!(result.work.observations, 1);
        assert_eq!(result.work.stale, 1);
        assert!(
            result.impact.dirty_paths.contains(&PathBuf::from("other")),
            "advancing to the receipt version must cover a concurrent producer"
        );
        assert_eq!(
            opened.state.index.attrs(Path::new("race")).expect("attrs").expect("retained").size,
            99
        );
        opened.close().expect("close");
    }

    #[test]
    fn close_cancels_verified_refresh_before_its_conditional_commit() {
        let controls = Arc::new(TestControls::default());
        let (root, opened) = opened(Arc::clone(&controls));
        std::fs::write(root.path().join("late"), b"late").expect("fixture");
        controls.gate(TestPoint::AfterRefreshVerification).arm();
        let refresher = opened.clone();
        let refresh = thread::spawn(move || refresher.refresh(&[PathBuf::from("late")]));
        controls.gate(TestPoint::AfterRefreshVerification).wait_reached();
        let closer = opened.clone();
        let close = thread::spawn(move || closer.close());
        let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
        while !opened.state.cancellation.is_cancelled() {
            assert!(std::time::Instant::now() < deadline, "close did not cancel refresh");
            thread::yield_now();
        }
        controls.gate(TestPoint::AfterRefreshVerification).release();

        assert!(matches!(refresh.join().expect("refresh thread"), Err(Error::OpenedIndexClosed)));
        close.join().expect("close thread").expect("joined close");
        assert_eq!(opened.state.index.kind(Path::new("late")).expect("lookup"), None);
        opened.close().expect("repeat close");
    }

    #[cfg(feature = "gitignore")]
    #[test]
    fn refresh_tracks_hidden_control_creation_edit_and_deletion() {
        let root = tempfile::tempdir().expect("temp root");
        std::fs::write(root.path().join("debug.log"), b"log").expect("fixture");
        std::fs::write(root.path().join("keep.rs"), b"keep").expect("fixture");
        let opened = OpenedIndex::open(
            root.path(),
            OpenOptions {
                hidden: Some(Arc::new(crate::HiddenPolicy::prune_hidden::<[&str; 0], &str>([]))),
                ..OpenOptions::default()
            },
        )
        .expect("opened root");
        assert_eq!(wait_until_settled(&opened).phase, crate::LifecyclePhase::Ready);

        std::fs::write(root.path().join(".gitignore"), b"*.log\n").expect("create control");
        let created = opened.refresh(&[PathBuf::from(".gitignore")]).expect("create refresh");
        assert_eq!(created.accepted, vec![PathBuf::from(".gitignore")]);
        let image = opened.state.index.snapshot().expect("snapshot");
        assert!(image.controls().source_is(Path::new(".gitignore"), b"*.log\n"));
        assert_eq!(image.is_ignored(Path::new("debug.log")), Some(true));

        std::fs::write(root.path().join(".gitignore"), b"*.tmp\n").expect("edit control");
        opened.refresh(&[PathBuf::from(".gitignore")]).expect("edit refresh");
        let image = opened.state.index.snapshot().expect("snapshot");
        assert!(image.controls().source_is(Path::new(".gitignore"), b"*.tmp\n"));
        assert_eq!(image.is_ignored(Path::new("debug.log")), Some(false));
        let unchanged =
            opened.refresh(&[PathBuf::from(".gitignore")]).expect("unchanged control refresh");
        assert_eq!(unchanged.work.observations, 1);

        std::fs::remove_file(root.path().join(".gitignore")).expect("delete control");
        opened.refresh(&[PathBuf::from(".gitignore")]).expect("delete refresh");
        let image = opened.state.index.snapshot().expect("snapshot");
        assert!(image.controls().is_empty());
        assert_eq!(image.partition_total().all, image.partition_total().unignored);
        opened.close().expect("close");
    }

    #[cfg(feature = "watch")]
    #[test]
    #[cfg(unix)]
    fn inaccessible_baseline_enters_watching_with_partial_coverage() {
        use std::os::unix::fs::PermissionsExt;

        if !crate::test_support::permission_bits_are_enforced() {
            eprintln!("skipped: this process is not subject to Unix permission bits");
            return;
        }

        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        let inaccessible = root.path().join("inaccessible");
        std::fs::create_dir(&inaccessible).expect("inaccessible directory");
        std::fs::write(inaccessible.join("secret"), b"secret").expect("fixture");
        std::fs::set_permissions(&inaccessible, std::fs::Permissions::from_mode(0o000))
            .expect("make directory inaccessible");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"").expect("script");

        let opened = OpenedIndex::open(root.path(), scripted_options(&script)).expect("open");
        let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
        let state = loop {
            let state = opened.state.index.state().expect("read state");
            if matches!(
                state.phase,
                crate::LifecyclePhase::Watching | crate::LifecyclePhase::Failed
            ) {
                break state;
            }
            assert!(std::time::Instant::now() < deadline, "observation handoff did not settle");
            std::thread::yield_now();
        };
        std::fs::set_permissions(&inaccessible, std::fs::Permissions::from_mode(0o700))
            .expect("restore directory permissions");

        assert_eq!(state.phase, crate::LifecyclePhase::Watching);
        assert_eq!(state.coverage, crate::Coverage::Partial(crate::CoverageReason::Inaccessible));
        assert_eq!(state.freshness, crate::Freshness::Partial);
        assert!(state.issues.retained > 0);
        opened.close().expect("close");
    }

    #[cfg(feature = "watch")]
    #[test]
    fn observation_is_captured_before_baseline_and_closes_the_handoff_gap() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        let path = root.path().join("during.txt");
        std::fs::write(&path, b"before").expect("fixture");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"modify\tduring.txt\n").expect("script");
        let controls = Arc::new(TestControls::default());
        controls.gate(TestPoint::BeforeDiscovery).arm();

        let opened = OpenedIndex::open_for_test(
            root.path(),
            scripted_options(&script),
            Arc::clone(&controls),
        )
        .expect("opened observed root");
        controls.gate(TestPoint::BeforeDiscovery).wait_reached();
        std::fs::write(&path, b"changed-during-baseline").expect("mutate during handoff");
        controls.gate(TestPoint::BeforeDiscovery).release();

        let state = wait_until_phase(&opened, crate::LifecyclePhase::Watching);
        assert_eq!(state.freshness, crate::Freshness::Fresh);
        assert_eq!(state.coverage, crate::Coverage::Complete);
        assert_eq!(
            opened
                .state
                .index
                .attrs(Path::new("during.txt"))
                .expect("attrs")
                .expect("retained")
                .size,
            23
        );
        let since = opened.state.index.since(crate::Clock::ZERO).expect("journal");
        assert!(since.commits.iter().any(|commit| {
            commit.state.iter().any(|transition| {
                matches!(
                    transition,
                    crate::StateTransition::IndexState { current, .. }
                        if current.phase == crate::LifecyclePhase::Reconciling
                )
            })
        }));
        assert!(since.commits.iter().any(|commit| {
            commit.state.iter().any(|transition| {
                matches!(
                    transition,
                    crate::StateTransition::IndexState { current, .. }
                        if current.phase == crate::LifecyclePhase::Watching
                )
            })
        }));
        opened.close().expect("joined close");
    }

    #[cfg(feature = "watch")]
    #[test]
    fn scripted_overflow_is_provider_recovery_not_a_consumer_reset() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        std::fs::create_dir(root.path().join("src")).expect("directory");
        std::fs::write(root.path().join("src/present"), b"present").expect("fixture");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"rescan\tsrc\n").expect("script");
        let opened = OpenedIndex::open(root.path(), scripted_options(&script)).expect("open");

        let state = wait_until_phase(&opened, crate::LifecyclePhase::Watching);
        assert_eq!(state.freshness, crate::Freshness::Fresh);
        assert!(state.issues.retained > 0);
        let since = opened.state.index.since(crate::Clock::ZERO).expect("journal");
        assert!(!since.truncated);
        assert!(since.commits.iter().any(|commit| commit.changes.iter().any(|change| {
            matches!(
                change,
                crate::EffectiveChange::Invalidated {
                    path,
                    reason: crate::InvalidateReason::WatchOverflow,
                } if path == Path::new("src")
            )
        })));
        assert!(
            opened
                .state
                .index
                .issues()
                .expect("issues")
                .iter()
                .any(|issue| issue.kind == crate::IssueKind::ObservationGap)
        );
        opened.close().expect("close");
    }

    #[cfg(feature = "watch")]
    #[test]
    fn scripted_directory_creation_closes_the_registration_gap() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        std::fs::create_dir(root.path().join("newdir")).expect("directory");
        std::fs::write(root.path().join("newdir/child"), b"child").expect("fixture");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"create-dir\tnewdir\n").expect("script");
        let opened = OpenedIndex::open(root.path(), scripted_options(&script)).expect("open");

        wait_until_phase(&opened, crate::LifecyclePhase::Watching);
        let since = opened.state.index.since(crate::Clock::ZERO).expect("journal");
        assert!(since.commits.iter().any(|commit| commit.changes.iter().any(|change| {
            matches!(
                change,
                crate::EffectiveChange::Invalidated {
                    path,
                    reason: crate::InvalidateReason::WatchSetupRace,
                } if path == Path::new("newdir")
            )
        })));
        assert_eq!(
            opened.state.index.kind(Path::new("newdir/child")).expect("lookup"),
            Some(EntryKind::File)
        );
        opened.close().expect("close");
    }

    #[cfg(feature = "watch")]
    #[test]
    fn scripted_observation_keeps_the_opened_index_live_after_handoff() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"").expect("script");
        let controls = Arc::new(TestControls::default());
        let opened = OpenedIndex::open_for_test(
            root.path(),
            scripted_options(&script),
            Arc::clone(&controls),
        )
        .expect("open scripted observer");
        wait_until_phase(&opened, crate::LifecyclePhase::Watching);
        let before = current_version(&opened);

        std::fs::write(root.path().join("live.txt"), b"live").expect("live mutation");
        controls.send_observation_hints("create\tlive.txt\n");
        let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
        loop {
            if opened.state.index.kind(Path::new("live.txt")).expect("lookup")
                == Some(EntryKind::File)
            {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "scripted hint was not applied");
            std::thread::yield_now();
        }
        let since = opened.state.index.since(before.sequence).expect("journal");
        assert!(since.commits.iter().any(|commit| commit.changes.iter().any(|change| {
            matches!(
                change,
                crate::EffectiveChange::Inserted { path, .. }
                    if path == Path::new("live.txt")
            )
        })));
        opened.close().expect("close");
    }

    #[cfg(feature = "watch")]
    #[test]
    fn live_observation_gap_recovers_before_reporting_freshness() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"").expect("script");
        let controls = Arc::new(TestControls::default());
        let opened = OpenedIndex::open_for_test(
            root.path(),
            scripted_options(&script),
            Arc::clone(&controls),
        )
        .expect("open scripted observer");
        wait_until_phase(&opened, crate::LifecyclePhase::Watching);
        let before = current_version(&opened);

        std::fs::write(root.path().join("recovered.txt"), b"recovered").expect("missed mutation");
        controls.send_observation_hints("rescan\t.\n");
        let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
        loop {
            let state = opened.state.index.state().expect("read state");
            let recovered = opened.state.index.kind(Path::new("recovered.txt")).expect("lookup")
                == Some(EntryKind::File);
            if recovered && state.freshness == crate::Freshness::Fresh {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "gap recovery did not finish");
            std::thread::yield_now();
        }

        let since = opened.state.index.since(before.sequence).expect("journal");
        assert!(since.commits.iter().any(|commit| commit.changes.iter().any(|change| {
            matches!(
                change,
                crate::EffectiveChange::Invalidated {
                    path,
                    reason: crate::InvalidateReason::WatchOverflow,
                } if path.as_os_str().is_empty()
            )
        })));
        assert!(since.commits.iter().any(|commit| commit.state.iter().any(|transition| {
            matches!(
                transition,
                crate::StateTransition::Freshness {
                    current: crate::Freshness::Reconciling | crate::Freshness::Stale,
                    ..
                }
            )
        })));
        assert!(since.commits.iter().any(|commit| commit.state.iter().any(|transition| {
            matches!(
                transition,
                crate::StateTransition::Freshness { current: crate::Freshness::Fresh, .. }
            )
        })));
        assert!(
            opened
                .state
                .index
                .issues()
                .expect("issues")
                .iter()
                .any(|issue| issue.kind == crate::IssueKind::ObservationGap)
        );
        opened.close().expect("close");
    }

    #[cfg(feature = "watch")]
    #[test]
    fn live_observation_shares_the_exact_opened_root_file_budget() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        std::fs::write(root.path().join("baseline.txt"), b"baseline").expect("fixture");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"").expect("script");
        let controls = Arc::new(TestControls::default());
        let mut options = scripted_options(&script);
        options.budget.max_files = Some(1);
        let opened = OpenedIndex::open_for_test(root.path(), options, Arc::clone(&controls))
            .expect("open scripted observer");
        wait_until_phase(&opened, crate::LifecyclePhase::Watching);

        std::fs::write(root.path().join("over-budget.txt"), b"refused")
            .expect("over-budget mutation");
        controls.send_observation_hints("create\tover-budget.txt\n");
        let state = wait_until_phase(&opened, crate::LifecyclePhase::Stopped);

        assert_eq!(state.coverage, crate::Coverage::Partial(crate::CoverageReason::Budget));
        assert_eq!(opened.state.index.kind(Path::new("over-budget.txt")).expect("lookup"), None);
        assert_eq!(opened.state.index.total().expect("total").files, 1);
        assert!(
            opened
                .state
                .index
                .issues()
                .expect("issues")
                .iter()
                .any(|issue| issue.kind == crate::IssueKind::ResourceBudget)
        );
        opened.close().expect("close");
    }

    #[cfg(feature = "watch")]
    #[test]
    fn close_after_observation_verification_prevents_publication() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"").expect("script");
        let controls = Arc::new(TestControls::default());
        let opened = OpenedIndex::open_for_test(
            root.path(),
            scripted_options(&script),
            Arc::clone(&controls),
        )
        .expect("open scripted observer");
        wait_until_phase(&opened, crate::LifecyclePhase::Watching);

        controls.gate(TestPoint::AfterObservationVerification).arm();
        std::fs::write(root.path().join("too-late.txt"), b"verified").expect("late mutation");
        controls.send_observation_hints("create\ttoo-late.txt\n");
        controls.gate(TestPoint::AfterObservationVerification).wait_reached();

        let closer = opened.clone();
        let close = thread::spawn(move || closer.close());
        let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
        while !opened.state.cancellation.is_cancelled() {
            assert!(std::time::Instant::now() < deadline, "close did not cancel observation");
            thread::yield_now();
        }
        assert!(!close.is_finished(), "close returned before the commit boundary released");
        controls.gate(TestPoint::AfterObservationVerification).release();
        close.join().expect("close thread").expect("joined close");

        assert_eq!(opened.state.index.kind(Path::new("too-late.txt")).expect("lookup"), None);
        opened.close().expect("repeat close");
    }

    #[cfg(feature = "watch")]
    #[test]
    fn stopped_discovery_never_claims_to_be_watching() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        std::fs::write(root.path().join("one"), b"one").expect("fixture");
        std::fs::write(root.path().join("two"), b"two").expect("fixture");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"").expect("script");
        let mut options = scripted_options(&script);
        options.budget.max_files = Some(1);
        let opened = OpenedIndex::open(root.path(), options).expect("open");

        wait_until_phase(&opened, crate::LifecyclePhase::Stopped);
        let since = opened.state.index.since(crate::Clock::ZERO).expect("journal");
        assert!(!since.commits.iter().any(|commit| commit.state.iter().any(|transition| {
            matches!(
                transition,
                crate::StateTransition::IndexState { current, .. }
                    if current.phase == crate::LifecyclePhase::Watching
            )
        })));
        opened.close().expect("close");
    }

    #[cfg(feature = "watch")]
    #[test]
    fn close_joins_an_observation_worker_blocked_at_a_named_boundary() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"").expect("script");
        let controls = Arc::new(TestControls::default());
        controls.gate(TestPoint::BeforeObservationPoll).arm();
        let opened = OpenedIndex::open_for_test(
            root.path(),
            scripted_options(&script),
            Arc::clone(&controls),
        )
        .expect("open");
        controls.gate(TestPoint::BeforeObservationPoll).wait_reached();

        let closer = opened.clone();
        let close = thread::spawn(move || closer.close());
        let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
        while !opened.state.cancellation.is_cancelled() {
            assert!(std::time::Instant::now() < deadline, "close did not cancel observation");
            thread::yield_now();
        }
        assert!(!close.is_finished(), "close returned before the owned worker was released");
        controls.gate(TestPoint::BeforeObservationPoll).release();
        close.join().expect("close thread").expect("joined close");
        opened.close().expect("repeat close");
    }

    #[cfg(feature = "watch")]
    #[test]
    fn malformed_script_fails_before_discovery_starts() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"teleport\tmissing\n").expect("script");
        let error = OpenedIndex::open(root.path(), scripted_options(&script))
            .expect_err("invalid observer configuration must fail open");
        assert!(matches!(error, Error::WatchScript(_)));
    }

    #[cfg(feature = "watch")]
    #[test]
    fn observation_rejects_a_restricted_scope_before_open_returns() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"").expect("script");
        let mut options = scripted_options(&script);
        options.one_filesystem = true;

        let error = OpenedIndex::open(root.path(), options)
            .expect_err("unsupported observed scope must fail open");
        assert!(matches!(error, Error::UnsupportedScanConfig(_)));
    }

    #[cfg(feature = "watch")]
    #[test]
    fn handoff_retries_a_benign_refresh_conflict_before_watching() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        let path = root.path().join("shared.txt");
        std::fs::write(&path, b"before").expect("fixture");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"").expect("script");
        let controls = Arc::new(TestControls::default());
        controls.gate(TestPoint::AfterObservationVerification).arm();
        let opened = OpenedIndex::open_for_test(
            root.path(),
            scripted_options(&script),
            Arc::clone(&controls),
        )
        .expect("open scripted observer");
        controls.gate(TestPoint::AfterObservationVerification).wait_reached();

        std::fs::write(&path, b"updated-by-refresh").expect("concurrent mutation");
        let refreshed =
            opened.refresh(&[PathBuf::from("shared.txt")]).expect("overlapping refresh succeeds");
        assert_eq!(refreshed.work.stale, 0);
        controls.gate(TestPoint::AfterObservationVerification).release();

        wait_until_phase(&opened, crate::LifecyclePhase::Watching);
        assert_eq!(
            opened
                .state
                .index
                .attrs(Path::new("shared.txt"))
                .expect("attrs")
                .expect("retained")
                .size,
            18
        );
        opened.close().expect("close");
    }

    #[cfg(feature = "watch")]
    #[test]
    fn budget_stop_wins_a_race_with_the_transition_to_watching() {
        let root = tempfile::tempdir().expect("temp root");
        let scripts = tempfile::tempdir().expect("script root");
        std::fs::write(root.path().join("baseline.txt"), b"baseline").expect("fixture");
        let script = scripts.path().join("events.script");
        std::fs::write(&script, b"").expect("script");
        let controls = Arc::new(TestControls::default());
        controls.gate(TestPoint::BeforeObservationWatching).arm();
        let mut options = scripted_options(&script);
        options.budget.max_files = Some(1);
        let opened =
            OpenedIndex::open_for_test(root.path(), options, Arc::clone(&controls)).expect("open");
        controls.gate(TestPoint::BeforeObservationWatching).wait_reached();

        std::fs::write(root.path().join("over-budget.txt"), b"refused")
            .expect("over-budget mutation");
        let refreshed = opened
            .refresh(&[PathBuf::from("over-budget.txt")])
            .expect("resource refusal is a typed result");
        assert_eq!(refreshed.work.resource_refused, 1);
        assert_eq!(refreshed.state.phase, crate::LifecyclePhase::Stopped);
        controls.gate(TestPoint::BeforeObservationWatching).release();

        let state = wait_until_phase(&opened, crate::LifecyclePhase::Stopped);
        assert_eq!(state.coverage, crate::Coverage::Partial(crate::CoverageReason::Budget));
        let since = opened.state.index.since(crate::Clock::ZERO).expect("journal");
        assert!(!since.commits.iter().any(|commit| commit.state.iter().any(|transition| {
            matches!(
                transition,
                crate::StateTransition::IndexState { current, .. }
                    if current.phase == crate::LifecyclePhase::Watching
            )
        })));
        opened.close().expect("close");
    }
}
