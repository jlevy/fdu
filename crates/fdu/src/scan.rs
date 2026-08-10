//! The scan layer: walking a tree, producing observations, and applying reconciliation.
//!
//! A cold scan is just a large batch of upserts, and a revalidation sweep is the diff
//! between what the index believes and what the filesystem says. Both speak the same
//! [`Observation`] vocabulary as the watch layer, which is what lets the index be ignorant of
//! where its changes came from.
//!
//! # Status
//!
//! This is the portable `read_dir` + `symlink_metadata` implementation. It is correct
//! and it is the reference the fast path must match, but it is **not** the walker the
//! performance goal calls for: raw `getdents64` into a large reused per-thread buffer,
//! dirfd-relative `statx` with a narrow field mask, `d_type` stat-avoidance, and a
//! work-stealing pool. Until that layer lands and the benchmark gate passes, no
//! performance claim should be made for this crate.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::ApplyStats;
use crate::index::{Index, IndexHandle};
use crate::types::{
    AppliedDelta, Attrs, EntryKind, Error, Observation, ObservationOp, Op, PathExpectation,
    PathState, Result, ScanScope,
};

/// How many ops accumulate before an observation is handed to the sink.
///
/// Batching matters for more than syscall economy: consumers coalesce per path within a
/// batch and stat once per batch, and a live UI wants partial results while a large tree
/// is still being walked rather than one delta at the end.
const DEFAULT_BATCH_SIZE: usize = 1024;

/// Largest producer batch accepted before work must be published incrementally.
pub const MAX_SCAN_BATCH_SIZE: usize = 64 * 1024;

/// Identity of the current built-in ignore policy. No ignore rules exist yet.
const IGNORE_RULES_FINGERPRINT: u64 = 0;

/// Identity of the current compound-extension classifier.
const TYPE_RULES_FINGERPRINT: u64 = 1;

/// Identity of the fixed stat-tier reducer set.
const REDUCERS_FINGERPRINT: u64 = 1;

/// Knobs for a scan.
#[derive(Clone, Debug)]
pub struct ScanConfig {
    /// Maximum relative entry depth to retain. Zero keeps only the index root and `None`
    /// means unlimited.
    pub max_depth: Option<usize>,
    /// Ops per emitted observation. Must be between one and [`MAX_SCAN_BATCH_SIZE`].
    pub batch_size: usize,
    /// Follow symlinks to directories. Off by default: following them turns a tree walk
    /// into a graph walk with cycles, and every surveyed tool defaults to off.
    pub follow_symlinks: bool,
    /// Stay on the filesystem the root lives on.
    pub one_filesystem: bool,
    /// Directory-reading worker threads.
    ///
    /// A tree walk is a pile of independent, latency-bound directory reads, so it
    /// scales with threads far better than most work does. One means the serial
    /// walker, which stays the reference implementation and the thing every result is
    /// checked against. [`None`] asks for a bounded default derived from the
    /// machine's available parallelism.
    ///
    /// This is an operational knob, not a semantic one: it changes how fast the same
    /// observations are produced, never which observations they are. That is why it
    /// stays out of [`ScanScope`] and cannot invalidate a cache.
    pub threads: Option<usize>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            max_depth: None,
            batch_size: DEFAULT_BATCH_SIZE,
            follow_symlinks: false,
            one_filesystem: false,
            threads: None,
        }
    }
}

impl ScanConfig {
    /// Semantic cache identity, excluding operational batching choices.
    pub const fn scope(&self) -> ScanScope {
        ScanScope {
            max_depth: self.max_depth,
            follow_symlinks: self.follow_symlinks,
            one_filesystem: self.one_filesystem,
            ignore_rules_fingerprint: IGNORE_RULES_FINGERPRINT,
            type_rules_fingerprint: TYPE_RULES_FINGERPRINT,
            reducers_fingerprint: REDUCERS_FINGERPRINT,
        }
    }

    /// Resolve [`Self::threads`] to a concrete, bounded worker count.
    ///
    /// A machine that will not report its parallelism gets one thread rather than a
    /// guess: the serial walker is always correct, and silently choosing a pool size
    /// out of thin air is how a benchmark ends up measuring the guess.
    fn worker_threads(&self) -> usize {
        match self.threads {
            Some(threads) => threads.clamp(1, MAX_SCAN_THREADS),
            None => std::thread::available_parallelism()
                .map_or(1, |value| value.get().clamp(1, DEFAULT_SCAN_THREADS_CAP)),
        }
    }

    fn validate(&self) -> Result<()> {
        if self.batch_size == 0 || self.batch_size > MAX_SCAN_BATCH_SIZE {
            return Err(Error::UnsupportedScanConfig(
                "batch_size must be nonzero and no greater than MAX_SCAN_BATCH_SIZE",
            ));
        }
        if self.follow_symlinks {
            return Err(Error::UnsupportedScanConfig(
                "follow_symlinks requires cycle, root-boundary, and filesystem-boundary semantics",
            ));
        }
        #[cfg(not(unix))]
        if self.one_filesystem {
            return Err(Error::UnsupportedScanConfig(
                "one_filesystem requires platform device identity",
            ));
        }
        Ok(())
    }

    pub(crate) fn validate_for_scope(&self, indexed: ScanScope) -> Result<()> {
        self.validate()?;
        let requested = self.scope();
        if indexed != requested {
            return Err(Error::ScanScopeMismatch { indexed, requested });
        }
        Ok(())
    }

    #[cfg(feature = "watch")]
    pub(crate) fn validate_for_watch_scope(&self, indexed: ScanScope) -> Result<()> {
        self.validate_for_scope(indexed)?;
        if self.max_depth.is_some() || self.one_filesystem {
            return Err(Error::UnsupportedScanConfig(
                "watch application for max_depth or one_filesystem requires event-scope filtering",
            ));
        }
        Ok(())
    }
}

impl Default for ScanScope {
    fn default() -> Self {
        ScanConfig::default().scope()
    }
}

/// What a scan did, including the errors it walked past.
///
/// Unreadable directories are skipped rather than aborting the scan — a permission-denied
/// subdirectory should not cost you the other 499,000 files — but they are reported
/// rather than swallowed, so a caller can tell a complete answer from a partial one.
#[derive(Debug, Default)]
pub struct ScanReport {
    /// Directories successfully listed.
    pub dirs_read: u64,
    /// Entries observed, directories included.
    pub entries: u64,
    /// Paths that could not be read, with the reason.
    pub errors: Vec<Error>,
}

impl ScanReport {
    /// True when every directory in scope was read successfully.
    pub fn is_complete(&self) -> bool {
        self.errors.is_empty()
    }

    /// Fold one worker's share of a parallel walk into the whole-walk report.
    fn absorb(&mut self, other: Self) {
        self.dirs_read += other.dirs_read;
        self.entries += other.entries;
        self.errors.extend(other.errors);
    }
}

/// Filesystem and index effects from an applying reconciliation pass.
#[derive(Debug, Default)]
pub struct ReconcileReport {
    /// Filesystem walk effects and partial errors.
    pub scan: ScanReport,
    /// Index arbitration and mutation effects.
    pub apply: ApplyStats,
}

impl ReconcileReport {
    /// True when the filesystem walk was complete and no conditional observation lost
    /// a race with another producer.
    pub fn is_complete(&self) -> bool {
        self.scan.is_complete() && self.apply.stale == 0
    }
}

enum ReconcileTarget<'a> {
    Direct(&'a mut Index),
    Shared(&'a IndexHandle),
}

impl ReconcileTarget<'_> {
    fn scope(&self) -> Result<ScanScope> {
        match self {
            Self::Direct(index) => Ok(index.scope()),
            Self::Shared(handle) => handle.scope(),
        }
    }

    fn root_path(&self) -> Result<PathBuf> {
        match self {
            Self::Direct(index) => Ok(index.root_path().to_path_buf()),
            Self::Shared(handle) => handle.root_path(),
        }
    }

    fn expectation(&self, path: &Path) -> Result<PathExpectation> {
        match self {
            Self::Direct(index) => Ok(index.expectation(path)),
            Self::Shared(handle) => handle.expectation(path),
        }
    }

    fn child_states(&self, path: &Path) -> Result<BTreeMap<OsString, PathExpectation>> {
        match self {
            Self::Direct(index) => Ok(collect_child_states(index, path)),
            Self::Shared(handle) => handle.child_states(path),
        }
    }

    fn apply(&mut self, observation: &Observation) -> Result<crate::ApplyOutcome> {
        match self {
            Self::Direct(index) => index.apply(observation),
            Self::Shared(handle) => handle.apply(observation),
        }
    }

    fn direct_upsert_is_unchanged(
        &self,
        baseline: PathExpectation,
        kind: EntryKind,
        attrs: Attrs,
    ) -> bool {
        matches!(self, Self::Direct(_)) && baseline.state == (PathState::Present { kind, attrs })
    }

    fn take_pending_invalidations(&mut self) -> Result<Vec<(PathBuf, crate::InvalidateReason)>> {
        match self {
            Self::Direct(index) => Ok(index.take_pending_invalidations()),
            Self::Shared(handle) => handle.take_pending_invalidations(),
        }
    }

    fn restore_pending_invalidations(
        &mut self,
        invalidations: Vec<(PathBuf, crate::InvalidateReason)>,
    ) -> Result<()> {
        match self {
            Self::Direct(index) => index.restore_pending_invalidations(invalidations),
            Self::Shared(handle) => handle.restore_pending_invalidations(invalidations)?,
        }
        Ok(())
    }

    fn begin_reconcile(&mut self, path: &Path) -> Result<u64> {
        match self {
            Self::Direct(index) => Ok(index.begin_reconcile(path)),
            Self::Shared(handle) => handle.begin_reconcile(path),
        }
    }

    fn finish_reconcile(&mut self, path: &Path, started_at: u64, complete: bool) -> Result<()> {
        match self {
            Self::Direct(index) => index.finish_reconcile(path, started_at, complete),
            Self::Shared(handle) => handle.finish_reconcile(path, started_at, complete)?,
        }
        Ok(())
    }
}

fn collect_child_states(index: &Index, path: &Path) -> BTreeMap<OsString, PathExpectation> {
    index.children(path).map_or_else(BTreeMap::new, |children| {
        children
            .map(|(name, _)| {
                let child_path = path.join(name);
                (name.to_os_string(), index.expectation(&child_path))
            })
            .collect()
    })
}

#[cfg(unix)]
fn metadata_for_fingerprint(entry: &fs::DirEntry) -> std::io::Result<fs::Metadata> {
    entry.metadata()
}

#[cfg(not(unix))]
fn metadata_for_fingerprint(entry: &fs::DirEntry) -> std::io::Result<fs::Metadata> {
    // Windows serves DirEntry metadata from directory-enumeration data, which the
    // platform permits to be stale. Fingerprints need a fresh non-following query.
    fs::symlink_metadata(entry.path())
}

/// Walk `root` and emit observations describing everything found.
pub fn scan(
    root: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(Observation),
) -> Result<ScanReport> {
    config.validate()?;
    let root_meta = fs::symlink_metadata(root).map_err(|e| Error::io(root, e))?;
    if !root_meta.is_dir() {
        return Err(Error::io(
            root,
            std::io::Error::new(std::io::ErrorKind::NotADirectory, "scan root is not a directory"),
        ));
    }
    let root_dev = attrs_from(&root_meta).dev;

    if config.max_depth != Some(0) && config.worker_threads() > 1 {
        return Ok(scan_concurrent(root, config, root_dev, sink));
    }

    let mut report = ScanReport::default();
    if config.max_depth == Some(0) {
        return Ok(report);
    }
    let mut batch: Vec<Op> = Vec::with_capacity(config.batch_size);
    // Depth-first keeps the portable reference implementation small and locality-friendly.
    // Traversal order becomes a measured choice once the parallel syscall layer lands.
    let mut queue: Vec<(PathBuf, usize)> = vec![(PathBuf::new(), 0)];

    while let Some((rel_dir, depth)) = queue.pop() {
        let abs_dir = root.join(&rel_dir);
        let listing = match fs::read_dir(&abs_dir) {
            Ok(listing) => listing,
            Err(e) => {
                report.errors.push(Error::io(abs_dir, e));
                continue;
            }
        };
        report.dirs_read += 1;

        for item in listing {
            let item = match item {
                Ok(item) => item,
                Err(e) => {
                    report.errors.push(Error::io(&abs_dir, e));
                    continue;
                }
            };
            let name = item.file_name();
            let rel_path = rel_dir.join(&name);
            let meta = match metadata_for_fingerprint(&item) {
                Ok(meta) => meta,
                Err(e) => {
                    report.errors.push(Error::io(item.path(), e));
                    continue;
                }
            };

            let attrs = attrs_from(&meta);
            let kind = kind_from(&meta);
            report.entries += 1;
            batch.push(Op::Upsert { path: rel_path.clone(), kind, attrs });
            if batch.len() >= config.batch_size {
                sink(Observation::new(std::mem::take(&mut batch)));
                batch.reserve(config.batch_size);
            }

            if should_descend(kind, attrs, depth, root_dev, config) {
                queue.push((rel_path, depth + 1));
            }
        }
    }

    if !batch.is_empty() {
        sink(Observation::new(batch));
    }
    Ok(report)
}

/// Largest worker pool a caller may ask for explicitly.
///
/// Well past anything measured to help. It exists so a caller that computes a thread
/// count from something silly cannot spawn thousands of threads.
const MAX_SCAN_THREADS: usize = 32;

/// Ceiling on the pool size chosen automatically.
///
/// Measured, not guessed. On a 10-core machine walking a 60k-entry `node_modules`
/// tree, wall time fell 37% at two workers and 50% at four, then stopped improving:
/// six matched four within noise and eight was 4% worse than four. The walk becomes
/// bound by the single index consumer, so past this point extra workers buy queue
/// contention and efficiency-core scheduling rather than throughput. See
/// `docs/project/reports/report-2026-08-10-fdu-performance-experiments.md`.
const DEFAULT_SCAN_THREADS_CAP: usize = 6;

/// Directories handed to a worker in one go.
///
/// Popping one directory at a time makes the queue lock the bottleneck on a wide,
/// shallow tree; taking a small run amortizes the lock without letting one worker
/// starve the others by hoarding the queue.
const DIR_CLAIM: usize = 4;

/// A parallel directory walk that produces exactly the observations the serial walk does.
///
/// The shape is deliberate. Workers read directories and *produce* observations; they
/// never touch an index. A single consumer — the caller's sink, on this thread —
/// applies them. That keeps the crate's one mutation contract intact: parallelism is a
/// property of the producer, and the index still sees one ordered stream of deltas.
///
/// Ordering across workers is not fixed, so an entry can arrive before its parent
/// directory does. The index already tolerates that, because watch events have never
/// arrived parent-first either, and it fills in a synthesized ancestor's real
/// attributes when the observation for it turns up. The resulting index is
/// byte-identical to the serial walker's, which the benchmark harness re-proves on
/// every trial by comparing engine digests against an independent oracle.
fn scan_concurrent(
    root: &Path,
    config: &ScanConfig,
    root_dev: u64,
    sink: &mut dyn FnMut(Observation),
) -> ScanReport {
    let workers = config.worker_threads();
    let queue = DirectoryQueue::new(vec![(PathBuf::new(), 0)]);
    let (sender, receiver) = std::sync::mpsc::channel::<Observation>();

    let mut report = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                let sender = sender.clone();
                let queue = &queue;
                scope.spawn(move || walk_worker(root, config, root_dev, queue, &sender))
            })
            .collect();
        // The loop below ends when every sender is gone, so this one must go first.
        drop(sender);

        for observation in receiver {
            sink(observation);
        }

        let mut report = ScanReport::default();
        for handle in handles {
            match handle.join() {
                Ok(worker) => report.absorb(worker),
                Err(_) => {
                    // A worker panicked. Its directories are unaccounted for, so the
                    // scan is partial; say so rather than reporting a short tree as
                    // complete.
                    report.errors.push(Error::io(
                        root,
                        std::io::Error::other("a scan worker thread panicked"),
                    ));
                }
            }
        }
        report
    });

    // Workers finish in whatever order the filesystem lets them, so a report assembled
    // from them is only reproducible if the errors are ordered here.
    report.errors.sort_by_cached_key(ToString::to_string);
    report
}

/// One worker's share of the walk: claim directories, read them, publish observations.
fn walk_worker(
    root: &Path,
    config: &ScanConfig,
    root_dev: u64,
    queue: &DirectoryQueue,
    sender: &std::sync::mpsc::Sender<Observation>,
) -> ScanReport {
    let mut report = ScanReport::default();
    let mut batch: Vec<Op> = Vec::with_capacity(config.batch_size);
    let mut claimed: Vec<(PathBuf, usize)> = Vec::with_capacity(DIR_CLAIM);
    let mut discovered: Vec<(PathBuf, usize)> = Vec::new();

    while queue.claim(&mut claimed) {
        for (rel_dir, depth) in claimed.drain(..) {
            let abs_dir = root.join(&rel_dir);
            let listing = match fs::read_dir(&abs_dir) {
                Ok(listing) => listing,
                Err(e) => {
                    report.errors.push(Error::io(abs_dir, e));
                    continue;
                }
            };
            report.dirs_read += 1;

            for item in listing {
                let item = match item {
                    Ok(item) => item,
                    Err(e) => {
                        report.errors.push(Error::io(&abs_dir, e));
                        continue;
                    }
                };
                let name = item.file_name();
                let rel_path = rel_dir.join(&name);
                let meta = match metadata_for_fingerprint(&item) {
                    Ok(meta) => meta,
                    Err(e) => {
                        report.errors.push(Error::io(item.path(), e));
                        continue;
                    }
                };

                let attrs = attrs_from(&meta);
                let kind = kind_from(&meta);
                report.entries += 1;
                let descend = should_descend(kind, attrs, depth, root_dev, config);
                batch.push(Op::Upsert { path: rel_path.clone(), kind, attrs });
                if batch.len() >= config.batch_size
                    && sender.send(Observation::new(std::mem::take(&mut batch))).is_err()
                {
                    // The consumer is gone; nothing further will be read.
                    return report;
                }
                if descend {
                    discovered.push((rel_path, depth + 1));
                }
            }
        }
        // Publish before releasing the claim so a worker that finds nothing new does
        // not hold work that others could be doing.
        if !discovered.is_empty() {
            queue.extend(discovered.drain(..));
        }
        queue.release();
    }

    if !batch.is_empty() {
        let _ = sender.send(Observation::new(batch));
    }
    report
}

/// Directories still to read, plus enough state to know when the walk is finished.
///
/// The termination condition is the only subtle part: the queue being empty does not
/// mean the walk is done, because a worker that is mid-directory may be about to push
/// its children. So a worker holds a claim from the moment it takes work until the
/// moment it has published everything that work produced, and the walk ends only when
/// the queue is empty *and* no claim is outstanding.
struct DirectoryQueue {
    state: std::sync::Mutex<DirectoryQueueState>,
    ready: std::sync::Condvar,
}

struct DirectoryQueueState {
    pending: Vec<(PathBuf, usize)>,
    outstanding: usize,
    finished: bool,
}

impl DirectoryQueue {
    fn new(initial: Vec<(PathBuf, usize)>) -> Self {
        Self {
            state: std::sync::Mutex::new(DirectoryQueueState {
                pending: initial,
                outstanding: 0,
                finished: false,
            }),
            ready: std::sync::Condvar::new(),
        }
    }

    /// Take up to [`DIR_CLAIM`] directories, blocking until there is work or the walk
    /// is over. Returns false once no more work will ever arrive.
    fn claim(&self, into: &mut Vec<(PathBuf, usize)>) -> bool {
        let mut state = self.lock();
        loop {
            if !state.pending.is_empty() {
                let take = state.pending.len().min(DIR_CLAIM);
                let start = state.pending.len() - take;
                into.extend(state.pending.drain(start..));
                state.outstanding += 1;
                return true;
            }
            if state.finished {
                return false;
            }
            if state.outstanding == 0 {
                state.finished = true;
                self.ready.notify_all();
                return false;
            }
            state = self.ready.wait(state).unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }

    fn extend(&self, directories: impl Iterator<Item = (PathBuf, usize)>) {
        let mut state = self.lock();
        state.pending.extend(directories);
        drop(state);
        self.ready.notify_all();
    }

    /// Give up a claim taken by [`claim`]. Wakes everyone if this was the last one.
    fn release(&self) {
        let mut state = self.lock();
        state.outstanding -= 1;
        if state.outstanding == 0 && state.pending.is_empty() {
            state.finished = true;
            drop(state);
            self.ready.notify_all();
        }
    }

    /// A poisoned queue means a worker panicked mid-walk. The data behind the lock is
    /// a plain work list with no invariant that a panic could have broken, and the
    /// caller already reports the panic as a scan error, so recovering the list is
    /// strictly better than propagating a second panic into every other worker.
    fn lock(&self) -> std::sync::MutexGuard<'_, DirectoryQueueState> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Walk `root` and return a fully populated index.
pub fn scan_into_index(root: &Path, config: &ScanConfig) -> Result<(Index, ScanReport)> {
    config.validate()?;
    let root = root.canonicalize().map_err(|error| Error::io(root, error))?;
    let mut index = Index::new_with_scope(&root, config.scope());
    let mut apply_error: Option<Error> = None;
    let report = scan(&root, config, &mut |observation| {
        if apply_error.is_none() {
            if let Err(error) = index.apply_baseline(&observation) {
                apply_error = Some(error);
            }
        }
    })?;
    if let Some(error) = apply_error {
        return Err(error);
    }
    index.set_initial_freshness(report.is_complete());
    Ok((index, report))
}

/// Diff the filesystem against an existing index and emit conditional observations.
///
/// This is cache tier 2: after a snapshot is loaded, a sweep like this is what makes the
/// answer trustworthy rather than merely fast. Unchanged entries produce upserts whose
/// fingerprints already match, which the index discards as no-ops, so the caller can
/// apply the whole stream without filtering it first.
///
/// Entries the index holds but the filesystem no longer has become [`Op::Remove`],
/// detected per directory rather than by accumulating every visited path in memory.
///
/// This observation-only reference API assumes its emitted stream is applied to the same
/// unchanged baseline after the borrow ends. Use [`reconcile`] or [`reconcile_handle`]
/// when other producers can write concurrently; those paths capture the stronger
/// generation/revision/absence expectations returned by [`Index::expectation`].
pub fn revalidate(
    index: &Index,
    config: &ScanConfig,
    sink: &mut dyn FnMut(Observation),
) -> Result<ScanReport> {
    config.validate_for_scope(index.scope())?;
    let root = index.root_path().to_path_buf();
    let root_meta = fs::symlink_metadata(&root).map_err(|error| Error::io(&root, error))?;
    if !root_meta.is_dir() {
        return Err(Error::io(
            &root,
            std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                "revalidation root is not a directory",
            ),
        ));
    }
    let root_dev = attrs_from(&root_meta).dev;
    let mut report = ScanReport::default();
    let batch_limit = config.batch_size.max(1);
    let mut batch: Vec<ObservationOp> = Vec::with_capacity(batch_limit);
    if config.max_depth == Some(0) {
        if let Some(children) = index.children(Path::new("")) {
            for (name, _) in children {
                let path = PathBuf::from(name);
                batch.push(ObservationOp::if_state(
                    Op::Remove { path: path.clone() },
                    index.relaxed_expectation(&path),
                ));
                if batch.len() >= batch_limit {
                    sink(Observation::from_ops(std::mem::take(&mut batch)));
                    batch.reserve(batch_limit);
                }
            }
        }
        if !batch.is_empty() {
            sink(Observation::from_ops(batch));
        }
        return Ok(report);
    }
    let mut queue: Vec<(PathBuf, usize)> = vec![(PathBuf::new(), 0)];

    while let Some((rel_dir, depth)) = queue.pop() {
        let abs_dir = root.join(&rel_dir);
        let listing = match fs::read_dir(&abs_dir) {
            Ok(listing) => listing,
            Err(e) => {
                report.errors.push(Error::io(abs_dir, e));
                continue;
            }
        };
        report.dirs_read += 1;

        let mut seen: BTreeSet<OsString> = BTreeSet::new();
        let mut listing_complete = true;
        for item in listing {
            let item = match item {
                Ok(item) => item,
                Err(e) => {
                    listing_complete = false;
                    report.errors.push(Error::io(&abs_dir, e));
                    continue;
                }
            };
            let name = item.file_name();
            seen.insert(name.clone());
            let rel_path = rel_dir.join(&name);
            let baseline = index.relaxed_expectation(&rel_path);
            let meta = match metadata_for_fingerprint(&item) {
                Ok(meta) => meta,
                Err(e) => {
                    report.errors.push(Error::io(item.path(), e));
                    continue;
                }
            };

            let kind = kind_from(&meta);
            let attrs = attrs_from(&meta);
            report.entries += 1;
            batch.push(ObservationOp::if_state(
                Op::Upsert { path: rel_path.clone(), kind, attrs },
                baseline,
            ));
            if batch.len() >= batch_limit {
                sink(Observation::from_ops(std::mem::take(&mut batch)));
                batch.reserve(batch_limit);
            }

            if should_descend(kind, attrs, depth, root_dev, config) {
                queue.push((rel_path, depth + 1));
            } else if kind.is_dir() {
                if let Some(children) = index.children(&rel_path) {
                    for (child_name, _) in children {
                        let child_path = rel_path.join(child_name);
                        batch.push(ObservationOp::if_state(
                            Op::Remove { path: child_path.clone() },
                            index.relaxed_expectation(&child_path),
                        ));
                        if batch.len() >= batch_limit {
                            sink(Observation::from_ops(std::mem::take(&mut batch)));
                            batch.reserve(batch_limit);
                        }
                    }
                }
            }
        }

        // Anything the index still lists here but the filesystem did not return is gone.
        if listing_complete {
            if let Some(known) = index.children(&rel_dir) {
                for (name, _) in known {
                    if !seen.contains(name) {
                        let path = rel_dir.join(name);
                        batch.push(ObservationOp::if_state(
                            Op::Remove { path: path.clone() },
                            index.relaxed_expectation(&path),
                        ));
                    }
                }
            }
        }
    }

    if !batch.is_empty() {
        sink(Observation::from_ops(batch));
    }
    Ok(report)
}

/// Reconcile the full index and publish each effective committed delta as it lands.
pub fn reconcile(
    index: &mut Index,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    reconcile_subtree(index, Path::new(""), config, sink)
}

/// Reconcile one relative subtree, applying effective changes during the walk.
///
/// If an ancestor vanished or became a non-directory, reconciliation widens to that
/// ancestor so a child invalidation can converge instead of retrying `ENOTDIR` forever.
pub fn reconcile_subtree(
    index: &mut Index,
    subtree: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    reconcile_target(&mut ReconcileTarget::Direct(index), subtree, config, sink)
}

/// Reconcile a shared index while allowing readers between applied batches.
pub fn reconcile_handle(
    handle: &IndexHandle,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    reconcile_subtree_handle(handle, Path::new(""), config, sink)
}

/// Reconcile one subtree of a shared index, widening to a missing/non-directory ancestor
/// when necessary.
pub fn reconcile_subtree_handle(
    handle: &IndexHandle,
    subtree: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    reconcile_target(&mut ReconcileTarget::Shared(handle), subtree, config, sink)
}

fn reconcile_target(
    target: &mut ReconcileTarget<'_>,
    subtree: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    config.validate_for_scope(target.scope()?)?;
    let subtree = normalize_subtree(subtree)?;
    if config.max_depth.is_some_and(|maximum| subtree.components().count() > maximum) {
        return Err(Error::SubtreeOutsideScanScope { path: subtree, scope: config.scope() });
    }
    let subtree = resolve_subtree_root(target, &subtree, config)?;
    let started_at = target.begin_reconcile(&subtree)?;
    match reconcile_target_inner(target, &subtree, config, sink) {
        Ok(report) => {
            target.finish_reconcile(&subtree, started_at, report.is_complete())?;
            Ok(report)
        }
        Err(error) => {
            target.finish_reconcile(&subtree, started_at, false)?;
            Err(error)
        }
    }
}

fn reconcile_target_inner(
    target: &mut ReconcileTarget<'_>,
    subtree: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    let root = target.root_path()?;
    let root_meta = fs::symlink_metadata(&root).map_err(|error| Error::io(&root, error))?;
    if !root_meta.is_dir() {
        return Err(Error::io(
            &root,
            std::io::Error::new(
                std::io::ErrorKind::NotADirectory,
                "reconciliation root is not a directory",
            ),
        ));
    }
    let root_dev = attrs_from(&root_meta).dev;
    let start_depth = subtree.components().count();
    let mut report = ReconcileReport::default();
    let mut batch: Vec<ObservationOp> = Vec::with_capacity(config.batch_size.max(1));

    if config.max_depth == Some(0) {
        remove_known_children(target, Path::new(""), config, &mut batch, sink, &mut report.apply)?;
        return Ok(report);
    }

    if !subtree.as_os_str().is_empty() {
        let baseline = target.expectation(subtree)?;
        let absolute = root.join(subtree);
        let meta = match fs::symlink_metadata(&absolute) {
            Ok(meta) => meta,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                batch.push(ObservationOp::if_state(
                    Op::Remove { path: subtree.to_path_buf() },
                    baseline,
                ));
                flush_reconcile_batch(target, &mut batch, sink, &mut report.apply)?;
                return Ok(report);
            }
            Err(error) => {
                report.scan.errors.push(Error::io(absolute, error));
                return Ok(report);
            }
        };
        let kind = kind_from(&meta);
        let attrs = attrs_from(&meta);
        report.scan.entries += 1;
        push_reconcile_upsert(
            target,
            subtree,
            kind,
            attrs,
            baseline,
            &mut batch,
            &mut report.apply,
        );
        flush_reconcile_batch(target, &mut batch, sink, &mut report.apply)?;
        if !should_descend(kind, attrs, start_depth.saturating_sub(1), root_dev, config) {
            if kind.is_dir() {
                remove_known_children(
                    target,
                    subtree,
                    config,
                    &mut batch,
                    sink,
                    &mut report.apply,
                )?;
            }
            return Ok(report);
        }
    }

    let mut queue = vec![(subtree.to_path_buf(), start_depth)];
    while let Some((rel_dir, depth)) = queue.pop() {
        let mut known = target.child_states(&rel_dir)?;
        let abs_dir = root.join(&rel_dir);
        let listing = match fs::read_dir(&abs_dir) {
            Ok(listing) => listing,
            Err(error) => {
                report.scan.errors.push(Error::io(abs_dir, error));
                continue;
            }
        };
        report.scan.dirs_read += 1;
        let mut listing_complete = true;

        for item in listing {
            let item = match item {
                Ok(item) => item,
                Err(error) => {
                    listing_complete = false;
                    report.scan.errors.push(Error::io(&abs_dir, error));
                    continue;
                }
            };
            let name = item.file_name();
            let rel_path = rel_dir.join(&name);
            let baseline = match known.remove(&name) {
                Some(baseline) => baseline,
                None => target.expectation(&rel_path)?,
            };
            let meta = match metadata_for_fingerprint(&item) {
                Ok(meta) => meta,
                Err(error) => {
                    report.scan.errors.push(Error::io(item.path(), error));
                    continue;
                }
            };
            let kind = kind_from(&meta);
            let attrs = attrs_from(&meta);
            report.scan.entries += 1;
            push_reconcile_upsert(
                target,
                &rel_path,
                kind,
                attrs,
                baseline,
                &mut batch,
                &mut report.apply,
            );
            if batch.len() >= config.batch_size.max(1) {
                flush_reconcile_batch(target, &mut batch, sink, &mut report.apply)?;
            }

            if should_descend(kind, attrs, depth, root_dev, config) {
                queue.push((rel_path, depth + 1));
            } else if kind.is_dir() {
                remove_known_children(
                    target,
                    &rel_path,
                    config,
                    &mut batch,
                    sink,
                    &mut report.apply,
                )?;
            }
        }

        if listing_complete {
            for (name, baseline) in known {
                batch.push(ObservationOp::if_state(
                    Op::Remove { path: rel_dir.join(name) },
                    baseline,
                ));
                if batch.len() >= config.batch_size.max(1) {
                    flush_reconcile_batch(target, &mut batch, sink, &mut report.apply)?;
                }
            }
        }
    }

    flush_reconcile_batch(target, &mut batch, sink, &mut report.apply)?;
    Ok(report)
}

/// Drain and reconcile every pending invalidation, collapsing nested requests.
pub fn reconcile_pending(
    index: &mut Index,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    let mut target = ReconcileTarget::Direct(index);
    reconcile_pending_target(&mut target, config, sink)
}

/// Drain and reconcile invalidations on a shared index.
pub fn reconcile_pending_handle(
    handle: &IndexHandle,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    let mut target = ReconcileTarget::Shared(handle);
    reconcile_pending_target(&mut target, config, sink)
}

fn reconcile_pending_target(
    target: &mut ReconcileTarget<'_>,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    config.validate_for_scope(target.scope()?)?;
    let roots = take_invalidation_roots(target)?;
    let mut combined = ReconcileReport::default();
    for (position, (root, reason)) in roots.iter().enumerate() {
        match reconcile_target(target, root, config, sink) {
            Ok(report) => {
                if !report.is_complete() {
                    target.restore_pending_invalidations(vec![(root.clone(), *reason)])?;
                }
                merge_reconcile_report(&mut combined, report);
            }
            Err(error) => {
                target.restore_pending_invalidations(roots[position..].to_vec())?;
                return Err(error);
            }
        }
    }
    Ok(combined)
}

fn take_invalidation_roots(
    target: &mut ReconcileTarget<'_>,
) -> Result<Vec<(PathBuf, crate::InvalidateReason)>> {
    let mut pending = target.take_pending_invalidations()?;
    pending.sort_by(|(left, _), (right, _)| {
        left.components().count().cmp(&right.components().count()).then_with(|| left.cmp(right))
    });
    let mut roots: Vec<(PathBuf, crate::InvalidateReason)> = Vec::new();
    for (path, reason) in pending {
        if roots.iter().any(|(root, _)| path.starts_with(root)) {
            continue;
        }
        roots.push((path, reason));
    }

    Ok(roots)
}

fn remove_known_children(
    target: &mut ReconcileTarget<'_>,
    path: &Path,
    config: &ScanConfig,
    batch: &mut Vec<ObservationOp>,
    sink: &mut dyn FnMut(&AppliedDelta),
    stats: &mut ApplyStats,
) -> Result<()> {
    for (name, baseline) in target.child_states(path)? {
        batch.push(ObservationOp::if_state(Op::Remove { path: path.join(name) }, baseline));
        if batch.len() >= config.batch_size.max(1) {
            flush_reconcile_batch(target, batch, sink, stats)?;
        }
    }
    flush_reconcile_batch(target, batch, sink, stats)
}

fn push_reconcile_upsert(
    target: &ReconcileTarget<'_>,
    path: &Path,
    kind: EntryKind,
    attrs: Attrs,
    baseline: PathExpectation,
    batch: &mut Vec<ObservationOp>,
    stats: &mut ApplyStats,
) {
    // An exclusive Index borrow cannot race another index producer. If filesystem
    // metadata exactly matches the captured state, applying this upsert can only be a
    // no-op, so avoid allocating an owned op and walking the index again. Shared
    // reconciliation keeps the conditional observation so ABA arbitration remains
    // authoritative between its read and write lock boundaries.
    if target.direct_upsert_is_unchanged(baseline, kind, attrs) {
        stats.unchanged += 1;
        return;
    }
    batch.push(ObservationOp::if_state(
        Op::Upsert { path: path.to_path_buf(), kind, attrs },
        baseline,
    ));
}

fn flush_reconcile_batch(
    target: &mut ReconcileTarget<'_>,
    batch: &mut Vec<ObservationOp>,
    sink: &mut dyn FnMut(&AppliedDelta),
    stats: &mut ApplyStats,
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }
    let outcome = target.apply(&Observation::from_ops(std::mem::take(batch)))?;
    merge_apply_stats(stats, outcome.stats);
    if let Some(applied) = &outcome.applied {
        sink(applied);
    }
    Ok(())
}

fn merge_apply_stats(total: &mut ApplyStats, addition: ApplyStats) {
    total.inserted += addition.inserted;
    total.updated += addition.updated;
    total.removed += addition.removed;
    total.unchanged += addition.unchanged;
    total.invalidated += addition.invalidated;
    total.stale += addition.stale;
}

fn merge_reconcile_report(total: &mut ReconcileReport, addition: ReconcileReport) {
    total.scan.dirs_read += addition.scan.dirs_read;
    total.scan.entries += addition.scan.entries;
    total.scan.errors.extend(addition.scan.errors);
    merge_apply_stats(&mut total.apply, addition.apply);
}

fn should_descend(
    kind: EntryKind,
    attrs: Attrs,
    parent_depth: usize,
    root_dev: u64,
    config: &ScanConfig,
) -> bool {
    let within_depth = config.max_depth.is_none_or(|max| parent_depth + 1 < max);
    let same_filesystem = !config.one_filesystem || attrs.dev == root_dev || attrs.dev == 0;
    kind.is_dir() && within_depth && same_filesystem
}

pub(crate) fn normalize_subtree(path: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(Error::PathEscapesRoot(path.to_path_buf()));
            }
        }
    }
    Ok(normalized)
}

fn resolve_subtree_root(
    target: &ReconcileTarget<'_>,
    subtree: &Path,
    config: &ScanConfig,
) -> Result<PathBuf> {
    if subtree.as_os_str().is_empty() {
        return Ok(PathBuf::new());
    }
    let root = target.root_path()?;
    let Ok(root_metadata) = fs::symlink_metadata(&root) else {
        // The applying pass reports operational root failures as partial.
        return Ok(subtree.to_path_buf());
    };
    if !root_metadata.is_dir() {
        return Ok(subtree.to_path_buf());
    }
    let root_dev = attrs_from(&root_metadata).dev;
    let mut prefix = PathBuf::new();
    let mut components = subtree.components().peekable();
    while let Some(component) = components.next() {
        if components.peek().is_none() {
            break; // The boundary entry itself remains visible even when descent stops.
        }
        prefix.push(component.as_os_str());
        let metadata = match fs::symlink_metadata(root.join(&prefix)) {
            Ok(metadata) => metadata,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) =>
            {
                return Ok(prefix);
            }
            Err(_) => break, // The applying pass records operational failures as partial.
        };
        if metadata.file_type().is_symlink() {
            return Err(Error::SubtreeOutsideScanScope {
                path: subtree.to_path_buf(),
                scope: config.scope(),
            });
        }
        if !metadata.is_dir() {
            return Ok(prefix);
        }
        let attrs = attrs_from(&metadata);
        if config.one_filesystem && attrs.dev != root_dev && attrs.dev != 0 {
            return Err(Error::SubtreeOutsideScanScope {
                path: subtree.to_path_buf(),
                scope: config.scope(),
            });
        }
    }
    Ok(subtree.to_path_buf())
}

/// Read an entry's kind and roll-up attributes out of its metadata.
///
/// Exposed so the watch layer verifies entries exactly the way the walker records them —
/// two stat interpretations that could drift would show up as an index that disagrees
/// with itself depending on which producer last touched a path.
pub fn observe(meta: &fs::Metadata) -> (EntryKind, Attrs) {
    (kind_from(meta), attrs_from(meta))
}

fn kind_from(meta: &fs::Metadata) -> EntryKind {
    let file_type = meta.file_type();
    if file_type.is_symlink() {
        EntryKind::Symlink
    } else if file_type.is_dir() {
        EntryKind::Dir
    } else if file_type.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    }
}

#[cfg(unix)]
fn attrs_from(meta: &fs::Metadata) -> Attrs {
    use std::os::unix::fs::MetadataExt;
    Attrs {
        size: meta.size(),
        // st_blocks is in 512-byte units by POSIX convention regardless of the
        // filesystem's own block size.
        allocated: meta.blocks().saturating_mul(512),
        mtime_ns: compose_ns(meta.mtime(), meta.mtime_nsec()),
        ctime_ns: compose_ns(meta.ctime(), meta.ctime_nsec()),
        inode: meta.ino(),
        dev: meta.dev(),
    }
}

#[cfg(unix)]
fn compose_ns(secs: i64, nanos: i64) -> i64 {
    secs.saturating_mul(1_000_000_000).saturating_add(nanos)
}

#[cfg(not(unix))]
fn attrs_from(meta: &fs::Metadata) -> Attrs {
    let mtime_ns = meta.modified().map(system_time_ns).unwrap_or(0);
    Attrs {
        size: meta.len(),
        // No allocated size without platform-specific calls; apparent size is the
        // honest fallback rather than a guess at block rounding.
        allocated: meta.len(),
        mtime_ns,
        // Windows has no ctime in the Unix sense. Leaving it zero means the fingerprint
        // degrades to size + mtime there, which is what every portable tool does.
        ctime_ns: 0,
        inode: 0,
        dev: 0,
    }
}

#[cfg(any(not(unix), test))]
fn system_time_ns(time: std::time::SystemTime) -> i64 {
    match time.duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => i64::try_from(duration.as_nanos()).unwrap_or(i64::MAX),
        Err(error) => {
            i64::try_from(error.duration().as_nanos()).map_or(i64::MIN, i64::saturating_neg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn write_file(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        let mut f = File::create(path).expect("create file");
        f.write_all(contents).expect("write");
    }

    fn sample_tree() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(&dir.path().join("a.txt"), b"hello");
        write_file(&dir.path().join("src/main.rs"), b"fn main() {}");
        write_file(&dir.path().join("src/deep/nested.rs"), b"// nested");
        dir
    }

    #[test]
    fn fingerprint_metadata_observes_mutation_after_directory_enumeration() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("changing.bin");
        write_file(&path, b"before");
        let entry = fs::read_dir(dir.path())
            .expect("read directory")
            .next()
            .expect("one entry")
            .expect("read entry");

        write_file(&path, b"after mutation");

        let metadata = metadata_for_fingerprint(&entry).expect("fresh metadata");
        assert_eq!(metadata.len(), b"after mutation".len() as u64);
    }

    /// A tree wide and deep enough that workers genuinely interleave.
    ///
    /// A three-file fixture would pass every one of these tests with a broken queue,
    /// because one worker would finish before another started.
    fn branching_tree() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        for top in 0..12 {
            for middle in 0..6 {
                for leaf in 0..7 {
                    write_file(
                        &dir.path().join(format!("t{top}/m{middle}/leaf-{leaf}.dat")),
                        &vec![b'x'; leaf * 13],
                    );
                }
            }
            // A deep chain alongside the wide fan-out, so depth and width are both
            // exercised by the same walk.
            write_file(&dir.path().join(format!("t{top}/a/b/c/d/e/deep.txt")), b"deep");
        }
        dir
    }

    fn index_fingerprint(index: &Index) -> Vec<(PathBuf, EntryKind, Attrs)> {
        let mut entries: Vec<(PathBuf, EntryKind, Attrs)> = Vec::new();
        let mut queue = vec![PathBuf::new()];
        while let Some(path) = queue.pop() {
            let Some(children) = index.children(&path) else {
                continue;
            };
            let names: Vec<PathBuf> = children.map(|(name, _id)| path.join(name)).collect();
            for child_path in names {
                let kind = index.kind(&child_path).expect("child has a kind");
                let attrs = *index.attrs(&child_path).expect("child has attrs");
                entries.push((child_path.clone(), kind, attrs));
                if kind.is_dir() {
                    queue.push(child_path);
                }
            }
        }
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        entries
    }

    #[test]
    fn parallel_and_serial_walks_produce_the_same_index() {
        let dir = branching_tree();
        let serial_config = ScanConfig { threads: Some(1), ..ScanConfig::default() };
        let (serial, serial_report) =
            scan_into_index(dir.path(), &serial_config).expect("serial scan");
        assert!(serial_report.is_complete());

        for threads in [2_usize, 3, 8] {
            let config = ScanConfig { threads: Some(threads), ..ScanConfig::default() };
            let (parallel, report) = scan_into_index(dir.path(), &config).expect("parallel scan");
            assert!(report.is_complete(), "{threads} threads reported errors");
            assert_eq!(report.entries, serial_report.entries, "{threads} threads");
            assert_eq!(report.dirs_read, serial_report.dirs_read, "{threads} threads");
            assert_eq!(parallel.total(), serial.total(), "{threads} threads roll-up");
            assert_eq!(
                index_fingerprint(&parallel),
                index_fingerprint(&serial),
                "{threads} threads produced a different index"
            );
        }
    }

    #[test]
    fn parallel_walk_emits_every_entry_exactly_once() {
        let dir = branching_tree();
        let config = ScanConfig { threads: Some(4), batch_size: 16, ..ScanConfig::default() };
        let mut seen: BTreeMap<PathBuf, usize> = BTreeMap::new();
        let report = scan(dir.path(), &config, &mut |observation| {
            for op in &observation.ops {
                if let Op::Upsert { path, .. } = &op.op {
                    *seen.entry(path.clone()).or_default() += 1;
                }
            }
        })
        .expect("parallel scan");

        assert!(report.is_complete());
        assert_eq!(seen.len() as u64, report.entries, "entry count disagrees with the report");
        let duplicated: Vec<_> =
            seen.iter().filter(|(_path, count)| **count != 1).map(|(path, _)| path).collect();
        assert!(duplicated.is_empty(), "paths emitted more than once: {duplicated:?}");
    }

    #[test]
    fn parallel_walk_honours_max_depth() {
        let dir = branching_tree();
        for threads in [1_usize, 4] {
            let config =
                ScanConfig { threads: Some(threads), max_depth: Some(2), ..ScanConfig::default() };
            let (index, report) = scan_into_index(dir.path(), &config).expect("scan");
            assert!(report.is_complete());
            for (path, _kind, _attrs) in index_fingerprint(&index) {
                assert!(
                    path.components().count() <= 2,
                    "{threads} threads kept {path:?} past the depth limit"
                );
            }
        }
    }

    #[test]
    fn worker_threads_are_bounded_and_never_zero() {
        let zero = ScanConfig { threads: Some(0), ..ScanConfig::default() };
        assert_eq!(zero.worker_threads(), 1, "zero threads must fall back to the serial walk");
        let absurd = ScanConfig { threads: Some(usize::MAX), ..ScanConfig::default() };
        assert_eq!(absurd.worker_threads(), MAX_SCAN_THREADS);
        // The automatic choice is capped well below what a caller may request, because
        // the measured knee is far below the core count on a large machine.
        let automatic = ScanConfig { threads: None, ..ScanConfig::default() };
        assert!((1..=DEFAULT_SCAN_THREADS_CAP).contains(&automatic.worker_threads()));
    }

    #[test]
    fn thread_count_does_not_change_the_cache_scope() {
        // Threads are an operational choice. If they leaked into the scope, changing
        // the pool size would invalidate every snapshot on disk.
        let serial = ScanConfig { threads: Some(1), ..ScanConfig::default() };
        let parallel = ScanConfig { threads: Some(8), ..ScanConfig::default() };
        assert_eq!(serial.scope(), parallel.scope());
    }

    #[test]
    fn scan_populates_an_index_end_to_end() {
        let dir = sample_tree();
        let (index, report) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");

        assert!(report.is_complete(), "unexpected errors: {:?}", report.errors);
        let total = index.total();
        assert_eq!(total.files, 3);
        assert_eq!(total.dirs, 2);
        assert_eq!(total.bytes, 5 + 12 + 9);
        assert_eq!(total.by_ext[".rs"].files, 2);
        assert_eq!(total.by_ext[".txt"].files, 1);

        let src = index.rollup(Path::new("src")).expect("src");
        assert_eq!(src.files, 2);
        assert_eq!(src.dirs, 1);
    }

    #[cfg(unix)]
    #[test]
    fn directory_entry_metadata_does_not_follow_symlinks() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().expect("root");
        let outside = tempfile::tempdir().expect("outside");
        write_file(&outside.path().join("must-not-be-scanned.txt"), b"outside");
        symlink(outside.path(), root.path().join("link")).expect("symlink");

        let (index, report) = scan_into_index(root.path(), &ScanConfig::default()).expect("scan");

        assert!(report.is_complete(), "unexpected errors: {:?}", report.errors);
        assert_eq!(index.kind(Path::new("link")), Some(EntryKind::Symlink));
        assert!(index.lookup(Path::new("link/must-not-be-scanned.txt")).is_none());
        assert_eq!(index.total().files, 0);
    }

    #[test]
    fn cold_scan_establishes_a_baseline_without_change_history() {
        let dir = sample_tree();
        let (index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");

        assert_eq!(index.clock(), crate::Clock::ZERO);
        assert!(index.since(crate::Clock::ZERO).deltas.is_empty());
    }

    #[test]
    fn max_depth_stops_descent() {
        let dir = sample_tree();
        let config = ScanConfig { max_depth: Some(1), ..ScanConfig::default() };
        let (index, _) = scan_into_index(dir.path(), &config).expect("scan");

        assert!(index.lookup(Path::new("src")).is_some());
        assert!(index.lookup(Path::new("src/main.rs")).is_none());
    }

    #[test]
    fn zero_max_depth_keeps_only_the_index_root() {
        let dir = sample_tree();
        let config = ScanConfig { max_depth: Some(0), ..ScanConfig::default() };
        let (index, report) = scan_into_index(dir.path(), &config).expect("scan");

        assert!(index.is_empty());
        assert_eq!(report.entries, 0);
        assert_eq!(report.dirs_read, 0);
    }

    #[test]
    fn direct_scan_records_the_canonical_root() {
        let dir = sample_tree();
        let aliased = dir.path().join(".");
        let (index, _) = scan_into_index(&aliased, &ScanConfig::default()).expect("scan");

        assert_eq!(index.root_path(), dir.path().canonicalize().expect("canonical root"));
    }

    #[test]
    fn unsupported_symlink_following_is_rejected_on_cold_and_warm_paths() {
        let dir = sample_tree();
        let unsupported = ScanConfig { follow_symlinks: true, ..ScanConfig::default() };
        assert!(matches!(
            scan_into_index(dir.path(), &unsupported),
            Err(Error::UnsupportedScanConfig(_))
        ));

        let (index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        assert!(matches!(
            revalidate(&index, &unsupported, &mut |_| {}),
            Err(Error::UnsupportedScanConfig(_))
        ));
    }

    #[test]
    fn revalidation_uses_the_same_depth_boundary_as_cold_scan() {
        let dir = sample_tree();
        let config = ScanConfig { max_depth: Some(1), ..ScanConfig::default() };
        let (mut index, _) = scan_into_index(dir.path(), &config).expect("scan");
        write_file(&dir.path().join("src/added-after-scan.txt"), b"new");

        let mut observations = Vec::new();
        revalidate(&index, &config, &mut |observation| observations.push(observation))
            .expect("revalidate");
        for observation in &observations {
            index.apply_ok(observation);
        }

        assert!(index.lookup(Path::new("src/added-after-scan.txt")).is_none());
    }

    #[test]
    fn zero_depth_revalidation_prunes_cached_root_children() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config = ScanConfig { max_depth: Some(0), ..ScanConfig::default() };
        let mut index = Index::new_with_scope(dir.path(), config.scope());
        index.apply_baseline_ok(&Observation::new(vec![Op::Upsert {
            path: PathBuf::from("stale.txt"),
            kind: EntryKind::File,
            attrs: Attrs::default(),
        }]));

        let mut observations = Vec::new();
        let report = revalidate(&index, &config, &mut |observation| {
            observations.push(observation);
        })
        .expect("revalidate");
        for observation in &observations {
            index.apply_ok(observation);
        }

        assert!(index.is_empty());
        assert_eq!(report.dirs_read, 0);
    }

    #[test]
    fn zero_depth_applying_reconciliation_prunes_cached_root_children() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config = ScanConfig { max_depth: Some(0), ..ScanConfig::default() };
        let mut index = Index::new_with_scope(dir.path(), config.scope());
        index.apply_baseline_ok(&Observation::new(vec![Op::Upsert {
            path: PathBuf::from("stale.txt"),
            kind: EntryKind::File,
            attrs: Attrs::default(),
        }]));

        let report = reconcile(&mut index, &config, &mut |_| {}).expect("reconcile");

        assert!(index.is_empty());
        assert_eq!(report.scan.dirs_read, 0);
    }

    #[test]
    fn filesystem_boundary_is_part_of_the_shared_descent_policy() {
        let config = ScanConfig { one_filesystem: true, ..ScanConfig::default() };
        let attrs = Attrs { dev: 22, ..Attrs::default() };
        assert!(!should_descend(EntryKind::Dir, attrs, 0, 11, &config));
        assert!(should_descend(EntryKind::Dir, Attrs { dev: 11, ..attrs }, 0, 11, &config,));
    }

    #[test]
    fn scanning_a_file_is_an_error_not_a_panic() {
        let dir = sample_tree();
        let err = scan_into_index(&dir.path().join("a.txt"), &ScanConfig::default());
        assert!(err.is_err());
    }

    #[test]
    fn deltas_arrive_in_batches_of_the_configured_size() {
        let dir = tempfile::tempdir().expect("tempdir");
        for i in 0..25 {
            write_file(&dir.path().join(format!("f{i}.txt")), b"x");
        }
        let config = ScanConfig { batch_size: 10, ..ScanConfig::default() };
        let mut sizes = Vec::new();
        scan(dir.path(), &config, &mut |d| sizes.push(d.len())).expect("scan");

        assert!(sizes.len() >= 3, "expected several batches, got {sizes:?}");
        assert!(sizes.iter().all(|&n| n <= 10));
        assert_eq!(sizes.iter().sum::<usize>(), 25);
    }

    #[test]
    fn invalid_batch_sizes_are_rejected_before_allocation() {
        let zero = ScanConfig { batch_size: 0, ..ScanConfig::default() };
        let unbounded = ScanConfig { batch_size: usize::MAX, ..ScanConfig::default() };

        assert!(matches!(zero.validate(), Err(Error::UnsupportedScanConfig(_))));
        assert!(matches!(unbounded.validate(), Err(Error::UnsupportedScanConfig(_))));
    }

    #[test]
    fn stale_arbitration_keeps_a_reconciliation_incomplete() {
        let report = ReconcileReport {
            scan: ScanReport::default(),
            apply: ApplyStats { stale: 1, ..ApplyStats::default() },
        };

        assert!(!report.is_complete());
    }

    #[test]
    fn portable_system_time_conversion_preserves_pre_epoch_values() {
        let before_epoch = std::time::UNIX_EPOCH
            // Windows timestamps have 100 ns granularity, so use a duration that every
            // supported platform can represent without rounding back to the epoch.
            .checked_sub(std::time::Duration::from_secs(1))
            .expect("represent pre-epoch fixture");

        assert_eq!(system_time_ns(before_epoch), -1_000_000_000);
        assert_eq!(system_time_ns(std::time::UNIX_EPOCH), 0);
    }

    #[cfg(not(unix))]
    #[test]
    fn one_filesystem_fails_when_device_identity_is_unavailable() {
        let config = ScanConfig { one_filesystem: true, ..ScanConfig::default() };

        assert!(matches!(config.validate(), Err(Error::UnsupportedScanConfig(_))));
    }

    #[test]
    fn revalidate_is_a_no_op_against_an_unchanged_tree() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        let before = index.total().clone();

        let mut deltas = Vec::new();
        revalidate(&index, &ScanConfig::default(), &mut |d| deltas.push(d)).expect("revalidate");
        let mut unchanged = 0;
        for delta in &deltas {
            unchanged += index.apply_ok(delta).unchanged;
        }

        assert_eq!(unchanged, 5, "3 files + 2 dirs all already known");
        assert_eq!(index.total(), &before);
    }

    #[test]
    fn direct_reconciliation_counts_unchanged_entries_without_publishing_deltas() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        let before_total = index.total().clone();
        let before_clock = index.clock();
        let mut deltas = Vec::new();

        let report = reconcile(&mut index, &ScanConfig::default(), &mut |delta| {
            deltas.push(delta.clone());
        })
        .expect("reconcile");

        assert!(report.is_complete());
        assert_eq!(report.apply.unchanged, 5, "3 files + 2 dirs all already known");
        assert!(deltas.is_empty());
        assert_eq!(index.clock(), before_clock);
        assert_eq!(index.total(), &before_total);
    }

    #[test]
    fn shared_reconciliation_retains_conditional_no_op_arbitration() {
        let dir = sample_tree();
        let (index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        let handle = crate::IndexHandle::new(index);
        let before_clock = handle.clock().expect("clock");
        let mut deltas = Vec::new();

        let report = reconcile_handle(&handle, &ScanConfig::default(), &mut |delta| {
            deltas.push(delta.clone());
        })
        .expect("reconcile");

        assert!(report.is_complete());
        assert_eq!(report.apply.unchanged, 5, "3 files + 2 dirs all already known");
        assert!(deltas.is_empty());
        assert_eq!(handle.clock().expect("clock"), before_clock);
    }

    #[test]
    fn revalidate_detects_additions_edits_and_deletions() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");

        fs::remove_file(dir.path().join("a.txt")).expect("remove");
        write_file(&dir.path().join("src/main.rs"), b"fn main() { longer }");
        write_file(&dir.path().join("added.md"), b"new");

        let mut deltas = Vec::new();
        revalidate(&index, &ScanConfig::default(), &mut |d| deltas.push(d)).expect("revalidate");
        let mut stats = crate::index::ApplyStats::default();
        for delta in &deltas {
            let s = index.apply_ok(delta);
            stats.inserted += s.inserted;
            stats.updated += s.updated;
            stats.removed += s.removed;
        }

        assert_eq!(stats.inserted, 1, "added.md");
        assert_eq!(stats.updated, 1, "main.rs grew");
        assert_eq!(stats.removed, 1, "a.txt is gone");

        let total = index.total();
        assert_eq!(total.files, 3);
        assert_eq!(total.bytes, 20 + 9 + 3);
        assert!(!total.by_ext.contains_key(".txt"));
        assert_eq!(total.by_ext[".md"].files, 1);
    }

    #[test]
    fn revalidate_removes_a_whole_vanished_directory() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        fs::remove_dir_all(dir.path().join("src")).expect("remove dir");

        let mut deltas = Vec::new();
        revalidate(&index, &ScanConfig::default(), &mut |d| deltas.push(d)).expect("revalidate");
        for delta in &deltas {
            index.apply_ok(delta);
        }

        let total = index.total();
        assert_eq!(total.files, 1);
        assert_eq!(total.dirs, 0);
        assert!(index.lookup(Path::new("src")).is_none());
    }

    #[test]
    fn pending_invalidation_reconciles_the_requested_subtree() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        write_file(&dir.path().join("src/added.rs"), b"new");
        index.apply_ok(&Observation::new(vec![Op::InvalidateSubtree {
            path: PathBuf::from("src"),
            reason: crate::InvalidateReason::Requested,
        }]));
        assert_eq!(index.freshness_at(Path::new("src")), crate::Freshness::Stale);

        let mut applied = Vec::new();
        let report = reconcile_pending(&mut index, &ScanConfig::default(), &mut |delta| {
            applied.push(delta.clone());
        })
        .expect("reconcile pending");

        assert!(report.is_complete());
        assert!(index.lookup(Path::new("src/added.rs")).is_some());
        assert_eq!(index.freshness_at(Path::new("src")), crate::Freshness::Fresh);
        assert!(index.take_pending_invalidations().is_empty());
        assert!(
            applied
                .iter()
                .any(|delta| { delta.ops.iter().any(|op| op.path() == Path::new("src/added.rs")) })
        );
    }

    #[test]
    fn handle_reconciliation_publishes_after_each_delta_is_applied() {
        let dir = sample_tree();
        let (index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        let handle = crate::IndexHandle::new(index);
        let reader = handle.clone();
        write_file(&dir.path().join("added.md"), b"new");

        let mut observed_after_apply = false;
        reconcile_handle(&handle, &ScanConfig::default(), &mut |delta| {
            if delta.ops.iter().any(|op| op.path() == Path::new("added.md")) {
                observed_after_apply =
                    reader.kind(Path::new("added.md")).expect("query index").is_some();
            }
        })
        .expect("reconcile handle");

        assert!(observed_after_apply);
    }

    #[test]
    fn reconciliation_does_not_clear_a_newer_invalidation() {
        let dir = sample_tree();
        let (index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        let handle = crate::IndexHandle::new(index);
        write_file(&dir.path().join("added.md"), b"new");

        let invalidator = handle.clone();
        let mut saw_reconciling = false;
        reconcile_handle(&handle, &ScanConfig::default(), &mut |delta| {
            if delta.ops.iter().any(|op| op.path() == Path::new("added.md")) {
                saw_reconciling =
                    invalidator.freshness().expect("query") == crate::Freshness::Reconciling;
                invalidator
                    .apply(&Observation::new(vec![Op::InvalidateSubtree {
                        path: PathBuf::new(),
                        reason: crate::InvalidateReason::WatchOverflow,
                    }]))
                    .expect("new invalidation");
            }
        })
        .expect("reconcile handle");

        assert!(saw_reconciling);
        assert_eq!(handle.freshness().expect("query"), crate::Freshness::Stale);
    }

    #[test]
    fn failed_reconciliation_marks_the_scope_partial() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        fs::remove_dir_all(dir.path()).expect("remove root");

        assert!(reconcile(&mut index, &ScanConfig::default(), &mut |_| {}).is_err());
        assert_eq!(index.freshness(), crate::Freshness::Partial);
    }

    #[test]
    fn failed_pending_reconciliation_remains_queued_for_retry() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        index.apply_ok(&Observation::new(vec![Op::InvalidateSubtree {
            path: PathBuf::new(),
            reason: crate::InvalidateReason::Requested,
        }]));
        fs::remove_dir_all(dir.path()).expect("remove root");

        assert!(reconcile_pending(&mut index, &ScanConfig::default(), &mut |_| {}).is_err());
        assert_eq!(
            index.take_pending_invalidations(),
            vec![(PathBuf::new(), crate::InvalidateReason::Requested)]
        );
        assert_eq!(index.freshness(), crate::Freshness::Partial);
    }

    #[cfg(unix)]
    #[test]
    fn partial_pending_reconciliation_remains_queued_for_retry() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("tempdir");
        write_file(&dir.path().join("blocked/known.txt"), b"known");
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        let blocked = dir.path().join("blocked");
        fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000)).expect("deny reads");
        index.apply_ok(&Observation::new(vec![Op::InvalidateSubtree {
            path: PathBuf::from("blocked"),
            reason: crate::InvalidateReason::VerificationFailed,
        }]));

        let report = reconcile_pending(&mut index, &ScanConfig::default(), &mut |_| {})
            .expect("permission failure is a partial report");
        let pending = index.take_pending_invalidations();
        fs::set_permissions(&blocked, fs::Permissions::from_mode(0o700)).expect("restore reads");
        if report.is_complete() {
            return; // Privileged test environments can read mode-000 directories.
        }

        assert_eq!(
            pending,
            vec![(PathBuf::from("blocked"), crate::InvalidateReason::VerificationFailed)]
        );
        assert_eq!(index.freshness_at(Path::new("blocked")), crate::Freshness::Partial);
    }

    #[test]
    fn pending_scope_mismatch_does_not_drain_the_retry_queue() {
        let dir = tempfile::tempdir().expect("tempdir");
        let shallow = ScanConfig { max_depth: Some(1), ..ScanConfig::default() };
        let (mut index, _) = scan_into_index(dir.path(), &shallow).expect("scan");
        index.apply_ok(&Observation::new(vec![Op::InvalidateSubtree {
            path: PathBuf::new(),
            reason: crate::InvalidateReason::Requested,
        }]));

        let error = reconcile_pending(&mut index, &ScanConfig::default(), &mut |_| {})
            .expect_err("mismatched scope must fail");

        assert!(matches!(error, Error::ScanScopeMismatch { .. }));
        assert_eq!(
            index.take_pending_invalidations(),
            vec![(PathBuf::new(), crate::InvalidateReason::Requested)]
        );
    }

    #[test]
    fn reconciliation_rejects_a_scope_mismatch_before_mutating() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(&dir.path().join("deep/nested.txt"), b"nested");
        let shallow = ScanConfig { max_depth: Some(1), ..ScanConfig::default() };
        let (mut index, _) = scan_into_index(dir.path(), &shallow).expect("scan");
        assert!(index.lookup(Path::new("deep/nested.txt")).is_none());

        let error = reconcile(&mut index, &ScanConfig::default(), &mut |_| {})
            .expect_err("mismatched scope must fail");

        assert!(matches!(error, Error::ScanScopeMismatch { .. }));
        assert!(index.lookup(Path::new("deep/nested.txt")).is_none());
        assert_eq!(index.freshness(), crate::Freshness::Fresh);
    }

    #[test]
    fn subtree_reconciliation_rejects_a_path_beyond_the_depth_scope() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(&dir.path().join("deep/nested.txt"), b"nested");
        let shallow = ScanConfig { max_depth: Some(1), ..ScanConfig::default() };
        let (mut index, _) = scan_into_index(dir.path(), &shallow).expect("scan");

        let result =
            reconcile_subtree(&mut index, Path::new("deep/nested.txt"), &shallow, &mut |_| {});

        assert!(matches!(result, Err(Error::SubtreeOutsideScanScope { .. })));
        assert!(index.lookup(Path::new("deep/nested.txt")).is_none());
        assert_eq!(index.freshness(), crate::Freshness::Fresh);
    }

    #[cfg(unix)]
    #[test]
    fn subtree_reconciliation_does_not_follow_an_ancestor_symlink() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().expect("root");
        let outside = tempfile::tempdir().expect("outside");
        write_file(&outside.path().join("secret.txt"), b"secret");
        symlink(outside.path(), root.path().join("link")).expect("symlink");
        let config = ScanConfig::default();
        let (mut index, _) = scan_into_index(root.path(), &config).expect("scan");

        let result =
            reconcile_subtree(&mut index, Path::new("link/secret.txt"), &config, &mut |_| {});

        assert!(matches!(result, Err(Error::SubtreeOutsideScanScope { .. })));
        assert!(index.lookup(Path::new("link/secret.txt")).is_none());
        assert_eq!(index.freshness(), crate::Freshness::Fresh);
    }

    #[test]
    fn subtree_reconciliation_widens_to_a_non_directory_ancestor() {
        let root = tempfile::tempdir().expect("root");
        write_file(&root.path().join("parent/child.txt"), b"old");
        let config = ScanConfig::default();
        let (mut index, _) = scan_into_index(root.path(), &config).expect("scan");
        fs::remove_dir_all(root.path().join("parent")).expect("remove directory");
        write_file(&root.path().join("parent"), b"replacement");

        let report =
            reconcile_subtree(&mut index, Path::new("parent/child.txt"), &config, &mut |_| {})
                .expect("reconcile widened ancestor");

        assert!(report.is_complete());
        assert_eq!(index.kind(Path::new("parent")), Some(EntryKind::File));
        assert!(index.lookup(Path::new("parent/child.txt")).is_none());
        assert_eq!(index.freshness(), crate::Freshness::Fresh);
    }

    #[test]
    fn subtree_reconciliation_widens_to_a_missing_ancestor() {
        let root = tempfile::tempdir().expect("root");
        write_file(&root.path().join("parent/child.txt"), b"old");
        let config = ScanConfig::default();
        let (mut index, _) = scan_into_index(root.path(), &config).expect("scan");
        fs::remove_dir_all(root.path().join("parent")).expect("remove directory");

        let report =
            reconcile_subtree(&mut index, Path::new("parent/child.txt"), &config, &mut |_| {})
                .expect("reconcile widened ancestor");

        assert!(report.is_complete());
        assert!(index.lookup(Path::new("parent")).is_none());
        assert_eq!(index.freshness(), crate::Freshness::Fresh);
    }

    #[test]
    fn observation_only_revalidation_rejects_a_scope_mismatch() {
        let dir = tempfile::tempdir().expect("tempdir");
        let shallow = ScanConfig { max_depth: Some(1), ..ScanConfig::default() };
        let (index, _) = scan_into_index(dir.path(), &shallow).expect("scan");
        let mut observations = Vec::new();

        let error = revalidate(&index, &ScanConfig::default(), &mut |observation| {
            observations.push(observation);
        })
        .expect_err("mismatched scope must fail");

        assert!(matches!(error, Error::ScanScopeMismatch { .. }));
        assert!(observations.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn a_new_filesystem_boundary_prunes_cached_descendants() {
        use std::os::unix::fs::MetadataExt;

        let root = Path::new("/");
        let root_dev = fs::symlink_metadata(root).expect("stat root").dev();
        let Some(mount) = [Path::new("/dev"), Path::new("/proc"), Path::new("/sys")]
            .into_iter()
            .find(|candidate| {
                fs::symlink_metadata(candidate)
                    .is_ok_and(|metadata| metadata.is_dir() && metadata.dev() != root_dev)
            })
        else {
            return; // This host exposes no convenient cross-device directory.
        };
        let relative = mount.strip_prefix(root).expect("mount is below root");
        let stale_child = relative.join(".fdu-stale-snapshot-entry");
        let config = ScanConfig { one_filesystem: true, ..ScanConfig::default() };
        let mount_meta = fs::symlink_metadata(mount).expect("stat mount");
        let mut index = Index::new_with_scope(root, config.scope());
        index.apply_baseline_ok(&Observation::new(vec![
            Op::Upsert {
                path: relative.to_path_buf(),
                kind: EntryKind::Dir,
                attrs: attrs_from(&mount_meta),
            },
            Op::Upsert {
                path: stale_child.clone(),
                kind: EntryKind::File,
                attrs: Attrs { size: 10, allocated: 10, ..Attrs::default() },
            },
        ]));

        let error = reconcile_subtree(&mut index, &stale_child, &config, &mut |_| {})
            .expect_err("a descendant below the mount boundary is outside scope");
        assert!(matches!(error, Error::SubtreeOutsideScanScope { .. }));
        assert!(index.lookup(&stale_child).is_some());

        reconcile_subtree(&mut index, relative, &config, &mut |_| {}).expect("reconcile mount");

        assert!(index.lookup(relative).is_some(), "the mount point itself stays visible");
        assert!(index.lookup(&stale_child).is_none(), "out-of-scope descendants are pruned");
    }

    #[test]
    fn subtree_reconciliation_rejects_paths_outside_the_root() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");

        assert!(matches!(
            reconcile_subtree(
                &mut index,
                Path::new("../outside"),
                &ScanConfig::default(),
                &mut |_| {},
            ),
            Err(Error::PathEscapesRoot(_))
        ));
        assert_eq!(index.freshness(), crate::Freshness::Fresh);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn scan_and_revalidate_keep_non_utf8_names_distinct() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let dir = tempfile::tempdir().expect("tempdir");
        let first = PathBuf::from(OsString::from_vec(vec![b'n', 0x80]));
        let second = PathBuf::from(OsString::from_vec(vec![b'n', 0x81]));
        write_file(&dir.path().join(&first), b"a");
        write_file(&dir.path().join(&second), b"bb");

        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        assert_eq!(index.total().files, 2);
        assert_eq!(index.total().bytes, 3);

        fs::remove_file(dir.path().join(&first)).expect("remove first");
        let mut observations = Vec::new();
        revalidate(&index, &ScanConfig::default(), &mut |observation| {
            observations.push(observation);
        })
        .expect("revalidate");
        for observation in &observations {
            index.apply_ok(observation);
        }
        assert!(index.lookup(&first).is_none());
        assert!(index.lookup(&second).is_some());
        assert_eq!(index.total().bytes, 2);
    }
}
