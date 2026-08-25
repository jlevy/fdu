//! The scan layer: walking a tree, producing observations, and applying reconciliation.
//!
//! A cold scan is just a large batch of upserts, and a revalidation sweep is the diff
//! between what the index believes and what the filesystem says. Both speak the same
//! [`Observation`] vocabulary as the watch layer, which is what lets the index be ignorant of
//! where its changes came from.
//!
//! # Status
//!
//! The serial walk is the portable `read_dir` plus non-following metadata reference.
//! Parallel scans use the same path on most platforms; on macOS they first try a
//! measured `getattrlistbulk` backend that returns directory entries and stat-tier
//! metadata together. Unsupported filesystems, malformed results, mount points, and
//! firmlinks fail closed to the portable path for the complete containing directory.
//! Every backend produces the same [`Observation`] contract.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::ffi::{OsStr, OsString};
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::ApplyStats;
use crate::engine_contract::{
    AppliedDelta, Attrs, CoverageReason, EntryKind, Error, Observation, ObservationOp, Op,
    PathExpectation, PathState, Result, ScanScope, Status,
};
use crate::index::{Index, IndexHandle, collect_child_expectations};

// Keep the FFI exception at the platform boundary. The rest of the engine, including
// every consumer of these observations, remains under the workspace's unsafe-code
// denial.
#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod macos_bulk;

/// How many ops accumulate before an observation is handed to the sink.
///
/// Batching matters for more than syscall economy: consumers coalesce per path within a
/// batch and stat once per batch, and a live UI wants partial results while a large tree
/// is still being walked rather than one delta at the end.
const DEFAULT_BATCH_SIZE: usize = crate::platform_tuning::tuning().batch_size.get();

/// Largest producer batch accepted before work must be published incrementally.
pub const MAX_SCAN_BATCH_SIZE: usize = 64 * 1024;

/// Most changed paths an exclusive parallel reconciliation may defer before applying.
///
/// Workers compare against one immutable index image, so mutations wait until the wave
/// joins. Bounding that change set keeps a churned tree from turning the fast unchanged
/// path into an unbounded allocation; overflow discards the wave and retries through
/// the incremental serial reconciler.
const MAX_DEFERRED_RECONCILE_OPS: usize = MAX_SCAN_BATCH_SIZE;

/// Directories compared against one immutable index baseline before changes are applied.
///
/// The wave is large enough to amortize scoped worker creation and small enough that a
/// changed tree publishes progress throughout a long reconciliation.
const RECONCILE_WAVE_DIRECTORIES: usize =
    crate::platform_tuning::tuning().reconcile_wave_directories.get();

/// Identity of the fixed stat-tier reducer set.
const REDUCERS_FINGERPRINT: u64 = 1;

/// The order directories are visited in.
///
/// This changes *when* observations are produced, never *which* ones: both orders
/// visit every entry exactly once and leave an identical index behind. It therefore
/// stays out of [`ScanScope`] and cannot invalidate a cache, exactly like the worker
/// count.
///
/// The choice only matters to a consumer that reads the index while the walk is still
/// running, and there it matters a great deal.
///
/// # Strength of the guarantee
///
/// **These are scheduling preferences, not strict orders, whenever more than one worker
/// is running** — which is the default.
///
/// The queue is ordered, but the *claims* are not. Workers take directories from the
/// shared queue in the policy's order; a worker that finishes early can enqueue its
/// children and another worker can claim them while a slower worker still holds
/// unfinished work from a shallower level. Nothing releases a level barrier, because
/// a barrier would idle every fast worker at each level boundary and give back most of
/// the parallel producer's win.
///
/// So:
///
/// - With `threads: Some(1)`, [`ScanOrder::BreadthFirst`] is strict: no directory is
///   read before one closer to the root.
/// - With several workers it is *shallow-first*: shallow work is always preferred when
///   a worker chooses, and deeper observations can still interleave.
///
/// That weaker property is what the browser use case actually needs — every top-level
/// subtree starts filling early, so a mid-scan ranking is meaningful — and it is the
/// property the tests pin. A caller that needs strict level order must ask for one
/// worker and pay for it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ScanOrder {
    /// Shallow directories before deep ones.
    ///
    /// The default, because it is the order whose partial results mean something.
    /// Roll-ups are maintained per directory as the walk proceeds, so a consumer that
    /// looks mid-scan sees top-level totals grow together — bars fill, rankings
    /// converge — instead of one subtree finishing while its siblings read zero.
    /// Interrupting early leaves a usefully complete picture of the top of the tree.
    ///
    /// Under several workers this is a preference rather than a guarantee; see the
    /// type-level note above.
    ///
    /// Note that totals only grow *while an additive walk is running*. Monotonicity
    /// comes from the producer being additive, not from the order — the order decides
    /// which subtrees get to grow early.
    #[default]
    BreadthFirst,
    /// One subtree toward completion before starting the next.
    ///
    /// Lower peak memory, since the frontier is bounded by depth rather than by the
    /// width of a level, and better locality within a subtree. The cost is that
    /// partial results are actively misleading: one child of the root approaches its
    /// final total while its siblings read zero, so anything ranking by size mid-scan
    /// ranks confidently and wrongly. Correct for a caller that only reads the
    /// finished index and wants the smallest footprint.
    ///
    /// Under several workers this too is a preference: several subtrees will be in
    /// flight at once, one per worker.
    DepthFirst,
}

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
    /// machine's available parallelism. The automatic pool starts conservatively and
    /// unlocks more latency-hiding workers only when initial chunk timing identifies a
    /// slow filesystem path.
    ///
    /// This is an operational knob, not a semantic one: it changes how fast the same
    /// observations are produced, never which observations they are. That is why it
    /// stays out of [`ScanScope`] and cannot invalidate a cache.
    pub threads: Option<usize>,
    /// The order directories are visited in. See [`ScanOrder`].
    pub order: ScanOrder,
    /// File-type rules to classify against, or `None` for the ones compiled into fdu.
    ///
    /// Unlike [`Self::threads`] this *is* semantic: a different taxonomy classifies the
    /// same tree differently, which is why its fingerprint rides in [`ScanScope`] and a
    /// change to it invalidates a snapshot. Shared rather than owned because a scan
    /// clones its config per wave and a registry is read-only once built.
    pub types: Option<std::sync::Arc<crate::classify::TypeRegistry>>,
    /// Tag rules to evaluate per entry, or `None` for none enabled.
    ///
    /// Semantic for the same reason [`Self::types`] is, and shared for the same reason.
    /// The default is the empty set, which fingerprints to zero and costs one branch per
    /// insert: a caller who asks for no tags pays for no tags.
    pub tags: Option<std::sync::Arc<crate::tags::TagRules>>,
    /// Which entries are inside the scan scope at all, or `None` to admit every one.
    ///
    /// Semantic in the strongest sense the word has here: a depth bound and a filesystem
    /// bound also decide what is retained, and like them this is fingerprinted into
    /// [`ScanScope`] so a snapshot recorded under one rule is never read under another.
    /// The default admits everything, fingerprints to zero, and costs one `bool` per entry
    /// -- fdu's own command line counts what is there, and pruning is opt-in.
    pub hidden: Option<std::sync::Arc<crate::admission::HiddenPolicy>>,
    /// Files this walk may observe before it stops descending, or `None` for unlimited.
    ///
    /// Semantic, and fingerprinted into [`ScanScope`] for a reason worth stating: a cap
    /// changes which entries exist in the index, and *which* ones depends on the order the
    /// walk reached them in. Two indexes of one tree under two caps are not nested sets a
    /// reader could reconcile, so a snapshot taken under one is never read under another.
    ///
    /// Files rather than entries, matching the bound a consumer contract actually states.
    /// Directories are the structure the files hang from; capping them would make the
    /// answer's *shape* depend on the cap rather than its size.
    ///
    /// The bound is checked between directories and never inside one. A directory is read
    /// whole or not at all, because a half-listed directory reports its own tallies as
    /// though they were complete -- silently, which is the one thing a bound must never be.
    /// So the observed count can overshoot the cap by the entries of the directories
    /// already in flight, and that overshoot is the price of never lying about a directory.
    pub max_files: Option<u64>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            max_depth: None,
            max_files: None,
            batch_size: DEFAULT_BATCH_SIZE,
            follow_symlinks: false,
            one_filesystem: false,
            threads: None,
            order: ScanOrder::default(),
            types: None,
            tags: None,
            hidden: None,
        }
    }
}

/// Why watching cannot narrow its scan scope, said once for every surface.
///
/// The CLI used to carry this guidance and the library carried "requires event-scope
/// filtering", which names the implementation rather than the caller's next move -- so a
/// library caller hitting the same wall got jargon and the CLI user got help. Two
/// messages for one rule also drift, and the parity harness could not tell they were the
/// same rule.
///
/// The knobs are named by the calling surface: `--scan-depth` on the command line,
/// `max_depth` through the API. Everything else is identical, so the harness can verify
/// mechanically that both surfaces state the same rule.
pub const WATCH_SCOPE_GUIDANCE: &str = concat!(
    "watching requires full scope and cannot be combined with max_depth, max_files or ",
    "one_filesystem: a watcher cannot filter backend events against a narrowed boundary. ",
    "Selection such as depth, include, and modified_since does work while watching, because ",
    "it filters the retained index rather than narrowing the scan"
);

impl ScanConfig {
    /// The file-type rules in effect: the supplied registry, or the compiled default.
    pub fn types(&self) -> &std::sync::Arc<crate::classify::TypeRegistry> {
        self.types.as_ref().unwrap_or_else(|| crate::classify::TypeRegistry::compiled())
    }

    /// The tag rules in effect: the supplied set, or none enabled.
    pub fn tags(&self) -> std::sync::Arc<crate::tags::TagRules> {
        self.tags
            .clone()
            .unwrap_or_else(|| std::sync::Arc::new(crate::tags::TagRules::none().clone()))
    }

    /// Semantic cache identity, excluding operational batching choices.
    ///
    /// No longer `const`: the type-rule fingerprint is now a property of the registry in
    /// effect rather than a compiled-in constant, which is the whole point of letting a
    /// caller supply one. A snapshot taken under different rules must not be reused.
    pub fn scope(&self) -> ScanScope {
        ScanScope {
            max_depth: self.max_depth,
            follow_symlinks: self.follow_symlinks,
            one_filesystem: self.one_filesystem,
            tag_rules_fingerprint: self.tags().fingerprint(),
            type_rules_fingerprint: self.types().fingerprint(),
            reducers_fingerprint: REDUCERS_FINGERPRINT,
            hidden_fingerprint: self.hidden().fingerprint(),
            max_files: self.max_files,
        }
    }

    /// The admission rule this scan applies, or the one that admits everything.
    pub fn hidden(&self) -> &crate::admission::HiddenPolicy {
        self.hidden.as_deref().unwrap_or_else(|| crate::admission::HiddenPolicy::keep_all())
    }

    /// Resolve [`Self::threads`] to the workers active when a scan begins.
    #[cfg(any(target_os = "macos", test))]
    fn worker_threads(&self) -> usize {
        self.worker_pool().initial
    }

    /// Resolve the worker count for immutable-baseline reconciliation waves.
    fn reconciliation_worker_threads(&self) -> usize {
        match self.threads {
            Some(threads) => threads.clamp(1, MAX_SCAN_THREADS),
            None => std::thread::available_parallelism()
                .map_or(1, std::num::NonZero::get)
                .clamp(1, DEFAULT_RECONCILE_THREADS_CAP),
        }
    }

    /// Resolve the initial and maximum worker counts for one scan.
    #[cfg(any(target_os = "macos", test))]
    fn worker_pool(&self) -> WorkerPool {
        self.worker_pool_for(std::thread::available_parallelism().map_or(1, std::num::NonZero::get))
    }

    /// Resolve the worker pool from one captured operating-system parallelism value.
    fn worker_pool_for(&self, available_parallelism: usize) -> WorkerPool {
        match self.threads {
            Some(threads) => WorkerPool::fixed(threads.clamp(1, MAX_SCAN_THREADS)),
            None => automatic_worker_pool(available_parallelism),
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
        if self.max_files == Some(0) {
            // Absent is how "no cap" is spelled. Zero would otherwise mean a walk that
            // stops before its first directory, which is a scan nobody asked for and is
            // indistinguishable at the surface from a typo.
            return Err(Error::UnsupportedScanConfig(
                "max_files must be nonzero; omit it for an unlimited walk",
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
            return Err(Error::ScanScopeMismatch {
                indexed: Box::new(indexed),
                requested: Box::new(requested),
            });
        }
        Ok(())
    }

    #[cfg(feature = "watch")]
    pub(crate) fn validate_for_watch_scope(&self, indexed: ScanScope) -> Result<()> {
        self.validate_for_scope(indexed)?;
        if self.max_depth.is_some() || self.max_files.is_some() || self.one_filesystem {
            // A budget belongs here for a reason of its own, beyond the shared one: a
            // watcher applies events to an index the cap left incomplete, so a creation
            // inside an unread subtree would be admitted while its siblings stay missing.
            // The index would then hold a subtree nobody walked, assembled one event at a
            // time, with nothing recording that it was never discovered.
            return Err(Error::UnsupportedScanConfig(WATCH_SCOPE_GUIDANCE));
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
    /// Regular files whose metadata was observed.
    pub files_walked: u64,
    /// Apparent bytes represented by the regular files whose metadata was observed.
    pub bytes_walked: u64,
    /// Paths that could not be read, with the reason.
    pub errors: Vec<Error>,
    /// Where the walk's time went, summed across workers.
    pub attribution: WalkAttribution,
    /// True when the file cap stopped the walk descending, so entries in scope were
    /// never read.
    ///
    /// Distinct from an error: nothing failed, and everything reported is correct. What is
    /// missing is unnamed and unnameable -- the walk stopped before discovering it -- which
    /// is why this is a flag rather than a list, and why it degrades coverage rather than
    /// freshness. The entries that *are* here were read and are current.
    pub budget_stopped: bool,
    /// Directories where the walk saw a control file it pruned rather than retained.
    ///
    /// Empty unless a hidden-path rule pruned one, which needs an enabled Path-tier rule to
    /// be interesting at all. It exists because binding a `gitignore` rule asks the index
    /// where the `.gitignore` files are, and the whole point of pruning is that they are
    /// not in the index. Directories rather than paths: binding takes a directory, and one
    /// directory holds at most one control file per rule.
    ///
    /// Unsorted and possibly repeated, because a walk has many workers and ordering it here
    /// would mean coordinating them; the index sorts and dedupes when it adopts them.
    pub control_dirs: Vec<PathBuf>,
}

/// The shared file counter one walk enforces its cap with.
///
/// Runtime state rather than configuration: the cap is what a caller asked for and lives
/// in [`ScanConfig`]; this is how much of it a particular walk has spent, and it exists
/// only for the duration of that walk.
///
/// `None` for an uncapped walk, which is the default, so an uncapped walk touches no
/// atomic at all -- the cost of the feature is paid only by callers who asked for it.
///
/// Charged **per directory**, not per file. A per-file `fetch_add` would put a contended
/// write on the hottest loop in the program for a bound almost no walk reaches; per
/// directory the write happens once per `read_dir`, which is already the expensive part,
/// and the read side is a relaxed load of a line that is otherwise shared.
#[derive(Debug)]
pub(crate) struct Budget {
    cap: u64,
    files: std::sync::atomic::AtomicU64,
}

impl Budget {
    /// The counter for `config`, or `None` when it set no cap.
    fn for_config(config: &ScanConfig) -> Option<std::sync::Arc<Self>> {
        config.max_files.map(|cap| {
            std::sync::Arc::new(Self { cap, files: std::sync::atomic::AtomicU64::new(0) })
        })
    }

    /// Record files observed in one directory.
    fn charge(&self, files: u64) {
        if files > 0 {
            self.files.fetch_add(files, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Whether the cap has been reached, so no further directory should be entered.
    fn spent(&self) -> bool {
        self.files.load(std::sync::atomic::Ordering::Relaxed) >= self.cap
    }
}

/// Whether a walk holding `budget` may still descend.
///
/// A free function taking an `Option` so both walkers ask the same question in the same
/// shape, and so the uncapped case is one `is_none` rather than a branch each walker
/// spells for itself.
fn within_budget(budget: Option<&std::sync::Arc<Budget>>) -> bool {
    budget.is_none_or(|budget| !budget.spent())
}

impl ScanReport {
    /// True when every directory in scope was read successfully.
    ///
    /// A budget-stopped walk is not complete even though it hit no error: the cap is a
    /// deliberate decision to stop reading scope, and an answer that omits scope is
    /// partial however cleanly it got that way.
    pub fn is_complete(&self) -> bool {
        self.errors.is_empty() && !self.budget_stopped
    }

    /// This walk's coverage, as the index records it.
    ///
    /// Derived here rather than at each call site so every producer of partial coverage
    /// names the same reason for the same condition. A walk that finished while skipping
    /// what it could not read is `Inaccessible` -- distinct from one that failed
    /// outright, which its caller reports as `Failed`.
    pub fn coverage(&self) -> Status {
        if !self.errors.is_empty() {
            // Worst wins, and [`CoverageReason`] declares which that is: `Inaccessible`
            // outranks `Budget`, so a walk that both hit the cap and failed to read
            // something reports the failure. The cap is a decision; a failure is not, and
            // only one of the two is worth a consumer's attention first.
            Status::Partial(CoverageReason::Inaccessible)
        } else if self.budget_stopped {
            Status::Partial(CoverageReason::Budget)
        } else {
            Status::Complete
        }
    }

    /// Everything this walk has to report as a typed condition.
    ///
    /// One place, so the reasons a result is partial and the issues explaining it cannot
    /// drift apart: `coverage()` above and this list are derived from the same two facts.
    pub(crate) fn issues(&self) -> Vec<crate::Issue> {
        let mut issues: Vec<crate::Issue> =
            self.errors.iter().map(crate::Issue::from_error).collect();
        if self.budget_stopped {
            issues.push(crate::Issue {
                kind: crate::IssueKind::ResourceStop,
                path: None,
                message: format!(
                    "the file budget stopped discovery after {} files",
                    self.files_walked
                ),
                os_error: None,
            });
        }
        issues
    }

    /// Fold one worker's share of a parallel walk into the whole-walk report.
    fn absorb(&mut self, other: Self) {
        self.dirs_read += other.dirs_read;
        self.entries += other.entries;
        self.files_walked += other.files_walked;
        self.bytes_walked += other.bytes_walked;
        self.errors.extend(other.errors);
        self.control_dirs.extend(other.control_dirs);
        self.budget_stopped |= other.budget_stopped;
        self.attribution.absorb(other.attribution);
    }

    /// Record one successfully stated directory entry.
    fn observe(&mut self, kind: EntryKind, attrs: Attrs) {
        self.entries += 1;
        if kind == EntryKind::File {
            self.files_walked += 1;
            self.bytes_walked += attrs.size;
        }
    }
}

/// Schema carried by [`ScanDiagnostics`].
///
/// Diagnostics are an opt-in measurement contract rather than stable human output.
/// Consumers must reject an unknown schema instead of guessing that fields retained
/// their meaning.
pub const SCAN_DIAGNOSTICS_SCHEMA: &str = "fdu-scan-diagnostics-v1";

/// Maximum policy-window records retained by one diagnostic scan.
///
/// The bound is on controller evaluations, not filesystem entries. A controller that
/// needs more history must mark the artifact truncated; claim-grade consumers reject
/// that artifact rather than silently analyzing an incomplete policy history.
const MAX_POLICY_TRACE_EVENTS: usize = 256;

/// Opt-in, run-scoped evidence about a filesystem scan.
///
/// Obtain this through [`scan_with_diagnostics`] or
/// [`scan_into_index_with_diagnostics`]. Keeping it out of [`ScanReport`] preserves the
/// existing scan API and keeps ordinary callers off the measurement path entirely.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanDiagnostics {
    /// Version of this diagnostic contract.
    pub schema: &'static str,
    /// Automatic worker-controller history and queue state.
    pub worker_policy: WorkerPolicyDiagnostics,
    /// Directory-enumeration backends used by this run.
    pub backend: ScanBackendDiagnostics,
}

/// Repository-only controller variants used by the performance evidence probe.
///
/// These variants are not selected by [`scan`] or [`scan_with_diagnostics`]; both keep
/// the shipped one-shot policy. The explicit experimental APIs make candidate behavior
/// measurable without hiding a production change behind an environment variable.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkerPolicyExperiment {
    /// The production controller: one prefix window and at most one expansion.
    #[default]
    ShippedOneShot,
    /// Re-evaluate independent windows until a slow phase requests the full reserve.
    RepeatedWindows,
    /// Re-evaluate independent windows, gate on useful frontier/handoff backlog, and grow
    /// the pool in stages.
    StagedGatedWindows,
}

/// Final state of the automatic worker controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerPolicyOutcome {
    /// The scan did no walking, for example because `max_depth` was zero.
    NotRun,
    /// A fixed pool had no adaptive decision to make.
    Fixed,
    /// The walk ended before an adaptive window became observable.
    Undecided,
    /// The controller measured a window and retained the initial pool.
    Held,
    /// The controller requested and activated the reserve workers.
    ScaledUp,
    /// Slow work was observed only after no useful queued or in-flight work remained.
    HeldNoUsefulWork,
}

/// One controller evaluation over a half-open range of completed entry ordinals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerPolicyWindow {
    /// Monotonic record number within this scan.
    pub sequence: u64,
    /// First completed-entry ordinal represented by this window, inclusive.
    pub start_entry_ordinal: u64,
    /// Ordinal immediately after the last represented entry.
    pub end_entry_ordinal: u64,
    /// Entries contributing to the service-time signal.
    pub observed_entries: u64,
    /// Completed directory claims contributing to the service-time signal.
    pub observed_chunks: u64,
    /// Worker time contributing to the service-time signal.
    pub observed_work_ns: u64,
    /// Derived service time, or null when no entry made the signal observable.
    pub work_ns_per_entry: Option<u64>,
    /// Why `work_ns_per_entry` is null.
    pub work_ns_per_entry_unavailable_reason: Option<&'static str>,
    /// Directories ready to claim when the controller evaluated the window.
    pub ready_directories: usize,
    /// Claimed directories still being processed at that point.
    pub in_flight_directories: usize,
    /// Live worker threads at that point, including workers waiting for a claim.
    pub active_workers: usize,
    /// Observation batches sent but not yet received by the consumer.
    pub handoff_backlog: usize,
    /// Worker target requested by a scale decision.
    pub requested_workers: Option<usize>,
    /// What the controller concluded from this window.
    pub decision: WorkerPolicyDecision,
}

/// Decision represented by a [`WorkerPolicyWindow`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerPolicyDecision {
    /// The walk ended before the window could support a decision.
    Undecided,
    /// The observed window retained the current pool.
    Hold,
    /// The observed window activated reserve workers.
    ScaleUp,
    /// The trigger fired after all useful work had drained.
    HoldNoUsefulWork,
    /// A complete window held because reserve workers had no useful frontier to claim.
    HoldInsufficientFrontier,
    /// A complete window held because the unbounded handoff backlog was already high.
    HoldHandoffBacklog,
    /// A post-decision observation window remained below the slow threshold.
    ObserveFast,
    /// A post-decision observation window met the slow threshold.
    ObserveSlow,
    /// A trailing partial window carried no new terminal decision.
    Incomplete,
    /// A trailing partial post-decision observation carried no policy decision.
    ObserveIncomplete,
}

/// Worker-controller configuration, trace, and terminal queue state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerPolicyDiagnostics {
    /// Controller variant exercised by this scan.
    pub controller: &'static str,
    /// Parallelism reported by the operating system when the scan began.
    pub available_parallelism: usize,
    /// Workers in the pool before any adaptive decision.
    pub initial_workers: usize,
    /// Hard maximum workers this scan could activate.
    pub maximum_workers: usize,
    /// Entry target for an adaptive window, or null for a fixed pool.
    pub calibration_window_entries: Option<u64>,
    /// Slow-service trigger, or null for a fixed pool.
    pub slow_threshold_ns_per_entry: Option<u64>,
    /// Directory chunks folded into live controller windows.
    pub calibration_chunks: u64,
    /// Entries folded into live controller windows.
    pub calibration_entries: u64,
    /// Worker time folded into live controller windows.
    pub calibration_work_ns: u64,
    /// Expansion messages that caused the consumer to create more workers.
    pub worker_expansions: u64,
    /// Terminal policy outcome.
    pub outcome: WorkerPolicyOutcome,
    /// Explanation when no adaptive evaluation exists.
    pub outcome_reason: Option<&'static str>,
    /// Total worker threads created during the walk.
    pub workers_spawned: usize,
    /// Maximum simultaneously live worker threads, including workers waiting for work.
    pub peak_active_workers: usize,
    /// Ready directories at scan completion; a complete walk must leave zero.
    pub ready_directories_at_finish: usize,
    /// In-flight directories at scan completion; a complete walk must leave zero.
    pub in_flight_directories_at_finish: usize,
    /// Observation batches outstanding at scan completion.
    pub handoff_backlog_at_finish: usize,
    /// Maximum outstanding observation batches during the scan.
    pub handoff_backlog_high_water: usize,
    /// Bounded controller history.
    pub windows: Vec<WorkerPolicyWindow>,
    /// True when controller history exceeded the 256-event diagnostic bound.
    pub events_truncated: bool,
}

/// Directory enumeration backends used by one scan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanBackendDiagnostics {
    /// Portable `read_dir` calls attempted.
    pub portable_attempts: u64,
    /// Portable directory listings completed successfully.
    pub portable_directory_reads: u64,
    /// macOS bulk enumeration attempts, or null off macOS.
    pub macos_bulk_attempts: Option<u64>,
    /// Successful macOS bulk listings, or null off macOS.
    pub macos_bulk_successes: Option<u64>,
    /// Bulk attempts that fell back to portable enumeration, or null off macOS.
    pub macos_bulk_fallbacks: Option<u64>,
    /// Why macOS fields are null.
    pub unavailable_reason: Option<&'static str>,
}

impl ScanDiagnostics {
    /// Serialize this versioned diagnostic contract as compact JSON.
    ///
    /// This deliberately lives beside the contract instead of in a benchmark binary:
    /// claim-grade installed-command measurements and the repository probe must emit
    /// byte-for-byte equivalent evidence without adding a serialization dependency to
    /// the core crate.
    pub fn to_json(&self) -> String {
        let policy = &self.worker_policy;
        let backend = &self.backend;
        let mut windows = String::from("[");
        for (index, window) in policy.windows.iter().enumerate() {
            if index > 0 {
                windows.push(',');
            }
            let _ = write!(
                windows,
                concat!(
                    "{{\"active_workers\":{},\"decision\":\"{}\",",
                    "\"end_entry_ordinal\":{},\"handoff_backlog\":{},",
                    "\"in_flight_directories\":{},\"observed_chunks\":{},",
                    "\"observed_entries\":{},",
                    "\"observed_work_ns\":{},\"ready_directories\":{},",
                    "\"requested_workers\":{},\"sequence\":{},\"start_entry_ordinal\":{},",
                    "\"work_ns_per_entry\":{},",
                    "\"work_ns_per_entry_unavailable_reason\":{}}}"
                ),
                window.active_workers,
                worker_policy_decision_name(window.decision),
                window.end_entry_ordinal,
                window.handoff_backlog,
                window.in_flight_directories,
                window.observed_chunks,
                window.observed_entries,
                window.observed_work_ns,
                window.ready_directories,
                json_optional_usize(window.requested_workers),
                window.sequence,
                window.start_entry_ordinal,
                json_optional_u64(window.work_ns_per_entry),
                json_optional_string(window.work_ns_per_entry_unavailable_reason),
            );
        }
        windows.push(']');
        format!(
            concat!(
                "{{\"backend\":{{\"macos_bulk_attempts\":{},",
                "\"macos_bulk_fallbacks\":{},\"macos_bulk_successes\":{},",
                "\"portable_attempts\":{},\"portable_directory_reads\":{},",
                "\"unavailable_reason\":{}}},",
                "\"schema\":\"{}\",\"worker_policy\":{{",
                "\"available_parallelism\":{},\"calibration_chunks\":{},",
                "\"calibration_entries\":{},\"calibration_window_entries\":{},",
                "\"calibration_work_ns\":{},",
                "\"controller\":\"{}\",",
                "\"events_truncated\":{},\"handoff_backlog_at_finish\":{},",
                "\"handoff_backlog_high_water\":{},\"in_flight_directories_at_finish\":{},",
                "\"initial_workers\":{},\"maximum_workers\":{},\"outcome\":\"{}\",",
                "\"outcome_reason\":{},\"peak_active_workers\":{},",
                "\"ready_directories_at_finish\":{},\"slow_threshold_ns_per_entry\":{},",
                "\"windows\":{},\"worker_expansions\":{},\"workers_spawned\":{}}}}}"
            ),
            json_optional_u64(backend.macos_bulk_attempts),
            json_optional_u64(backend.macos_bulk_fallbacks),
            json_optional_u64(backend.macos_bulk_successes),
            backend.portable_attempts,
            backend.portable_directory_reads,
            json_optional_string(backend.unavailable_reason),
            self.schema,
            policy.available_parallelism,
            policy.calibration_chunks,
            policy.calibration_entries,
            json_optional_u64(policy.calibration_window_entries),
            policy.calibration_work_ns,
            policy.controller,
            policy.events_truncated,
            policy.handoff_backlog_at_finish,
            policy.handoff_backlog_high_water,
            policy.in_flight_directories_at_finish,
            policy.initial_workers,
            policy.maximum_workers,
            worker_policy_outcome_name(policy.outcome),
            json_optional_string(policy.outcome_reason),
            policy.peak_active_workers,
            policy.ready_directories_at_finish,
            json_optional_u64(policy.slow_threshold_ns_per_entry),
            windows,
            policy.worker_expansions,
            policy.workers_spawned,
        )
    }
}

const fn worker_policy_outcome_name(value: WorkerPolicyOutcome) -> &'static str {
    match value {
        WorkerPolicyOutcome::NotRun => "not_run",
        WorkerPolicyOutcome::Fixed => "fixed",
        WorkerPolicyOutcome::Undecided => "undecided",
        WorkerPolicyOutcome::Held => "held",
        WorkerPolicyOutcome::ScaledUp => "scaled_up",
        WorkerPolicyOutcome::HeldNoUsefulWork => "held_no_useful_work",
    }
}

const fn worker_policy_decision_name(value: WorkerPolicyDecision) -> &'static str {
    match value {
        WorkerPolicyDecision::Undecided => "undecided",
        WorkerPolicyDecision::Hold => "hold",
        WorkerPolicyDecision::ScaleUp => "scale_up",
        WorkerPolicyDecision::HoldNoUsefulWork => "hold_no_useful_work",
        WorkerPolicyDecision::HoldInsufficientFrontier => "hold_insufficient_frontier",
        WorkerPolicyDecision::HoldHandoffBacklog => "hold_handoff_backlog",
        WorkerPolicyDecision::ObserveFast => "observe_fast",
        WorkerPolicyDecision::ObserveSlow => "observe_slow",
        WorkerPolicyDecision::Incomplete => "incomplete",
        WorkerPolicyDecision::ObserveIncomplete => "observe_incomplete",
    }
}

fn json_optional_string(value: Option<&str>) -> String {
    value.map_or_else(|| "null".into(), |value| format!("\"{}\"", json_escape(value)))
}

fn json_optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "null".into(), |value| value.to_string())
}

fn json_optional_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "null".into(), |value| value.to_string())
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character <= '\u{1f}' => {
                let _ = write!(escaped, "\\u{:04x}", u32::from(character));
            }
            character => escaped.push(character),
        }
    }
    escaped
}

/// Where a walk's time went, so "blocked" is never one undifferentiated number.
///
/// The performance loop's standing question is whether a walk is bound by disk I/O,
/// by CPU, or by coordination, and process-level counters cannot answer it: user and
/// system time say how much CPU was burned, but a fused "blocked" number cannot say
/// whether workers were waiting on the filesystem, on the queue lock, or on nothing
/// at all because the queue was empty. These counters split that out at the source.
///
/// Everything is measured in *chunks*, never per file: one timing pair per claimed
/// run of directories, per contended lock, per batch handoff. On the 60k-entry
/// reference tree that is a few thousand `Instant` reads against hundreds of
/// milliseconds of walking — the instrumentation follows the same amortization rule
/// it exists to verify.
///
/// In a parallel walk the fields sum over workers, so `wall_ns` is worker-seconds
/// (it can exceed the scan's wall clock) and every other duration is a disjoint
/// slice of it: `work_ns + starved_ns + lock_wait_ns + send_ns <= wall_ns`, with the
/// remainder being uninstrumented odds and ends (uncontended lock ops, loop
/// bookkeeping). A serial walk fills only `wall_ns`, `work_ns`, and `send_ns` —
/// there is no coordination to attribute.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WalkAttribution {
    /// Total time workers spent in the walk loop, summed across workers.
    pub wall_ns: u64,
    /// Reading directories and stating entries — the real work, syscalls plus the
    /// compute between them. Separating disk from CPU *within* this span needs the
    /// process-level user/system counters alongside; per-syscall timing would break
    /// the chunk-amortization rule.
    pub work_ns: u64,
    /// Waiting on the queue's condvar because no work was available. Starvation:
    /// either the frontier is momentarily narrower than the worker pool, or the walk
    /// is ending.
    pub starved_ns: u64,
    /// Waiting to acquire the queue lock when another worker held it. This is the
    /// contention the shared-queue design bets stays negligible; now it is measured
    /// instead of argued.
    pub lock_wait_ns: u64,
    /// Handing observation batches to the consumer: the channel send in a parallel
    /// walk, the inline sink call — which is the consumer actually running — in a
    /// serial one.
    pub send_ns: u64,
    /// Chunks of directories claimed from the queue.
    pub claims: u64,
    /// Queue lock acquisitions, contended or not.
    pub lock_ops: u64,
    /// Lock acquisitions that found the lock already held.
    pub lock_contended: u64,
}

impl WalkAttribution {
    /// Fold one worker's counters into the whole-walk totals.
    fn absorb(&mut self, other: Self) {
        self.wall_ns += other.wall_ns;
        self.work_ns += other.work_ns;
        self.starved_ns += other.starved_ns;
        self.lock_wait_ns += other.lock_wait_ns;
        self.send_ns += other.send_ns;
        self.claims += other.claims;
        self.lock_ops += other.lock_ops;
        self.lock_contended += other.lock_contended;
    }

    /// Time attributed to a named cause, as opposed to `wall_ns`'s total.
    pub fn accounted_ns(&self) -> u64 {
        self.work_ns + self.starved_ns + self.lock_wait_ns + self.send_ns
    }
}

/// Filesystem and index effects from an applying reconciliation pass.
#[derive(Debug, Default)]
pub struct ReconcileReport {
    /// Filesystem walk effects and partial errors.
    ///
    /// [`ScanReport::attribution`] remains zero for reconciliation because neither the
    /// serial nor parallel path has complete, comparable instrumentation yet. Zero
    /// means "not measured" here, not "no work".
    pub scan: ScanReport,
    /// Index arbitration and mutation effects.
    pub apply: ApplyStats,
}

impl ReconcileReport {
    /// True when the filesystem walk was complete and no conditional observation lost
    /// a race with another producer.
    pub fn coverage(&self) -> Status {
        if self.is_complete() { Status::Complete } else { self.scan.coverage() }
    }

    /// Whether this reconciliation read everything in scope and applied it all.
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

    /// Hand over the directories this pass saw a pruned control file in.
    ///
    /// A no-op for the overwhelmingly common empty case, which is checked before the write
    /// guard is taken: a shared index serves readers throughout a reconciliation, and
    /// taking the write lock to add nothing would stall them for no reason.
    fn adopt_pruned_control_dirs(&mut self, dirs: &[PathBuf]) -> Result<()> {
        if dirs.is_empty() {
            return Ok(());
        }
        match self {
            Self::Direct(index) => index.adopt_pruned_control_dirs(dirs.iter().cloned()),
            Self::Shared(handle) => {
                handle.adopt_pruned_control_dirs(dirs.iter().cloned())?;
            }
        }
        Ok(())
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
            Self::Direct(index) => Ok(collect_child_expectations(index, path)),
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

    fn begin_reconcile(&mut self, path: &Path) -> Result<(u64, AppliedDelta)> {
        match self {
            Self::Direct(index) => index.begin_reconcile(path),
            Self::Shared(handle) => handle.begin_reconcile(path),
        }
    }

    fn finish_reconcile(
        &mut self,
        path: &Path,
        started_at: u64,
        coverage: Status,
    ) -> Result<AppliedDelta> {
        match self {
            Self::Direct(index) => index.finish_reconcile(path, started_at, coverage),
            Self::Shared(handle) => handle.finish_reconcile(path, started_at, coverage),
        }
    }
}

#[cfg(unix)]
fn metadata_for_fingerprint(entry: &fs::DirEntry) -> std::io::Result<fs::Metadata> {
    crate::counters::bump(|c| c.stats += 1);
    entry.metadata()
}

#[cfg(not(unix))]
fn metadata_for_fingerprint(entry: &fs::DirEntry) -> std::io::Result<fs::Metadata> {
    crate::counters::bump(|c| c.stats += 1);
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
    scan_internal(root, config, sink, false, WorkerPolicyExperiment::ShippedOneShot)
        .map(|(report, _diagnostics)| report)
}

/// Walk `root`, emitting observations and a bounded run-scoped diagnostic trace.
///
/// This is the measurement counterpart to [`scan`]. It produces the same observation
/// stream and report while recording controller and backend evidence that ordinary
/// scans intentionally do not collect.
pub fn scan_with_diagnostics(
    root: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(Observation),
) -> Result<(ScanReport, ScanDiagnostics)> {
    scan_with_policy_diagnostics(root, config, sink, WorkerPolicyExperiment::ShippedOneShot)
}

/// Exercise a repository-only worker-controller candidate and retain its trace.
#[doc(hidden)]
pub fn scan_with_policy_diagnostics(
    root: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(Observation),
    policy: WorkerPolicyExperiment,
) -> Result<(ScanReport, ScanDiagnostics)> {
    let (report, diagnostics) = scan_internal(root, config, sink, true, policy)?;
    Ok((report, diagnostics.expect("diagnostic scan creates a recorder")))
}

fn scan_internal(
    root: &Path,
    config: &ScanConfig,
    sink: &mut dyn FnMut(Observation),
    collect_diagnostics: bool,
    policy: WorkerPolicyExperiment,
) -> Result<(ScanReport, Option<ScanDiagnostics>)> {
    config.validate()?;
    let root_meta = {
        crate::counters::bump(|c| c.stats += 1);
        fs::symlink_metadata(root)
    }
    .map_err(|e| Error::io(root, e))?;
    if !root_meta.is_dir() {
        return Err(Error::io(
            root,
            std::io::Error::new(std::io::ErrorKind::NotADirectory, "scan root is not a directory"),
        ));
    }
    let root_dev = attrs_from(&root_meta).dev;
    let available_parallelism =
        std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    let pool = config.worker_pool_for(available_parallelism);
    let diagnostics = collect_diagnostics
        .then(|| ScanDiagnosticsRecorder::new(pool, available_parallelism, policy));

    if config.max_depth != Some(0) && pool.initial > 1 {
        let report =
            scan_concurrent(root, config, root_dev, sink, pool, diagnostics.as_ref(), policy);
        return Ok((report, diagnostics.as_ref().map(|value| value.finish())));
    }

    let mut report = ScanReport::default();
    if config.max_depth == Some(0) {
        if let Some(diagnostics) = &diagnostics {
            diagnostics.mark_not_run();
            diagnostics.record_queue_finish(0, 0);
        }
        return Ok((report, diagnostics.as_ref().map(|value| value.finish())));
    }
    let worker_guard = diagnostics.as_ref().map(ScanDiagnosticsRecorder::worker_guard);
    let walk_started = std::time::Instant::now();
    let mut batch: Vec<Op> = Vec::with_capacity(config.batch_size);
    let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::from(vec![(PathBuf::new(), 0)]);
    let budget = Budget::for_config(config);

    while let Some((rel_dir, depth)) = take_next(&mut queue, config.order) {
        if !within_budget(budget.as_ref()) {
            // Asked here as well as at enqueue time, and this is the one that does the
            // work on a wide tree: the root holds no files of its own, so nothing is
            // charged before all of its children are queued, and a check that only guarded
            // the enqueue would let every one of them be read. Between directories, so the
            // one being listed is never torn.
            report.budget_stopped = true;
            break;
        }
        let abs_dir = root.join(&rel_dir);
        crate::counters::bump(|c| c.dir_opens += 1);
        if let Some(diagnostics) = &diagnostics {
            diagnostics.portable_attempted();
        }
        let listing = match fs::read_dir(&abs_dir) {
            Ok(listing) => {
                if let Some(diagnostics) = &diagnostics {
                    diagnostics.portable_succeeded();
                }
                listing
            }
            Err(e) => {
                report.errors.push(Error::io(abs_dir, e));
                continue;
            }
        };
        report.dirs_read += 1;
        let files_before = report.files_walked;

        for item in listing {
            let item = match item {
                Ok(item) => item,
                Err(e) => {
                    report.errors.push(Error::io(&abs_dir, e));
                    continue;
                }
            };
            crate::counters::bump(|c| c.dir_entries += 1);
            let name = item.file_name();
            if !admits(&name, config) {
                note_pruned_control_file(&name, &rel_dir, config, &mut report);
                continue;
            }
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
            report.observe(kind, attrs);
            batch.push(Op::Upsert { path: rel_path.clone(), kind, attrs });
            if batch.len() >= config.batch_size {
                let send_started = std::time::Instant::now();
                sink(Observation::new(std::mem::take(&mut batch)));
                report.attribution.send_ns += elapsed_ns(send_started);
                batch.reserve(config.batch_size);
            }

            if should_descend(kind, attrs, depth, root_dev, config) {
                if within_budget(budget.as_ref()) {
                    queue.push_back((rel_path, depth + 1));
                } else {
                    // Not enqueued, and the walk says so. Silently dropping the subtree
                    // would leave a directory reporting a complete roll-up over a subtree
                    // nobody read.
                    report.budget_stopped = true;
                }
            }
        }
        // Once per directory rather than once per file: the write lands next to the
        // `read_dir` that already dominates this loop's cost.
        if let Some(budget) = &budget {
            budget.charge(report.files_walked - files_before);
        }
    }

    if !batch.is_empty() {
        let send_started = std::time::Instant::now();
        sink(Observation::new(batch));
        report.attribution.send_ns += elapsed_ns(send_started);
    }
    // A serial walk has no coordination to attribute: wall is the loop, "send" is the
    // inline sink — which is the consumer actually running — and work is the rest.
    report.attribution.wall_ns = elapsed_ns(walk_started);
    report.attribution.work_ns =
        report.attribution.wall_ns.saturating_sub(report.attribution.send_ns);
    drop(worker_guard);
    if let Some(diagnostics) = &diagnostics {
        diagnostics.record_queue_finish(0, 0);
    }
    Ok((report, diagnostics.as_ref().map(|value| value.finish())))
}

/// Take the next directory in the configured order.
///
/// Both orders push to the back; only the end they are taken from differs, which is
/// what keeps this a one-line policy rather than two walkers.
fn take_next(queue: &mut VecDeque<(PathBuf, usize)>, order: ScanOrder) -> Option<(PathBuf, usize)> {
    match order {
        ScanOrder::BreadthFirst => queue.pop_front(),
        ScanOrder::DepthFirst => queue.pop_back(),
    }
}

/// Largest worker pool a caller may ask for explicitly.
///
/// Well past anything measured to help. It exists so a caller that computes a thread
/// count from something silly cannot spawn thousands of threads.
const MAX_SCAN_THREADS: usize = 32;

/// Ceiling on the workers active at the start of an automatic scan.
///
/// Measured, not guessed. On a 10-core machine walking a 60k-entry `node_modules`
/// tree, wall time fell 37% at two workers and 50% at four, then stopped improving:
/// six matched four within noise and eight was 4% worse than four. The walk becomes
/// bound by the single index consumer, so past this point extra workers buy queue
/// contention and efficiency-core scheduling rather than throughput. See
/// `docs/project/reports/report-2026-08-10-fdu-performance-experiments.md`.
///
/// That measurement was taken on macOS, and every constant in this group now reads its
/// value from [`crate::platform_tuning`], which records per platform whether the number
/// was measured there or inherited. On Linux these are inherited.
const DEFAULT_SCAN_THREADS_CAP: usize = crate::platform_tuning::tuning().scan_threads_cap.get();

/// Ceiling on automatic workers for an immutable-baseline reconciliation wave.
///
/// Reconciliation reads both filesystem and index state. Its measured knee arrives
/// before the cold producer's because additional metadata calls amplify kernel work
/// after the index comparisons already saturate the performance cores.
const DEFAULT_RECONCILE_THREADS_CAP: usize =
    crate::platform_tuning::tuning().reconcile_threads_cap.get();

/// Ceiling an automatic scan may unlock after it establishes that the tree is large.
///
/// Sixteen was the knee on the 720k-entry cache-pressure corpus in exp-015. Thirty-two
/// did not improve on it and spent substantially more worker time waiting at the end.
const ADAPTIVE_SCAN_THREADS_CAP: usize =
    crate::platform_tuning::tuning().adaptive_scan_threads_cap.get();

/// Maximum reserve depth relative to the host's reported parallelism.
const ADAPTIVE_SCAN_PARALLELISM_MULTIPLIER: usize =
    crate::platform_tuning::tuning().adaptive_scan_parallelism_multiplier.get();

/// Entries used to calibrate the initial workers' filesystem service time.
const ADAPTIVE_SCAN_CALIBRATION_ENTRIES: u64 =
    crate::platform_tuning::tuning().adaptive_scan_calibration_entries.get();

/// Average worker time per observed entry that identifies a latency-bound scan.
///
/// Whole-run attribution separated the measured regimes: roughly 18 microseconds on
/// the 60k tree, 22 on the 120k boundary, and 42 or more on the 720k cache-pressure
/// tree. Thirty leaves margin between them. The calibration uses the same chunk timing
/// already collected for attribution, so it adds no per-entry clock reads.
///
/// Those are APFS regimes. The Linux warm floor is about 1.5 µs per entry, twenty times
/// below this threshold, so the trigger may never fire there — which is exactly the kind
/// of inherited constant [`crate::platform_tuning`] exists to make visible (H84).
const ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY: u64 =
    crate::platform_tuning::tuning().adaptive_scan_slow_work_ns_per_entry.get();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WorkerPool {
    initial: usize,
    maximum: usize,
    calibration: Option<WorkerCalibration>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WorkerCalibration {
    minimum_entries: u64,
    slow_work_ns_per_entry: u64,
    entries: u64,
    work_ns: u64,
    chunks: u64,
}

#[derive(Clone, Copy, Debug)]
struct PolicyWindowSnapshot {
    sequence: u64,
    start_entry_ordinal: u64,
    end_entry_ordinal: u64,
    observed_entries: u64,
    observed_chunks: u64,
    observed_work_ns: u64,
    ready_directories: usize,
    in_flight_directories: usize,
    active_workers: usize,
    handoff_backlog: usize,
    requested_workers: Option<usize>,
    decision: WorkerPolicyDecision,
}

struct PolicyTraceState {
    outcome: WorkerPolicyOutcome,
    outcome_reason: Option<&'static str>,
    outcome_sequence: Option<u64>,
    windows: Vec<WorkerPolicyWindow>,
    events_truncated: bool,
    ready_directories_at_finish: usize,
    in_flight_directories_at_finish: usize,
}

/// Shared state used only by the opt-in diagnostic scan APIs.
///
/// Normal scans pass no recorder and therefore never touch these atomics or locks. The
/// trace mutex is deliberately separate from the directory queue: recording a policy
/// window may add diagnostic cost, but it cannot alter the queue's synchronization or
/// the controller's decision.
struct ScanDiagnosticsRecorder {
    available_parallelism: usize,
    pool: WorkerPool,
    policy: WorkerPolicyExperiment,
    trace: std::sync::Mutex<PolicyTraceState>,
    workers_spawned: std::sync::atomic::AtomicUsize,
    active_workers: std::sync::atomic::AtomicUsize,
    peak_active_workers: std::sync::atomic::AtomicUsize,
    handoff_backlog: std::sync::atomic::AtomicUsize,
    handoff_backlog_high_water: std::sync::atomic::AtomicUsize,
    calibration_chunks: std::sync::atomic::AtomicU64,
    calibration_entries: std::sync::atomic::AtomicU64,
    calibration_work_ns: std::sync::atomic::AtomicU64,
    worker_expansions: std::sync::atomic::AtomicU64,
    portable_attempts: std::sync::atomic::AtomicU64,
    portable_successes: std::sync::atomic::AtomicU64,
    #[cfg(target_os = "macos")]
    macos_bulk_attempts: std::sync::atomic::AtomicU64,
    #[cfg(target_os = "macos")]
    macos_bulk_successes: std::sync::atomic::AtomicU64,
    #[cfg(target_os = "macos")]
    macos_bulk_fallbacks: std::sync::atomic::AtomicU64,
}

impl ScanDiagnosticsRecorder {
    fn new(
        pool: WorkerPool,
        available_parallelism: usize,
        policy: WorkerPolicyExperiment,
    ) -> std::sync::Arc<Self> {
        let (outcome, outcome_reason) = if pool.calibration.is_some() {
            (
                WorkerPolicyOutcome::Undecided,
                Some("the adaptive calibration window has not completed"),
            )
        } else {
            (WorkerPolicyOutcome::Fixed, Some("this worker pool has no adaptive reserve"))
        };
        std::sync::Arc::new(Self {
            available_parallelism,
            pool,
            policy,
            trace: std::sync::Mutex::new(PolicyTraceState {
                outcome,
                outcome_reason,
                outcome_sequence: None,
                windows: Vec::new(),
                events_truncated: false,
                ready_directories_at_finish: 0,
                in_flight_directories_at_finish: 0,
            }),
            workers_spawned: std::sync::atomic::AtomicUsize::new(0),
            active_workers: std::sync::atomic::AtomicUsize::new(0),
            peak_active_workers: std::sync::atomic::AtomicUsize::new(0),
            handoff_backlog: std::sync::atomic::AtomicUsize::new(0),
            handoff_backlog_high_water: std::sync::atomic::AtomicUsize::new(0),
            calibration_chunks: std::sync::atomic::AtomicU64::new(0),
            calibration_entries: std::sync::atomic::AtomicU64::new(0),
            calibration_work_ns: std::sync::atomic::AtomicU64::new(0),
            worker_expansions: std::sync::atomic::AtomicU64::new(0),
            portable_attempts: std::sync::atomic::AtomicU64::new(0),
            portable_successes: std::sync::atomic::AtomicU64::new(0),
            #[cfg(target_os = "macos")]
            macos_bulk_attempts: std::sync::atomic::AtomicU64::new(0),
            #[cfg(target_os = "macos")]
            macos_bulk_successes: std::sync::atomic::AtomicU64::new(0),
            #[cfg(target_os = "macos")]
            macos_bulk_fallbacks: std::sync::atomic::AtomicU64::new(0),
        })
    }

    fn worker_guard(self: &std::sync::Arc<Self>) -> ScanWorkerGuard {
        let active = self
            .active_workers
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .saturating_add(1);
        self.workers_spawned.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        atomic_update_max(&self.peak_active_workers, active);
        ScanWorkerGuard { recorder: self.clone() }
    }

    fn record_policy_window(&self, snapshot: PolicyWindowSnapshot) {
        let mut trace = self.trace.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let sequence = snapshot.sequence;
        let supersedes = trace.outcome_sequence.is_none_or(|current| sequence >= current);
        match snapshot.decision {
            WorkerPolicyDecision::Undecided if supersedes => {
                trace.outcome = WorkerPolicyOutcome::Undecided;
                trace.outcome_reason =
                    Some("the walk ended before the adaptive calibration window completed");
                trace.outcome_sequence = Some(sequence);
            }
            WorkerPolicyDecision::Hold
                if trace.outcome != WorkerPolicyOutcome::ScaledUp && supersedes =>
            {
                trace.outcome = WorkerPolicyOutcome::Held;
                trace.outcome_reason = None;
                trace.outcome_sequence = Some(sequence);
            }
            WorkerPolicyDecision::ScaleUp => {
                trace.outcome = WorkerPolicyOutcome::ScaledUp;
                trace.outcome_reason = None;
                trace.outcome_sequence = Some(sequence);
            }
            WorkerPolicyDecision::HoldNoUsefulWork
                if trace.outcome != WorkerPolicyOutcome::ScaledUp && supersedes =>
            {
                trace.outcome = WorkerPolicyOutcome::HeldNoUsefulWork;
                trace.outcome_reason =
                    Some("the slow trigger fired only after ready and in-flight work had drained");
                trace.outcome_sequence = Some(sequence);
            }
            WorkerPolicyDecision::HoldInsufficientFrontier
                if trace.outcome != WorkerPolicyOutcome::ScaledUp && supersedes =>
            {
                trace.outcome = WorkerPolicyOutcome::Held;
                trace.outcome_reason =
                    Some("the observed frontier could not use additional workers");
                trace.outcome_sequence = Some(sequence);
            }
            WorkerPolicyDecision::HoldHandoffBacklog
                if trace.outcome != WorkerPolicyOutcome::ScaledUp && supersedes =>
            {
                trace.outcome = WorkerPolicyOutcome::Held;
                trace.outcome_reason =
                    Some("the observation handoff backlog was already at the controller limit");
                trace.outcome_sequence = Some(sequence);
            }
            WorkerPolicyDecision::Incomplete
                if supersedes && trace.outcome == WorkerPolicyOutcome::Undecided =>
            {
                trace.outcome_reason =
                    Some("the walk ended before any adaptive calibration window completed");
                trace.outcome_sequence = Some(sequence);
            }
            _ => {}
        }
        if sequence >= MAX_POLICY_TRACE_EVENTS as u64 {
            trace.events_truncated = true;
            return;
        }
        let work_ns_per_entry = (snapshot.observed_entries > 0)
            .then(|| snapshot.observed_work_ns / snapshot.observed_entries);
        trace.windows.push(WorkerPolicyWindow {
            sequence,
            start_entry_ordinal: snapshot.start_entry_ordinal,
            end_entry_ordinal: snapshot.end_entry_ordinal,
            observed_entries: snapshot.observed_entries,
            observed_chunks: snapshot.observed_chunks,
            observed_work_ns: snapshot.observed_work_ns,
            work_ns_per_entry,
            work_ns_per_entry_unavailable_reason: work_ns_per_entry
                .is_none()
                .then_some("the window observed no entries"),
            ready_directories: snapshot.ready_directories,
            in_flight_directories: snapshot.in_flight_directories,
            active_workers: snapshot.active_workers,
            handoff_backlog: snapshot.handoff_backlog,
            requested_workers: snapshot.requested_workers,
            decision: snapshot.decision,
        });
    }

    fn mark_not_run(&self) {
        let mut trace = self.trace.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        trace.outcome = WorkerPolicyOutcome::NotRun;
        trace.outcome_reason = Some("max_depth zero requested no directory walk");
    }

    fn record_queue_finish(&self, ready_directories: usize, in_flight_directories: usize) {
        let mut trace = self.trace.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        trace.ready_directories_at_finish = ready_directories;
        trace.in_flight_directories_at_finish = in_flight_directories;
    }

    fn handoff_sent(&self) {
        let backlog = self
            .handoff_backlog
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .saturating_add(1);
        atomic_update_max(&self.handoff_backlog_high_water, backlog);
    }

    fn handoff_received(&self) {
        let previous = self.handoff_backlog.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        debug_assert!(previous > 0, "received handoff must have been sent");
    }

    fn calibration_chunk(&self, entries: u64, work_ns: u64) {
        self.calibration_chunks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.calibration_entries.fetch_add(entries, std::sync::atomic::Ordering::Relaxed);
        self.calibration_work_ns.fetch_add(work_ns, std::sync::atomic::Ordering::Relaxed);
    }

    fn worker_expanded(&self) {
        self.worker_expansions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn portable_attempted(&self) {
        self.portable_attempts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn portable_succeeded(&self) {
        self.portable_successes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    #[cfg(target_os = "macos")]
    fn macos_bulk_attempted(&self) {
        self.macos_bulk_attempts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    #[cfg(target_os = "macos")]
    fn macos_bulk_succeeded(&self) {
        self.macos_bulk_successes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    #[cfg(target_os = "macos")]
    fn macos_bulk_fell_back(&self) {
        self.macos_bulk_fallbacks.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn finish(&self) -> ScanDiagnostics {
        let trace = self.trace.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let calibration = self.pool.calibration;
        let backend = ScanBackendDiagnostics {
            portable_attempts: self.portable_attempts.load(std::sync::atomic::Ordering::Relaxed),
            portable_directory_reads: self
                .portable_successes
                .load(std::sync::atomic::Ordering::Relaxed),
            #[cfg(target_os = "macos")]
            macos_bulk_attempts: Some(
                self.macos_bulk_attempts.load(std::sync::atomic::Ordering::Relaxed),
            ),
            #[cfg(not(target_os = "macos"))]
            macos_bulk_attempts: None,
            #[cfg(target_os = "macos")]
            macos_bulk_successes: Some(
                self.macos_bulk_successes.load(std::sync::atomic::Ordering::Relaxed),
            ),
            #[cfg(not(target_os = "macos"))]
            macos_bulk_successes: None,
            #[cfg(target_os = "macos")]
            macos_bulk_fallbacks: Some(
                self.macos_bulk_fallbacks.load(std::sync::atomic::Ordering::Relaxed),
            ),
            #[cfg(not(target_os = "macos"))]
            macos_bulk_fallbacks: None,
            #[cfg(target_os = "macos")]
            unavailable_reason: None,
            #[cfg(not(target_os = "macos"))]
            unavailable_reason: Some(
                "macOS bulk directory enumeration is unavailable on this platform",
            ),
        };
        ScanDiagnostics {
            schema: SCAN_DIAGNOSTICS_SCHEMA,
            worker_policy: WorkerPolicyDiagnostics {
                controller: worker_policy_experiment_name(self.policy),
                available_parallelism: self.available_parallelism,
                initial_workers: self.pool.initial,
                maximum_workers: self.pool.maximum,
                calibration_window_entries: calibration.map(|value| value.minimum_entries),
                slow_threshold_ns_per_entry: calibration.map(|value| value.slow_work_ns_per_entry),
                calibration_chunks: self
                    .calibration_chunks
                    .load(std::sync::atomic::Ordering::Relaxed),
                calibration_entries: self
                    .calibration_entries
                    .load(std::sync::atomic::Ordering::Relaxed),
                calibration_work_ns: self
                    .calibration_work_ns
                    .load(std::sync::atomic::Ordering::Relaxed),
                worker_expansions: self
                    .worker_expansions
                    .load(std::sync::atomic::Ordering::Relaxed),
                outcome: trace.outcome,
                outcome_reason: trace.outcome_reason,
                workers_spawned: self.workers_spawned.load(std::sync::atomic::Ordering::Relaxed),
                peak_active_workers: self
                    .peak_active_workers
                    .load(std::sync::atomic::Ordering::Relaxed),
                ready_directories_at_finish: trace.ready_directories_at_finish,
                in_flight_directories_at_finish: trace.in_flight_directories_at_finish,
                handoff_backlog_at_finish: self
                    .handoff_backlog
                    .load(std::sync::atomic::Ordering::Relaxed),
                handoff_backlog_high_water: self
                    .handoff_backlog_high_water
                    .load(std::sync::atomic::Ordering::Relaxed),
                windows: {
                    let mut windows = trace.windows.clone();
                    windows.sort_by_key(|window| window.sequence);
                    windows
                },
                events_truncated: trace.events_truncated,
            },
            backend,
        }
    }
}

const fn worker_policy_experiment_name(value: WorkerPolicyExperiment) -> &'static str {
    match value {
        WorkerPolicyExperiment::ShippedOneShot => "shipped_one_shot",
        WorkerPolicyExperiment::RepeatedWindows => "repeated_windows",
        WorkerPolicyExperiment::StagedGatedWindows => "staged_gated_windows",
    }
}

struct ScanWorkerGuard {
    recorder: std::sync::Arc<ScanDiagnosticsRecorder>,
}

impl Drop for ScanWorkerGuard {
    fn drop(&mut self) {
        let previous =
            self.recorder.active_workers.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        debug_assert!(previous > 0, "worker guard must balance worker start");
    }
}

fn atomic_update_max(target: &std::sync::atomic::AtomicUsize, value: usize) {
    let mut observed = target.load(std::sync::atomic::Ordering::Relaxed);
    while value > observed {
        match target.compare_exchange_weak(
            observed,
            value,
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(actual) => observed = actual,
        }
    }
}

enum WalkMessage {
    Observation(Observation),
    ScaleUp { sender: std::sync::mpsc::Sender<Self>, target_workers: usize },
}

impl WorkerPool {
    const fn fixed(workers: usize) -> Self {
        Self { initial: workers, maximum: workers, calibration: None }
    }
}

impl WorkerCalibration {
    const fn new(minimum_entries: u64, slow_work_ns_per_entry: u64) -> Self {
        Self { minimum_entries, slow_work_ns_per_entry, entries: 0, work_ns: 0, chunks: 0 }
    }

    fn observe(&mut self, entries: u64, work_ns: u64) -> Option<bool> {
        self.chunks = self.chunks.saturating_add(1);
        self.entries = self.entries.saturating_add(entries);
        self.work_ns = self.work_ns.saturating_add(work_ns);
        (self.entries >= self.minimum_entries)
            .then(|| self.work_ns / self.entries >= self.slow_work_ns_per_entry)
    }
}

#[derive(Clone, Copy, Debug)]
struct CalibrationWindow {
    start_entry_ordinal: u64,
    end_entry_ordinal: u64,
    entries: u64,
    chunks: u64,
    work_ns: u64,
    slow: bool,
}

#[derive(Debug)]
struct RepeatedCalibration {
    minimum_entries: u64,
    slow_work_ns_per_entry: u64,
    window_start: u64,
    entries: u64,
    chunks: u64,
    work_ns: u64,
    completed_windows: u64,
}

impl RepeatedCalibration {
    const fn new(calibration: WorkerCalibration) -> Self {
        Self {
            minimum_entries: calibration.minimum_entries,
            slow_work_ns_per_entry: calibration.slow_work_ns_per_entry,
            window_start: 0,
            entries: 0,
            chunks: 0,
            work_ns: 0,
            completed_windows: 0,
        }
    }

    const fn starting_at(calibration: WorkerCalibration, window_start: u64) -> Self {
        let mut repeated = Self::new(calibration);
        repeated.window_start = window_start;
        repeated
    }

    fn observe(&mut self, entries: u64, work_ns: u64) -> Option<CalibrationWindow> {
        self.chunks = self.chunks.saturating_add(1);
        self.entries = self.entries.saturating_add(entries);
        self.work_ns = self.work_ns.saturating_add(work_ns);
        if self.entries < self.minimum_entries {
            return None;
        }
        let end_entry_ordinal = self.window_start.saturating_add(self.entries);
        let window = CalibrationWindow {
            start_entry_ordinal: self.window_start,
            end_entry_ordinal,
            entries: self.entries,
            chunks: self.chunks,
            work_ns: self.work_ns,
            slow: self.work_ns / self.entries >= self.slow_work_ns_per_entry,
        };
        self.window_start = end_entry_ordinal;
        self.entries = 0;
        self.chunks = 0;
        self.work_ns = 0;
        self.completed_windows = self.completed_windows.saturating_add(1);
        Some(window)
    }
}

#[derive(Debug)]
enum WorkerController {
    OneShot(WorkerCalibration),
    Repeated { calibration: RepeatedCalibration, staged_gated: bool },
}

impl WorkerController {
    fn new(calibration: WorkerCalibration, policy: WorkerPolicyExperiment) -> Self {
        match policy {
            WorkerPolicyExperiment::ShippedOneShot => Self::OneShot(calibration),
            WorkerPolicyExperiment::RepeatedWindows => Self::Repeated {
                calibration: RepeatedCalibration::new(calibration),
                staged_gated: false,
            },
            WorkerPolicyExperiment::StagedGatedWindows => Self::Repeated {
                calibration: RepeatedCalibration::new(calibration),
                staged_gated: true,
            },
        }
    }

    fn observe(&mut self, entries: u64, work_ns: u64) -> Option<CalibrationWindow> {
        match self {
            Self::OneShot(calibration) => {
                let slow = calibration.observe(entries, work_ns)?;
                Some(CalibrationWindow {
                    start_entry_ordinal: 0,
                    end_entry_ordinal: calibration.entries,
                    entries: calibration.entries,
                    chunks: calibration.chunks,
                    work_ns: calibration.work_ns,
                    slow,
                })
            }
            Self::Repeated { calibration, .. } => calibration.observe(entries, work_ns),
        }
    }

    fn partial_window(&self) -> (CalibrationWindow, WorkerPolicyDecision) {
        match self {
            Self::OneShot(calibration) => (
                CalibrationWindow {
                    start_entry_ordinal: 0,
                    end_entry_ordinal: calibration.entries,
                    entries: calibration.entries,
                    chunks: calibration.chunks,
                    work_ns: calibration.work_ns,
                    slow: false,
                },
                WorkerPolicyDecision::Undecided,
            ),
            Self::Repeated { calibration, .. } => (
                CalibrationWindow {
                    start_entry_ordinal: calibration.window_start,
                    end_entry_ordinal: calibration.window_start.saturating_add(calibration.entries),
                    entries: calibration.entries,
                    chunks: calibration.chunks,
                    work_ns: calibration.work_ns,
                    slow: false,
                },
                if calibration.completed_windows == 0 {
                    WorkerPolicyDecision::Undecided
                } else {
                    WorkerPolicyDecision::Incomplete
                },
            ),
        }
    }

    const fn is_staged_gated(&self) -> bool {
        matches!(self, Self::Repeated { staged_gated: true, .. })
    }

    const fn is_one_shot(&self) -> bool {
        matches!(self, Self::OneShot(_))
    }

    const fn calibration_spec(&self) -> WorkerCalibration {
        match self {
            Self::OneShot(calibration) => WorkerCalibration::new(
                calibration.minimum_entries,
                calibration.slow_work_ns_per_entry,
            ),
            Self::Repeated { calibration, .. } => WorkerCalibration::new(
                calibration.minimum_entries,
                calibration.slow_work_ns_per_entry,
            ),
        }
    }
}

fn automatic_worker_pool(available: usize) -> WorkerPool {
    let initial = available.clamp(1, DEFAULT_SCAN_THREADS_CAP);
    // Preserve the serial fallback when the platform cannot report more than one
    // available processor. There is no measured basis for inventing parallelism there.
    if initial == 1 {
        return WorkerPool::fixed(1);
    }
    let maximum = available
        .saturating_mul(ADAPTIVE_SCAN_PARALLELISM_MULTIPLIER)
        .clamp(initial, ADAPTIVE_SCAN_THREADS_CAP);
    WorkerPool {
        initial,
        maximum,
        calibration: (maximum > initial).then_some(WorkerCalibration::new(
            ADAPTIVE_SCAN_CALIBRATION_ENTRIES,
            ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY,
        )),
    }
}

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
    pool: WorkerPool,
    diagnostics: Option<&std::sync::Arc<ScanDiagnosticsRecorder>>,
    policy: WorkerPolicyExperiment,
) -> ScanReport {
    let diagnostics = diagnostics.cloned();
    let queue = DirectoryQueue::new_with_policy(
        (PathBuf::new(), 0),
        config.order,
        pool.calibration,
        diagnostics.clone(),
        pool.initial,
        pool.maximum,
        policy,
    );
    let (sender, receiver) = std::sync::mpsc::channel::<WalkMessage>();
    // One counter for the whole walk, not one per worker: a per-worker cap would admit
    // `cap * workers` files and make the bound depend on a thread count that is explicitly
    // not part of the scope.
    let budget = Budget::for_config(config);

    let mut report = std::thread::scope(|scope| {
        let mut handles: Vec<_> = (0..pool.initial)
            .map(|_| {
                let sender = sender.clone();
                let queue = &queue;
                let diagnostics = diagnostics.clone();
                let budget = budget.clone();
                scope.spawn(move || {
                    walk_worker(
                        root,
                        config,
                        root_dev,
                        queue,
                        &sender,
                        diagnostics.as_ref(),
                        budget.as_ref(),
                    )
                })
            })
            .collect();
        // The loop below ends when every sender is gone, so this one must go first.
        drop(sender);

        let mut spawned_workers = pool.initial;
        for message in receiver {
            match message {
                WalkMessage::Observation(observation) => {
                    if let Some(diagnostics) = &diagnostics {
                        diagnostics.handoff_received();
                    }
                    sink(observation);
                }
                WalkMessage::ScaleUp { sender, target_workers }
                    if target_workers > spawned_workers =>
                {
                    let target_workers = target_workers.min(pool.maximum);
                    record_adaptive_worker_expansion(diagnostics.as_ref());
                    for _ in spawned_workers..target_workers {
                        let sender = sender.clone();
                        let queue = &queue;
                        let diagnostics = diagnostics.clone();
                        let budget = budget.clone();
                        handles.push(scope.spawn(move || {
                            walk_worker(
                                root,
                                config,
                                root_dev,
                                queue,
                                &sender,
                                diagnostics.as_ref(),
                                budget.as_ref(),
                            )
                        }));
                    }
                    spawned_workers = target_workers;
                }
                WalkMessage::ScaleUp { .. } => {}
            }
        }

        // A walk that ends before its calibration window fills never observed enough to
        // decide anything. That is an *unobservable* policy, not a decision to hold the
        // initial pool, and an artifact that conflated the two would report a held pool
        // as if the walk had measured one and chosen it.
        let queue_finish = {
            let mut state = queue.lock();
            let mut trailing_window = None;
            if let Some(controller) = &state.controller {
                let (window, decision) = controller.partial_window();
                if decision == WorkerPolicyDecision::Undecided {
                    crate::counters::bump(|counts| {
                        counts.adaptive_policy_undecided =
                            counts.adaptive_policy_undecided.saturating_add(1);
                    });
                }
                if diagnostics.is_some() {
                    let sequence = state.allocate_policy_sequence();
                    trailing_window = Some(PolicyWindowSnapshot {
                        sequence,
                        start_entry_ordinal: window.start_entry_ordinal,
                        end_entry_ordinal: window.end_entry_ordinal,
                        observed_entries: window.entries,
                        observed_chunks: window.chunks,
                        observed_work_ns: window.work_ns,
                        ready_directories: state.ready_directories,
                        in_flight_directories: state.in_flight_directories,
                        active_workers: diagnostics.as_ref().map_or(0, |diagnostics| {
                            diagnostics.active_workers.load(std::sync::atomic::Ordering::Relaxed)
                        }),
                        handoff_backlog: diagnostics.as_ref().map_or(0, |diagnostics| {
                            diagnostics.handoff_backlog.load(std::sync::atomic::Ordering::Relaxed)
                        }),
                        requested_workers: None,
                        decision,
                    });
                }
            } else if diagnostics.is_some() {
                let shadow = state.shadow_calibration.as_ref().and_then(|shadow| {
                    (shadow.entries > 0).then_some((
                        shadow.window_start,
                        shadow.entries,
                        shadow.chunks,
                        shadow.work_ns,
                    ))
                });
                if let Some((window_start, entries, chunks, work_ns)) = shadow {
                    let sequence = state.allocate_policy_sequence();
                    trailing_window = Some(PolicyWindowSnapshot {
                        sequence,
                        start_entry_ordinal: window_start,
                        end_entry_ordinal: window_start.saturating_add(entries),
                        observed_entries: entries,
                        observed_chunks: chunks,
                        observed_work_ns: work_ns,
                        ready_directories: state.ready_directories,
                        in_flight_directories: state.in_flight_directories,
                        active_workers: diagnostics.as_ref().map_or(0, |diagnostics| {
                            diagnostics.active_workers.load(std::sync::atomic::Ordering::Relaxed)
                        }),
                        handoff_backlog: diagnostics.as_ref().map_or(0, |diagnostics| {
                            diagnostics.handoff_backlog.load(std::sync::atomic::Ordering::Relaxed)
                        }),
                        requested_workers: None,
                        decision: WorkerPolicyDecision::ObserveIncomplete,
                    });
                }
            }
            (state.ready_directories, state.in_flight_directories, trailing_window)
        };
        if let Some(diagnostics) = &diagnostics {
            diagnostics.record_queue_finish(queue_finish.0, queue_finish.1);
            if let Some(window) = queue_finish.2 {
                diagnostics.record_policy_window(window);
            }
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
    sender: &std::sync::mpsc::Sender<WalkMessage>,
    diagnostics: Option<&std::sync::Arc<ScanDiagnosticsRecorder>>,
    budget: Option<&std::sync::Arc<Budget>>,
) -> ScanReport {
    let _counter_guard = crate::counters::thread_flush_guard();
    let _worker_guard = diagnostics.map(ScanDiagnosticsRecorder::worker_guard);
    let worker_started = std::time::Instant::now();
    let mut report = ScanReport::default();
    let mut batch: Vec<Op> = Vec::with_capacity(config.batch_size);
    let mut claimed: Vec<(PathBuf, usize, RegionId)> = Vec::with_capacity(DIR_CLAIM);
    let mut discovered: Vec<(PathBuf, usize, RegionId)> = Vec::new();
    let mut consumer_gone = false;
    #[cfg(target_os = "macos")]
    let mut bulk_reader = macos_bulk::Reader::new();

    'walk: while let Some(claim) = queue.claim(&mut claimed, &mut report.attribution) {
        // A claim's directories are read even if the cap falls partway through it. That is
        // the overshoot the between-directories rule buys: a directory is whole or absent,
        // and no new directory enters the queue once the cap is reached, so the queue
        // drains and the walk ends rather than deadlocking on partially-claimed work.
        // One timing pair per claimed chunk, never per entry: the chunk is the unit
        // the amortization argument is made in, so it is the unit the evidence is
        // collected in.
        let chunk_started = std::time::Instant::now();
        let mut chunk_send_ns: u64 = 0;
        let entries_before = report.entries;
        for (rel_dir, depth, region) in claimed.drain(..) {
            if !within_budget(budget) {
                // Drained rather than abandoned: the queue counts a claimed directory as
                // dealt with, so skipping it here lets the walk wind down through the same
                // path it always does. Nothing below it is discovered, which is the point.
                report.budget_stopped = true;
                continue;
            }
            let abs_dir = root.join(&rel_dir);
            let files_before = report.files_walked;
            #[cfg(target_os = "macos")]
            {
                if let Some(diagnostics) = diagnostics {
                    diagnostics.macos_bulk_attempted();
                }
                if let Some(entries) = bulk_reader.read(&abs_dir) {
                    if let Some(diagnostics) = diagnostics {
                        diagnostics.macos_bulk_succeeded();
                    }
                    report.dirs_read += 1;
                    for entry in entries {
                        // The same admission the portable loop applies, and it was missing
                        // here: `admits` is asked from every listing loop, and the bulk
                        // reader is a listing loop that does not look like one -- it hands
                        // over name, kind and attrs together, so there is no `read_dir`
                        // item to attach the check to and it reads as a fast path rather
                        // than as a fourth place the rule has to hold.
                        if !admits(&entry.name, config) {
                            note_pruned_control_file(&entry.name, &rel_dir, config, &mut report);
                            continue;
                        }
                        if !record_walk_entry(
                            &rel_dir,
                            depth,
                            region,
                            entry.name,
                            entry.kind,
                            entry.attrs,
                            root_dev,
                            config,
                            budget,
                            &mut batch,
                            &mut discovered,
                            &mut report,
                            sender,
                            &mut chunk_send_ns,
                            diagnostics.map(AsRef::as_ref),
                        ) {
                            consumer_gone = true;
                            break 'walk;
                        }
                    }
                    if let Some(budget) = budget {
                        budget.charge(report.files_walked - files_before);
                    }
                    continue;
                }
                if let Some(diagnostics) = diagnostics {
                    diagnostics.macos_bulk_fell_back();
                }
            }

            crate::counters::bump(|c| c.dir_opens += 1);
            if let Some(diagnostics) = diagnostics {
                diagnostics.portable_attempted();
            }

            let listing = match fs::read_dir(&abs_dir) {
                Ok(listing) => {
                    if let Some(diagnostics) = diagnostics {
                        diagnostics.portable_succeeded();
                    }
                    listing
                }
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
                crate::counters::bump(|c| c.dir_entries += 1);
                let name = item.file_name();
                if !admits(&name, config) {
                    note_pruned_control_file(&name, &rel_dir, config, &mut report);
                    continue;
                }
                let meta = match metadata_for_fingerprint(&item) {
                    Ok(meta) => meta,
                    Err(e) => {
                        report.errors.push(Error::io(item.path(), e));
                        continue;
                    }
                };

                let attrs = attrs_from(&meta);
                let kind = kind_from(&meta);
                if !record_walk_entry(
                    &rel_dir,
                    depth,
                    region,
                    name,
                    kind,
                    attrs,
                    root_dev,
                    config,
                    budget,
                    &mut batch,
                    &mut discovered,
                    &mut report,
                    sender,
                    &mut chunk_send_ns,
                    diagnostics.map(AsRef::as_ref),
                ) {
                    // The consumer is gone; nothing further will be read.
                    consumer_gone = true;
                    break 'walk;
                }
            }
            // Once per directory, after it is fully listed. The `read_dir` failure above
            // returns here having observed nothing, so its charge is zero and costs a
            // branch rather than a write.
            if let Some(budget) = budget {
                budget.charge(report.files_walked - files_before);
            }
        }
        report.attribution.send_ns += chunk_send_ns;
        let chunk_work_ns = elapsed_ns(chunk_started).saturating_sub(chunk_send_ns);
        report.attribution.work_ns += chunk_work_ns;
        // Publish before releasing the claim so a worker that finds nothing new does
        // not hold work that others could be doing.
        if !discovered.is_empty() {
            queue.extend(discovered.drain(..), &mut report.attribution);
        }
        if let Some(target_workers) = claim.release(
            report.entries.saturating_sub(entries_before),
            chunk_work_ns,
            &mut report.attribution,
        ) {
            // Carry a sender in-band so the consumer can create the reserve workers
            // without retaining a channel endpoint that would keep a small scan alive.
            // Only the release that completes a slow calibration returns true, so one
            // message expands the pool exactly once.
            let _ = sender.send(WalkMessage::ScaleUp { sender: sender.clone(), target_workers });
        }
    }

    if !consumer_gone && !batch.is_empty() {
        let send_started = std::time::Instant::now();
        let _ = send_observation(sender, Observation::new(batch), diagnostics.map(AsRef::as_ref));
        report.attribution.send_ns += elapsed_ns(send_started);
    }
    report.attribution.wall_ns = elapsed_ns(worker_started);
    report
}

#[allow(clippy::too_many_arguments)]
fn record_walk_entry(
    rel_dir: &Path,
    depth: usize,
    region: RegionId,
    name: OsString,
    kind: EntryKind,
    attrs: Attrs,
    root_dev: u64,
    config: &ScanConfig,
    budget: Option<&std::sync::Arc<Budget>>,
    batch: &mut Vec<Op>,
    discovered: &mut Vec<(PathBuf, usize, RegionId)>,
    report: &mut ScanReport,
    sender: &std::sync::mpsc::Sender<WalkMessage>,
    chunk_send_ns: &mut u64,
    diagnostics: Option<&ScanDiagnosticsRecorder>,
) -> bool {
    let rel_path = rel_dir.join(name);
    let mut descend = should_descend(kind, attrs, depth, root_dev, config);
    if descend && !within_budget(budget) {
        // In scope, and not read, because the cap was reached first. Recorded rather than
        // dropped: a silently skipped subtree leaves its parent reporting a complete
        // roll-up over entries nobody looked at.
        descend = false;
        report.budget_stopped = true;
    }
    report.observe(kind, attrs);
    batch.push(Op::Upsert { path: rel_path.clone(), kind, attrs });
    if batch.len() >= config.batch_size {
        let send_started = std::time::Instant::now();
        let sent = send_observation(sender, Observation::new(std::mem::take(batch)), diagnostics);
        *chunk_send_ns += elapsed_ns(send_started);
        if !sent {
            return false;
        }
    }
    if descend {
        // A child of the root seeds a new region; everything deeper inherits its
        // parent's. Region membership therefore costs one integer copy and never
        // inspects a path.
        let child_region = if depth == 0 { RegionId::UNASSIGNED } else { region };
        discovered.push((rel_path, depth + 1, child_region));
    }
    true
}

fn send_observation(
    sender: &std::sync::mpsc::Sender<WalkMessage>,
    observation: Observation,
    diagnostics: Option<&ScanDiagnosticsRecorder>,
) -> bool {
    if let Some(diagnostics) = diagnostics {
        diagnostics.handoff_sent();
    }
    let sent = sender.send(WalkMessage::Observation(observation)).is_ok();
    if !sent {
        // Balance the reservation when the receiver disappeared before accepting it.
        if let Some(diagnostics) = diagnostics {
            diagnostics.handoff_received();
        }
    }
    sent
}

/// Nanoseconds since `started`, saturating rather than panicking on the absurd.
fn elapsed_ns(started: std::time::Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

/// A top-level subtree, used to spread workers across the breadth of the tree.
///
/// Every directory below the root belongs to the region seeded by its depth-1
/// ancestor, inherited from its parent rather than recomputed from its path. The root
/// itself is [`RegionId::ROOT`], which exists only to bootstrap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct RegionId(usize);

impl RegionId {
    /// The root's own region, which exists only to bootstrap the walk.
    const ROOT: Self = Self(0);
    /// "Allocate a fresh region for this directory." Resolved by
    /// [`DirectoryQueueState::push`], which is the only place holding the lock that
    /// owns the region table.
    const UNASSIGNED: Self = Self(usize::MAX);
}

/// Directories still to read, plus enough state to know when the walk is finished.
///
/// The termination condition is the only subtle part: the queue being empty does not
/// mean the walk is done, because a worker that is mid-directory may be about to push
/// its children. So a worker holds a claim from the moment it takes work until the
/// moment it has published everything that work produced, and the walk ends only when
/// the queue is empty *and* no claim is outstanding.
///
/// # Why breadth-first is region-scheduled rather than a global FIFO
///
/// A global FIFO orders the *queue*, but claims are unordered: workers take whatever
/// is at the front, which on a real tree means several workers grinding through the
/// same top-level subtree while others sit untouched. Measured on the branching
/// fixture, that left a global-FIFO walk starting the same 7–8 of 12 subtrees at the
/// halfway mark as depth-first did — the ordering bought nothing a consumer could see.
/// It also made the pending set hold a whole level of the tree, which is where the
/// +1.5–3.7% peak RSS in exp-012 came from.
///
/// So breadth-first keeps work in per-region buckets and hands each free worker a
/// *different* region, round-robin. Within a region the bucket is LIFO, which restores
/// depth-first's locality and spine-bounded memory. Nothing waits on a level boundary:
/// if only one region has work, every worker takes it. The result is a scheduler whose
/// shallow preference is expressed in *which subtree a worker picks up*, not in the
/// order a single queue drains — which is the property progressive consumers actually
/// need.
struct DirectoryQueue {
    state: std::sync::Mutex<DirectoryQueueState>,
    ready: std::sync::Condvar,
    order: ScanOrder,
    diagnostics: Option<std::sync::Arc<ScanDiagnosticsRecorder>>,
}

/// One outstanding claim, held for exactly as long as the worker owes the queue the
/// work it took.
///
/// Giving the claim back is the queue's liveness condition, not a courtesy: [`claim`]
/// parks every other worker on the condvar while `outstanding` is nonzero, so a single
/// claim that is never returned stops the whole walk and the scoped join that waits on
/// it. That makes `Drop` the only safe place to put the release, because the paths that
/// skip a hand-written call are exactly the ones that matter — the `break` taken when
/// the consumer disconnects, and an unwinding panic inside a directory read.
///
/// [`claim`]: DirectoryQueue::claim
struct DirectoryClaim<'a> {
    queue: &'a DirectoryQueue,
    /// Directories represented by the claim, for exact in-flight accounting.
    directories: usize,
    /// Whether the worker already returned this claim through [`Self::release`].
    released: bool,
}

impl DirectoryClaim<'_> {
    /// Return the claim at the end of a completed chunk, feeding the chunk's own
    /// measurements to the shared calibration.
    ///
    /// Returns the queue's scale-up decision, which is why the normal path cannot be
    /// `Drop`: a destructor has neither the chunk's timing nor anywhere to put an
    /// answer.
    fn release(
        mut self,
        entries: u64,
        work_ns: u64,
        timing: &mut WalkAttribution,
    ) -> Option<usize> {
        self.released = true;
        self.queue.release(self.directories, entries, work_ns, timing)
    }
}

impl Drop for DirectoryClaim<'_> {
    fn drop(&mut self) {
        if self.released {
            return;
        }
        // An abandoned chunk: the consumer went away mid-directory, or a read panicked.
        // Either way the partial timing describes an aborted chunk rather than the cost
        // of reading directories, so it must not reach the calibration that sizes the
        // worker pool. Returning the claim is the whole job.
        self.queue.abandon(self.directories);
    }
}

struct DirectoryQueueState {
    /// Depth-first's single stack. Unused under breadth-first.
    pending: VecDeque<(PathBuf, usize, RegionId)>,
    /// Breadth-first's per-region work, indexed by [`RegionId`]. Each is a LIFO stack.
    regions: Vec<Vec<(PathBuf, usize, RegionId)>>,
    /// Regions with work, in round-robin order. A region appears at most once; the
    /// flag array is what keeps that true without scanning the ring.
    ready_ring: VecDeque<RegionId>,
    /// Whether each region is currently in `ready_ring`.
    enqueued: Vec<bool>,
    /// Directories currently available for a future claim.
    ready_directories: usize,
    /// Directories held by outstanding claims.
    in_flight_directories: usize,
    outstanding: usize,
    finished: bool,
    controller: Option<WorkerController>,
    /// Observation-only windows retained after the shipped one-shot decision.
    shadow_calibration: Option<RepeatedCalibration>,
    /// Completion-order sequence assigned under the queue lock.
    next_policy_sequence: u64,
    worker_target: usize,
    maximum_workers: usize,
}

impl DirectoryQueueState {
    fn seeded(
        root: (PathBuf, usize),
        order: ScanOrder,
        calibration: Option<WorkerCalibration>,
        initial_workers: usize,
        maximum_workers: usize,
        policy: WorkerPolicyExperiment,
    ) -> Self {
        let mut state = Self {
            pending: VecDeque::new(),
            regions: Vec::new(),
            ready_ring: VecDeque::new(),
            enqueued: Vec::new(),
            ready_directories: 0,
            in_flight_directories: 0,
            outstanding: 0,
            finished: false,
            controller: calibration.map(|value| WorkerController::new(value, policy)),
            shadow_calibration: None,
            next_policy_sequence: 0,
            worker_target: initial_workers,
            maximum_workers,
        };
        state.push((root.0, root.1, RegionId::ROOT), order);
        state
    }

    /// Push one directory into the structure the order uses.
    fn push(&mut self, item: (PathBuf, usize, RegionId), order: ScanOrder) {
        self.ready_directories = self.ready_directories.saturating_add(1);
        match order {
            ScanOrder::DepthFirst => self.pending.push_back(item),
            ScanOrder::BreadthFirst => {
                let region = if item.2 == RegionId::UNASSIGNED {
                    // One region per top-level subtree, numbered as they are found.
                    self.regions.len().max(1)
                } else {
                    item.2.0
                };
                if region >= self.regions.len() {
                    self.regions.resize_with(region + 1, Vec::new);
                    self.enqueued.resize(region + 1, false);
                }
                // Resolve the id *into* the item, so every directory discovered beneath
                // this one inherits a concrete region instead of the sentinel. Without
                // this the sentinel propagates and each directory allocates a region of
                // its own, degenerating the scheduler into round-robin over the whole
                // frontier.
                let mut item = item;
                item.2 = RegionId(region);
                self.regions[region].push(item);
                if !self.enqueued[region] {
                    self.enqueued[region] = true;
                    self.ready_ring.push_back(RegionId(region));
                }
            }
        }
    }

    /// Whether any work is available.
    fn is_empty(&self, order: ScanOrder) -> bool {
        match order {
            ScanOrder::DepthFirst => self.pending.is_empty(),
            ScanOrder::BreadthFirst => self.ready_ring.is_empty(),
        }
    }

    /// Take up to `limit` directories from the next region in the round-robin ring.
    ///
    /// Every region holding work is in the ring exactly once, so popping it always
    /// finds work and always moves to a *different* subtree than the previous claim.
    /// An earlier version preferred the caller's previous region for locality, which
    /// pinned each worker to one subtree: with twelve deep chains and six workers only
    /// six subtrees ever advanced, and depth-first — whose four-directory claims
    /// happen to fan across the root's children — spread wider than breadth-first did.
    /// Locality still comes from the claim being a run of directories out of one
    /// region; it must not come from a worker refusing to leave.
    fn take(
        &mut self,
        limit: usize,
        order: ScanOrder,
        into: &mut Vec<(PathBuf, usize, RegionId)>,
    ) -> usize {
        let before = into.len();
        match order {
            ScanOrder::DepthFirst => {
                let take = self.pending.len().min(limit);
                let start = self.pending.len() - take;
                into.extend(self.pending.drain(start..));
            }
            ScanOrder::BreadthFirst => {
                let Some(region) = self.ready_ring.pop_front() else { return 0 };
                self.enqueued[region.0] = false;
                let bucket = &mut self.regions[region.0];
                let take = bucket.len().min(limit);
                let start = bucket.len() - take;
                into.extend(bucket.drain(start..));
                // Re-arm the region only if work remains and it is not already queued,
                // so a busy region cannot appear twice and starve the others.
                if !bucket.is_empty() && !self.enqueued[region.0] {
                    self.enqueued[region.0] = true;
                    self.ready_ring.push_back(region);
                }
            }
        }
        into.len().saturating_sub(before)
    }

    fn allocate_policy_sequence(&mut self) -> u64 {
        let sequence = self.next_policy_sequence;
        self.next_policy_sequence = self.next_policy_sequence.saturating_add(1);
        sequence
    }
}

impl DirectoryQueue {
    /// Seed the queue with the root, which is region zero until its children fan out.
    #[cfg(test)]
    fn new(
        root: (PathBuf, usize),
        order: ScanOrder,
        calibration: Option<WorkerCalibration>,
        diagnostics: Option<std::sync::Arc<ScanDiagnosticsRecorder>>,
    ) -> Self {
        Self::new_with_policy(
            root,
            order,
            calibration,
            diagnostics,
            1,
            2,
            WorkerPolicyExperiment::ShippedOneShot,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_policy(
        root: (PathBuf, usize),
        order: ScanOrder,
        calibration: Option<WorkerCalibration>,
        diagnostics: Option<std::sync::Arc<ScanDiagnosticsRecorder>>,
        initial_workers: usize,
        maximum_workers: usize,
        policy: WorkerPolicyExperiment,
    ) -> Self {
        let state = DirectoryQueueState::seeded(
            root,
            order,
            calibration,
            initial_workers,
            maximum_workers,
            policy,
        );
        Self {
            state: std::sync::Mutex::new(state),
            ready: std::sync::Condvar::new(),
            order,
            diagnostics,
        }
    }

    /// Take up to [`DIR_CLAIM`] directories, blocking until there is work or the walk
    /// is over. Returns `None` once no more work will ever arrive.
    ///
    /// Time spent waiting is charged to `timing`: lock acquisition to `lock_wait_ns`
    /// when contended, condvar waits to `starved_ns`. The condvar span includes the
    /// lock re-acquisition on wake, which slightly overstates starvation rather than
    /// understating contention — the fail-honest direction for the number that is
    /// supposed to stay near zero.
    fn claim<'a>(
        &'a self,
        into: &mut Vec<(PathBuf, usize, RegionId)>,
        timing: &mut WalkAttribution,
    ) -> Option<DirectoryClaim<'a>> {
        let mut state = self.lock_timed(timing);
        loop {
            if !state.is_empty(self.order) {
                let directories = state.take(DIR_CLAIM, self.order, into);
                state.ready_directories = state.ready_directories.saturating_sub(directories);
                state.in_flight_directories =
                    state.in_flight_directories.saturating_add(directories);
                state.outstanding += 1;
                timing.claims += 1;
                return Some(DirectoryClaim { queue: self, directories, released: false });
            }
            if state.finished {
                return None;
            }
            if state.outstanding == 0 {
                state.finished = true;
                self.ready.notify_all();
                return None;
            }
            let started = std::time::Instant::now();
            state = self.ready.wait(state).unwrap_or_else(std::sync::PoisonError::into_inner);
            timing.starved_ns += elapsed_ns(started);
        }
    }

    fn extend(
        &self,
        directories: impl Iterator<Item = (PathBuf, usize, RegionId)>,
        timing: &mut WalkAttribution,
    ) {
        let mut state = self.lock_timed(timing);
        for item in directories {
            state.push(item, self.order);
        }
        drop(state);
        self.ready.notify_all();
    }

    /// Give up a claim whose chunk never finished. Wakes everyone if it was the last.
    ///
    /// Reached only from [`DirectoryClaim::drop`], where there is no `WalkAttribution`
    /// to charge and nothing worth charging: an abandoned chunk read some unknown
    /// fraction of its directories, so its lock wait says nothing about contention
    /// during the walk.
    fn abandon(&self, directories: usize) {
        let mut state = self.lock();
        state.outstanding -= 1;
        state.in_flight_directories = state.in_flight_directories.saturating_sub(directories);
        if state.outstanding == 0 && state.is_empty(self.order) {
            state.finished = true;
            drop(state);
            self.ready.notify_all();
        }
    }

    /// Give up a claim taken by [`claim`]. Wakes everyone if this was the last one.
    ///
    /// The chunk's own entry count and work time feed the shared service-time
    /// calibration, so the decision uses the timing the walk already collects for
    /// attribution rather than a second clock. Returns the new worker target when a
    /// controller requests expansion. A release that ends the walk returns no target,
    /// because there is no longer useful work for a reserve worker to take.
    fn release(
        &self,
        directories: usize,
        observed_entries: u64,
        observed_work_ns: u64,
        timing: &mut WalkAttribution,
    ) -> Option<usize> {
        let mut state = self.lock_timed(timing);
        // Whether this chunk reached the calibration at all. Only chunks released while
        // it is still live are policy history; later ones are ordinary walk work.
        let calibrating = state.controller.is_some();
        let one_shot = state.controller.as_ref().is_some_and(WorkerController::is_one_shot);
        let controller_spec = state.controller.as_ref().map(WorkerController::calibration_spec);
        let staged_gated = state.controller.as_ref().is_some_and(WorkerController::is_staged_gated);
        let completed_window = state
            .controller
            .as_mut()
            .and_then(|value| value.observe(observed_entries, observed_work_ns));
        let observed_window = state
            .shadow_calibration
            .as_mut()
            .and_then(|value| value.observe(observed_entries, observed_work_ns));
        let window_completed = completed_window.is_some();
        state.outstanding -= 1;
        state.in_flight_directories = state.in_flight_directories.saturating_sub(directories);
        let finished = state.outstanding == 0 && state.is_empty(self.order);
        if finished {
            state.finished = true;
        }
        let handoff_backlog = self.diagnostics.as_ref().map_or(0, |diagnostics| {
            diagnostics.handoff_backlog.load(std::sync::atomic::Ordering::Relaxed)
        });
        let mut requested_workers = None;
        let policy_window = completed_window.map(|window| {
            let useful_frontier =
                state.ready_directories.saturating_add(state.in_flight_directories);
            let decision = if !window.slow {
                WorkerPolicyDecision::Hold
            } else if finished {
                WorkerPolicyDecision::HoldNoUsefulWork
            } else if staged_gated && useful_frontier <= state.worker_target {
                WorkerPolicyDecision::HoldInsufficientFrontier
            } else if staged_gated && handoff_backlog >= state.worker_target {
                WorkerPolicyDecision::HoldHandoffBacklog
            } else {
                let target = if staged_gated {
                    state.worker_target.saturating_mul(2).min(state.maximum_workers)
                } else {
                    state.maximum_workers
                };
                if target > state.worker_target {
                    state.worker_target = target;
                    requested_workers = Some(target);
                    WorkerPolicyDecision::ScaleUp
                } else {
                    WorkerPolicyDecision::HoldInsufficientFrontier
                }
            };
            let sequence = state.allocate_policy_sequence();
            PolicyWindowSnapshot {
                sequence,
                start_entry_ordinal: window.start_entry_ordinal,
                end_entry_ordinal: window.end_entry_ordinal,
                observed_entries: window.entries,
                observed_chunks: window.chunks,
                observed_work_ns: window.work_ns,
                ready_directories: state.ready_directories,
                in_flight_directories: state.in_flight_directories,
                active_workers: self.diagnostics.as_ref().map_or(0, |diagnostics| {
                    diagnostics.active_workers.load(std::sync::atomic::Ordering::Relaxed)
                }),
                handoff_backlog,
                requested_workers,
                decision,
            }
        });
        let shadow_window = observed_window.map(|window| {
            let sequence = state.allocate_policy_sequence();
            PolicyWindowSnapshot {
                sequence,
                start_entry_ordinal: window.start_entry_ordinal,
                end_entry_ordinal: window.end_entry_ordinal,
                observed_entries: window.entries,
                observed_chunks: window.chunks,
                observed_work_ns: window.work_ns,
                ready_directories: state.ready_directories,
                in_flight_directories: state.in_flight_directories,
                active_workers: self.diagnostics.as_ref().map_or(0, |diagnostics| {
                    diagnostics.active_workers.load(std::sync::atomic::Ordering::Relaxed)
                }),
                handoff_backlog,
                requested_workers: None,
                decision: if window.slow {
                    WorkerPolicyDecision::ObserveSlow
                } else {
                    WorkerPolicyDecision::ObserveFast
                },
            }
        });
        let controller_terminates = (one_shot || finished) && window_completed
            || requested_workers.is_some_and(|target| target == state.maximum_workers);
        if controller_terminates {
            // Continue observing after every terminal decision, including a candidate
            // that reached the maximum pool. Without this shadow history a slow-prefix
            // expansion makes a later fast phase unobservable, precisely the
            // irreversible over-expansion case the evidence matrix must detect.
            if let (Some(spec), Some(window)) = (controller_spec, completed_window)
                && self.diagnostics.is_some()
                && !finished
            {
                state.shadow_calibration =
                    Some(RepeatedCalibration::starting_at(spec, window.end_entry_ordinal));
            }
            state.controller = None;
        }
        drop(state);

        // The sequence was assigned while the queue was locked, but the trace lock and
        // bounded-vector update stay outside that critical section. Recorder arrival
        // may differ from completion order; `finish` sorts the retained prefix.
        if let (Some(diagnostics), Some(window)) = (&self.diagnostics, policy_window) {
            diagnostics.record_policy_window(window);
        }
        if let (Some(diagnostics), Some(window)) = (&self.diagnostics, shadow_window) {
            diagnostics.record_policy_window(window);
        }

        // Recorded outside the lock: the sampling is off by default, and a disabled
        // counter must not lengthen the critical section it observes.
        if calibrating {
            record_adaptive_calibration_chunk(
                self.diagnostics.as_ref(),
                observed_entries,
                observed_work_ns,
            );
        }

        if finished {
            self.ready.notify_all();
        }
        requested_workers
    }

    /// Acquire the state lock, charging any contention to `timing`.
    ///
    /// The fast path is a `try_lock` that succeeds and costs one counter increment;
    /// only the contended path pays for reading the clock. Poisoning is tolerated for
    /// the same reason as [`Self::lock`].
    fn lock_timed(
        &self,
        timing: &mut WalkAttribution,
    ) -> std::sync::MutexGuard<'_, DirectoryQueueState> {
        timing.lock_ops += 1;
        match self.state.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => {
                timing.lock_contended += 1;
                let started = std::time::Instant::now();
                let guard = self.lock();
                timing.lock_wait_ns += elapsed_ns(started);
                guard
            }
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

fn record_adaptive_calibration_chunk(
    diagnostics: Option<&std::sync::Arc<ScanDiagnosticsRecorder>>,
    entries: u64,
    work_ns: u64,
) {
    if let Some(diagnostics) = diagnostics {
        diagnostics.calibration_chunk(entries, work_ns);
    }
    crate::counters::bump(|counts| {
        counts.adaptive_calibration_chunks = counts.adaptive_calibration_chunks.saturating_add(1);
        counts.adaptive_calibration_entries =
            counts.adaptive_calibration_entries.saturating_add(entries);
        counts.adaptive_calibration_work_us =
            counts.adaptive_calibration_work_us.saturating_add(work_ns / 1_000);
    });
}

fn record_adaptive_worker_expansion(diagnostics: Option<&std::sync::Arc<ScanDiagnosticsRecorder>>) {
    if let Some(diagnostics) = diagnostics {
        diagnostics.worker_expanded();
    }
    crate::counters::bump(|counts| {
        counts.adaptive_scale_ups = counts.adaptive_scale_ups.saturating_add(1);
    });
}

/// Walk `root` and return a fully populated index.
pub fn scan_into_index(root: &Path, config: &ScanConfig) -> Result<(Index, ScanReport)> {
    config.validate()?;
    let root = root.canonicalize().map_err(|error| Error::io(root, error))?;
    let mut index = Index::new_with_scope(&root, config.scope())
        .with_types(config.types().clone())
        .with_tag_rules(config.tags());
    let mut apply_error: Option<Error> = None;
    let report = scan(&root, config, &mut |observation| {
        if apply_error.is_none()
            && let Err(error) = index.apply_baseline(&observation)
        {
            apply_error = Some(error);
        }
    })?;
    if let Some(error) = apply_error {
        return Err(error);
    }
    index.adopt_pruned_control_dirs(report.control_dirs.iter().cloned());
    index.set_initial_coverage(report.coverage());
    Ok((index, report))
}

/// Walk `root` into an index and retain the opt-in diagnostic trace for that run.
///
/// The index and [`ScanReport`] have the same semantics as [`scan_into_index`]. The
/// additional trace is versioned independently so evidence tooling can fail closed on
/// changes without coupling cache or engine behavior to measurement details.
pub fn scan_into_index_with_diagnostics(
    root: &Path,
    config: &ScanConfig,
) -> Result<(Index, ScanReport, ScanDiagnostics)> {
    scan_into_index_with_policy_diagnostics(root, config, WorkerPolicyExperiment::ShippedOneShot)
}

/// Exercise a repository-only worker-controller candidate while building an index.
#[doc(hidden)]
pub fn scan_into_index_with_policy_diagnostics(
    root: &Path,
    config: &ScanConfig,
    policy: WorkerPolicyExperiment,
) -> Result<(Index, ScanReport, ScanDiagnostics)> {
    config.validate()?;
    let root = root.canonicalize().map_err(|error| Error::io(root, error))?;
    let mut index = Index::new_with_scope(&root, config.scope())
        .with_types(config.types().clone())
        .with_tag_rules(config.tags());
    let mut apply_error: Option<Error> = None;
    let (report, diagnostics) = scan_with_policy_diagnostics(
        &root,
        config,
        &mut |observation| {
            if apply_error.is_none()
                && let Err(error) = index.apply_baseline(&observation)
            {
                apply_error = Some(error);
            }
        },
        policy,
    )?;
    if let Some(error) = apply_error {
        return Err(error);
    }
    index.adopt_pruned_control_dirs(report.control_dirs.iter().cloned());
    index.set_initial_coverage(report.coverage());
    Ok((index, report, diagnostics))
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
    let root_meta = {
        crate::counters::bump(|c| c.stats += 1);
        fs::symlink_metadata(&root)
    }
    .map_err(|error| Error::io(&root, error))?;
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
    let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::from(vec![(PathBuf::new(), 0)]);

    while let Some((rel_dir, depth)) = take_next(&mut queue, config.order) {
        let abs_dir = root.join(&rel_dir);
        crate::counters::bump(|c| c.dir_opens += 1);
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
            if !admits(&name, config) {
                // Before `seen`, deliberately: an index that somehow holds a name the rule
                // now excludes should have it removed, and leaving it in `seen` would say
                // it is still there.
                note_pruned_control_file(&name, &rel_dir, config, &mut report);
                continue;
            }
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
            report.observe(kind, attrs);
            batch.push(ObservationOp::if_state(
                Op::Upsert { path: rel_path.clone(), kind, attrs },
                baseline,
            ));
            if batch.len() >= batch_limit {
                sink(Observation::from_ops(std::mem::take(&mut batch)));
                batch.reserve(batch_limit);
            }

            if should_descend(kind, attrs, depth, root_dev, config) {
                queue.push_back((rel_path, depth + 1));
            } else if kind.is_dir()
                && let Some(children) = index.children(&rel_path)
            {
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

        // Anything the index still lists here but the filesystem did not return is gone.
        if listing_complete && let Some(known) = index.children(&rel_dir) {
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
        return Err(Error::SubtreeOutsideScanScope {
            path: subtree,
            scope: Box::new(config.scope()),
        });
    }
    let subtree = resolve_subtree_root(target, &subtree, config)?;
    // The sweep's own transitions go through the sink like every other committed delta.
    // They are effective commits -- they advance the clock and enter the journal -- and a
    // caller told it would receive "every effective committed delta" was quietly receiving
    // only the ones that carried operations.
    let (started_at, opened) = target.begin_reconcile(&subtree)?;
    sink(&opened);
    match reconcile_target_inner(target, &subtree, config, MAX_DEFERRED_RECONCILE_OPS, sink) {
        Ok(report) => {
            // Before the close, so a consumer woken by the finish delta sees an index that
            // can already bind. Additive on the index side, because a scoped refresh speaks
            // only for the subtree it read.
            target.adopt_pruned_control_dirs(&report.scan.control_dirs)?;
            let closed = target.finish_reconcile(&subtree, started_at, report.coverage())?;
            sink(&closed);
            Ok(report)
        }
        Err(error) => {
            // The pass returned an error rather than completing around one, which is
            // a different fact from an unreadable directory and reads differently to a
            // consumer deciding whether to retry.
            let closed = target.finish_reconcile(
                &subtree,
                started_at,
                Status::Partial(CoverageReason::Failed),
            )?;
            sink(&closed);
            Err(error)
        }
    }
}

fn reconcile_target_inner(
    target: &mut ReconcileTarget<'_>,
    subtree: &Path,
    config: &ScanConfig,
    max_deferred_ops: usize,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<ReconcileReport> {
    let root = target.root_path()?;
    let root_meta = {
        crate::counters::bump(|c| c.stats += 1);
        fs::symlink_metadata(&root)
    }
    .map_err(|error| Error::io(&root, error))?;
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
    let mut retry_frontier = None;
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
        report.scan.observe(kind, attrs);
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

    if subtree.as_os_str().is_empty()
        && config.reconciliation_worker_threads() > 1
        && let ReconcileTarget::Direct(index) = target
    {
        match reconcile_direct_parallel(index, &root, root_dev, config, max_deferred_ops, sink)? {
            DirectParallelOutcome::Complete(parallel) => return Ok(parallel),
            DirectParallelOutcome::RetrySerial { prefix, remaining } => {
                report = prefix;
                retry_frontier = Some(remaining);
            }
        }
    }

    let mut queue: VecDeque<(PathBuf, usize)> = retry_frontier
        .unwrap_or_else(|| VecDeque::from(vec![(subtree.to_path_buf(), start_depth)]));
    #[cfg(target_os = "macos")]
    let mut bulk_reader = (config.worker_threads() > 1).then(macos_bulk::Reader::new);
    while let Some((rel_dir, depth)) = take_next(&mut queue, config.order) {
        let mut known = target.child_states(&rel_dir)?;
        let abs_dir = root.join(&rel_dir);
        let mut listing_complete = true;
        let process_entry = |name: OsString,
                             kind: EntryKind,
                             attrs: Attrs,
                             baseline: PathExpectation,
                             target: &mut ReconcileTarget<'_>,
                             queue: &mut VecDeque<(PathBuf, usize)>,
                             batch: &mut Vec<ObservationOp>,
                             sink: &mut dyn FnMut(&AppliedDelta),
                             report: &mut ReconcileReport|
         -> Result<()> {
            let rel_path = rel_dir.join(&name);
            report.scan.observe(kind, attrs);
            push_reconcile_upsert(
                target,
                &rel_path,
                kind,
                attrs,
                baseline,
                batch,
                &mut report.apply,
            );
            if batch.len() >= config.batch_size.max(1) {
                flush_reconcile_batch(target, batch, sink, &mut report.apply)?;
            }

            if should_descend(kind, attrs, depth, root_dev, config) {
                queue.push_back((rel_path, depth + 1));
            } else if kind.is_dir() {
                remove_known_children(target, &rel_path, config, batch, sink, &mut report.apply)?;
            }
            Ok(())
        };

        #[cfg(target_os = "macos")]
        let used_bulk =
            if let Some(entries) = bulk_reader.as_mut().and_then(|reader| reader.read(&abs_dir)) {
                report.scan.dirs_read += 1;
                for entry in entries {
                    if !admits(&entry.name, config) {
                        note_pruned_control_file(
                            &entry.name,
                            rel_dir.as_path(),
                            config,
                            &mut report.scan,
                        );
                        continue;
                    }
                    let baseline = match known.remove(&entry.name) {
                        Some(baseline) => baseline,
                        None => target.expectation(&rel_dir.join(&entry.name))?,
                    };
                    process_entry(
                        entry.name,
                        entry.kind,
                        entry.attrs,
                        baseline,
                        target,
                        &mut queue,
                        &mut batch,
                        sink,
                        &mut report,
                    )?;
                }
                true
            } else {
                false
            };
        #[cfg(not(target_os = "macos"))]
        let used_bulk = false;

        if !used_bulk {
            crate::counters::bump(|c| c.dir_opens += 1);
            let listing = match fs::read_dir(&abs_dir) {
                Ok(listing) => listing,
                Err(error) => {
                    report.scan.errors.push(Error::io(abs_dir, error));
                    continue;
                }
            };
            report.scan.dirs_read += 1;
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
                if !admits(&name, config) {
                    // Before `known.remove`, so an entry the rule now excludes stays in
                    // the missing set and is removed rather than silently retained.
                    note_pruned_control_file(&name, rel_dir.as_path(), config, &mut report.scan);
                    continue;
                }
                // Seeing the name proves it is not absent even if the following
                // metadata lookup fails. Remove it from the missing set before that
                // fallible lookup so an operational error cannot turn an existing
                // entry into a deletion.
                let baseline = match known.remove(&name) {
                    Some(baseline) => baseline,
                    None => target.expectation(&rel_dir.join(&name))?,
                };
                let meta = match metadata_for_fingerprint(&item) {
                    Ok(meta) => meta,
                    Err(error) => {
                        report.scan.errors.push(Error::io(item.path(), error));
                        continue;
                    }
                };
                process_entry(
                    name,
                    kind_from(&meta),
                    attrs_from(&meta),
                    baseline,
                    target,
                    &mut queue,
                    &mut batch,
                    sink,
                    &mut report,
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
    report.scan.errors.sort_by_cached_key(ToString::to_string);
    Ok(report)
}

#[derive(Debug, Default)]
struct DeferredReconcile {
    scan: ScanReport,
    unchanged: u64,
    operations: Vec<Op>,
    discovered: Vec<(PathBuf, usize, RegionId)>,
}

enum DirectParallelOutcome {
    Complete(ReconcileReport),
    RetrySerial { prefix: ReconcileReport, remaining: VecDeque<(PathBuf, usize)> },
}

/// Reconcile an exclusive full tree in bounded immutable-baseline waves.
///
/// No index write occurs while a wave's workers hold shared baseline references. That
/// lets each worker discard exact no-ops where they are observed instead of funnelling
/// every entry through one consumer. Effective changes still enter through ordinary
/// observations between waves, preserving both the index's sole mutation contract and
/// progressive delta delivery.
///
/// Unlike every other reconciliation path, the operations a wave defers are
/// unconditional: they carry no [`ObservationOp::if_state`] guard. Three properties
/// have to hold together for that to be safe, and a change to any one of them puts the
/// guards back. The target is an exclusive `&mut Index`, so no other producer can
/// commit between a worker's read and the wave's write. Nothing is applied while
/// workers run, so no baseline a worker compared against can go stale beneath it. And
/// a directory is only reconciled in a wave after the wave that discovered it has
/// committed, so a parent is never absent when its children arrive.
fn reconcile_direct_parallel(
    index: &mut Index,
    root: &Path,
    root_dev: u64,
    config: &ScanConfig,
    max_deferred_ops: usize,
    sink: &mut dyn FnMut(&AppliedDelta),
) -> Result<DirectParallelOutcome> {
    let mut frontier = DirectoryQueueState::seeded(
        (PathBuf::new(), 0),
        config.order,
        None,
        1,
        1,
        WorkerPolicyExperiment::ShippedOneShot,
    );
    let mut report = ReconcileReport::default();
    while !frontier.is_empty(config.order) {
        let mut wave = Vec::with_capacity(RECONCILE_WAVE_DIRECTORIES);
        while wave.len() < RECONCILE_WAVE_DIRECTORIES && !frontier.is_empty(config.order) {
            let remaining = RECONCILE_WAVE_DIRECTORIES - wave.len();
            frontier.take(remaining.min(DIR_CLAIM), config.order, &mut wave);
        }

        let next = std::sync::atomic::AtomicUsize::new(0);
        let deferred_count = std::sync::atomic::AtomicUsize::new(0);
        let overflowed = std::sync::atomic::AtomicBool::new(false);
        let workers = config.reconciliation_worker_threads().min(wave.len());
        let baseline: &Index = index;
        let results: Vec<DeferredReconcile> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..workers)
                .map(|_| {
                    scope.spawn(|| {
                        reconcile_wave_worker(
                            baseline,
                            root,
                            root_dev,
                            config,
                            &wave,
                            &next,
                            &deferred_count,
                            &overflowed,
                            max_deferred_ops,
                        )
                    })
                })
                .collect();

            handles
                .into_iter()
                .map(|handle| {
                    if let Ok(worker) = handle.join() {
                        return worker;
                    }
                    let mut worker = DeferredReconcile::default();
                    worker.scan.errors.push(Error::io(
                        root,
                        std::io::Error::other("a reconciliation worker thread panicked"),
                    ));
                    worker
                })
                .collect()
        });

        // Nothing from an overflowing wave was applied, so the ordinary incremental
        // reconciler can resume at that wave. Completed waves and their statistics are
        // retained exactly once; restarting from the root would count their unchanged
        // entries again and misreport the logical reconciliation pass.
        if overflowed.load(std::sync::atomic::Ordering::Relaxed) {
            let mut remaining: VecDeque<_> =
                wave.into_iter().map(|(path, depth, _region)| (path, depth)).collect();
            let mut deferred = Vec::with_capacity(DIR_CLAIM);
            while !frontier.is_empty(config.order) {
                frontier.take(DIR_CLAIM, config.order, &mut deferred);
                remaining.extend(deferred.drain(..).map(|(path, depth, _region)| (path, depth)));
            }
            return Ok(DirectParallelOutcome::RetrySerial { prefix: report, remaining });
        }

        let operation_count = deferred_count.load(std::sync::atomic::Ordering::Relaxed);
        let mut operations = Vec::with_capacity(operation_count);
        for worker in results {
            report.scan.absorb(worker.scan);
            report.apply.unchanged += worker.unchanged;
            operations.extend(worker.operations);
            for directory in worker.discovered {
                frontier.push(directory, config.order);
            }
        }
        apply_deferred_reconcile(index, &mut operations, config, sink, &mut report.apply)?;
    }
    report.scan.errors.sort_by_cached_key(ToString::to_string);
    Ok(DirectParallelOutcome::Complete(report))
}

fn apply_deferred_reconcile(
    index: &mut Index,
    operations: &mut Vec<Op>,
    config: &ScanConfig,
    sink: &mut dyn FnMut(&AppliedDelta),
    stats: &mut ApplyStats,
) -> Result<()> {
    // Parent upserts establish real directory attributes before children arrive.
    // Removals run deepest first so a parent removal never precedes an independently
    // observed descendant operation. The index supports out-of-order producers, but a
    // deterministic causal order also makes emitted deltas stable for callers.
    operations.sort_by(|left, right| {
        let left_remove = matches!(left, Op::Remove { .. });
        let right_remove = matches!(right, Op::Remove { .. });
        left_remove.cmp(&right_remove).then_with(|| {
            let left_depth = left.path().components().count();
            let right_depth = right.path().components().count();
            if left_remove {
                right_depth.cmp(&left_depth).then_with(|| left.path().cmp(right.path()))
            } else {
                left_depth.cmp(&right_depth).then_with(|| left.path().cmp(right.path()))
            }
        })
    });

    let batch_limit = config.batch_size.max(1);
    let mut batch = Vec::with_capacity(batch_limit.min(operations.len()));
    for operation in operations.drain(..) {
        batch.push(operation);
        if batch.len() >= batch_limit {
            flush_direct_reconcile_batch(index, &mut batch, sink, stats)?;
        }
    }
    flush_direct_reconcile_batch(index, &mut batch, sink, stats)
}

#[allow(clippy::too_many_arguments)]
fn reconcile_wave_worker(
    index: &Index,
    root: &Path,
    root_dev: u64,
    config: &ScanConfig,
    wave: &[(PathBuf, usize, RegionId)],
    next: &std::sync::atomic::AtomicUsize,
    deferred_count: &std::sync::atomic::AtomicUsize,
    overflowed: &std::sync::atomic::AtomicBool,
    max_deferred_ops: usize,
) -> DeferredReconcile {
    let _counter_guard = crate::counters::thread_flush_guard();
    let mut result = DeferredReconcile::default();
    #[cfg(target_os = "macos")]
    let mut bulk_reader = macos_bulk::Reader::new();

    loop {
        let start = next.fetch_add(DIR_CLAIM, std::sync::atomic::Ordering::Relaxed);
        if start >= wave.len() {
            break;
        }
        let end = start.saturating_add(DIR_CLAIM).min(wave.len());
        for (rel_dir, depth, region) in &wave[start..end] {
            let mut known = collect_child_expectations(index, rel_dir);
            let abs_dir = root.join(rel_dir);
            let mut listing_complete = true;
            // Collected beside the closure rather than through it: `process_entry` holds a
            // mutable borrow of `result.scan` for the whole block, and a control file the
            // rule pruned never reaches it anyway.
            let mut pruned_controls: Vec<PathBuf> = Vec::new();

            {
                let mut process_entry =
                    |name: OsString, kind: EntryKind, attrs: Attrs, baseline: PathExpectation| {
                        let rel_path = rel_dir.join(&name);
                        result.scan.entries += 1;
                        if kind == EntryKind::File {
                            result.scan.files_walked += 1;
                            result.scan.bytes_walked += attrs.size;
                        }
                        if baseline.state == (PathState::Present { kind, attrs }) {
                            result.unchanged += 1;
                        } else {
                            defer_reconcile_op(
                                Op::Upsert { path: rel_path.clone(), kind, attrs },
                                &mut result.operations,
                                deferred_count,
                                overflowed,
                                max_deferred_ops,
                            );
                        }

                        if should_descend(kind, attrs, *depth, root_dev, config) {
                            let child_region =
                                if *depth == 0 { RegionId::UNASSIGNED } else { *region };
                            result.discovered.push((rel_path, depth + 1, child_region));
                        } else if kind.is_dir() {
                            for name in collect_child_expectations(index, &rel_path).into_keys() {
                                defer_reconcile_op(
                                    Op::Remove { path: rel_path.join(name) },
                                    &mut result.operations,
                                    deferred_count,
                                    overflowed,
                                    max_deferred_ops,
                                );
                            }
                        }
                    };

                #[cfg(target_os = "macos")]
                let used_bulk = if let Some(entries) = bulk_reader.read(&abs_dir) {
                    result.scan.dirs_read += 1;
                    for entry in entries {
                        if !admits(&entry.name, config) {
                            if config.tags().is_control_file(Path::new(&entry.name)) {
                                pruned_controls.push(rel_dir.clone());
                            }
                            continue;
                        }
                        let baseline = known
                            .remove(&entry.name)
                            .unwrap_or_else(|| index.expectation(&rel_dir.join(&entry.name)));
                        process_entry(entry.name, entry.kind, entry.attrs, baseline);
                    }
                    true
                } else {
                    false
                };
                #[cfg(not(target_os = "macos"))]
                let used_bulk = false;

                if !used_bulk {
                    crate::counters::bump(|c| c.dir_opens += 1);
                    let listing = match fs::read_dir(&abs_dir) {
                        Ok(listing) => listing,
                        Err(error) => {
                            result.scan.errors.push(Error::io(abs_dir, error));
                            continue;
                        }
                    };
                    result.scan.dirs_read += 1;
                    for item in listing {
                        let item = match item {
                            Ok(item) => item,
                            Err(error) => {
                                listing_complete = false;
                                result.scan.errors.push(Error::io(&abs_dir, error));
                                continue;
                            }
                        };
                        let name = item.file_name();
                        if !admits(&name, config) {
                            if config.tags().is_control_file(Path::new(&name)) {
                                pruned_controls.push(rel_dir.clone());
                            }
                            continue;
                        }
                        // Match the serial path: an entry whose name was enumerated is
                        // not missing merely because its metadata could not be read.
                        let baseline = known
                            .remove(&name)
                            .unwrap_or_else(|| index.expectation(&rel_dir.join(&name)));
                        let meta = match metadata_for_fingerprint(&item) {
                            Ok(meta) => meta,
                            Err(error) => {
                                result.scan.errors.push(Error::io(item.path(), error));
                                continue;
                            }
                        };
                        process_entry(name, kind_from(&meta), attrs_from(&meta), baseline);
                    }
                }
            }
            result.scan.control_dirs.append(&mut pruned_controls);
            if listing_complete {
                for (name, _) in known {
                    defer_reconcile_op(
                        Op::Remove { path: rel_dir.join(name) },
                        &mut result.operations,
                        deferred_count,
                        overflowed,
                        max_deferred_ops,
                    );
                }
            }
        }
    }
    result
}

fn defer_reconcile_op(
    operation: Op,
    operations: &mut Vec<Op>,
    deferred_count: &std::sync::atomic::AtomicUsize,
    overflowed: &std::sync::atomic::AtomicBool,
    max_deferred_ops: usize,
) {
    let position = deferred_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if position < max_deferred_ops {
        operations.push(operation);
    } else {
        overflowed.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

fn flush_direct_reconcile_batch(
    index: &mut Index,
    batch: &mut Vec<Op>,
    sink: &mut dyn FnMut(&AppliedDelta),
    stats: &mut ApplyStats,
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }
    let outcome = index.apply(&Observation::new(std::mem::take(batch)))?;
    merge_apply_stats(stats, outcome.stats);
    if let Some(applied) = &outcome.applied {
        sink(applied);
    }
    Ok(())
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
    total.scan.files_walked += addition.scan.files_walked;
    total.scan.bytes_walked += addition.scan.bytes_walked;
    total.scan.errors.extend(addition.scan.errors);
    total.scan.control_dirs.extend(addition.scan.control_dirs);
    merge_apply_stats(&mut total.apply, addition.apply);
}

/// Whether one directory entry is inside the scan scope, by name alone.
///
/// The one place the admission rule is asked, called from every listing loop the engine
/// has -- the serial walk, the parallel walk, and both reconciliation paths. A rule spelled
/// out at each site is a rule that diverges at one of them, and a scan and a refresh
/// disagreeing about which entries exist reads as corruption rather than as a rule.
///
/// By name, before the `stat`. An entry outside scope costs one `bool` and nothing else:
/// no metadata read, no path join, no observation. That ordering is the point of pruning at
/// admission rather than filtering afterwards.
fn admits(name: &OsStr, config: &ScanConfig) -> bool {
    config.hidden().admits(name)
}

/// Note where the walk saw a governing control file it is not going to retain.
///
/// A `.gitignore` is hidden and governs entries that are not, so pruning it silently would
/// answer a gitignore question with "nothing is ignored" -- a wrong answer rather than a
/// missing one. Recording the directory is enough: binding reads the file from disk by
/// path, and the index never holds a row for it.
fn note_pruned_control_file(
    name: &OsStr,
    rel_dir: &Path,
    config: &ScanConfig,
    report: &mut ScanReport,
) {
    if config.tags().is_control_file(Path::new(name)) {
        report.control_dirs.push(rel_dir.to_path_buf());
    }
}

/// Whether a directory entry is inside the scan scope, so the walk should enter it.
///
/// Deliberately **not** where the file budget is asked. Reconciliation calls this too, and
/// reads a `false` for a directory as "out of scope, so remove what the index holds below
/// it" -- correct for a depth or filesystem bound, which are fixed for the life of an
/// index, and catastrophic for a bound that flips partway through a walk. A budget that
/// answered here would delete subtrees for the crime of being discovered late.
///
/// The budget is asked at the two discovery sites instead, where the only consequence of
/// `false` is that a directory is not enqueued.
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
    crate::counters::bump(|c| c.stats += 1);
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
                scope: Box::new(config.scope()),
            });
        }
        if !metadata.is_dir() {
            return Ok(prefix);
        }
        let attrs = attrs_from(&metadata);
        if config.one_filesystem && attrs.dev != root_dev && attrs.dev != 0 {
            return Err(Error::SubtreeOutsideScanScope {
                path: subtree.to_path_buf(),
                scope: Box::new(config.scope()),
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
pub(crate) fn attrs_from(meta: &fs::Metadata) -> Attrs {
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
pub(crate) fn attrs_from(meta: &fs::Metadata) -> Attrs {
    let mtime_ns = meta.modified().map_or(0, system_time_ns);
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

    /// A counter that silently reads zero is worse than a missing one, because a report
    /// full of zeroes invites the conclusion that the work did not happen.
    ///
    /// This has already gone wrong twice: once when the per-entry counter was added to
    /// the serial walk while the parallel walk went uninstrumented, and once when a
    /// clippy fix hoisted a `read_dir` out of a match scrutinee and took the counter
    /// with it. Both builds compiled, passed every other test, and reported zero. This
    /// asserts the relationships a real walk must satisfy, so the next such edit fails
    /// here instead of in a report someone believes.
    #[test]
    fn a_walk_moves_every_counter_it_should() {
        // Both walkers, because they are separate loops with separate call sites. The
        // first version of this test only exercised the parallel one, and deleting the
        // serial walker's counter still passed — a guard covering one path gives false
        // confidence about the other.
        let _serial = crate::counters::test_serial();
        crate::counters::enable(true);
        for threads in [Some(1), Some(4)] {
            let dir = sample_tree();
            let config = ScanConfig { threads, ..ScanConfig::default() };

            // Deltas around the scan, not absolute totals. The counters are
            // process-global, so a test running beside this one can add to them — and
            // `test_serial` cannot prevent that, since it only serializes tests that
            // take it, not every test that happens to walk a tree.
            let before = crate::counters::snapshot();
            let report = scan(dir.path(), &config, &mut |_| {}).expect("scan");
            crate::counters::flush_thread();
            let after = crate::counters::snapshot();
            let observed_entries = after.dir_entries - before.dir_entries;
            let observed_opens = after.dir_opens - before.dir_opens;
            let observed_stats = after.stats - before.stats;

            // `>=` rather than `==`, and the direction is the whole point: concurrent
            // work can only inflate these, never deflate them. So a counter that is too
            // low means a path ran uninstrumented, which is the failure worth catching
            // and the one that has actually happened — the macOS bulk reader reported
            // zero opens against three real ones. Equality would catch double-counting
            // too, and would be flaky for it.
            assert!(
                observed_entries >= report.entries,
                "every enumerated entry is counted at {threads:?}: {observed_entries} < {}",
                report.entries
            );
            assert!(
                observed_opens >= report.dirs_read,
                "every directory open is counted at {threads:?}: {observed_opens} < {}",
                report.dirs_read
            );
            assert!(
                observed_stats >= report.entries,
                "every entry is stated at {threads:?}: {observed_stats} < {}",
                report.entries
            );

            // Deliberately not asserted: `allocs` stays zero in a library test, because
            // allocation counting needs a binary to install `CountingAlloc` as its
            // global allocator and a test harness installs its own. The probe covers
            // that half; this covers the counters the library itself drives.
        }
        crate::counters::enable(false);
    }

    /// An automatic walk too short to fill its calibration window must say so.
    ///
    /// The failure this guards is quiet: such a walk runs on its initial pool, which is
    /// indistinguishable in the artifacts from a walk that measured the filesystem and
    /// chose to hold — unless the undecided case is recorded separately. Reading the
    /// first as the second is how a policy with no evidence behind it comes to look
    /// like a policy with evidence behind it.
    #[test]
    fn a_short_automatic_walk_records_an_undecided_policy() {
        let _serial = crate::counters::test_serial();
        let available = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        if automatic_worker_pool(available).calibration.is_none() {
            // A host reporting one processor has no reserve to unlock, so there is no
            // policy here to leave undecided.
            return;
        }

        crate::counters::enable(true);
        let dir = sample_tree();
        let config = ScanConfig { threads: None, ..ScanConfig::default() };
        let before = crate::counters::snapshot();
        scan(dir.path(), &config, &mut |_| {}).expect("scan");
        crate::counters::flush_thread();
        let after = crate::counters::snapshot();
        crate::counters::enable(false);

        // A strict increase, so a counter inflated by a test running beside this one
        // cannot turn the assertion into a false pass.
        assert!(
            after.adaptive_policy_undecided > before.adaptive_policy_undecided,
            "a three-file tree cannot fill a {ADAPTIVE_SCAN_CALIBRATION_ENTRIES}-entry window"
        );
    }

    #[test]
    fn diagnostics_make_a_fixed_pool_and_backend_choice_explicit() {
        let dir = sample_tree();
        let config = ScanConfig { threads: Some(1), ..ScanConfig::default() };

        let (report, diagnostics) =
            scan_with_diagnostics(dir.path(), &config, &mut |_| {}).expect("diagnostic scan");

        assert_eq!(diagnostics.schema, SCAN_DIAGNOSTICS_SCHEMA);
        assert_eq!(diagnostics.worker_policy.outcome, WorkerPolicyOutcome::Fixed);
        assert_eq!(diagnostics.worker_policy.initial_workers, 1);
        assert_eq!(diagnostics.worker_policy.maximum_workers, 1);
        assert_eq!(diagnostics.worker_policy.peak_active_workers, 1);
        assert!(diagnostics.worker_policy.windows.is_empty());
        assert!(!diagnostics.worker_policy.events_truncated);
        assert_eq!(diagnostics.worker_policy.ready_directories_at_finish, 0);
        assert_eq!(diagnostics.worker_policy.in_flight_directories_at_finish, 0);
        assert_eq!(diagnostics.backend.portable_directory_reads, report.dirs_read);

        #[cfg(target_os = "macos")]
        {
            assert_eq!(diagnostics.backend.macos_bulk_attempts, Some(0));
            assert_eq!(diagnostics.backend.macos_bulk_successes, Some(0));
            assert_eq!(diagnostics.backend.macos_bulk_fallbacks, Some(0));
            assert!(diagnostics.backend.unavailable_reason.is_none());
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert_eq!(diagnostics.backend.macos_bulk_attempts, None);
            assert_eq!(diagnostics.backend.macos_bulk_successes, None);
            assert_eq!(diagnostics.backend.macos_bulk_fallbacks, None);
            assert_eq!(
                diagnostics.backend.unavailable_reason,
                Some("macOS bulk directory enumeration is unavailable on this platform")
            );
        }
    }

    #[test]
    fn diagnostics_fail_closed_when_an_automatic_window_is_incomplete() {
        let available = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        let pool = automatic_worker_pool(available);
        if pool.calibration.is_none() {
            return;
        }
        let dir = sample_tree();
        let config = ScanConfig { threads: None, ..ScanConfig::default() };

        let (report, diagnostics) =
            scan_with_diagnostics(dir.path(), &config, &mut |_| {}).expect("diagnostic scan");

        assert_eq!(diagnostics.worker_policy.outcome, WorkerPolicyOutcome::Undecided);
        assert_eq!(diagnostics.worker_policy.available_parallelism, available);
        assert_eq!(diagnostics.worker_policy.initial_workers, pool.initial);
        assert_eq!(diagnostics.worker_policy.maximum_workers, pool.maximum);
        assert_eq!(
            diagnostics.worker_policy.calibration_window_entries,
            Some(ADAPTIVE_SCAN_CALIBRATION_ENTRIES)
        );
        assert_eq!(
            diagnostics.worker_policy.slow_threshold_ns_per_entry,
            Some(ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY)
        );
        assert_eq!(diagnostics.worker_policy.windows.len(), 1);
        let window = &diagnostics.worker_policy.windows[0];
        assert_eq!(window.sequence, 0);
        assert_eq!(window.start_entry_ordinal, 0);
        assert_eq!(window.end_entry_ordinal, report.entries);
        assert_eq!(window.observed_entries, report.entries);
        assert_eq!(window.decision, WorkerPolicyDecision::Undecided);
        assert!(window.end_entry_ordinal < ADAPTIVE_SCAN_CALIBRATION_ENTRIES);
        assert!(window.active_workers <= diagnostics.worker_policy.peak_active_workers);
        assert_eq!(diagnostics.worker_policy.ready_directories_at_finish, 0);
        assert_eq!(diagnostics.worker_policy.in_flight_directories_at_finish, 0);
        assert!(diagnostics.worker_policy.handoff_backlog_high_water >= 1);
    }

    #[test]
    fn diagnostic_trace_is_bounded_and_marks_truncation() {
        let recorder = ScanDiagnosticsRecorder::new(
            WorkerPool::fixed(2),
            2,
            WorkerPolicyExperiment::ShippedOneShot,
        );
        for sequence in 0..=MAX_POLICY_TRACE_EVENTS {
            recorder.record_policy_window(PolicyWindowSnapshot {
                sequence: sequence as u64,
                start_entry_ordinal: sequence as u64,
                end_entry_ordinal: sequence as u64 + 1,
                observed_entries: 1,
                observed_chunks: 1,
                observed_work_ns: 10,
                ready_directories: 1,
                in_flight_directories: 1,
                active_workers: 1,
                handoff_backlog: 0,
                requested_workers: None,
                decision: WorkerPolicyDecision::Hold,
            });
        }

        let diagnostics = recorder.finish();
        assert_eq!(diagnostics.worker_policy.windows.len(), MAX_POLICY_TRACE_EVENTS);
        assert!(diagnostics.worker_policy.events_truncated);
    }

    #[test]
    fn diagnostic_trace_preserves_queue_order_when_recorders_arrive_out_of_order() {
        let pool =
            WorkerPool { initial: 2, maximum: 4, calibration: Some(WorkerCalibration::new(1, 1)) };
        let recorder =
            ScanDiagnosticsRecorder::new(pool, 2, WorkerPolicyExperiment::RepeatedWindows);
        let snapshot = |sequence, decision| PolicyWindowSnapshot {
            sequence,
            start_entry_ordinal: sequence,
            end_entry_ordinal: sequence + 1,
            observed_entries: 1,
            observed_chunks: 1,
            observed_work_ns: 1,
            ready_directories: 0,
            in_flight_directories: 0,
            active_workers: 1,
            handoff_backlog: 0,
            requested_workers: None,
            decision,
        };

        recorder.record_policy_window(snapshot(1, WorkerPolicyDecision::HoldNoUsefulWork));
        recorder.record_policy_window(snapshot(0, WorkerPolicyDecision::Hold));

        let diagnostics = recorder.finish();
        assert_eq!(
            diagnostics
                .worker_policy
                .windows
                .iter()
                .map(|window| window.sequence)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(diagnostics.worker_policy.outcome, WorkerPolicyOutcome::HeldNoUsefulWork);
    }

    #[test]
    fn diagnostic_policy_aggregates_cross_check_runtime_counters() {
        let _serial = crate::counters::test_serial();
        let pool = WorkerPool {
            initial: 2,
            maximum: 4,
            calibration: Some(WorkerCalibration::new(17, 100)),
        };
        let recorder =
            ScanDiagnosticsRecorder::new(pool, 2, WorkerPolicyExperiment::ShippedOneShot);

        crate::counters::enable(true);
        let before = crate::counters::snapshot();
        record_adaptive_calibration_chunk(Some(&recorder), 17, 2_100);
        record_adaptive_worker_expansion(Some(&recorder));
        crate::counters::flush_thread();
        let after = crate::counters::snapshot();
        crate::counters::enable(false);

        recorder.record_policy_window(PolicyWindowSnapshot {
            sequence: 0,
            start_entry_ordinal: 0,
            end_entry_ordinal: 17,
            observed_entries: 17,
            observed_chunks: 1,
            observed_work_ns: 2_100,
            ready_directories: 2,
            in_flight_directories: 2,
            active_workers: 2,
            handoff_backlog: 0,
            requested_workers: Some(4),
            decision: WorkerPolicyDecision::ScaleUp,
        });
        let diagnostics = recorder.finish();
        let policy = diagnostics.worker_policy;
        assert_eq!(policy.calibration_chunks, 1);
        assert_eq!(policy.calibration_entries, 17);
        assert_eq!(policy.calibration_work_ns, 2_100);
        assert_eq!(policy.worker_expansions, 1);
        assert_eq!(policy.windows[0].observed_chunks, policy.calibration_chunks);
        assert_eq!(policy.windows[0].observed_entries, policy.calibration_entries);
        assert_eq!(policy.windows[0].observed_work_ns, policy.calibration_work_ns);

        // Other tests can record while this process-global interval is enabled, so the
        // counter delta may be larger but must never be smaller than this run-scoped
        // trace. The shared helpers above make the two observations one event.
        assert!(
            after.adaptive_calibration_chunks - before.adaptive_calibration_chunks
                >= policy.calibration_chunks
        );
        assert!(
            after.adaptive_calibration_entries - before.adaptive_calibration_entries
                >= policy.calibration_entries
        );
        assert!(
            after.adaptive_calibration_work_us - before.adaptive_calibration_work_us
                >= policy.calibration_work_ns / 1_000
        );
        assert!(after.adaptive_scale_ups - before.adaptive_scale_ups >= policy.worker_expansions);
    }

    #[test]
    fn diagnostic_index_scan_preserves_the_regular_result() {
        let dir = branching_tree();
        let config = ScanConfig { threads: Some(4), ..ScanConfig::default() };
        let (plain, plain_report) = scan_into_index(dir.path(), &config).expect("plain scan");
        let (diagnostic, diagnostic_report, diagnostics) =
            scan_into_index_with_diagnostics(dir.path(), &config).expect("diagnostic scan");

        assert_eq!(index_fingerprint(&plain), index_fingerprint(&diagnostic));
        assert_eq!(plain_report.entries, diagnostic_report.entries);
        assert_eq!(diagnostics.worker_policy.outcome, WorkerPolicyOutcome::Fixed);
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

    /// A small tree whose mutation crosses every structural reconciliation boundary.
    fn reconciliation_transition_tree() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(&dir.path().join("changed.txt"), b"before");
        write_file(&dir.path().join("removed.txt"), b"remove me");
        write_file(&dir.path().join("directory-to-file/old.rs"), b"old child");
        write_file(&dir.path().join("file-to-directory"), b"old file");
        write_file(&dir.path().join("removed-tree/nested/gone.md"), b"gone");
        write_file(&dir.path().join("stable/deep/kept.rs"), b"kept");
        dir
    }

    fn mutate_reconciliation_transition_tree(root: &Path) {
        write_file(&root.join("changed.txt"), b"after, with a distinct size");
        fs::remove_file(root.join("removed.txt")).expect("remove root file");

        fs::remove_dir_all(root.join("directory-to-file")).expect("remove old directory");
        write_file(&root.join("directory-to-file"), b"replacement file");

        fs::remove_file(root.join("file-to-directory")).expect("remove old file");
        write_file(&root.join("file-to-directory/new.txt"), b"replacement child");

        fs::remove_dir_all(root.join("removed-tree")).expect("remove nested tree");
        write_file(&root.join("added-tree/nested/new.md"), b"new nested file");
    }

    fn effective_ops(deltas: &[AppliedDelta]) -> Vec<Op> {
        let mut operations: Vec<_> =
            deltas.iter().flat_map(|delta| delta.ops.iter().cloned()).collect();
        operations.sort_by(|left, right| left.path().cmp(right.path()));
        operations
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
            assert_eq!(report.files_walked, serial_report.files_walked, "{threads} threads");
            assert_eq!(report.bytes_walked, serial_report.bytes_walked, "{threads} threads");
            // Public roll-ups carry extension names even though the internal merge path
            // uses ids whose assignment order differs between serial and parallel walks.
            let (serial_total, parallel_total) = (serial.total(), parallel.total());
            assert_eq!(
                (
                    parallel_total.files,
                    parallel_total.dirs,
                    parallel_total.bytes,
                    parallel_total.allocated,
                    parallel_total.newest_mtime_ns,
                ),
                (
                    serial_total.files,
                    serial_total.dirs,
                    serial_total.bytes,
                    serial_total.allocated,
                    serial_total.newest_mtime_ns,
                ),
                "{threads} threads roll-up"
            );
            assert_eq!(
                parallel_total.by_ext, serial_total.by_ext,
                "{threads} threads per-extension roll-up"
            );
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
    fn scan_order_never_changes_the_resulting_index() {
        let dir = branching_tree();
        let depth_first =
            ScanConfig { order: ScanOrder::DepthFirst, threads: Some(1), ..ScanConfig::default() };
        let (expected, expected_report) =
            scan_into_index(dir.path(), &depth_first).expect("depth-first scan");

        for (order, threads) in
            [(ScanOrder::BreadthFirst, 1), (ScanOrder::BreadthFirst, 4), (ScanOrder::DepthFirst, 4)]
        {
            let config = ScanConfig { order, threads: Some(threads), ..ScanConfig::default() };
            let (index, report) = scan_into_index(dir.path(), &config).expect("scan");
            assert_eq!(report.entries, expected_report.entries, "{order:?}/{threads}");
            assert_eq!(report.dirs_read, expected_report.dirs_read, "{order:?}/{threads}");
            // Public roll-ups resolve internal ids, so their named maps are stable even
            // when traversal order changes id assignment.
            let (totals, expected_totals) = (index.total(), expected.total());
            assert_eq!(
                (totals.files, totals.dirs, totals.bytes, totals.allocated),
                (
                    expected_totals.files,
                    expected_totals.dirs,
                    expected_totals.bytes,
                    expected_totals.allocated
                ),
                "{order:?}/{threads} roll-up"
            );
            assert_eq!(
                totals.newest_mtime_ns, expected_totals.newest_mtime_ns,
                "{order:?}/{threads} newest mtime"
            );
            assert_eq!(
                totals.by_ext, expected_totals.by_ext,
                "{order:?}/{threads} extension tallies"
            );
            assert_eq!(
                index_fingerprint(&index),
                index_fingerprint(&expected),
                "{order:?}/{threads} produced a different index"
            );
        }
    }

    #[test]
    fn a_single_worker_breadth_first_walk_is_strictly_level_ordered() {
        // The strict guarantee, which holds only with one worker. With several, the
        // queue is ordered but the claims are not: a fast worker can enqueue and claim
        // depth d+2 while a slow worker still holds depth d+1. See
        // `breadth_first_starts_every_top_level_subtree_early` for the property the
        // default configuration actually provides, which is the one consumers rely on.
        let dir = branching_tree();
        let config = ScanConfig {
            order: ScanOrder::BreadthFirst,
            threads: Some(1),
            batch_size: 1,
            ..ScanConfig::default()
        };
        let mut depths_in_order: Vec<usize> = Vec::new();
        scan(dir.path(), &config, &mut |observation| {
            for op in &observation.ops {
                if let Op::Upsert { path, kind, .. } = &op.op
                    && kind.is_dir()
                {
                    depths_in_order.push(path.components().count());
                }
            }
        })
        .expect("scan");

        assert!(depths_in_order.len() > 10, "fixture should have many directories");
        assert!(
            depths_in_order.windows(2).all(|pair| pair[0] <= pair[1]),
            "directory depths were not non-decreasing: {depths_in_order:?}"
        );
    }

    /// How many of the fixture's twelve top-level subtrees have received any file by
    /// the time half the files have been emitted.
    ///
    /// This is the product metric — "is a mid-scan ranking meaningful?" — rather than
    /// first-touch, which cannot distinguish the orders at all: reading the root
    /// enumerates all twelve children at once either way. What a ranking needs is that
    /// the subtrees grow *together*.
    fn subtrees_started_at_halfway(order: ScanOrder, threads: usize, dir: &Path) -> usize {
        let config =
            ScanConfig { order, batch_size: 1, threads: Some(threads), ..ScanConfig::default() };

        let mut files: Vec<PathBuf> = Vec::new();
        scan(dir, &config, &mut |observation| {
            for op in &observation.ops {
                if let Op::Upsert { path, kind, .. } = &op.op
                    && !kind.is_dir()
                {
                    files.push(path.clone());
                }
            }
        })
        .expect("scan");

        let halfway = files.len() / 2;
        let mut started: BTreeSet<PathBuf> = BTreeSet::new();
        for path in files.iter().take(halfway) {
            if let Some(top) = path.components().next() {
                started.insert(PathBuf::from(top.as_os_str()));
            }
        }
        started.len()
    }

    #[test]
    fn a_parallel_walk_accounts_for_where_its_time_went() {
        // The attribution identity: every named cause is a disjoint slice of worker
        // wall time, so the parts can never exceed the whole, and the counters that
        // amortization depends on are actually incremented. This is the instrument
        // the scheduler experiments will read; if it drifts, they measure noise.
        let dir = branching_tree();
        let config = ScanConfig { threads: Some(4), batch_size: 64, ..ScanConfig::default() };
        let report = scan(dir.path(), &config, &mut |_| {}).expect("scan");
        let a = report.attribution;

        assert!(a.claims > 0, "a parallel walk claims chunks: {a:?}");
        assert!(a.work_ns > 0, "reading directories takes time: {a:?}");
        assert!(a.wall_ns > 0);
        // claim() locks at least once per successful claim, and release() locks once
        // per claim cycle too.
        assert!(a.lock_ops >= a.claims * 2, "lock ops out of step with claims: {a:?}");
        assert!(
            a.accounted_ns() <= a.wall_ns,
            "attributed slices are disjoint intervals inside worker wall: {a:?}"
        );
    }

    #[test]
    fn a_serial_walk_has_no_coordination_to_attribute() {
        // Serial semantics: wall is the loop, "send" is the inline sink (the consumer
        // actually running), work is the rest — and the coordination counters stay
        // zero because there is no queue lock and no channel.
        let dir = branching_tree();
        let config = ScanConfig { threads: Some(1), batch_size: 64, ..ScanConfig::default() };
        let mut observations = 0usize;
        let report = scan(dir.path(), &config, &mut |_| observations += 1).expect("scan");
        let a = report.attribution;

        assert!(observations > 0, "the sink ran, so send_ns measured something real");
        assert!(a.work_ns > 0 && a.wall_ns >= a.work_ns);
        assert_eq!(
            (a.claims, a.lock_ops, a.lock_contended, a.starved_ns, a.lock_wait_ns),
            (0, 0, 0, 0, 0),
            "no queue, no lock, nothing to wait on: {a:?}"
        );
    }

    /// Twelve top-level subtrees, each a branching tree several levels deep.
    ///
    /// Branching matters: an earlier fixture gave every level exactly one child, which
    /// pinned the frontier at twelve directories and made both orders behave
    /// identically — a LIFO cannot dive when there is nothing to dive into. With two
    /// children per level, depth-first pushes siblings and immediately descends into
    /// the last one, which is the behaviour that leaves other subtrees behind.
    ///
    /// It is also deliberately uniform. A version using one deep spur beside shallow
    /// siblings made the result depend on whether `readdir` returned the spur early:
    /// it passed on APFS and failed on ext4.
    fn deep_forest() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        for top in 0..12 {
            let mut level: Vec<PathBuf> = vec![dir.path().join(format!("t{top}"))];
            for _ in 0..5 {
                let mut next = Vec::new();
                for parent in &level {
                    for child in 0..2 {
                        let path = parent.join(format!("c{child}"));
                        for file in 0..3 {
                            write_file(&path.join(format!("f{file}.dat")), b"xxxxxxxxxx");
                        }
                        next.push(path);
                    }
                }
                level = next;
            }
        }
        dir
    }

    /// Files accumulated by the *least advanced* top-level subtree in the first
    /// quarter of the walk.
    ///
    /// Counting subtrees merely *started* cannot discriminate on a tree whose root
    /// fans out twelve ways: every scheduler touches all twelve immediately, because
    /// reading the root enumerates them. What differs is whether they then advance
    /// together, so the question is how far behind the laggard is.
    fn leanest_subtree_early(order: ScanOrder, threads: usize, dir: &Path) -> usize {
        let config =
            ScanConfig { order, batch_size: 1, threads: Some(threads), ..ScanConfig::default() };
        let mut files: Vec<PathBuf> = Vec::new();
        scan(dir, &config, &mut |observation| {
            for op in &observation.ops {
                if let Op::Upsert { path, kind, .. } = &op.op
                    && !kind.is_dir()
                {
                    files.push(path.clone());
                }
            }
        })
        .expect("scan");

        let quarter = files.len() / 4;
        let mut per_top: BTreeMap<PathBuf, usize> = BTreeMap::new();
        for path in files.iter().take(quarter) {
            if let Some(top) = path.components().next() {
                *per_top.entry(PathBuf::from(top.as_os_str())).or_default() += 1;
            }
        }
        (0..12)
            .map(|top| per_top.get(&PathBuf::from(format!("t{top}"))).copied().unwrap_or(0))
            .min()
            .unwrap_or(0)
    }

    #[test]
    fn deep_subtrees_do_not_delay_their_siblings() {
        // The orientation property, and the reason breadth-first is the default: when
        // every top-level subtree is deep, depth-first pours its early effort down
        // whichever ones it picked up and leaves the rest at zero, while the region
        // scheduler advances all twelve together. A user watching the top level fill
        // in sees a meaningful ranking in the first case and a misleading one in the
        // second.
        //
        // Asserted at one worker only, and that bound is deliberate. This metric reads
        // *emission* order, and under several workers emission reflects which worker
        // finished first as much as which region was claimed — so it varies with core
        // count. Measured on a six-core machine the margin is wide (33-37 files against
        // 6); on a CI runner with fewer cores both orders can report zero. That makes it
        // a benchmark-grade observation, recorded in exp-013, not a unit-test assertion.
        //
        // The scheduling property itself *is* asserted deterministically, against the
        // queue rather than through a walk, by
        // `the_region_scheduler_spreads_workers_over_distinct_subtrees`.
        let dir = deep_forest();
        let breadth = leanest_subtree_early(ScanOrder::BreadthFirst, 1, dir.path());
        let depth = leanest_subtree_early(ScanOrder::DepthFirst, 1, dir.path());
        assert!(
            breadth > depth,
            "breadth-first should leave its least advanced top-level subtree further \
             along: {breadth} files against {depth}"
        );
    }

    #[test]
    fn the_region_scheduler_spreads_workers_over_distinct_subtrees() {
        // The scheduler invariant, checked directly on the queue rather than through a
        // walk: consecutive claims by *different* workers must land in different
        // regions while several regions have work. This is what the round-robin ready
        // ring buys, and it is the thing a global FIFO could not promise.
        let queue = DirectoryQueue::new((PathBuf::new(), 0), ScanOrder::BreadthFirst, None, None);
        let mut timing = WalkAttribution::default();

        // Bootstrap: drain the root, then seed four top-level regions.
        let mut claimed = Vec::new();
        let root = queue.claim(&mut claimed, &mut timing).expect("root is claimable");
        claimed.clear();
        queue.extend(
            (0..4).map(|top| (PathBuf::from(format!("t{top}")), 1, RegionId::UNASSIGNED)),
            &mut timing,
        );
        assert!(root.release(0, 0, &mut timing).is_none());

        // Four workers with no affinity must each be handed a different region. The
        // claims are held for the whole loop, as four concurrent workers would hold
        // them, because releasing between them would let one worker take every region.
        let mut regions = BTreeSet::new();
        let mut held = Vec::new();
        for _ in 0..4 {
            let mut claimed = Vec::new();
            held.push(queue.claim(&mut claimed, &mut timing).expect("a region has work"));
            regions.insert(claimed[0].2.0);
            assert_eq!(claimed.len(), 1, "one directory per region so far");
        }
        assert_eq!(regions.len(), 4, "each claim took a distinct region: {regions:?}");
    }

    #[test]
    fn breadth_first_spreads_early_work_across_top_level_subtrees() {
        // The justification for making breadth-first the default: at the halfway point
        // more of the tree's top-level subtrees have started filling, so a consumer
        // ranking by size mid-scan is comparing partial values rather than a mix of
        // final values and zeros.
        //
        // Pinned with one worker, where the ordering guarantee is strict and the result
        // is deterministic. The multi-worker case is deliberately NOT asserted here:
        // measured on this fixture the advantage disappears under the default worker
        // count (both orders start 7-8 subtrees, run to run), because emission order is
        // then dominated by worker scheduling rather than by queue order. That is a
        // real limitation of the current design, recorded in the plan and tracked
        // rather than papered over with a test tuned until it passed.
        let dir = branching_tree();
        let breadth = subtrees_started_at_halfway(ScanOrder::BreadthFirst, 1, dir.path());
        let depth = subtrees_started_at_halfway(ScanOrder::DepthFirst, 1, dir.path());

        assert!(
            breadth > depth,
            "breadth-first should have more top-level subtrees underway at the halfway \
             point, but started {breadth} against depth-first's {depth}"
        );
    }

    #[test]
    fn scan_order_does_not_change_the_cache_scope() {
        // Order is operational, like the worker count: it changes when observations
        // appear, never which ones, so it must not be able to invalidate a snapshot.
        let breadth = ScanConfig { order: ScanOrder::BreadthFirst, ..ScanConfig::default() };
        let depth = ScanConfig { order: ScanOrder::DepthFirst, ..ScanConfig::default() };
        assert_eq!(breadth.scope(), depth.scope());
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
    fn automatic_worker_pool_keeps_a_conservative_start_and_bounded_reserve() {
        assert_eq!(automatic_worker_pool(1), WorkerPool::fixed(1));
        assert_eq!(
            automatic_worker_pool(4),
            WorkerPool {
                initial: 4,
                maximum: 8,
                calibration: Some(WorkerCalibration::new(
                    ADAPTIVE_SCAN_CALIBRATION_ENTRIES,
                    ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY,
                )),
            }
        );
        assert_eq!(
            automatic_worker_pool(10),
            WorkerPool {
                initial: DEFAULT_SCAN_THREADS_CAP,
                maximum: ADAPTIVE_SCAN_THREADS_CAP,
                calibration: Some(WorkerCalibration::new(
                    ADAPTIVE_SCAN_CALIBRATION_ENTRIES,
                    ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY,
                )),
            }
        );
    }

    #[test]
    fn an_abandoned_claim_does_not_strand_the_other_workers() {
        // The liveness property behind `DirectoryClaim`. A worker that stops mid-chunk
        // — consumer gone, or a panic unwinding through the directory read — still owes
        // the queue its claim, and `claim` parks everyone else until `outstanding`
        // reaches zero. Before the claim was an RAII guard both of those exits skipped
        // the release, and every remaining worker waited on the condvar forever while
        // the scoped join waited on them.
        let queue = std::sync::Arc::new(DirectoryQueue::new(
            (PathBuf::new(), 0),
            ScanOrder::BreadthFirst,
            None,
            None,
        ));
        let mut timing = WalkAttribution::default();
        let mut claimed = Vec::new();

        // One worker takes the root and abandons it without publishing anything.
        drop(queue.claim(&mut claimed, &mut timing).expect("root is claimable"));

        // A second worker must now be told the walk is over rather than parking.
        let waiter = queue.clone();
        let (done, finished) = std::sync::mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let mut timing = WalkAttribution::default();
            let mut claimed = Vec::new();
            let outcome = waiter.claim(&mut claimed, &mut timing).is_some();
            done.send(outcome).expect("publish the claim outcome");
        });

        assert_eq!(
            finished.recv_timeout(std::time::Duration::from_secs(5)),
            Ok(false),
            "the queue must report the walk finished instead of parking the worker"
        );
    }

    #[test]
    fn automatic_queue_activates_its_reserve_only_for_slow_initial_work() {
        let slow = WorkerCalibration::new(3, 10);
        let queue =
            DirectoryQueue::new((PathBuf::new(), 0), ScanOrder::BreadthFirst, Some(slow), None);
        let mut timing = WalkAttribution::default();
        let mut claimed = Vec::new();

        let claim = queue.claim(&mut claimed, &mut timing).expect("root is claimable");
        queue.extend([(PathBuf::from("child"), 1, RegionId::UNASSIGNED)].into_iter(), &mut timing);
        assert!(claim.release(2, 20, &mut timing).is_none());

        claimed.clear();
        let claim = queue.claim(&mut claimed, &mut timing).expect("child is claimable");
        queue.extend([(PathBuf::from("grandchild"), 2, claimed[0].2)].into_iter(), &mut timing);
        assert_eq!(claim.release(1, 10, &mut timing), Some(2));

        let fast = WorkerCalibration::new(3, 11);
        let queue =
            DirectoryQueue::new((PathBuf::new(), 0), ScanOrder::BreadthFirst, Some(fast), None);
        let mut timing = WalkAttribution::default();
        let mut claimed = Vec::new();
        let claim = queue.claim(&mut claimed, &mut timing).expect("root is claimable");
        assert!(claim.release(3, 30, &mut timing).is_none());
        assert!(queue.lock().controller.is_none(), "calibration decides only once");
    }

    #[test]
    fn repeated_windows_reconsider_a_late_slow_phase_in_entry_order() {
        let calibration = WorkerCalibration::new(4, 10);
        let mut controller =
            WorkerController::new(calibration, WorkerPolicyExperiment::RepeatedWindows);

        let fast = controller.observe(4, 20).expect("first complete window");
        let slow = controller.observe(4, 80).expect("second complete window");

        assert!(!fast.slow);
        assert!(slow.slow);
        assert_eq!((fast.start_entry_ordinal, fast.end_entry_ordinal), (0, 4));
        assert_eq!((slow.start_entry_ordinal, slow.end_entry_ordinal), (4, 8));
    }

    #[test]
    fn shipped_trace_retains_post_decision_windows_without_changing_policy() {
        let calibration = WorkerCalibration::new(2, 10);
        let pool = WorkerPool { initial: 2, maximum: 4, calibration: Some(calibration) };
        let recorder =
            ScanDiagnosticsRecorder::new(pool, 2, WorkerPolicyExperiment::ShippedOneShot);
        let queue = DirectoryQueue::new_with_policy(
            (PathBuf::new(), 0),
            ScanOrder::BreadthFirst,
            Some(calibration),
            Some(recorder.clone()),
            pool.initial,
            pool.maximum,
            WorkerPolicyExperiment::ShippedOneShot,
        );
        let mut timing = WalkAttribution::default();
        let mut claimed = Vec::new();

        let first = queue.claim(&mut claimed, &mut timing).expect("first window");
        queue.extend([(PathBuf::from("late"), 1, RegionId::UNASSIGNED)].into_iter(), &mut timing);
        assert_eq!(first.release(2, 10, &mut timing), None, "fast prefix holds");

        claimed.clear();
        let late = queue.claim(&mut claimed, &mut timing).expect("late phase");
        queue.extend([(PathBuf::from("tail"), 2, RegionId::UNASSIGNED)].into_iter(), &mut timing);
        assert_eq!(late.release(2, 40, &mut timing), None, "shadow cannot scale");

        let diagnostics = recorder.finish();
        assert_eq!(diagnostics.worker_policy.outcome, WorkerPolicyOutcome::Held);
        assert_eq!(
            diagnostics
                .worker_policy
                .windows
                .iter()
                .map(|window| window.decision)
                .collect::<Vec<_>>(),
            vec![WorkerPolicyDecision::Hold, WorkerPolicyDecision::ObserveSlow]
        );
    }

    #[test]
    fn staged_controller_requires_a_useful_frontier_then_stays_bounded() {
        let calibration = WorkerCalibration::new(1, 10);
        let pool = WorkerPool { initial: 2, maximum: 8, calibration: Some(calibration) };
        let recorder =
            ScanDiagnosticsRecorder::new(pool, 4, WorkerPolicyExperiment::StagedGatedWindows);
        let queue = DirectoryQueue::new_with_policy(
            (PathBuf::new(), 0),
            ScanOrder::BreadthFirst,
            Some(calibration),
            Some(recorder.clone()),
            pool.initial,
            pool.maximum,
            WorkerPolicyExperiment::StagedGatedWindows,
        );
        let mut timing = WalkAttribution::default();
        let mut claimed = Vec::new();

        let root = queue.claim(&mut claimed, &mut timing).expect("root");
        queue.extend([(PathBuf::from("narrow"), 1, RegionId::UNASSIGNED)].into_iter(), &mut timing);
        assert_eq!(root.release(1, 20, &mut timing), None);

        claimed.clear();
        let narrow = queue.claim(&mut claimed, &mut timing).expect("narrow child");
        queue.extend(
            (0..9).map(|index| (PathBuf::from(format!("wide-{index}")), 2, RegionId::UNASSIGNED)),
            &mut timing,
        );
        assert_eq!(narrow.release(1, 20, &mut timing), Some(4));

        claimed.clear();
        let wide = queue.claim(&mut claimed, &mut timing).expect("wide claim");
        queue.extend(
            (0..9).map(|index| (PathBuf::from(format!("wider-{index}")), 3, RegionId::UNASSIGNED)),
            &mut timing,
        );
        assert_eq!(wide.release(1, 20, &mut timing), Some(8));

        let diagnostics = recorder.finish();
        let decisions: Vec<_> = diagnostics
            .worker_policy
            .windows
            .iter()
            .map(|window| (window.decision, window.requested_workers))
            .collect();
        assert_eq!(
            decisions,
            vec![
                (WorkerPolicyDecision::HoldInsufficientFrontier, None),
                (WorkerPolicyDecision::ScaleUp, Some(4)),
                (WorkerPolicyDecision::ScaleUp, Some(8)),
            ]
        );
        assert!(
            diagnostics
                .worker_policy
                .windows
                .windows(2)
                .all(|pair| pair[0].end_entry_ordinal <= pair[1].start_entry_ordinal)
        );
        assert!(diagnostics.worker_policy.windows.iter().all(|window| {
            window.requested_workers.is_none_or(|workers| workers <= pool.maximum)
        }));
    }

    #[test]
    fn staged_controller_does_not_add_producers_to_a_delayed_handoff() {
        let calibration = WorkerCalibration::new(1, 10);
        let pool = WorkerPool { initial: 2, maximum: 8, calibration: Some(calibration) };
        let recorder =
            ScanDiagnosticsRecorder::new(pool, 4, WorkerPolicyExperiment::StagedGatedWindows);
        recorder.handoff_sent();
        recorder.handoff_sent();
        let queue = DirectoryQueue::new_with_policy(
            (PathBuf::new(), 0),
            ScanOrder::BreadthFirst,
            Some(calibration),
            Some(recorder.clone()),
            pool.initial,
            pool.maximum,
            WorkerPolicyExperiment::StagedGatedWindows,
        );
        let mut timing = WalkAttribution::default();
        let mut claimed = Vec::new();
        let claim = queue.claim(&mut claimed, &mut timing).expect("root");
        queue.extend(
            (0..9).map(|index| (PathBuf::from(format!("ready-{index}")), 1, RegionId::UNASSIGNED)),
            &mut timing,
        );

        assert_eq!(claim.release(1, 20, &mut timing), None);
        recorder.handoff_received();
        recorder.handoff_received();
        let diagnostics = recorder.finish();
        assert_eq!(
            diagnostics.worker_policy.windows[0].decision,
            WorkerPolicyDecision::HoldHandoffBacklog
        );
    }

    #[test]
    fn candidate_retains_post_expansion_shadow_history() {
        let calibration = WorkerCalibration::new(1, 10);
        let pool = WorkerPool { initial: 2, maximum: 4, calibration: Some(calibration) };
        let recorder =
            ScanDiagnosticsRecorder::new(pool, 4, WorkerPolicyExperiment::RepeatedWindows);
        let queue = DirectoryQueue::new_with_policy(
            (PathBuf::new(), 0),
            ScanOrder::BreadthFirst,
            Some(calibration),
            Some(recorder.clone()),
            pool.initial,
            pool.maximum,
            WorkerPolicyExperiment::RepeatedWindows,
        );
        let mut timing = WalkAttribution::default();
        let mut claimed = Vec::new();

        let slow = queue.claim(&mut claimed, &mut timing).expect("slow prefix");
        queue.extend(
            (0..4).map(|index| (PathBuf::from(format!("fast-{index}")), 1, RegionId::UNASSIGNED)),
            &mut timing,
        );
        assert_eq!(slow.release(1, 20, &mut timing), Some(4));

        claimed.clear();
        let fast = queue.claim(&mut claimed, &mut timing).expect("fast suffix");
        queue.extend([(PathBuf::from("tail"), 2, RegionId::UNASSIGNED)].into_iter(), &mut timing);
        assert_eq!(fast.release(1, 1, &mut timing), None);

        let diagnostics = recorder.finish();
        assert_eq!(
            diagnostics
                .worker_policy
                .windows
                .iter()
                .map(|window| window.decision)
                .collect::<Vec<_>>(),
            vec![WorkerPolicyDecision::ScaleUp, WorkerPolicyDecision::ObserveFast]
        );
    }

    #[test]
    fn every_experimental_controller_preserves_exactness_and_shutdown() {
        let dir = branching_tree();
        let serial = ScanConfig { threads: Some(1), ..ScanConfig::default() };
        let (reference, _) = scan_into_index(dir.path(), &serial).expect("serial reference");
        let automatic = ScanConfig { threads: None, ..ScanConfig::default() };

        for policy in [
            WorkerPolicyExperiment::ShippedOneShot,
            WorkerPolicyExperiment::RepeatedWindows,
            WorkerPolicyExperiment::StagedGatedWindows,
        ] {
            let (index, report, diagnostics) =
                scan_into_index_with_policy_diagnostics(dir.path(), &automatic, policy)
                    .expect("candidate scan finishes");
            assert!(report.is_complete(), "{policy:?}: {:?}", report.errors);
            assert_eq!(index_fingerprint(&reference), index_fingerprint(&index), "{policy:?}");
            assert_eq!(diagnostics.worker_policy.ready_directories_at_finish, 0);
            assert_eq!(diagnostics.worker_policy.in_flight_directories_at_finish, 0);
            assert_eq!(diagnostics.worker_policy.handoff_backlog_at_finish, 0);
            assert!(
                diagnostics.worker_policy.workers_spawned
                    <= diagnostics.worker_policy.maximum_workers
            );
        }
    }

    /// A deterministic model of the automatic worker policy under *completion* order.
    ///
    /// The scaling decision is driven by chunk releases, and chunks complete in whatever
    /// order the filesystem and the workers produce them — not in traversal order. On a
    /// homogeneous tree that distinction is invisible, because every prefix looks like
    /// every other. On a heterogeneous one it decides the answer.
    ///
    /// These tests exist because the alternative is a stopwatch on a real tree, which
    /// measures one host on one day and cannot separate a policy defect from ambient
    /// noise. Replaying an explicit completion order through the shipped calibration
    /// isolates the policy exactly, and does so identically on every platform.
    ///
    /// They characterize behavior; they do not endorse a replacement. Which controller
    /// is *faster* is a question only the held-out Apple Silicon/APFS matrix can answer.
    mod completion_order {
        use super::{ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY, WorkerCalibration};

        /// One chunk release: entries observed and worker time spent observing them.
        #[derive(Clone, Copy, Debug)]
        struct Chunk {
            entries: u64,
            work_ns: u64,
        }

        impl Chunk {
            /// A run of `entries` entries costing `per_entry_ns` each.
            const fn at(entries: u64, per_entry_ns: u64) -> Self {
                Self { entries, work_ns: entries.saturating_mul(per_entry_ns) }
            }
        }

        /// What a policy concluded over one completion order.
        #[derive(Debug, PartialEq, Eq)]
        enum Outcome {
            /// The policy found the filesystem slow and expanded the reserve.
            ScaledUp { after_chunks: usize },
            /// The policy found the filesystem fast and held the initial pool.
            Held { after_chunks: usize },
            /// The walk ended before the policy observed enough to conclude anything.
            ///
            /// Distinct from [`Outcome::Held`] on purpose: nothing was measured, so a
            /// held pool here is an absence of evidence rather than a decision.
            Undecided,
        }

        /// Mean cost per entry over a whole trace, which is what the threshold *means*.
        fn whole_trace_ns_per_entry(trace: &[Chunk]) -> u64 {
            let entries: u64 = trace.iter().map(|chunk| chunk.entries).sum();
            let work_ns: u64 = trace.iter().map(|chunk| chunk.work_ns).sum();
            assert!(entries > 0, "a trace must observe entries");
            work_ns / entries
        }

        /// Replay a completion order through the *shipped* calibration.
        ///
        /// This drives [`WorkerCalibration::observe`] itself rather than restating its
        /// arithmetic, so the model cannot quietly drift from the policy it is evidence
        /// about. The loop mirrors `DirectoryQueue::release`: fold each chunk in, and
        /// stop at the first one that produces a verdict.
        fn shipped(window: u64, threshold_ns: u64, trace: &[Chunk]) -> Outcome {
            let mut calibration = WorkerCalibration::new(window, threshold_ns);
            for (index, chunk) in trace.iter().enumerate() {
                if let Some(slow) = calibration.observe(chunk.entries, chunk.work_ns) {
                    let after_chunks = index + 1;
                    return if slow {
                        Outcome::ScaledUp { after_chunks }
                    } else {
                        Outcome::Held { after_chunks }
                    };
                }
            }
            Outcome::Undecided
        }

        /// Entries per chunk in the traces below. Four fill the 16,384-entry window.
        const CHUNK: u64 = 4_096;
        /// A shallow, cache-warm phase: metadata already resident.
        const FAST: Chunk = Chunk::at(CHUNK, 2_000);
        /// A deep, cold phase: the latency-bound regime the reserve exists to hide.
        const SLOW: Chunk = Chunk::at(CHUNK, 90_000);

        #[test]
        fn completion_order_alone_flips_the_shipped_decision() {
            // The defect, stated as an experiment: hold the *tree* constant and vary
            // only the order its chunks complete in. Both traces contain the same four
            // fast and four slow chunks, so they describe the same filesystem work.
            let fast_phase_first = [FAST, FAST, FAST, FAST, SLOW, SLOW, SLOW, SLOW];
            let interleaved = [SLOW, FAST, SLOW, FAST, SLOW, FAST, SLOW, FAST];

            let window = 4 * CHUNK;
            let threshold = ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY;

            // Whole-walk truth is identical, and by the policy's own threshold both
            // walks are latency-bound: 46 µs per entry against a 30 µs trigger.
            let truth = whole_trace_ns_per_entry(&fast_phase_first);
            assert_eq!(truth, whole_trace_ns_per_entry(&interleaved));
            assert!(
                truth >= threshold,
                "both traces are slow walks by the shipped threshold: {truth} < {threshold}"
            );

            // Yet the decision depends entirely on which chunks happened to finish
            // first. One walk hides latency; the other runs the whole slow phase on the
            // starting pool, having concluded from an unrepresentative prefix.
            assert_eq!(
                shipped(window, threshold, &fast_phase_first),
                Outcome::Held { after_chunks: 4 },
                "a fast prefix holds the pool for a walk that is slow overall"
            );
            assert_eq!(
                shipped(window, threshold, &interleaved),
                Outcome::ScaledUp { after_chunks: 4 },
                "the same tree scales up when its slow chunks land in the window"
            );
        }

        #[test]
        fn a_slow_phase_after_the_window_is_never_reconsidered() {
            // The heterogeneous-tree case from the field report. A small fast region
            // fills the window, and everything after it is slow — but the calibration
            // is already gone, so no amount of later evidence can reopen the decision.
            let mut trace = vec![FAST; 4];
            trace.extend(std::iter::repeat_n(SLOW, 400));

            let observed = whole_trace_ns_per_entry(&trace);
            assert!(
                observed >= 2 * ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY,
                "the walk is overwhelmingly latency-bound: {observed} ns per entry"
            );

            assert_eq!(
                shipped(4 * CHUNK, ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY, &trace),
                Outcome::Held { after_chunks: 4 },
                "1% of the walk decided the worker policy for the other 99%"
            );
        }

        #[test]
        fn slow_in_flight_work_is_censored_by_fast_completions() {
            // Four slow chunks have already been claimed, but their filesystem calls
            // remain in flight while four cache-warm chunks complete. Completion-order
            // calibration cannot see owed work: the fast completions close the window
            // and permanently hold before any slow claim returns.
            let completed_before_slow_returns = [FAST, FAST, FAST, FAST];
            assert_eq!(
                shipped(
                    4 * CHUNK,
                    ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY,
                    &completed_before_slow_returns,
                ),
                Outcome::Held { after_chunks: 4 }
            );

            let mut eventual_completions = completed_before_slow_returns.to_vec();
            eventual_completions.extend([SLOW, SLOW, SLOW, SLOW]);
            assert!(
                whole_trace_ns_per_entry(&eventual_completions)
                    >= ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY
            );
            assert_eq!(
                shipped(4 * CHUNK, ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY, &eventual_completions,),
                Outcome::Held { after_chunks: 4 }
            );
        }

        #[test]
        fn a_slow_prefix_can_scale_a_walk_that_is_fast_overall() {
            // The mirror-image error. A one-way expansion reacts correctly to the
            // prefix by its local threshold, but the prefix is under 1% of this walk
            // and the whole trace is firmly in the fast regime. A repeated trigger
            // alone cannot undo an expansion; staged growth limits exposure but does
            // not make reversible parking unnecessary.
            let mut trace = vec![SLOW; 4];
            trace.extend(std::iter::repeat_n(FAST, 400));
            let observed = whole_trace_ns_per_entry(&trace);
            assert!(observed < ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY);
            assert_eq!(
                shipped(4 * CHUNK, ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY, &trace),
                Outcome::ScaledUp { after_chunks: 4 }
            );
            assert_eq!(
                sliding(4 * CHUNK, ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY, &trace),
                Outcome::ScaledUp { after_chunks: 4 }
            );
        }

        #[test]
        fn a_walk_shorter_than_the_window_decides_nothing() {
            // Fails closed rather than reporting a held pool: a walk this short never
            // observed enough to have an opinion, and an artifact that recorded `Held`
            // would claim a measurement that was never taken.
            let trace = [FAST, SLOW];
            assert_eq!(
                shipped(4 * CHUNK, ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY, &trace),
                Outcome::Undecided
            );
        }

        /// A screening-only candidate: a window that slides instead of closing once.
        ///
        /// Present as *evidence about a design*, not as a proposed change. It keeps the
        /// shipped trigger and pool bounds and alters only when the question is asked,
        /// which is the narrowest edit that could address the order sensitivity above.
        /// Whether it is faster on a real tree is unmeasured here and unmeasurable in a
        /// virtualized non-APFS environment; selecting it would need the held-out Apple
        /// Silicon matrix that this workstream has not yet been able to run.
        struct SlidingWindow {
            window_entries: u64,
            threshold_ns: u64,
            recent: std::collections::VecDeque<Chunk>,
            entries: u64,
            work_ns: u64,
        }

        impl SlidingWindow {
            fn new(window_entries: u64, threshold_ns: u64) -> Self {
                Self {
                    window_entries,
                    threshold_ns,
                    recent: std::collections::VecDeque::new(),
                    entries: 0,
                    work_ns: 0,
                }
            }

            /// Fold in a chunk and re-ask the question over the trailing window.
            fn observe(&mut self, chunk: Chunk) -> Option<bool> {
                self.recent.push_back(chunk);
                self.entries = self.entries.saturating_add(chunk.entries);
                self.work_ns = self.work_ns.saturating_add(chunk.work_ns);

                // Drop from the front while the window stays full without the oldest
                // chunk, so the answer describes recent work rather than the whole walk.
                while let Some(oldest) = self.recent.front().copied() {
                    if self.entries - oldest.entries < self.window_entries {
                        break;
                    }
                    self.recent.pop_front();
                    self.entries -= oldest.entries;
                    self.work_ns -= oldest.work_ns;
                }

                (self.entries >= self.window_entries)
                    .then(|| self.work_ns / self.entries >= self.threshold_ns)
            }
        }

        /// Replay a completion order through the candidate, stopping at its first
        /// scale-up. The shipped pool only grows, so a later verdict cannot undo one.
        fn sliding(window: u64, threshold_ns: u64, trace: &[Chunk]) -> Outcome {
            let mut policy = SlidingWindow::new(window, threshold_ns);
            let mut decided = None;
            for (index, chunk) in trace.iter().enumerate() {
                if let Some(slow) = policy.observe(*chunk) {
                    let after_chunks = index + 1;
                    if slow {
                        return Outcome::ScaledUp { after_chunks };
                    }
                    decided.get_or_insert(Outcome::Held { after_chunks });
                }
            }
            decided.unwrap_or(Outcome::Undecided)
        }

        #[test]
        fn screening_a_sliding_window_against_the_order_sensitivity() {
            let window = 4 * CHUNK;
            let threshold = ADAPTIVE_SCAN_SLOW_WORK_NS_PER_ENTRY;

            // The pair that splits the shipped policy reaches one answer here, and it
            // is the answer the whole-trace mean supports in both orders.
            let fast_phase_first = [FAST, FAST, FAST, FAST, SLOW, SLOW, SLOW, SLOW];
            let interleaved = [SLOW, FAST, SLOW, FAST, SLOW, FAST, SLOW, FAST];
            // Both reach the same verdict; they differ only in how long the fast prefix
            // delays it, which is the behavior a trailing window is supposed to have.
            assert_eq!(
                sliding(window, threshold, &fast_phase_first),
                Outcome::ScaledUp { after_chunks: 6 }
            );
            assert_eq!(
                sliding(window, threshold, &interleaved),
                Outcome::ScaledUp { after_chunks: 4 }
            );

            // And the late slow phase is reached rather than missed: two slow chunks
            // after the window closes are enough to pull the trailing mean over.
            let mut late = vec![FAST; 4];
            late.extend(std::iter::repeat_n(SLOW, 400));
            assert_eq!(sliding(window, threshold, &late), Outcome::ScaledUp { after_chunks: 6 });

            // A genuinely fast tree must still hold the pool: the candidate has to keep
            // the property the shipped policy gets right, or it is not a candidate.
            let uniformly_fast = vec![FAST; 40];
            assert_eq!(
                sliding(window, threshold, &uniformly_fast),
                Outcome::Held { after_chunks: 4 }
            );

            // A short walk still decides nothing, for the same reason as above.
            assert_eq!(sliding(window, threshold, &[FAST, SLOW]), Outcome::Undecided);
        }
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
        assert!(
            index
                .since(crate::Cursor::start(index.session()))
                .expect("own session")
                .deltas
                .is_empty()
        );
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
        let before = index.total();

        let mut deltas = Vec::new();
        revalidate(&index, &ScanConfig::default(), &mut |d| deltas.push(d)).expect("revalidate");
        let mut unchanged = 0;
        for delta in &deltas {
            unchanged += index.apply_ok(delta).unchanged;
        }

        assert_eq!(unchanged, 5, "3 files + 2 dirs all already known");
        assert_eq!(index.total(), before);
    }

    #[test]
    fn direct_reconciliation_counts_unchanged_entries_without_publishing_deltas() {
        let dir = sample_tree();
        let (mut index, _) = scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        let before_total = index.total();
        let before_clock = index.clock();
        let mut deltas = Vec::new();

        let report = reconcile(&mut index, &ScanConfig::default(), &mut |delta| {
            deltas.push(delta.clone());
        })
        .expect("reconcile");

        assert!(report.is_complete());
        assert_eq!(report.apply.unchanged, 5, "3 files + 2 dirs all already known");
        assert!(
            deltas.iter().all(|delta| delta.ops.is_empty()),
            "an unchanged tree publishes no mutation: {deltas:?}"
        );
        // The sweep itself is still a transition, and the two commits are the whole of it:
        // the subtree entered reconciliation, and the sweep that ended verified it. Every
        // one of those five unchanged entries had its provenance promoted from cached to
        // revalidated, and the interval is where the index records that -- so a clock that
        // had not moved would be an index claiming nothing had changed about how far its
        // answers can be trusted, immediately after checking every one of them.
        //
        // Asserted through the sink rather than only against the index clock. This callback
        // is documented as receiving every effective committed delta, and checking the
        // clock alone let it receive none of them and still pass.
        assert_sweep_deltas(&deltas, before_clock, Path::new(""));
        assert_eq!(index.clock().0, before_clock.0 + 2);
        assert_eq!(index.total(), before_total);
    }

    /// The two deltas a completed sweep publishes, in order, at consecutive clocks.
    #[track_caller]
    fn assert_sweep_deltas(deltas: &[AppliedDelta], before: crate::Clock, subtree: &Path) {
        let state: Vec<(u64, &crate::StateChange)> = deltas
            .iter()
            .flat_map(|delta| delta.state.iter().map(move |change| (delta.clock.0, change)))
            .collect();
        assert_eq!(
            state,
            vec![
                (
                    before.0 + 1,
                    &crate::StateChange::Freshness {
                        path: subtree.to_path_buf(),
                        freshness: crate::Freshness::Reconciling,
                        reason: None,
                    }
                ),
                (
                    before.0 + 2,
                    &crate::StateChange::Freshness {
                        path: subtree.to_path_buf(),
                        freshness: crate::Freshness::Fresh,
                        reason: None,
                    }
                ),
                (before.0 + 2, &crate::StateChange::Verified { path: subtree.to_path_buf() }),
            ],
            "the sweep announced itself and then recorded what it verified: {deltas:?}"
        );
    }

    #[test]
    fn parallel_and_serial_reconciliation_produce_the_same_index() {
        let dir = sample_tree();
        let portable_config =
            ScanConfig { threads: Some(1), batch_size: 2, ..ScanConfig::default() };
        let (mut portable, _) =
            scan_into_index(dir.path(), &portable_config).expect("portable baseline");
        let mut bulk = portable.clone();

        fs::remove_file(dir.path().join("a.txt")).expect("remove file");
        write_file(&dir.path().join("src/main.rs"), b"fn main() { much longer }");
        write_file(&dir.path().join("src/added.md"), b"new file");

        let portable_report = reconcile(&mut portable, &portable_config, &mut |_| {})
            .expect("portable reconciliation");
        let bulk_config = ScanConfig { threads: Some(2), ..portable_config };
        let bulk_report =
            reconcile(&mut bulk, &bulk_config, &mut |_| {}).expect("bulk reconciliation");

        assert!(portable_report.is_complete());
        assert!(bulk_report.is_complete());
        assert_eq!(bulk_report.scan.attribution, WalkAttribution::default());
        assert_eq!(bulk_report.scan.entries, portable_report.scan.entries);
        assert_eq!(bulk_report.scan.dirs_read, portable_report.scan.dirs_read);
        assert_eq!(bulk_report.apply, portable_report.apply);
        assert_eq!(index_fingerprint(&bulk), index_fingerprint(&portable));
        assert_eq!(bulk.total(), portable.total());
    }

    #[test]
    fn parallel_reconciliation_workers_publish_directory_counters() {
        let _serial = crate::counters::test_serial();
        let dir = sample_tree();
        let baseline = ScanConfig { threads: Some(1), ..ScanConfig::default() };
        let (mut index, scan) = scan_into_index(dir.path(), &baseline).expect("baseline scan");
        assert!(scan.is_complete());

        crate::counters::enable(true);
        crate::counters::reset();
        let config = ScanConfig { threads: Some(4), ..baseline };
        let report = reconcile(&mut index, &config, &mut |_| {}).expect("reconciliation");
        crate::counters::flush_thread();
        let counts = crate::counters::snapshot();
        crate::counters::reset();
        crate::counters::enable(false);

        assert!(report.is_complete());
        assert!(
            counts.dir_opens >= report.scan.dirs_read,
            "parallel worker directory opens were folded: {counts:?}, report={report:?}"
        );
    }

    #[test]
    fn parallel_reconciliation_matches_serial_across_structural_transitions() {
        for max_depth in [None, Some(1), Some(2)] {
            for order in [ScanOrder::BreadthFirst, ScanOrder::DepthFirst] {
                let dir = reconciliation_transition_tree();
                let reference_config = ScanConfig {
                    order,
                    max_depth,
                    threads: Some(1),
                    batch_size: 2,
                    ..ScanConfig::default()
                };
                let (baseline, baseline_report) =
                    scan_into_index(dir.path(), &reference_config).expect("baseline scan");
                assert!(baseline_report.is_complete());
                mutate_reconciliation_transition_tree(dir.path());

                let mut serial = baseline.clone();
                let mut serial_deltas = Vec::new();
                let serial_report = reconcile(&mut serial, &reference_config, &mut |delta| {
                    serial_deltas.push(delta.clone());
                })
                .expect("serial reconciliation");
                let (fresh, fresh_report) =
                    scan_into_index(dir.path(), &reference_config).expect("fresh oracle");
                assert!(serial_report.is_complete(), "serial {order:?}/{max_depth:?}");
                assert!(fresh_report.is_complete(), "fresh {order:?}/{max_depth:?}");
                assert_eq!(
                    index_fingerprint(&serial),
                    index_fingerprint(&fresh),
                    "serial did not converge to a fresh scan for {order:?}/{max_depth:?}"
                );

                for workers in [2, 4] {
                    let mut parallel = baseline.clone();
                    let config = ScanConfig { threads: Some(workers), ..reference_config.clone() };
                    let mut parallel_deltas = Vec::new();
                    let report = reconcile(&mut parallel, &config, &mut |delta| {
                        parallel_deltas.push(delta.clone());
                    })
                    .expect("parallel reconciliation");
                    let context = format!("{order:?}/{max_depth:?}/{workers} workers");

                    assert!(report.is_complete(), "{context}: unexpected partial report");
                    assert_eq!(report.scan.entries, serial_report.scan.entries, "{context}");
                    assert_eq!(report.scan.dirs_read, serial_report.scan.dirs_read, "{context}");
                    assert_eq!(report.apply, serial_report.apply, "{context}");
                    assert_eq!(
                        effective_ops(&parallel_deltas),
                        effective_ops(&serial_deltas),
                        "{context}: effective delta differs"
                    );
                    assert_eq!(
                        index_fingerprint(&parallel),
                        index_fingerprint(&serial),
                        "{context}: final index differs"
                    );
                    let (parallel_total, serial_total) = (parallel.total(), serial.total());
                    assert_eq!(
                        (
                            parallel_total.files,
                            parallel_total.dirs,
                            parallel_total.bytes,
                            parallel_total.allocated,
                            parallel_total.newest_mtime_ns,
                        ),
                        (
                            serial_total.files,
                            serial_total.dirs,
                            serial_total.bytes,
                            serial_total.allocated,
                            serial_total.newest_mtime_ns,
                        ),
                        "{context}: roll-up differs"
                    );
                    assert_eq!(
                        parallel_total.by_ext, serial_total.by_ext,
                        "{context}: extension roll-up differs"
                    );
                }
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn reconciliation_metadata_errors_do_not_delete_enumerated_entries() {
        use std::os::unix::fs::PermissionsExt;

        if !crate::test_support::permission_bits_are_enforced() {
            // A privileged process still reads the directory it was denied, so the
            // fixture cannot reach the metadata-error boundary this pins.
            eprintln!("skipped: this process is not subject to Unix permission bits");
            return;
        }

        // macOS's parallel path may satisfy the whole directory through
        // getattrlistbulk even without search permission. One worker pins the portable
        // fallback there; Linux also exercises the parallel portable worker.
        let worker_counts = if cfg!(target_os = "macos") { vec![1] } else { vec![1, 2] };
        for workers in worker_counts {
            let dir = sample_tree();
            let config =
                ScanConfig { threads: Some(workers), batch_size: 2, ..ScanConfig::default() };
            let (mut index, baseline_report) =
                scan_into_index(dir.path(), &config).expect("baseline scan");
            assert!(baseline_report.is_complete());
            let before = index_fingerprint(&index);
            let original_permissions =
                fs::metadata(dir.path()).expect("root metadata").permissions();

            // Reading names requires read permission; looking up their metadata also
            // requires search permission. This makes enumeration succeed and each
            // metadata lookup fail, the boundary where an encountered name used to be
            // misclassified as a deletion.
            fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o400))
                .expect("remove search permission");
            let outcome = reconcile(&mut index, &config, &mut |_| {});
            fs::set_permissions(dir.path(), original_permissions).expect("restore permissions");

            let report = outcome.expect("operational metadata errors are a partial report");
            assert!(!report.scan.errors.is_empty(), "the fixture did not induce metadata errors");
            assert!(!report.is_complete());
            assert_eq!(index_fingerprint(&index), before);
            assert!(index.attrs(Path::new("a.txt")).is_some(), "existing entry was removed");
        }
    }

    #[test]
    fn deferred_change_overflow_retries_without_applying_a_partial_wave() {
        let dir = sample_tree();
        let config = ScanConfig { threads: Some(2), batch_size: 2, ..ScanConfig::default() };
        let (mut index, _) = scan_into_index(dir.path(), &config).expect("baseline");
        let before = index_fingerprint(&index);

        fs::remove_file(dir.path().join("a.txt")).expect("remove file");
        write_file(&dir.path().join("added.md"), b"new file");
        write_file(&dir.path().join("src/main.rs"), b"fn main() { much longer }");

        let root = index.root_path().to_path_buf();
        let root_meta = {
            crate::counters::bump(|c| c.stats += 1);
            fs::symlink_metadata(&root)
        }
        .expect("root metadata");
        let mut deltas = Vec::new();
        let outcome = reconcile_direct_parallel(
            &mut index,
            &root,
            attrs_from(&root_meta).dev,
            &config,
            1,
            &mut |delta| deltas.push(delta.clone()),
        )
        .expect("parallel attempt");
        let DirectParallelOutcome::RetrySerial { prefix, remaining } = outcome else {
            panic!("the deliberately tiny deferred budget must trigger the retry");
        };

        assert_eq!(prefix.apply, ApplyStats::default());
        assert_eq!(remaining, VecDeque::from([(PathBuf::new(), 0)]));
        assert!(deltas.is_empty());
        assert_eq!(index_fingerprint(&index), before);

        let serial = ScanConfig { threads: Some(1), ..config };
        let report = reconcile(&mut index, &serial, &mut |_| {}).expect("serial retry");
        let (expected, expected_report) = scan_into_index(dir.path(), &serial).expect("oracle");
        assert!(report.is_complete());
        assert!(expected_report.is_complete());
        assert_eq!(index_fingerprint(&index), index_fingerprint(&expected));
        assert_eq!(index.total().files, expected.total().files);
        assert_eq!(index.total().dirs, expected.total().dirs);
        assert_eq!(index.total().bytes, expected.total().bytes);
        assert_eq!(index.total().allocated, expected.total().allocated);
        assert_eq!(index.total().newest_mtime_ns, expected.total().newest_mtime_ns);
        assert_eq!(index.total().by_ext, expected.total().by_ext);
    }

    #[test]
    fn late_overflow_resumes_without_double_counting_completed_waves() {
        let dir = tempfile::tempdir().expect("tempdir");
        // The root wave discovers more than one full wave of directories. A change in
        // the second wave then forces the serial fallback only after the first wave's
        // unchanged entries have already been counted.
        for directory in 0..=RECONCILE_WAVE_DIRECTORIES {
            write_file(&dir.path().join(format!("d{directory:04}/file.txt")), b"unchanged");
        }
        let parallel = ScanConfig { threads: Some(2), ..ScanConfig::default() };
        let (baseline, _) = scan_into_index(dir.path(), &parallel).expect("baseline");
        let mut candidate = baseline.clone();
        let mut serial_oracle = baseline;

        for directory in 0..=RECONCILE_WAVE_DIRECTORIES {
            write_file(
                &dir.path().join(format!("d{directory:04}/file.txt")),
                b"changed after the first wave",
            );
        }

        let candidate_report = reconcile_target_inner(
            &mut ReconcileTarget::Direct(&mut candidate),
            Path::new(""),
            &parallel,
            0,
            &mut |_| {},
        )
        .expect("late-overflow reconciliation");
        let serial = ScanConfig { threads: Some(1), ..parallel };
        let oracle_report =
            reconcile(&mut serial_oracle, &serial, &mut |_| {}).expect("serial oracle");

        assert_eq!(candidate_report.apply, oracle_report.apply);
        assert_eq!(candidate_report.scan.entries, oracle_report.scan.entries);
        assert_eq!(candidate_report.scan.dirs_read, oracle_report.scan.dirs_read);
        assert_eq!(index_fingerprint(&candidate), index_fingerprint(&serial_oracle));
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
        assert!(
            deltas.iter().all(|delta| delta.ops.is_empty()),
            "an unchanged tree publishes no mutation: {deltas:?}"
        );
        // The same two deltas through the same sink, so the shared path cannot quietly
        // publish less than the direct one. Arbitration of the no-ops is what the test is
        // about, and it is unaffected -- no op reached a delta.
        assert_sweep_deltas(&deltas, before_clock, Path::new(""));
        assert_eq!(handle.clock().expect("clock").0, before_clock.0 + 2);
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
        let root_dev = {
            crate::counters::bump(|c| c.stats += 1);
            fs::symlink_metadata(root)
        }
        .expect("stat root")
        .dev();
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
