//! **fdu** — a fast, incremental file roll-up engine.
//!
//! fdu answers, for any directory in a tree: how big is it, how many files does it hold,
//! what changed most recently, and what kinds of files live in it — hierarchically, for
//! every directory at once, from a single walk.
//!
//! # The shape: three artifacts, one contract
//!
//! 1. **The index** ([`Index`]) — the in-memory hierarchical structure: entry
//!    records plus per-directory roll-up state.
//! 2. **The snapshot** ([`snapshot`]) — that index, serialized.
//! 3. **The change contract** ([`Observation`] and [`Commit`]) —
//!    producers submit verified observations; the index commits clocked effective
//!    changes.
//!
//! Everything else is a producer of observations or a consumer of exact commits. The
//! walker establishes a baseline from upsert observations; the reconciler submits the
//! conditional diff between indexed state and reality; the watch layer submits verified,
//! coalesced observations. The index arbitrates them and re-rolls its reducers; a change
//! feed consumes exact effective changes and state transitions from [`Commit`].
//!
//! A deliberate consequence: **watching is not tied to the roll-up logic.** The index
//! knows `apply(Observation)` and nothing about filesystem events, so a batch scan, a test
//! feeding synthetic observations, and a live watcher are indistinguishable to it.
//!
//! # Freshness is a ladder, not a set of alternatives
//!
//! [`open`] is the conservative, blocking entry point: it loads a compatible snapshot,
//! reconciles the configured filesystem scope, and only then returns. It does not serve
//! the loaded baseline concurrently. Applications that want that model can own an
//! [`IndexHandle`], call the applying reconciliation APIs, and inspect [`Freshness`]
//! while readers continue between short write batches. With the `watch` feature,
//! `watch::Watcher::apply_next` verifies event hints and closes invalidations through
//! subtree reconciliation; neither `open` nor the Python binding starts it implicitly.
//! [`OpenedIndex`] is the additive long-lived owner: its clones share one live identity,
//! cancellation domain, index, and joined shutdown. A cloned [`Index`] remains a
//! detached image and never inherits that authority.
//!
//! ```no_run
//! use fdu_core::{OpenConfig, open};
//! use std::path::Path;
//!
//! let (index, report) = open(Path::new("."), &OpenConfig::default())?;
//! let total = index.total();
//! println!("{} files, {} bytes ({:?})", total.files, total.bytes, report.path_taken);
//! # Ok::<(), fdu_core::Error>(())
//! ```
//!
//! # Build features
//!
//! - `watch` — the OS-native watch layer.
//!
//! `fdu-core` has no default build features. The command and Python packages opt into
//! watch, while embedding consumers can retain the smaller one-shot engine.
//!
//! `.gitignore` handling is not a build feature: it has no dependency, so it is always
//! compiled in, and whether a scan reads control files is decided at runtime by
//! [`ScanConfig::read_controls`].

pub mod admission;
pub mod cache;
pub mod classify;
pub mod content;
pub mod control;
pub mod counters;
mod emit;
mod engine_contract;
mod execution;
mod index;
mod opened;
mod platform_tuning;
pub mod query;
pub mod scan;
pub mod snapshot;
mod stored_state;
#[cfg(test)]
mod test_support;

/// The crate README's Rust examples, compiled and run as doctests.
///
/// crates.io shows the README as this crate's front page, so its examples are the first
/// code a reader copies. Nothing compiled them, and they kept naming an `AnalysisProfile`
/// type for a release after the content axis became `AnalysisSet`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

// Ungated: rendering is not a command-line concern. It was behind `cli` only because it
// took its ANSI colour types from clap, so the library could produce a report and not
// print it -- and a display note added elsewhere on this branch called into here and
// broke the no-default-features build, which is that gap showing itself.
pub mod report_format;

#[cfg(feature = "watch")]
pub mod watch_session;

#[cfg(feature = "watch")]
pub mod watch;

#[cfg(feature = "watch")]
pub use crate::watch_session as session;

pub use crate::admission::HiddenPolicy;
pub use crate::cache::{
    CacheScope, CacheState, CacheStatus, ClearSummary, ContentInfo, ContentState, ContentStatus,
    LeftoverKind, SnapshotInfo, StaleReason, cache_status, clear_all_caches, clear_cache,
    list_caches,
};
pub use crate::control::{
    CONTROL_FILE_NAME, ControlAdmission, ControlCoverage, ControlIdentity, ControlLimits,
    ControlMatcher, ControlObservation, ControlRefusalReason, ControlTable, DEFAULT_CONTROL_BUDGET,
    DEFAULT_CONTROL_LINE_LIMIT, RefusedControl, is_control_file,
};
pub use crate::engine_contract::{
    Attrs, ChangeOutcome, ChangePoll, ChangeRequest, Clock, Commit, ContinuationId, CountResult,
    Coverage, CoverageReason, DEFAULT_COUNT_CAP, DiscoveryProgress, EffectiveChange, EngineVersion,
    EntryKind, EntryValue, Error, Expectation, Fingerprint, FlatPage, Freshness, Impact,
    ImpactDomain, IndexState, InvalidateReason, Issue, IssueKind, IssueSummary, Knowledge,
    LifecyclePhase, LimitedProjection, MAX_CONTINUATION_RECORD_BYTES, MAX_COUNT_CAP,
    MAX_DIRTY_PATHS, MAX_ISSUE_MESSAGE_BYTES, MAX_ISSUE_PATH_BYTES, MAX_PAGE_ROWS, MAX_PAGE_WORK,
    MAX_READ_PROJECTIONS, MAX_REPORT_VIEWS, MAX_RETAINED_ISSUES, MIN_JOURNAL_CAPACITY_BYTES,
    Observation, ObservationOp, Op, PageRequest, PathExpectation, PathState, PortablePath,
    ProjectionRefusal, ProjectionResult, Provenance, QueryLimit, ReadDiagnostics, ReadProjection,
    ReadRequest, ReadResponse, RefreshRejection, RefreshResult, RejectedRefreshPath, ReportRequest,
    Result, RowShape, ScanScope, SemanticIdentity, SessionId, Source, StateTransition, Status,
    TreePage, Work,
};
pub use crate::index::{
    ApplyOutcome, ApplyStats, ChildSnapshot, DEFAULT_JOURNAL_CAPACITY_BYTES, EntryId, ExtTally,
    Index, IndexHandle, PartitionRollUp, PartitionRollUpSummary, RollUp, RollUpSummary, Since,
};
pub use crate::opened::{
    DiscoveryBudget, MAX_PRIORITY_PATHS, MAX_REFRESH_PATHS, OpenOptions, OpenedIndex,
};
// Ungated with report_format, for the same reason: one-shot planning is an execution
// strategy, not a front end. A caller wanting one report without retaining an index was
// previously required to compile the command line to get it (fdu-z7sp).
pub use crate::execution::{
    Load, PerformanceSummary, Plan, Route, Verify, plan, prepare_report,
    prepare_report_with_scan_diagnostics,
};
pub use crate::scan::{ReconcileReport, ScanConfig, ScanOrder, ScanReport};
pub use crate::stored_state::{
    AnalyzerProvenance, ContentTierIdentity, ControlTierIdentity, EntryScope, EntryTierIdentity,
    Serves, SnapshotIdentity, serves_snapshot,
};
#[cfg(feature = "watch")]
pub use crate::watch_session::{Batch, Change, ChangeKind, Session};

use crate::execution::{RunFacts, SaveTargets};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// How to open a tree.
#[cfg(test)]
#[derive(Clone, Debug, Default)]
pub(crate) struct OpenFixture {
    /// Walk settings.
    pub scan: ScanConfig,
    /// Where the snapshot for this root lives.
    ///
    /// `None` disables the cache regardless of policy, which is what a caller with no
    /// writable cache directory gets.
    pub cache_path: Option<PathBuf>,
    /// How the snapshot may be used.
    pub policy: CachePolicy,
    /// Optional streaming content analysis. Disabled preserves metadata-only behavior.
    pub analysis: content::AnalysisRequest,
}

#[cfg(test)]
impl OpenFixture {
    /// Today's open configuration, composed from the basis and the delivery that carry a
    /// request.
    ///
    /// One direction of a temporary bridge, and the only place it is spliced: the
    /// execution plan model replaces `OpenFixture` with `Basis` and `Delivery`, and one
    /// splice is one thing to delete rather than four. A basis rather than a whole
    /// request, because opening a root is what a basis is for and no query has been named
    /// yet on two of the routes that open one. Scan workers ride in the basis's scope and
    /// content workers in the delivery, because that is where each waits until one
    /// `Workers` takes both.
    pub fn of(basis: &query::Basis, delivery: &query::Delivery) -> Self {
        Self {
            scan: basis.scope.scan_config(delivery),
            cache_path: delivery.cache_path.clone(),
            policy: delivery.cache,
            analysis: content::AnalysisRequest {
                profile: basis.content,
                workers: delivery.workers.analysis,
            },
        }
    }

    /// The other direction, for a caller that still holds a configuration: the basis and
    /// the delivery it spells, over `root`.
    ///
    /// The inverse of [`Self::of`] and deleted with it. Fixtures and probes that name one
    /// configuration read it this way rather than each writing the division out, so the
    /// two halves are divided in one place whichever way a caller crosses the bridge.
    pub fn split(&self, root: impl Into<PathBuf>) -> (query::Basis, query::Delivery) {
        (
            query::Basis {
                root: root.into(),
                scope: self.scan.clone().into(),
                content: self.analysis.profile,
            },
            query::Delivery {
                cache: self.policy,
                cache_path: self.cache_path.clone(),
                accept_partial: false,
                watch: None,
                workers: query::Workers {
                    scan: self.scan.threads,
                    analysis: self.analysis.workers,
                },
                batch_size: self.scan.batch_size,
                order: self.scan.order,
            },
        )
    }
}

#[cfg(test)]
pub(crate) fn open_fixture(root: &Path, config: &OpenFixture) -> Result<(Index, OpenReport)> {
    let (basis, delivery) = config.split(root);
    open(&basis, &delivery)
}
#[cfg(test)]
pub(crate) fn open_fixture_with_pending_save(
    root: &Path,
    config: &OpenFixture,
) -> Result<(std::sync::Arc<Index>, OpenReport, PendingSave)> {
    let (basis, delivery) = config.split(root);
    open_with_pending_save(&basis, &delivery)
}

/// How an [`open`] may use the snapshot cache.
///
/// One explicit axis rather than a pair of booleans, because "did this answer touch the
/// filesystem" and "did it leave a trace" are the two questions a caller actually has,
/// and a boolean pair can express combinations that have no meaning.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CachePolicy {
    /// Read the snapshot, revalidate it, and write it back when the scan is complete.
    ///
    /// A root has one cache path, and its snapshot carries the scan scope that wrote it.
    /// A read under another scope normally treats that snapshot as absent and scans cold.
    /// The lawful exception is a controls-off request with the same entry identity: it
    /// projects a controls-on snapshot into the requested blind scope and never replaces
    /// the stronger image with that projection.
    ///
    /// Every default request observes `.gitignore` control state -- the one-shot
    /// `fdu <dir>` and [`prepare_report`], `fdu --watch <dir>`, and a default [`open`] --
    /// so they share one scope: an [`open`] or a watch starts warm from a one-shot report's
    /// snapshot, and a report that reads the snapshot, as content analysis does, starts
    /// warm from theirs. A request that turns [`ScanConfig::read_controls`] off is a second
    /// scope, but every route can start from a default snapshot by discarding its control
    /// tier while loading. A summary-only report that turned observation off saves nothing
    /// and replaces nothing.
    #[default]
    Auto,
    /// Ignore any snapshot, scan cold, and rewrite it. The benchmark control.
    Refresh,
    /// Read and revalidate, but never write. A warm answer that leaves no trace.
    ReadOnly,
    /// Answer from the snapshot alone, without touching the tree.
    ///
    /// Fails when no usable snapshot exists: there is no data to answer with, and
    /// silently falling back to a scan would make the fast path unpredictable.
    Only,
    /// Ignore the snapshot entirely and leave nothing behind.
    Off,
}

impl CachePolicy {
    /// Whether this policy may read an existing snapshot.
    fn reads(self) -> bool {
        matches!(self, Self::Auto | Self::ReadOnly | Self::Only)
    }

    /// Whether this policy may write a snapshot back.
    ///
    /// Public because a caller deciding whether to prepare a cache directory, or to warn
    /// that a run will leave nothing behind, is asking about the policy it was handed --
    /// and the alternative is matching on the variants, which is the same knowledge
    /// copied into every caller.
    pub fn writes(self) -> bool {
        matches!(self, Self::Auto | Self::Refresh)
    }

    /// Whether this policy may touch the filesystem at all.
    fn scans(self) -> bool {
        !matches!(self, Self::Only)
    }
}

/// Which tier of the freshness ladder an [`open`] actually used.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpenPath {
    /// No usable snapshot: the tree was walked from scratch.
    ColdScan,
    /// A snapshot was loaded and reconciled against the filesystem.
    WarmRevalidate,
    /// A snapshot answered on its own; the filesystem was never consulted.
    ///
    /// The only tier that can be stale, and it says so rather than implying currency.
    CacheOnly,
}

/// A snapshot write running alongside rendering.
///
/// The index is read-only by the time this starts, so the writer and the renderer are
/// two readers of the same data. The handle exists so the process can join before it
/// exits: an abandoned write would leave a half-written snapshot for the next run to
/// reject, turning a warm start into a cold one for no reason.
#[derive(Debug)]
#[must_use = "join the save before exiting or the snapshot may be abandoned"]
pub struct PendingSave {
    workers: Vec<(&'static str, std::thread::JoinHandle<Result<()>>)>,
}

impl PendingSave {
    /// Nothing to wait for.
    pub(crate) fn none() -> Self {
        Self { workers: Vec::new() }
    }

    /// Wait for the write to finish, returning its result.
    ///
    /// A failed save is the caller's to report, not to die on: the answer already
    /// rendered is still correct, and only the next run's warmth is lost.
    pub fn join(mut self) -> Result<()> {
        let mut first_error = None;
        for (name, worker) in self.workers.drain(..) {
            let outcome = worker
                .join()
                .unwrap_or_else(|_| Err(Error::Snapshot(format!("{name} cache writer panicked"))));
            if first_error.is_none() {
                first_error = outcome.err();
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}

impl Drop for PendingSave {
    fn drop(&mut self) {
        // A dropped handle still waits: losing the write silently would be worse than
        // the brief delay, and this only happens on a path that forgot to join.
        for (_, worker) in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

/// What [`open`] did.
#[derive(Debug)]
pub struct OpenReport {
    /// Cache tier used to produce the returned index.
    pub path_taken: OpenPath,
    /// Filesystem walk results, including any partial errors.
    pub scan: ScanReport,
    /// Content-analysis work performed after metadata reconciliation.
    pub analysis: Option<content::AnalysisReport>,
    /// Reusable records restored from the independently versioned content sidecar.
    pub content_cache: content::ContentCacheLoad,
    /// Whether the returned index was projected from a stronger controls-on snapshot.
    pub projected: bool,
}

impl OpenReport {
    /// Whether every path in the requested scan scope was read successfully.
    pub fn is_complete(&self) -> bool {
        self.scan.is_complete()
            && self.analysis.as_ref().is_none_or(content::AnalysisReport::is_complete)
    }

    /// Per-path errors that make this result partial.
    pub fn errors(&self) -> &[Error] {
        &self.scan.errors
    }

    /// Human-readable diagnostics for every operational condition that makes this result partial.
    pub fn error_messages(&self) -> Vec<String> {
        let mut errors = self.scan.errors.iter().map(ToString::to_string).collect::<Vec<_>>();
        if let Some(message) =
            self.analysis.as_ref().and_then(content::AnalysisReport::failure_message)
        {
            errors.push(message);
        }
        errors
    }
}

/// Open a tree, using the snapshot cache when one is usable.
///
/// On the warm path the snapshot is loaded and then reconciled against the filesystem
/// before being returned. Errors are represented as partial freshness and the previous
/// complete snapshot is left untouched; callers must inspect [`OpenReport::is_complete`]
/// or [`Index::freshness`] before treating totals as complete.
///
/// The index observes control state as [`ScanConfig::read_controls`] says, and the
/// default is on: the index exposes [`Index::controls`] and [`Index::is_ignored`], and a
/// watch over it maintains them. A one-shot report from [`prepare_report`] observes it on
/// the same terms, so a default `open` and a default report share one snapshot scope and
/// each starts warm from the other's snapshot. A caller wanting a single answer should use
/// [`prepare_report`].
///
/// A caller that reads no ignore classification may turn the field off. Its `open` reads
/// no `.gitignore`, and its index answers [`Index::is_ignored`] and [`Index::controls`]
/// with [`Error::ControlStateNotObserved`] rather than calling every entry unignored. Its
/// snapshot is of another scope. When its entry identity matches a default snapshot, the
/// loader discards that snapshot's control tier and constructs the returned index directly
/// in the requested controls-off scope. The projected index never replaces the stronger
/// snapshot, including during a watch session.
pub fn open(basis: &query::Basis, delivery: &query::Delivery) -> Result<(Index, OpenReport)> {
    let (index, report, pending) = open_with_pending_save(basis, delivery)?;
    // Joining first is what makes the unwrap infallible: the writer held the only other
    // reference, and this is the blocking entry point, so by here it has finished and
    // dropped it. `try_unwrap` rather than a clone keeps the owned-`Index` signature
    // honest — a fallback clone here would quietly reintroduce the copy the shared
    // writer exists to avoid.
    pending.join()?;
    let index = std::sync::Arc::into_inner(index)
        .expect("the joined writer released the only other reference");
    Ok((index, report))
}

/// Open a tree, returning the snapshot write for the caller to join.
///
/// The blocking [`open`] is the right default; a caller that renders its own output can
/// use this to overlap the write with rendering and join before exiting.
///
/// This path always loads a usable snapshot, because its callers — live sessions and
/// library consumers holding the index — amortise the load across everything they do
/// with it. A one-shot report cannot; the internal report planner decides per request
/// whether the read pays and routes through the gated variant below.
pub fn open_with_pending_save(
    basis: &query::Basis,
    delivery: &query::Delivery,
) -> Result<(std::sync::Arc<Index>, OpenReport, PendingSave)> {
    let request =
        query::Request::new(basis.clone(), query::Query::default(), std::time::SystemTime::now());
    let plan = plan(&request, &delivery, Route::Retained).map_err(Error::InvalidRequest)?;
    execute(&plan, &request.basis, false)
        .map(|(index, report, pending, _diagnostics)| (index, report, pending))
}

/// Reverify a retained index, refresh requested content, and persist according to delivery.
///
/// A partial pass retains its verified facts and may save only the content tier beside
/// a compatible complete snapshot. The returned report describes metadata changes.
pub fn refresh(
    index: &mut Index,
    basis: &query::Basis,
    delivery: &query::Delivery,
) -> Result<ReconcileReport> {
    let request =
        query::Request::new(basis.clone(), query::Query::default(), std::time::SystemTime::now());
    let plan = plan(&request, delivery, Route::Refresh).map_err(Error::InvalidRequest)?;
    request.validate_read(&query::Basis::held_by(index)).map_err(Error::InvalidRequest)?;
    let scan = basis.scope.scan_config(delivery);
    scan.validate_for_scope(index.scope())?;
    if plan.verify() == Verify::None {
        return Err(Error::Snapshot(
            "the `only` cache policy cannot refresh filesystem state".into(),
        ));
    }
    let report = scan::reconcile(index, &scan, &mut |_| {})?;
    load_content(index, basis, delivery)?;
    if basis.content.is_enabled() {
        content::analyze_index(
            index,
            content::AnalysisRequest { profile: basis.content, workers: delivery.workers.analysis },
        );
    }
    persist_index(index, &plan)?;
    Ok(report)
}

/// Why a policy that cannot scan has no snapshot to answer from, and what recovers.
///
/// [`CachePolicy::Only`] is the one policy that cannot fall back to a scan, so its failure
/// is the only place a caller learns that the snapshot is missing or of another scope. Two
/// mismatches are common enough to name. Control state: every default request observes it,
/// so a snapshot without it was written by a request that turned observation off, or by a
/// release from before observation was the default, and a default cache-only request after
/// it would otherwise fail with no hint that a snapshot exists at all. Control limits: both
/// scopes observe, and their identities are hashes, so "a different scan scope" would
/// describe a request whose only difference is a limit the caller chose and can choose
/// again.
fn unusable_snapshot_message(refused: Option<SnapshotIdentity>, wanted: &ScanConfig) -> String {
    // Not "run once under `auto` to write one": a compact summary scans without retaining
    // an index and writes nothing, so that remedy would fail again for exactly that query.
    const PREFIX: &str = "no usable snapshot for this root and scan scope";
    const NEVER_SCANS: &str = "the `only` cache policy never scans";
    let wanted_scope = wanted.scope();
    let differs_only_in_ignore_rules = |stored: ScanScope| {
        ScanScope { ignore_rules_fingerprint: wanted_scope.ignore_rules_fingerprint, ..stored }
            == wanted_scope
    };
    let Some(refused) = refused else {
        return format!("{PREFIX}; {NEVER_SCANS}, so use `auto`, which scans when none serves");
    };
    let stored = refused.scan_scope();
    if differs_only_in_ignore_rules(stored) {
        // Named without a knob, because the command line and the library spell the switch
        // differently and this message is the engine's.
        if stored.ignore_rules_fingerprint == 0 {
            return format!(
                "{PREFIX}: the cached snapshot has no .gitignore state, because the request \
                 that wrote it did not observe it, and this request does; {NEVER_SCANS}, so \
                 use `auto`, or turn .gitignore observation off as that request did"
            );
        }
        if wanted_scope.observes_controls() {
            let stored_limits = match refused.controls {
                ControlTierIdentity::Observed { limits } => limits,
                ControlTierIdentity::NotObserved => crate::control::ControlLimits::default(),
            };
            let changed = changed_control_limits(stored_limits, wanted.control_limits);
            if !changed.is_empty() {
                return format!(
                    "{PREFIX}: the cached snapshot was taken under other .gitignore limits \
                     ({changed}); {NEVER_SCANS}, so repeat the request with the snapshot's \
                     limits, or use `auto`, which scans when none serves"
                );
            }
        }
    }
    format!(
        "{PREFIX}: the cached snapshot has a different scan scope; {NEVER_SCANS}, so use \
         `auto`, which scans when none serves"
    )
}

/// Each control limit that differs between a snapshot and a request, both values named.
fn changed_control_limits(
    stored: crate::control::ControlLimits,
    wanted: crate::control::ControlLimits,
) -> String {
    let display = crate::control::limit_display;
    let changed: Vec<String> = [
        ("budget", stored.budget, wanted.budget),
        ("line limit", stored.line_limit, wanted.line_limit),
    ]
    .into_iter()
    .filter(|(_, stored, wanted)| stored != wanted)
    .map(|(name, stored, wanted)| {
        format!("{name} {}, where this request asks for {}", display(stored), display(wanted))
    })
    .collect();
    changed.join(", and ")
}

/// [`open_with_pending_save`] with the snapshot read under the caller's control.
///
/// `read_snapshot: false` skips loading an existing snapshot and takes the cold-scan
/// path: for a one-shot metadata query, revalidation stats every entry regardless, so
/// the load and the reconciliation against it are additive cost with nothing to
/// amortise them — measured on macOS/APFS over 494,031 entries, warm revalidation cost
/// 4.8 s against 3.6 s for the cold path, whose write-behind the read could at best
/// have saved ~50 ms of. Persistence is unaffected: the cold path still writes per
/// [`Plan::writes`], so the snapshot stays fresh for [`CachePolicy::Only`]
/// and for content-analysis reuse.
///
/// A policy that cannot scan reads regardless of the flag — for [`CachePolicy::Only`]
/// the snapshot is the contract, not a cost choice.
pub(crate) fn execute(
    plan: &Plan,
    basis: &query::Basis,
    collect_scan_diagnostics: bool,
) -> Result<(std::sync::Arc<Index>, OpenReport, PendingSave, Option<scan::ScanDiagnostics>)> {
    let scan_config = basis.scope.scan_config(plan.delivery());
    let analysis_request = content::AnalysisRequest {
        profile: basis.content,
        workers: plan.delivery.workers.analysis,
    };
    let delivery = plan.delivery();
    let root = &basis.root;
    let root = root.canonicalize().map_err(|e| Error::io(root, e))?;
    // Before the snapshot, not at the scan that may never happen: a scope this build cannot
    // honour has no answer at any delivery, and checking it where the scan runs made
    // `--cache only` report a snapshot miss for a request every other policy refuses --
    // which failure a run named then depended on how it was delivered (`refusal-order`).
    scan_config.validate()?;
    // A snapshot for this root that could not serve, kept so a policy that cannot scan says
    // why it has no answer rather than only that it has none.
    let mut refused_snapshot = None;
    let mut projected = false;
    let loaded = match (plan.load() == Load::Snapshot, &delivery.cache_path) {
        (true, Some(cache_path)) => {
            match snapshot::load_serving(
                cache_path,
                scan_config.types_shared(),
                scan_config.snapshot_identity(),
            )? {
                snapshot::LoadOutcome::Served(index, serves) if index.root_path() == root => {
                    projected = serves == Serves::ProjectControlsOff;
                    Some(index)
                }
                snapshot::LoadOutcome::Refused(identity) => {
                    refused_snapshot = Some(identity);
                    None
                }
                snapshot::LoadOutcome::Served(_, _) | snapshot::LoadOutcome::Absent => None,
            }
        }
        _ => None,
    };

    if plan.verify() == Verify::None {
        let Some(mut index) = loaded else {
            return Err(Error::Snapshot(unusable_snapshot_message(refused_snapshot, &scan_config)));
        };
        // Deliberately no reconciliation: this tier never touches the tree. The index is
        // marked unverified so the answer cannot claim a currency it has not earned — a
        // snapshot records the freshness it was written with, which was true then.
        index.mark_unverified();
        let content_cache = load_content(&mut index, basis, delivery)?;
        // A sidecar serves only its own identity, so restoring one record per visited
        // regular file means the sidecar holds the complete answer to this request.
        // Restore already walked that set; compare `hits` to files visited, not a
        // second walk and not unique `PathBuf` keys.
        if basis.content.is_enabled()
            && (!content_cache.usable || content_cache.hits != content_cache.candidates)
        {
            return Err(Error::Snapshot(
                "no complete usable content sidecar for this root and analysis profile".into(),
            ));
        }
        return Ok((
            std::sync::Arc::new(index),
            OpenReport {
                path_taken: OpenPath::CacheOnly,
                scan: ScanReport::default(),
                analysis: None,
                content_cache,
                projected,
            },
            PendingSave::none(),
            None,
        ));
    }

    if let Some(mut index) = loaded {
        let reconciled = scan::reconcile(&mut index, &scan_config, &mut |_| {})?;
        let scan_report = reconciled.scan;
        index.establish_baseline();
        let content_cache = load_content(&mut index, basis, delivery)?;
        let analysis = basis
            .content
            .is_enabled()
            .then(|| content::analyze_index(&mut index, analysis_request));
        // A reconciliation that mutated nothing leaves an index that serializes to the
        // bytes already on disk, so rewriting it is pure cost: the clone, the encode,
        // and the write all produce a file identical to the one just read. Each artifact
        // is judged separately because content and metadata are invalidated separately.
        let facts = run_facts(
            &index,
            basis,
            delivery,
            reconciled.apply.mutated(),
            analysis.as_ref().is_some_and(|report| report.applied > 0) || content_cache.stale > 0,
            projected,
        );
        let index = std::sync::Arc::new(index);
        let pending = spawn_save(&index, &plan.delivery, plan.writes(&facts));
        return Ok((
            index,
            OpenReport {
                path_taken: OpenPath::WarmRevalidate,
                scan: scan_report,
                analysis,
                content_cache,
                projected,
            },
            pending,
            None,
        ));
    }

    let (mut index, scan_report, scan_diagnostics) = if collect_scan_diagnostics {
        let (index, report, diagnostics) =
            scan::scan_into_index_with_diagnostics(&root, &scan_config)?;
        (index, report, Some(diagnostics))
    } else {
        let (index, report) = scan::scan_into_index(&root, &scan_config)?;
        (index, report, None)
    };
    let content_cache = load_content(&mut index, basis, delivery)?;
    let analysis =
        basis.content.is_enabled().then(|| content::analyze_index(&mut index, analysis_request));
    let facts = run_facts(&index, basis, delivery, true, true, false);
    let index = std::sync::Arc::new(index);
    let pending = spawn_save(&index, &plan.delivery, plan.writes(&facts));
    Ok((
        index,
        OpenReport {
            path_taken: OpenPath::ColdScan,
            scan: scan_report,
            analysis,
            content_cache,
            projected: false,
        },
        pending,
        scan_diagnostics,
    ))
}

fn run_facts(
    index: &Index,
    basis: &query::Basis,
    delivery: &query::Delivery,
    entries_changed: bool,
    content_changed: bool,
    projected: bool,
) -> RunFacts {
    let entries_verified = stored_state::entries_writable(index);
    let paired_entries = !entries_verified
        && delivery
            .cache_path
            .as_ref()
            .and_then(|path| snapshot::read_header(path).ok().flatten())
            .is_some_and(|stored| {
                stored.root == index.root_path()
                    && stored.identity.entries == index.snapshot_identity().entries
            });
    RunFacts {
        entries_verified,
        entries_changed,
        content_changed,
        content_requested: basis.content.is_enabled(),
        projected,
        paired_entries,
    }
}

/// Execute the persistence policy for a live index without retaining a lock while writing.
pub(crate) fn persist_index(index: &Index, plan: &Plan) -> Result<bool> {
    let basis = query::Basis::held_by(index);
    let delivery = plan.delivery();
    let stored =
        delivery.cache_path.as_ref().and_then(|path| snapshot::read_header(path).ok().flatten());
    let projected = stored.as_ref().is_some_and(|header| {
        header.root == index.root_path()
            && serves_snapshot(header.identity, index.snapshot_identity())
                == Serves::ProjectControlsOff
    });
    let writes = plan.writes(&run_facts(index, &basis, delivery, true, true, projected));
    let Some(path) = delivery.cache_path.as_ref() else {
        return Ok(false);
    };
    if writes.metadata {
        snapshot::save(index, path)?;
    }
    if writes.content {
        content::save_content_cache(index, &content::content_cache_path(path))?;
    }
    Ok(writes.metadata)
}

fn load_content(
    index: &mut Index,
    basis: &query::Basis,
    delivery: &query::Delivery,
) -> Result<content::ContentCacheLoad> {
    let (true, Some(snapshot_path)) = (delivery.cache.reads(), delivery.cache_path.as_deref())
    else {
        return Ok(content::ContentCacheLoad::default());
    };
    let wanted = index.content_identity(basis.content);
    content::load_content_cache(index, &wanted, &content::content_cache_path(snapshot_path))
}

/// Start the cache writes a completed open still needs, each tier under its own rule.
///
/// The snapshot is written only after a complete pass
/// ([`stored_state::entries_writable`]): a snapshot recording a partial view would be
/// served as fact on the next run, and the existing complete snapshot is better than that.
/// The content sidecar keeps the records the pass verified
/// ([`stored_state::content_tier_writable`]); a partial pass may replace it only beside a
/// stored snapshot of the same entry tier.
fn spawn_save(
    index: &std::sync::Arc<Index>,
    delivery: &query::Delivery,
    writes: SaveTargets,
) -> PendingSave {
    let Some(cache_path) = delivery.cache_path.clone().filter(|_| !writes.none()) else {
        return PendingSave::none();
    };

    // The index is read-only from here, so the writer and the caller's rendering are two
    // readers of one index rather than of two copies. This used to deep-clone — every
    // boxed entry, both stored copies of every name, and every `BTreeMap` — on the
    // caller's thread, before rendering could start, on every cache-writing run.
    // Sharing is what buys the independence a clone was buying; a run with nothing to
    // write still returns above rather than reaching this point.
    let snapshot_source = std::sync::Arc::clone(index);
    let mut workers = Vec::with_capacity(2);
    if writes.metadata {
        let metadata_source = std::sync::Arc::clone(&snapshot_source);
        let metadata_path = cache_path.clone();
        if let Ok(worker) =
            std::thread::Builder::new().name("fdu-snapshot".to_string()).spawn(move || {
                let _counter_guard = counters::thread_flush_guard();
                snapshot::save(&metadata_source, &metadata_path)
            })
        {
            workers.push(("metadata", worker));
        }
    }
    if writes.content {
        let content_path = content::content_cache_path(&cache_path);
        if let Ok(worker) =
            std::thread::Builder::new().name("fdu-content-cache".to_string()).spawn(move || {
                let _counter_guard = counters::thread_flush_guard();
                content::save_content_cache(&snapshot_source, &content_path)
            })
        {
            workers.push(("content", worker));
        }
    }
    // A machine that cannot spawn either thread can still answer; it just answers cold
    // next time.
    PendingSave { workers }
}

/// The conventional snapshot location for a root.
///
/// Snapshots are keyed by a hash of the canonical root path under the user cache
/// directory, so two roots never collide and a moved tree simply misses rather than
/// reading another tree's data.
pub fn default_cache_path(root: &Path) -> Option<PathBuf> {
    let canonical = root.canonicalize().ok()?;
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in canonical.as_os_str().as_encoded_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    Some(user_cache_dir()?.join("fdu").join(cache::snapshot_file_name(hash)))
}

fn user_cache_dir() -> Option<PathBuf> {
    if let Some(xdg) = nonempty_env("XDG_CACHE_HOME") {
        return Some(PathBuf::from(xdg));
    }
    platform_cache_dir()
}

fn nonempty_env(name: &str) -> Option<OsString> {
    std::env::var_os(name).filter(|value| !value.is_empty())
}

#[cfg(target_os = "windows")]
fn platform_cache_dir() -> Option<PathBuf> {
    windows_cache_dir(
        nonempty_env("LOCALAPPDATA"),
        nonempty_env("USERPROFILE"),
        nonempty_env("HOME"),
    )
}

#[cfg(target_os = "windows")]
fn windows_cache_dir(
    local_app_data: Option<OsString>,
    user_profile: Option<OsString>,
    home: Option<OsString>,
) -> Option<PathBuf> {
    local_app_data
        .map(PathBuf::from)
        .or_else(|| user_profile.map(|path| PathBuf::from(path).join("AppData").join("Local")))
        .or_else(|| home.map(|path| PathBuf::from(path).join(".cache")))
}

#[cfg(target_os = "macos")]
fn platform_cache_dir() -> Option<PathBuf> {
    Some(PathBuf::from(nonempty_env("HOME")?).join("Library").join("Caches"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn platform_cache_dir() -> Option<PathBuf> {
    Some(PathBuf::from(nonempty_env("HOME")?).join(".cache"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_file(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write");
    }

    fn controls_config(
        policy: CachePolicy,
        snapshot_path: PathBuf,
        read_controls: bool,
    ) -> OpenFixture {
        OpenFixture {
            scan: ScanConfig { read_controls, ..ScanConfig::default() },
            cache_path: Some(snapshot_path),
            policy,
            ..OpenFixture::default()
        }
    }

    fn seed_controls_snapshot(root: &Path, snapshot_path: PathBuf) {
        let seed = controls_config(CachePolicy::Auto, snapshot_path, true);
        let (index, report) = open_fixture(root, &seed).expect("seed controls-on snapshot");
        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert!(
            !index.controls().expect("control state observed").is_empty(),
            "the fixture must retain a control source"
        );
    }

    /// A default `open` observes control state, so its index answers ignore questions
    /// exactly. A request that turns observation off reads no control file, so a control
    /// line no index could retain does not end it, and its index says it cannot classify
    /// ignored entries rather than calling every entry unignored.
    #[test]
    fn a_default_open_answers_ignore_questions_and_an_opt_out_refuses_them() {
        let root = tempfile::tempdir().expect("tempdir");
        write_file(&root.path().join(".gitignore"), b"*.log\n");
        write_file(&root.path().join("debug.log"), b"ignored");
        write_file(&root.path().join("keep.rs"), b"kept");
        let uncached = OpenFixture { policy: CachePolicy::Off, ..OpenFixture::default() };
        let opted_out = OpenFixture {
            scan: ScanConfig { read_controls: false, ..ScanConfig::default() },
            ..uncached.clone()
        };

        let (index, report) = open_fixture(root.path(), &uncached).expect("default open");
        assert!(report.is_complete(), "{:?}", report.errors());
        assert!(index.observes_controls());
        assert_eq!(index.is_ignored(Path::new("debug.log")).ok(), Some(Some(true)));
        assert_eq!(index.is_ignored(Path::new("keep.rs")).ok(), Some(Some(false)));
        assert_eq!(index.is_ignored(Path::new("absent")).ok(), Some(None));
        assert!(
            index
                .controls()
                .is_ok_and(|controls| controls.source_is(Path::new(".gitignore"), b"*.log\n"))
        );
        assert_eq!(index.partition_total().expect("control state observed").unignored.files, 2);

        let mut oversized = vec![b'x'; crate::control::DEFAULT_CONTROL_LINE_LIMIT + 1];
        oversized.extend_from_slice(b"\n*.log\n");
        write_file(&root.path().join(".gitignore"), &oversized);
        let (index, report) =
            open_fixture(root.path(), &uncached).expect("a refused control ends nothing");
        assert!(report.is_complete(), "{:?}", report.errors());
        let crate::control::ControlCoverage::Observed(coverage) = index.control_coverage() else {
            panic!("a default open reads the control file the opt-out skips");
        };
        assert_eq!((coverage.applied, coverage.refused), (0, 1));

        let (index, report) = open_fixture(root.path(), &opted_out)
            .expect("an opted-out open reads no control line, however long");
        assert!(report.is_complete(), "{:?}", report.errors());
        assert!(!index.observes_controls());
        for path in ["debug.log", "keep.rs", "absent"] {
            assert!(
                matches!(index.is_ignored(Path::new(path)), Err(Error::ControlStateNotObserved)),
                "{path} must not be called unignored by an index that read no rule"
            );
        }
        assert!(matches!(index.controls(), Err(Error::ControlStateNotObserved)));
        assert!(matches!(index.partition_total(), Err(Error::ControlStateNotObserved)));
        assert_eq!(index.total().files, 3);
    }

    /// Lifting a control limit scans cold once, and the snapshot it writes then serves
    /// those limits warm, with the coverage it recorded; the other limits' request misses.
    #[test]
    fn a_snapshot_serves_only_the_control_limits_it_was_taken_under() {
        let root = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        let mut long_line = b"*.log\n".to_vec();
        long_line.extend(std::iter::repeat_n(b'x', crate::control::DEFAULT_CONTROL_LINE_LIMIT + 1));
        write_file(&root.path().join(".gitignore"), &long_line);
        write_file(&root.path().join("debug.log"), b"ignored");
        let default = controls_config(CachePolicy::Auto, snapshot_path.clone(), true);
        let lifted_limits = crate::control::ControlLimits {
            line_limit: None,
            ..crate::control::ControlLimits::default()
        };
        let lifted = OpenFixture {
            scan: ScanConfig { control_limits: lifted_limits, ..default.scan.clone() },
            ..controls_config(CachePolicy::Auto, snapshot_path, true)
        };
        let refused = |index: &Index| match index.control_coverage() {
            crate::control::ControlCoverage::Observed(coverage) => coverage.refused,
            crate::control::ControlCoverage::NotObserved => panic!("observed"),
        };

        let (index, report) = open_fixture(root.path(), &default).expect("default limits");
        assert_eq!((report.path_taken, refused(&index)), (OpenPath::ColdScan, 1));
        let (index, report) = open_fixture(root.path(), &default).expect("default limits again");
        assert_eq!((report.path_taken, refused(&index)), (OpenPath::WarmRevalidate, 1));

        let (index, report) = open_fixture(root.path(), &lifted).expect("lifted line limit");
        assert_eq!((report.path_taken, refused(&index)), (OpenPath::ColdScan, 0));
        assert_eq!(index.is_ignored(Path::new("debug.log")).ok(), Some(Some(true)));
        let (index, report) = open_fixture(root.path(), &lifted).expect("lifted line limit again");
        assert_eq!((report.path_taken, refused(&index)), (OpenPath::WarmRevalidate, 0));

        let (_, report) = open_fixture(root.path(), &default).expect("back to the default");
        assert_eq!(report.path_taken, OpenPath::ColdScan);
    }

    #[test]
    fn controls_on_snapshot_projects_to_controls_off_auto_open_without_replacing_it() {
        let root = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&root.path().join(".gitignore"), b"ignored.log\n");
        write_file(&root.path().join("ignored.log"), b"ignored");
        seed_controls_snapshot(root.path(), snapshot_path.clone());
        let stronger = fs::read(&snapshot_path).expect("stronger snapshot");
        write_file(&root.path().join("new.txt"), b"new");

        let controls_off = controls_config(CachePolicy::Auto, snapshot_path.clone(), false);
        let (index, report) =
            open_fixture(root.path(), &controls_off).expect("projected warm open");

        assert_eq!(report.path_taken, OpenPath::WarmRevalidate);
        assert!(report.projected);
        assert_eq!(index.scope(), controls_off.scan.scope());
        assert!(matches!(index.controls(), Err(Error::ControlStateNotObserved)));
        assert!(matches!(index.path_state(Path::new("new.txt")), PathState::Present { .. }));
        assert_eq!(fs::read(snapshot_path).expect("snapshot retained"), stronger);
    }

    /// A cache-only open refused for the control limits names them and what recovers.
    ///
    /// Both scopes observe control state and differ only in their ignore-rules identity,
    /// which is a hash: without naming the limits, the caller is told "a different scan
    /// scope" about a request whose only difference is a limit they chose.
    #[test]
    fn a_cache_only_open_after_a_limit_change_names_the_limits_that_differ() {
        let root = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&root.path().join(".gitignore"), b"ignored.log\n");
        write_file(&root.path().join("ignored.log"), b"ignored");
        seed_controls_snapshot(root.path(), snapshot_path.clone());

        let lifted = crate::control::ControlLimits {
            line_limit: None,
            ..crate::control::ControlLimits::default()
        };
        let mut wanted = controls_config(CachePolicy::Only, snapshot_path, true);
        wanted.scan.control_limits = lifted;
        let Err(Error::Snapshot(message)) = open_fixture(root.path(), &wanted) else {
            panic!("a snapshot taken under other limits must not serve a cache-only open");
        };
        assert!(
            message.contains("line limit 16 KiB, where this request asks for all"),
            "names the limit that differs and both values: {message}"
        );
        assert!(!message.contains("budget"), "the budget is unchanged: {message}");
        assert!(!message.contains("different scan scope"), "says which scope differs: {message}");
        assert!(
            message.contains("repeat the request with the snapshot's limits"),
            "names the remedy: {message}"
        );
        assert!(message.contains("`auto`"), "names the other remedy: {message}");
    }

    #[test]
    fn controls_on_snapshot_projects_to_controls_off_cache_only_open() {
        let root = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&root.path().join(".gitignore"), b"ignored.log\n");
        write_file(&root.path().join("ignored.log"), b"ignored");
        seed_controls_snapshot(root.path(), snapshot_path.clone());

        let controls_off = controls_config(CachePolicy::Only, snapshot_path, false);
        let (index, report) =
            open_fixture(root.path(), &controls_off).expect("projected cache-only open");
        assert_eq!(report.path_taken, OpenPath::CacheOnly);
        assert!(report.projected);
        assert_eq!(index.scope(), controls_off.scan.scope());
        assert!(matches!(index.controls(), Err(Error::ControlStateNotObserved)));
    }

    /// Supplied rules reach the answer, and invalidate a snapshot taken under others.
    ///
    /// The end-to-end property behind the registry: a consumer whose taxonomy differs
    /// from this repository's classifies its own way without rebuilding the crate, and a
    /// snapshot written under one taxonomy is never served under another. The second half
    /// is the one that would fail silently -- the entry counts and byte totals are
    /// identical either way, so a stale snapshot looks entirely correct.
    #[test]
    fn supplied_type_rules_change_the_answer_and_invalidate_the_snapshot() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("main.rs"), b"fn main() {}\n");

        let default_config = OpenFixture {
            cache_path: Some(snapshot_path.clone()),
            analysis: content::AnalysisRequest {
                profile: content::AnalysisSet::NONE.with_lines(),
                ..content::AnalysisRequest::default()
            },
            ..OpenFixture::default()
        };
        let (index, _) = open_fixture(dir.path(), &default_config).expect("default open");
        assert_eq!(index.classify(Path::new("main.rs")).file_type.as_str(), "rust");
        drop(index);

        let mine = std::sync::Arc::new(
            classify::TypeRegistry::from_manifest(
                "[[kind]]\nid = \"notes\"\nfamily = \"prose\"\nextensions = [\"rs\"]\n",
            )
            .expect("a minimal manifest"),
        );
        let custom_config = OpenFixture {
            scan: scan::ScanConfig::default().with_types(mine.clone()),
            ..default_config.clone()
        };

        assert_ne!(
            custom_config.scan.scope(),
            default_config.scan.scope(),
            "different rules are a different scan scope"
        );

        let (index, report) = open_fixture(dir.path(), &custom_config).expect("custom open");
        assert_eq!(
            report.path_taken,
            OpenPath::ColdScan,
            "the snapshot was written under other rules and must not be reused"
        );
        assert_eq!(index.classify(Path::new("main.rs")).file_type.as_str(), "notes");
        assert_eq!(index.types().fingerprint(), mine.fingerprint());
        let content = index
            .content()
            .and_then(|content| content.file(Path::new("main.rs")))
            .expect("custom analysis record");
        assert_eq!(content.detection.file_type.as_str(), "notes");
        assert_eq!(
            index
                .content()
                .and_then(content::ContentIndex::provenance)
                .expect("content provenance")
                .type_rules_fingerprint,
            mine.fingerprint()
        );

        // And the snapshot the custom run wrote is reusable by a run under the same rules.
        let (_, report) = open_fixture(dir.path(), &custom_config).expect("second custom open");
        assert_eq!(report.path_taken, OpenPath::WarmRevalidate, "same rules, same snapshot");
        assert_eq!(report.content_cache.hits, 1, "the matching sidecar is reusable");
        assert_eq!(report.analysis.expect("analysis report").candidates, 0);

        assert!(
            snapshot::load(&snapshot_path).expect("default-registry load").is_none(),
            "a direct default-registry load must reject a custom-registry snapshot"
        );
        let loaded = snapshot::load_with_types(&snapshot_path, mine)
            .expect("custom-registry load")
            .expect("the matching custom registry makes the snapshot usable");
        assert_eq!(loaded.classify(Path::new("main.rs")).file_type.as_str(), "notes");
    }

    /// The behaviour table from the design, asserted rather than described.
    #[test]
    fn each_cache_policy_reads_scans_and_writes_as_documented() {
        for (policy, expect_write) in [
            (CachePolicy::Auto, true),
            (CachePolicy::Refresh, true),
            (CachePolicy::ReadOnly, false),
            (CachePolicy::Off, false),
        ] {
            let dir = tempfile::tempdir().expect("tempdir");
            let cache = tempfile::tempdir().expect("cache dir");
            let snapshot_path = cache.path().join("snap.fdu");
            write_file(&dir.path().join("a.txt"), b"hello");

            let config = OpenFixture {
                cache_path: Some(snapshot_path.clone()),
                policy,
                ..OpenFixture::default()
            };
            let (index, report) = open_fixture(dir.path(), &config).expect("open");

            assert_eq!(index.total().files, 1, "{policy:?} lost an entry");
            assert_eq!(report.path_taken, OpenPath::ColdScan, "{policy:?} without a snapshot");
            assert_eq!(
                snapshot_path.exists(),
                expect_write,
                "{policy:?} wrote a snapshot: {}",
                snapshot_path.exists()
            );
        }
    }

    #[test]
    fn read_only_takes_the_warm_path_without_rewriting_the_snapshot() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        // Seed a snapshot with auto, then read it without leaving a trace.
        let seed = OpenFixture {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenFixture::default()
        };
        open_fixture(dir.path(), &seed).expect("seed");
        let before = fs::metadata(&snapshot_path).expect("snapshot exists").len();

        let read_only = OpenFixture { policy: CachePolicy::ReadOnly, ..seed };
        let (index, report) = open_fixture(dir.path(), &read_only).expect("warm open");
        assert_eq!(report.path_taken, OpenPath::WarmRevalidate);
        assert_eq!(index.total().files, 1);
        assert_eq!(fs::metadata(&snapshot_path).expect("still there").len(), before);
    }

    /// A verified warm open over an unchanged tree rewrote a byte-identical snapshot on
    /// every run, paying a full index clone, encode, and write to reproduce the file it
    /// had just read.  The bytes are the assertion: if a future change makes an
    /// unchanged reconciliation produce different serialized state, this fails loudly
    /// rather than letting the skip silently drop it.
    #[test]
    fn an_unchanged_warm_open_leaves_the_snapshot_alone_and_a_changed_one_rewrites_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");
        write_file(&dir.path().join("sub/b.txt"), b"world");

        let auto = OpenFixture {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenFixture::default()
        };
        open_fixture(dir.path(), &auto).expect("seed");
        let seeded = fs::read(&snapshot_path).expect("seeded snapshot");

        let (index, report) = open_fixture(dir.path(), &auto).expect("warm open");
        assert_eq!(report.path_taken, OpenPath::WarmRevalidate);
        assert_eq!(index.total().files, 2);
        assert_eq!(
            fs::read(&snapshot_path).expect("snapshot still there"),
            seeded,
            "an unchanged warm open must not rewrite the snapshot"
        );

        write_file(&dir.path().join("sub/c.txt"), b"new file");
        let (index, report) = open_fixture(dir.path(), &auto).expect("warm open after a change");
        assert_eq!(report.path_taken, OpenPath::WarmRevalidate);
        assert_eq!(index.total().files, 3);
        let after_add = fs::read(&snapshot_path).expect("rewritten snapshot");
        assert_ne!(after_add, seeded, "a warm open that found a new file must persist it");

        fs::remove_file(dir.path().join("sub/c.txt")).expect("remove");
        let (index, _) = open_fixture(dir.path(), &auto).expect("warm open after a removal");
        assert_eq!(index.total().files, 2);
        assert_ne!(
            fs::read(&snapshot_path).expect("rewritten snapshot"),
            after_add,
            "a warm open that found a removal must persist it"
        );

        // The skip must leave a snapshot a later cache-only open can still serve.
        let cache_only = OpenFixture { policy: CachePolicy::Only, ..auto };
        let (restored, _) = open_fixture(dir.path(), &cache_only).expect("cache-only open");
        assert_eq!(restored.total().files, 2);
    }

    #[test]
    fn refresh_ignores_an_existing_snapshot_and_rewrites_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        let auto = OpenFixture {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenFixture::default()
        };
        open_fixture(dir.path(), &auto).expect("seed");

        // A second auto open would be warm; refresh must scan cold anyway, which is what
        // makes it usable as a benchmark control.
        let refresh = OpenFixture { policy: CachePolicy::Refresh, ..auto };
        let (_, report) = open_fixture(dir.path(), &refresh).expect("refresh open");
        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert!(snapshot_path.exists());
    }

    #[test]
    fn cache_only_answers_from_the_snapshot_without_touching_the_tree() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        let auto = OpenFixture {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenFixture::default()
        };
        open_fixture(dir.path(), &auto).expect("seed");

        // Change the tree after the snapshot was taken. A cache-only answer must report
        // what it has, not what is there now — and its freshness must say so.
        write_file(&dir.path().join("b.txt"), b"new file");

        let only = OpenFixture { policy: CachePolicy::Only, ..auto };
        let (index, report) = open_fixture(dir.path(), &only).expect("cache-only open");
        assert_eq!(report.path_taken, OpenPath::CacheOnly);
        assert_eq!(index.total().files, 1, "the new file must not appear");
        assert_ne!(index.freshness(), Freshness::Fresh, "a stale answer must not claim currency");
    }

    #[test]
    fn cache_only_fails_closed_when_no_snapshot_is_usable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&dir.path().join("a.txt"), b"hello");

        // Guessing a scan here would make the fast path unpredictable: sometimes instant,
        // sometimes a full walk, with nothing in the output to say which happened.
        let only = OpenFixture {
            cache_path: Some(cache.path().join("absent.fdu")),
            policy: CachePolicy::Only,
            ..OpenFixture::default()
        };
        let Err(Error::Snapshot(message)) = open_fixture(dir.path(), &only) else {
            panic!("a cache-only open with no snapshot must fail");
        };
        // A diagnostic names its remedy: `only` is the one policy that cannot recover, so
        // the message says which policy can.
        assert!(message.contains("`auto`"), "names the remedy: {message}");
    }

    #[test]
    fn content_sidecar_skips_unchanged_reads_and_serves_cache_only() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("notes.md"), b"one two\n");
        let analysis = content::AnalysisRequest {
            profile: content::AnalysisSet::NONE.with_lines(),
            ..content::AnalysisRequest::default()
        };
        let auto = OpenFixture {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            analysis,
            ..OpenFixture::default()
        };

        let (first, first_report) = open_fixture(dir.path(), &auto).expect("cold analyzed open");
        assert_eq!(first_report.analysis.expect("analysis").lines.analyzed, 1);
        assert_eq!(
            first.content_rollup(Path::new("")).expect("content").total.lines.metrics.raw_words,
            2
        );
        assert!(content::content_cache_path(&snapshot_path).exists());

        let (_, warm_report) = open_fixture(dir.path(), &auto).expect("warm analyzed open");
        assert_eq!(warm_report.content_cache.hits, 1);
        assert_eq!(warm_report.content_cache.bytes, 8);
        assert_eq!(warm_report.analysis.expect("analysis").candidates, 0);

        fs::remove_file(dir.path().join("notes.md")).expect("remove source");
        let only = OpenFixture { policy: CachePolicy::Only, ..auto };
        let (cached, cached_report) = open_fixture(dir.path(), &only).expect("cache-only content");
        assert_eq!(cached_report.content_cache.hits, 1);
        assert_eq!(cached_report.content_cache.bytes, 8);
        assert_eq!(
            cached.content_rollup(Path::new("")).expect("content").total.lines.metrics.raw_words,
            2
        );
    }

    #[test]
    fn cached_coverage_exclusions_remain_visible_without_making_the_run_partial() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("invalid.txt"), b"valid prefix\xff");
        let auto = OpenFixture {
            cache_path: Some(snapshot_path),
            policy: CachePolicy::Auto,
            analysis: content::AnalysisRequest {
                profile: content::AnalysisSet::NONE.with_lines(),
                ..content::AnalysisRequest::default()
            },
            ..OpenFixture::default()
        };

        let (_, cold_report) = open_fixture(dir.path(), &auto).expect("cold analyzed open");
        assert!(cold_report.is_complete());
        assert_eq!(cold_report.analysis.expect("analysis").lines.invalid_utf8, 1);
        assert!(cold_report.error_messages().is_empty());

        let (_, warm_report) = open_fixture(dir.path(), &auto).expect("warm analyzed open");
        assert_eq!(warm_report.content_cache.hits, 1);
        assert_eq!(warm_report.content_cache.coverage_exclusions, 1);
        assert_eq!(warm_report.analysis.expect("analysis").candidates, 0);
        assert!(warm_report.is_complete());
        assert!(warm_report.error_messages().is_empty());

        let only = OpenFixture { policy: CachePolicy::Only, ..auto };
        let (_, cached_report) = open_fixture(dir.path(), &only).expect("cache-only analyzed open");
        assert_eq!(cached_report.content_cache.coverage_exclusions, 1);
        assert!(cached_report.is_complete());
        assert!(cached_report.error_messages().is_empty());
    }

    #[test]
    fn cache_only_analysis_fails_closed_without_its_sidecar() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("notes.md"), b"one two\n");
        let metadata_only = OpenFixture {
            cache_path: Some(snapshot_path),
            policy: CachePolicy::Auto,
            ..OpenFixture::default()
        };
        open_fixture(dir.path(), &metadata_only).expect("seed metadata");

        let only = OpenFixture {
            policy: CachePolicy::Only,
            analysis: content::AnalysisRequest {
                profile: content::AnalysisSet::NONE.with_lines(),
                ..content::AnalysisRequest::default()
            },
            ..metadata_only
        };
        assert!(matches!(open_fixture(dir.path(), &only), Err(Error::Snapshot(_))));

        // A sidecar of another analyzer set is not this request's, whether it is wider or
        // narrower: cache-only fails closed rather than answering with the stored set.
        let with_set = |policy, profile| OpenFixture {
            policy,
            analysis: content::AnalysisRequest { profile, ..content::AnalysisRequest::default() },
            ..only.clone()
        };
        let lines = content::AnalysisSet::NONE.with_lines();
        for (stored, wanted) in
            [(content::AnalysisSet::ALL, lines), (lines, content::AnalysisSet::ALL)]
        {
            open_fixture(dir.path(), &with_set(CachePolicy::Auto, stored))
                .expect("write a sidecar");
            let refused = open_fixture(dir.path(), &with_set(CachePolicy::Only, wanted));
            assert!(
                matches!(refused, Err(Error::Snapshot(_))),
                "a {stored:?} sidecar must not serve a cache-only {wanted:?} request"
            );
        }

        open_fixture(dir.path(), &with_set(CachePolicy::Auto, lines))
            .expect("write the lines sidecar");
        let (cached, report) = open_fixture(dir.path(), &only).expect("restore the analyzed state");
        assert!(report.content_cache.usable);
        assert_eq!(report.content_cache.hits, 1, "one record per candidate is complete");
        assert_eq!(
            report.content_cache.candidates, report.content_cache.hits,
            "completeness uses the count restore already walked"
        );
        assert_eq!(cached.content_set(), lines);
    }

    #[test]
    fn cache_only_analysis_fails_closed_when_the_sidecar_is_incomplete() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("notes.md"), b"one two\n");
        let analysis = content::AnalysisRequest {
            profile: content::AnalysisSet::NONE.with_lines(),
            ..content::AnalysisRequest::default()
        };
        let auto = OpenFixture {
            cache_path: Some(snapshot_path),
            policy: CachePolicy::Auto,
            analysis,
            ..OpenFixture::default()
        };
        open_fixture(dir.path(), &auto).expect("seed one analyzed file");

        // A later metadata-only pass widens the snapshot without rewriting the sidecar,
        // so cache-only analysis must refuse rather than report a partial content answer.
        write_file(&dir.path().join("extra.txt"), b"three\n");
        let metadata_only =
            OpenFixture { analysis: content::AnalysisRequest::default(), ..auto.clone() };
        open_fixture(dir.path(), &metadata_only).expect("widen the snapshot");

        let only = OpenFixture { policy: CachePolicy::Only, ..auto };
        assert!(
            matches!(open_fixture(dir.path(), &only), Err(Error::Snapshot(_))),
            "a one-record sidecar must not serve a two-file cache-only analysis request"
        );
    }

    #[cfg(unix)]
    #[test]
    fn cache_only_serves_checksummed_native_non_utf8_names() {
        use crate::execution::{RunFacts, SaveTargets};
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        let root = dir.path().canonicalize().expect("canonical root");
        let native = PathBuf::from(OsString::from_vec(vec![b'n', 0x80]));
        let mut index = Index::new(&root);
        index.apply_ok(&Observation::new(vec![
            Op::Upsert {
                path: PathBuf::from("ok.txt"),
                kind: EntryKind::File,
                attrs: Attrs {
                    size: 1,
                    allocated: 512,
                    mtime_ns: 1,
                    ctime_ns: 1,
                    inode: 1,
                    dev: 1,
                },
            },
            Op::Upsert {
                path: native.clone(),
                kind: EntryKind::File,
                attrs: Attrs {
                    size: 2,
                    allocated: 512,
                    mtime_ns: 2,
                    ctime_ns: 2,
                    inode: 2,
                    dev: 1,
                },
            },
        ]));
        snapshot::save(&index, &snapshot_path).expect("save native names");

        let only = OpenFixture {
            cache_path: Some(snapshot_path),
            policy: CachePolicy::Only,
            ..OpenFixture::default()
        };
        let (cached, report) = open_fixture(&root, &only).expect("cache-only native name");
        assert_eq!(cached.total().files, 2);
        assert!(cached.lookup(&native).is_some());
        assert!(report.is_complete());
    }

    #[test]
    fn cache_only_empty_analysis_still_requires_a_usable_sidecar() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        let metadata_only = OpenFixture {
            cache_path: Some(snapshot_path),
            policy: CachePolicy::Auto,
            ..OpenFixture::default()
        };
        open_fixture(dir.path(), &metadata_only).expect("seed empty metadata");

        let only = OpenFixture {
            policy: CachePolicy::Only,
            analysis: content::AnalysisRequest {
                profile: content::AnalysisSet::NONE.with_lines(),
                ..content::AnalysisRequest::default()
            },
            ..metadata_only
        };
        assert!(matches!(open_fixture(dir.path(), &only), Err(Error::Snapshot(_))));
    }

    #[test]
    fn a_snapshot_for_another_root_is_treated_as_absent() {
        let one = tempfile::tempdir().expect("tempdir");
        let two = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&one.path().join("a.txt"), b"hello");
        write_file(&two.path().join("b.txt"), b"other tree");

        let config = OpenFixture {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenFixture::default()
        };
        open_fixture(one.path(), &config).expect("seed from the first root");

        // Reading another tree's snapshot would be worse than a cache miss.
        let (index, report) = open_fixture(two.path(), &config).expect("second root");
        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert_eq!(index.total().files, 1);
        assert_eq!(index.root_path(), two.path().canonicalize().expect("canonical").as_path());
    }

    #[test]
    fn open_without_a_cache_always_scans_cold() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(&dir.path().join("a.txt"), b"hello");

        let (index, report) = open_fixture(dir.path(), &OpenFixture::default()).expect("open");
        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert_eq!(index.total().files, 1);
    }

    #[test]
    fn second_open_takes_the_warm_path_and_stays_correct() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&dir.path().join("a.txt"), b"hello");
        write_file(&dir.path().join("src/main.rs"), b"fn main() {}");

        let config = OpenFixture {
            cache_path: Some(cache.path().join("snap.fdu")),
            policy: CachePolicy::Auto,
            ..OpenFixture::default()
        };

        let (first, first_report) = open_fixture(dir.path(), &config).expect("cold open");
        assert_eq!(first_report.path_taken, OpenPath::ColdScan);
        assert_eq!(first.total().files, 2);

        // Change the tree between opens: the warm path must notice.
        write_file(&dir.path().join("added.md"), b"new");
        fs::remove_file(dir.path().join("a.txt")).expect("remove");

        let (second, second_report) = open_fixture(dir.path(), &config).expect("warm open");
        assert_eq!(second_report.path_taken, OpenPath::WarmRevalidate);
        assert_eq!(second.total().files, 2);
        assert!(second.lookup(Path::new("added.md")).is_some());
        assert!(second.lookup(Path::new("a.txt")).is_none());
    }

    #[test]
    fn a_snapshot_from_another_root_is_ignored() {
        let a = tempfile::tempdir().expect("tempdir a");
        let b = tempfile::tempdir().expect("tempdir b");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&a.path().join("only-in-a.txt"), b"x");
        write_file(&b.path().join("only-in-b.txt"), b"y");

        let cache_path = cache.path().join("snap.fdu");
        let config = OpenFixture {
            cache_path: Some(cache_path),
            policy: CachePolicy::Auto,
            ..OpenFixture::default()
        };

        open_fixture(a.path(), &config).expect("open a");
        let (index, report) = open_fixture(b.path(), &config).expect("open b");

        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert!(index.lookup(Path::new("only-in-b.txt")).is_some());
        assert!(index.lookup(Path::new("only-in-a.txt")).is_none());
    }

    #[test]
    fn snapshot_scope_mismatch_forces_a_cold_scan() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&dir.path().join("top.txt"), b"top");
        write_file(&dir.path().join("deep/nested.txt"), b"nested");

        let cache_path = cache.path().join("snap.fdu");
        let full = OpenFixture {
            cache_path: Some(cache_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenFixture::default()
        };
        open_fixture(dir.path(), &full).expect("full open");

        let shallow = OpenFixture {
            scan: ScanConfig { max_depth: Some(1), ..ScanConfig::default() },
            cache_path: Some(cache_path),
            policy: CachePolicy::ReadOnly,
            analysis: content::AnalysisRequest::default(),
        };
        let (index, report) = open_fixture(dir.path(), &shallow).expect("shallow open");

        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert!(index.lookup(Path::new("deep")).is_some());
        assert!(index.lookup(Path::new("deep/nested.txt")).is_none());
    }

    #[test]
    fn admission_scope_mismatch_cannot_reinterpret_a_snapshot() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&dir.path().join(".hidden"), b"hidden");
        let cache_path = cache.path().join("snap.fdu");
        let seed = OpenFixture {
            cache_path: Some(cache_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenFixture::default()
        };
        open_fixture(dir.path(), &seed).expect("seed snapshot");

        let changed_scopes = [
            ScanConfig {
                hidden: Some(std::sync::Arc::new(
                    HiddenPolicy::prune_hidden::<[&str; 0], &str>([]),
                )),
                ..ScanConfig::default()
            },
            ScanConfig { exclude_special: true, ..ScanConfig::default() },
        ];
        for scan in changed_scopes {
            let only = OpenFixture {
                scan,
                cache_path: Some(cache_path.clone()),
                policy: CachePolicy::Only,
                analysis: content::AnalysisRequest::default(),
            };
            assert!(matches!(open_fixture(dir.path(), &only), Err(Error::Snapshot(_))));
        }
    }

    #[test]
    fn operational_batch_size_does_not_invalidate_a_snapshot() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&dir.path().join("a.txt"), b"a");
        let cache_path = cache.path().join("snap.fdu");

        let first = OpenFixture {
            scan: ScanConfig { batch_size: 1, ..ScanConfig::default() },
            cache_path: Some(cache_path.clone()),
            policy: CachePolicy::Auto,
            analysis: content::AnalysisRequest::default(),
        };
        open_fixture(dir.path(), &first).expect("first open");

        let second = OpenFixture {
            scan: ScanConfig { batch_size: 17, ..ScanConfig::default() },
            cache_path: Some(cache_path),
            policy: CachePolicy::ReadOnly,
            analysis: content::AnalysisRequest::default(),
        };
        let (_, report) = open_fixture(dir.path(), &second).expect("second open");
        assert_eq!(report.path_taken, OpenPath::WarmRevalidate);
    }

    #[test]
    fn cache_paths_differ_per_root() {
        let a = tempfile::tempdir().expect("tempdir a");
        let b = tempfile::tempdir().expect("tempdir b");
        let (Some(pa), Some(pb)) = (default_cache_path(a.path()), default_cache_path(b.path()))
        else {
            return; // No HOME in this environment; nothing to assert.
        };
        assert_ne!(pa, pb);
        assert_eq!(pa, default_cache_path(a.path()).expect("stable"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_cache_discovery_prefers_native_locations() {
        let local = OsString::from(r"C:\Users\tester\AppData\Local");
        let profile = OsString::from(r"D:\Profile");
        let home = OsString::from(r"E:\Home");

        assert_eq!(
            windows_cache_dir(Some(local.clone()), Some(profile.clone()), Some(home.clone())),
            Some(PathBuf::from(local))
        );
        assert_eq!(
            windows_cache_dir(None, Some(profile.clone()), Some(home.clone())),
            Some(PathBuf::from(profile).join("AppData").join("Local"))
        );
        assert_eq!(
            windows_cache_dir(None, None, Some(home.clone())),
            Some(PathBuf::from(home).join(".cache"))
        );
        assert_eq!(windows_cache_dir(None, None, None), None);
    }
}

#[cfg(test)]
mod save_tests {
    use super::*;
    use std::fs;

    fn write_file(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write");
    }

    fn config(snapshot_path: &Path, policy: CachePolicy) -> OpenFixture {
        OpenFixture {
            cache_path: Some(snapshot_path.to_path_buf()),
            policy,
            ..OpenFixture::default()
        }
    }

    #[test]
    fn a_pending_save_completes_when_joined() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        let (_index, _report, pending) =
            open_fixture_with_pending_save(dir.path(), &config(&snapshot_path, CachePolicy::Auto))
                .expect("open");
        pending.join().expect("save succeeds");
        assert!(snapshot_path.exists(), "a joined save must have landed");
    }

    #[test]
    fn a_dropped_save_still_lands() {
        // Dropping without joining is a caller mistake, not a reason to lose the write:
        // the next run would otherwise pay for a cold scan this one already did.
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        {
            let (_index, _report, _pending) = open_fixture_with_pending_save(
                dir.path(),
                &config(&snapshot_path, CachePolicy::Auto),
            )
            .expect("open");
        }
        assert!(snapshot_path.exists());
    }

    #[test]
    #[cfg(unix)]
    fn a_partial_scan_leaves_the_previous_snapshot_alone() {
        // Writing a partial view would serve it as fact on the next run, and the
        // existing complete snapshot is better than that.
        use std::os::unix::fs::PermissionsExt;

        if !crate::test_support::require_permission_bits() {
            return;
        }

        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        let settings = config(&snapshot_path, CachePolicy::Auto);
        open_fixture(dir.path(), &settings).expect("seed a complete snapshot");
        let complete_len = fs::metadata(&snapshot_path).expect("exists").len();

        let denied = dir.path().join("denied");
        fs::create_dir(&denied).expect("create");
        write_file(&denied.join("hidden.txt"), b"hidden");
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).expect("deny");

        let opened = open_fixture(dir.path(), &settings);
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o700)).expect("restore");
        let (_index, report) = opened.expect("partial open still returns a result");

        assert!(!report.is_complete(), "the scan should be partial");
        assert_eq!(
            fs::metadata(&snapshot_path).expect("still there").len(),
            complete_len,
            "a partial scan must not overwrite a complete snapshot"
        );
    }

    /// Each tier is written by its own rule: a partial pass writes no snapshot, because an
    /// absent entry would change totals, but may write every verified content record beside
    /// a stored snapshot of the same entry identity. It still writes nothing under another
    /// identity, where replacing the sidecar would separate it from the snapshot it names.
    #[test]
    #[cfg(unix)]
    fn a_partial_scan_writes_only_verified_content_for_the_stored_entry_tier() {
        use std::os::unix::fs::PermissionsExt;

        if !crate::test_support::require_permission_bits() {
            return;
        }

        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        let sidecar_path = content::content_cache_path(&snapshot_path);
        write_file(&dir.path().join("notes.md"), b"one two\n");
        write_file(&dir.path().join("locked/old.md"), b"three\n");
        let analysis = content::AnalysisRequest {
            profile: content::AnalysisSet::NONE.with_lines(),
            ..content::AnalysisRequest::default()
        };
        let auto = OpenFixture {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            analysis,
            ..OpenFixture::default()
        };
        let (_, seeded) = open_fixture(dir.path(), &auto).expect("seed both tiers");
        assert!(seeded.is_complete());
        let snapshot_before = fs::read(&snapshot_path).expect("a snapshot");

        // Change a file the next pass can verify, and lock the directory holding another.
        write_file(&dir.path().join("notes.md"), b"one two three\n");
        let locked = dir.path().join("locked");
        let run = |config: &OpenFixture| {
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("deny");
            let opened = open_fixture(dir.path(), config);
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o700)).expect("restore");
            let (_, report) = opened.expect("a partial open still answers");
            assert!(!report.is_complete(), "the pass should be partial");
        };

        // Under another entry identity: neither tier is written.
        let sidecar_before = fs::read(&sidecar_path).expect("a sidecar");
        let other_scope = OpenFixture {
            scan: ScanConfig { max_depth: Some(8), ..ScanConfig::default() },
            ..auto.clone()
        };
        run(&other_scope);
        assert_eq!(fs::read(&snapshot_path).expect("snapshot"), snapshot_before);
        assert_eq!(
            fs::read(&sidecar_path).expect("sidecar"),
            sidecar_before,
            "a partial run under another entry identity must not evict the paired sidecar"
        );

        // Under the stored snapshot's identity, the snapshot stays whole while the sidecar
        // keeps only content whose entry facts this pass verified.
        run(&auto);
        assert_eq!(
            fs::read(&snapshot_path).expect("snapshot"),
            snapshot_before,
            "a partial scan must not overwrite a complete snapshot"
        );
        assert_ne!(fs::read(&sidecar_path).expect("sidecar"), sidecar_before);
        // The verified changed file is reusable; no record survives under the directory
        // this pass could not list.
        let (mut fresh, _) =
            scan::scan_into_index(dir.path(), &ScanConfig::default()).expect("scan");
        let wanted = fresh.content_identity(analysis.profile);
        let loaded = content::load_content_cache(&mut fresh, &wanted, &sidecar_path).expect("load");
        assert_eq!((loaded.usable, loaded.hits, loaded.stale), (true, 1, 0), "{loaded:?}");
        let content = fresh.content().expect("content");
        assert!(content.file(Path::new("locked/old.md")).is_none());
        assert_eq!(
            content
                .file(Path::new("notes.md"))
                .expect("verified record")
                .lines
                .value()
                .expect("line metrics")
                .raw_words,
            3
        );
    }

    /// A warm partial pass keeps every record whose entry the pass verified, including
    /// unchanged files, and removes records below an unlistable directory.
    #[test]
    #[cfg(unix)]
    fn a_warm_partial_pass_keeps_all_and_only_verified_content_records() {
        use std::os::unix::fs::PermissionsExt;

        if !crate::test_support::require_permission_bits() {
            return;
        }

        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        let sidecar_path = content::content_cache_path(&snapshot_path);
        for index in 0..7 {
            write_file(&dir.path().join(format!("f{index}.md")), b"alpha\nbeta\n");
        }
        write_file(&dir.path().join("locked/old.md"), b"three\n");
        let analysis = content::AnalysisRequest {
            profile: content::AnalysisSet::NONE.with_lines(),
            ..content::AnalysisRequest::default()
        };
        let auto = OpenFixture {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            analysis,
            ..OpenFixture::default()
        };

        let records_in_sidecar = |path: &Path| {
            let (mut fresh, _) = scan::scan_into_index(path, &ScanConfig::default()).expect("scan");
            let wanted = fresh.content_identity(analysis.profile);
            let loaded =
                content::load_content_cache(&mut fresh, &wanted, &sidecar_path).expect("load");
            assert!(loaded.usable, "the sidecar should still be readable");
            loaded.hits + loaded.stale
        };

        let (_, seeded) = open_fixture(dir.path(), &auto).expect("seed both tiers");
        assert!(seeded.is_complete());
        assert_eq!(records_in_sidecar(dir.path()), 8, "one record per seeded file");

        // One file changes, one directory becomes unlistable: the warm pass is partial.
        write_file(&dir.path().join("f0.md"), b"alpha\nbeta\ngamma\n");
        let locked = dir.path().join("locked");
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("deny");
        let opened = open_fixture(dir.path(), &auto);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o700)).expect("restore");
        let (_, report) = opened.expect("a partial open still answers");
        assert!(!report.is_complete(), "the pass should be partial");

        assert_eq!(
            records_in_sidecar(dir.path()),
            7,
            "all seven verified files survive and the inaccessible record does not"
        );
    }

    #[test]
    fn no_snapshot_is_written_when_the_policy_forbids_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        for policy in [CachePolicy::ReadOnly, CachePolicy::Off] {
            let (_index, _report, pending) =
                open_fixture_with_pending_save(dir.path(), &config(&snapshot_path, policy))
                    .expect("open");
            pending.join().expect("nothing to join");
            assert!(!snapshot_path.exists(), "{policy:?} wrote a snapshot");
        }
    }
}
