//! Ownership and joined shutdown for one long-lived opened root.
//!
//! [`OpenedIndex`] is the public behavior surface. Its private shared state contains
//! data and synchronization only; it is deliberately not a second API-shaped service.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

use crate::index::{DiscoveryCommit, DiscoveryTransition};
use crate::{EntryKind, Error, Index, IndexHandle, Observation, Op, Result, ScanConfig, SessionId};

mod continuation;
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
/// display depth is absent by design and belongs to a read request. Progressive
/// Observation policy is added with its capability; the current options already bind
/// progressive-discovery and exact-history bounds.
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
        let opened = Self { state: Arc::new(state) };
        opened.start_discovery()?;
        Ok(opened)
    }

    #[cfg(test)]
    fn build(root: &Path, options: OpenOptions, controls: Arc<TestControls>) -> Result<Self> {
        let state = OpenedState::new(root, options, controls)?;
        let opened = Self { state: Arc::new(state) };
        if !opened.state.test_controls.discovery_disabled.load(Ordering::Acquire) {
            opened.start_discovery()?;
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
        #[cfg(test)]
        let controls = Arc::clone(&self.state.test_controls);
        self.spawn_worker("discovery", move |cancellation| {
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
    /// Retained for discovery and later verified producers; semantic identity is also
    /// fixed in `index`, so this cannot reinterpret already-retained facts.
    #[allow(dead_code)]
    scan: ScanConfig,
    budget: DiscoveryBudget,
    frontier: Arc<DiscoveryFrontier>,
    continuations: Mutex<continuation::ContinuationTable>,
    journal: Arc<journal::JournalWait>,
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
        let (root, index, scan, budget) = bind_root(root, options)?;
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
            lifecycle: Mutex::new(Lifecycle::default()),
            lifecycle_changed: Condvar::new(),
        })
    }

    #[cfg(test)]
    fn build(root: &Path, options: OpenOptions, controls: Arc<TestControls>) -> Result<Self> {
        let (root, index, scan, budget) = bind_root(root, options)?;
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
                    self.cancellation.cancel();
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
    let index = IndexHandle::new(Index::new_with_scope_types_and_journal_capacity(
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
    let mut retained_files = 0_u64;

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
            &mut retained_files,
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
    retained_files: &mut u64,
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
                push_discovery_op(index, journal, scan.batch_size, &mut batch, control)?;
            }
            continue;
        }

        if kind == EntryKind::File && budget.max_files.is_some_and(|max| *retained_files >= max) {
            commit_discovery_batch(
                index,
                journal,
                &mut batch,
                None,
                Some(DiscoveryTransition::BudgetRefused(crate::Issue::resource_budget(
                    budget.max_files.expect("budget refusal requires a configured limit"),
                ))),
            )?;
            frontier.stop();
            return Ok(DiscoveryStep::Stopped);
        }
        if kind == EntryKind::File {
            *retained_files = retained_files.saturating_add(1);
        }
        push_discovery_op(
            index,
            journal,
            scan.batch_size,
            &mut batch,
            Op::Upsert { path: path.clone(), kind, attrs },
        )?;
        if let Some(control) = control {
            push_discovery_op(index, journal, scan.batch_size, &mut batch, control)?;
        }
        if descend {
            discovered.push(PendingDirectory { path, depth: directory.depth.saturating_add(1) });
        }
    }

    let incomplete = !issues.is_empty() || omitted_issues > 0;
    let transition =
        incomplete.then_some(DiscoveryTransition::Inaccessible { issues, omitted: omitted_issues });
    let complete = (!incomplete).then(|| directory.path.clone());
    commit_discovery_batch(index, journal, &mut batch, complete, transition)?;
    frontier.extend(discovered);
    Ok(DiscoveryStep::Continue)
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
) -> Result<()> {
    batch.push(op);
    if batch.len() >= batch_size {
        commit_discovery_batch(index, journal, batch, None, None)?;
    }
    Ok(())
}

fn commit_discovery_batch(
    index: &IndexHandle,
    journal: &journal::JournalWait,
    batch: &mut Vec<Op>,
    directory_complete: Option<PathBuf>,
    transition: Option<DiscoveryTransition>,
) -> Result<()> {
    let observation = Observation::new(std::mem::take(batch));
    let outcome =
        index.apply_discovery(&observation, DiscoveryCommit { directory_complete, transition })?;
    if outcome.commit.is_some() {
        journal.notify_commit();
    }
    Ok(())
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
    BeforeDiscovery,
    AfterRootDirectory,
    BeforeJournalWait,
}

#[cfg(test)]
#[derive(Default)]
struct TestControls {
    before_worker_exit: TestGate,
    before_close_wait: TestGate,
    before_discovery: TestGate,
    after_root_directory: TestGate,
    before_journal_wait: TestGate,
    discovery_disabled: AtomicBool,
}

#[cfg(test)]
impl TestControls {
    fn gate(&self, point: TestPoint) -> &TestGate {
        match point {
            TestPoint::BeforeWorkerExit => &self.before_worker_exit,
            TestPoint::BeforeCloseWait => &self.before_close_wait,
            TestPoint::BeforeDiscovery => &self.before_discovery,
            TestPoint::AfterRootDirectory => &self.after_root_directory,
            TestPoint::BeforeJournalWait => &self.before_journal_wait,
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
                    selection: crate::query::Selection::default(),
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
            first_page.rows.iter().map(|row| row.portable_path.as_deref()).collect::<Vec<_>>(),
            vec![Some("a.txt"), Some("b.txt")]
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
            second_page.rows.iter().map(|row| row.portable_path.as_deref()).collect::<Vec<_>>(),
            vec![Some("c.txt")]
        );
        assert!(second_page.next.is_none());
        assert!(second.work.rows_visited <= 2, "continuation resumed from retained position");
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
                        selection: crate::query::Selection::default(),
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

    #[cfg(unix)]
    #[test]
    fn portable_pages_expose_exact_path_loss_and_fail_closed_on_absence() {
        use std::os::unix::ffi::OsStringExt;

        let (_root, opened) = opened(Arc::new(TestControls::default()));
        let invalid = PathBuf::from(OsString::from_vec(vec![b'x', 0xff]));
        let mut long_bytes = vec![b'y'; crate::MAX_PORTABLE_PATH_EXAMPLE_BYTES + 1];
        *long_bytes.last_mut().expect("nonempty path") = 0xfe;
        let long_invalid = PathBuf::from(OsString::from_vec(long_bytes));
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
                    path: long_invalid.clone(),
                    kind: EntryKind::File,
                    attrs: crate::Attrs { size: 1, ..crate::Attrs::default() },
                },
            ]))
            .expect("seed unrepresentable entry");

        let response = opened
            .read(crate::ReadRequest {
                projections: vec![
                    crate::ReadProjection::Lookup { path: PathBuf::from("missing") },
                    crate::ReadProjection::Tree {
                        path: PathBuf::new(),
                        page: crate::PageRequest {
                            limit: crate::MAX_PAGE_ROWS,
                            max_work: crate::MAX_PAGE_WORK,
                        },
                    },
                    crate::ReadProjection::Flat {
                        selection: crate::query::Selection::default(),
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
        assert!(matches!(
            response.results[0],
            crate::ProjectionResult::Lookup(crate::Knowledge::Unknown { .. })
        ));
        let crate::ProjectionResult::Tree(crate::Knowledge::Present(tree)) = &response.results[1]
        else {
            panic!("tree page");
        };
        assert!(tree.rows.is_empty());
        assert!(tree.native_complete);
        assert!(!tree.portable_complete);
        assert_eq!(tree.portable_issue.as_ref().map(|issue| issue.omitted), Some(2));
        let example = &tree.portable_issue.as_ref().expect("path issue").examples[0];
        assert_eq!(example.encoding, crate::PortablePathEncoding::UnixBytes);
        assert!(example.encoded_hex.ends_with("78ff"));
        assert!(!example.truncated);
        let long_example = &tree.portable_issue.as_ref().expect("path issue").examples[1];
        assert_eq!(long_example.encoded_hex.len(), crate::MAX_PORTABLE_PATH_EXAMPLE_BYTES * 2);
        assert!(long_example.truncated);
        let crate::ProjectionResult::Flat(flat) = &response.results[2] else {
            panic!("flat page");
        };
        assert!(flat.rows.is_empty());
        assert_eq!(flat.portable_issue.as_ref().map(|issue| issue.omitted), Some(2));
        assert!(matches!(
            &response.results[3],
            crate::ProjectionResult::RollUp(crate::Knowledge::Present(rollup))
                if rollup.all.files == 2 && rollup.all.bytes == 10
        ));

        opened
            .state
            .index
            .apply(&Observation::new(vec![
                Op::Remove { path: invalid },
                Op::Remove { path: long_invalid },
            ]))
            .expect("remove unrepresentable entry");
        assert!(matches!(
            opened
                .read(crate::ReadRequest {
                    projections: vec![crate::ReadProjection::Lookup {
                        path: PathBuf::from("missing"),
                    }],
                    ..crate::ReadRequest::default()
                })
                .expect("known absence")
                .results[0],
            crate::ProjectionResult::Lookup(crate::Knowledge::Absent)
        ));
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
                    selection: crate::query::Selection::default(),
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
                    selection: crate::query::Selection::default(),
                    count_cap: 0,
                    max_work: 1,
                }],
                ..crate::ReadRequest::default()
            }),
            Err(Error::CountCapLimit { attempted: 0, .. })
        ));
        let bounded_path = opened
            .read(crate::ReadRequest {
                projections: vec![crate::ReadProjection::Tree {
                    path: PathBuf::from("missing/deep"),
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
                        selection: crate::query::Selection::default(),
                        count_cap: 1,
                        max_work: 1,
                    },
                    crate::ReadProjection::Aggregate {
                        selection: crate::query::Selection {
                            kinds: vec![EntryKind::File],
                            ..crate::query::Selection::default()
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
}
