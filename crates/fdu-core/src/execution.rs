//! Planning and executing one-shot reports with the least retained state they require.
//!
//! The command surface stays composable: callers describe cache policy and a query, not
//! an implementation strategy.  This module derives that strategy.  Most reports need
//! the complete [`Index`](crate::Index), either because another view needs hierarchy or
//! paths, or because the cache must retain reusable state.  An unfiltered summary needs
//! only its aggregate values and, when it observes `.gitignore`, the ignored share of
//! them, so that one plan reduces the scan's observations directly and never builds an
//! index.

use std::time::SystemTime;

use crate::query::{
    Delivery, Report, ReportProvenance, ReportSource, Request, SummaryRow, TreeStatus, ViewSpec,
    report, report_summary,
};
use crate::{CachePolicy, EntryKind, Error, OpenPath, PendingSave, Progress, Result, execute};

/// The minimum state a one-shot report plan retains while scanning.
///
/// This is deliberately a small closed set.  Add another tier only when a measured view
/// can be answered exactly from materially less state than an index; callers should not
/// have to select it themselves.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum RetainedState {
    /// One aggregate row; no path or hierarchy records survive the scan.
    ///
    /// A scan that observes control state keeps the control table and the few ignored
    /// directories that head an ignored subtree, which is all it needs to classify each
    /// entry as the index would ([`SummaryFold`]).
    Summary,
    /// The complete reusable metadata index.
    FullIndex,
}

/// The engine lifecycle that will deliver an answer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Route {
    /// A single report may retain only an aggregate.
    OneShot,
    /// A reusable index returned to the caller.
    Retained,
    /// Verification of an index already held by the caller.
    Refresh,
    /// A continuing observation session.
    Watch,
    /// Progressive discovery and serving of an opened root.
    Opened,
}

/// Which persisted state execution may read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Load {
    /// Start without a metadata snapshot.
    None,
    /// Attempt to restore a serving metadata snapshot.
    Snapshot,
}

/// Whether execution must verify the filesystem.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verify {
    /// Answer only from persisted facts, marked unverified.
    None,
    /// Observe the requested filesystem scope.
    Filesystem,
}

/// Whether an answer fulfills the caller's delivery contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutcomeClass {
    /// A complete answer, or a partial answer the caller explicitly accepts.
    Success,
    /// An incomplete answer the caller did not accept.
    Partial,
}

/// Validated policy shared by all engine execution routes.
#[derive(Clone, Debug)]
pub struct Plan {
    pub(crate) basis: crate::query::Basis,
    pub(crate) route: Route,
    pub(crate) retained: RetainedState,
    pub(crate) load: Load,
    pub(crate) verify: Verify,
    pub(crate) persist: bool,
    pub(crate) delivery: Delivery,
}

impl Plan {
    /// Classify the answer using the caller's partial-answer policy.
    pub fn outcome(&self, status: &TreeStatus) -> OutcomeClass {
        if status.complete || self.delivery.accept_partial {
            OutcomeClass::Success
        } else {
            OutcomeClass::Partial
        }
    }
    /// The semantic basis validated when the plan was constructed.
    pub fn basis(&self) -> &crate::query::Basis {
        &self.basis
    }
    /// The lifecycle this plan executes.
    pub const fn route(&self) -> Route {
        self.route
    }
    /// The persisted state this plan may read.
    pub const fn load(&self) -> Load {
        self.load
    }
    /// The verification this plan performs.
    pub const fn verify(&self) -> Verify {
        self.verify
    }
    /// Whether this plan may write the snapshot and its content sidecar.
    ///
    /// Each tier still writes only under its own rule (`Plan::writes`); this is the
    /// authorization those rules start from, and a caller deciding whether to warn that a
    /// run leaves nothing behind asks it here rather than of the policy, whose meaning
    /// under `Auto` depends on the route and the analysis.
    pub const fn persists(&self) -> bool {
        self.persist
    }
    /// The caller's operational choices after validation.
    pub fn delivery(&self) -> &Delivery {
        &self.delivery
    }
}

/// Stored tier declarations and restoration evidence presented to a plan.
pub(crate) struct StoreHeader<'a> {
    pub(crate) root: &'a std::path::Path,
    pub(crate) snapshot: crate::SnapshotIdentity,
    pub(crate) content: Option<&'a crate::ContentTierIdentity>,
    pub(crate) content_complete: bool,
}

/// Why persisted state cannot deliver a planned answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Admission {
    Serve(crate::Serves),
    NoLocation,
    Missing,
    WrongRoot,
    WrongScope,
    IncompleteContent,
}

impl Plan {
    /// The one decision of whether stored state answers this plan's basis.
    ///
    /// Every route that reads a snapshot admits it here, warm and cache-only alike, and
    /// persistence asks the same question of the header on disk before it decides what an
    /// unchanged pass owes the store. The content arm applies only to a plan that verifies
    /// nothing, because a verifying route re-reads what its sidecar lacks.
    pub(crate) fn admit(
        &self,
        stored: Option<&StoreHeader<'_>>,
        basis: &crate::query::Basis,
    ) -> Admission {
        if self.delivery.cache_path.is_none() {
            return Admission::NoLocation;
        }
        let Some(stored) = stored else {
            return Admission::Missing;
        };
        if stored.root != basis.root {
            return Admission::WrongRoot;
        }
        let relation = crate::serves_snapshot(stored.snapshot, basis.scope.snapshot_identity());
        if relation == crate::Serves::Refuse {
            return Admission::WrongScope;
        }
        if self.verify == Verify::None && basis.content.is_enabled() {
            let wanted = crate::ContentTierIdentity::for_request(
                basis.scope.snapshot_identity().entries,
                basis.content,
            );
            if !stored.content_complete
                || stored.content.and_then(|identity| wanted.admit(identity)).is_none()
            {
                return Admission::IncompleteContent;
            }
        }
        Admission::Serve(relation)
    }
}

/// Facts observed by execution, independent of the route that observed them.
#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct RunFacts {
    pub(crate) entries_verified: bool,
    pub(crate) entries_changed: bool,
    pub(crate) content_changed: bool,
    pub(crate) content_requested: bool,
    pub(crate) projected: bool,
    pub(crate) paired_entries: bool,
}

/// Artifacts the plan authorizes its executor to write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SaveTargets {
    pub(crate) metadata: bool,
    pub(crate) content: bool,
}

impl SaveTargets {
    pub(crate) const fn none(self) -> bool {
        !self.metadata && !self.content
    }
}

impl Plan {
    pub(crate) fn writes(&self, run: RunFacts) -> SaveTargets {
        let allowed = self.persist && self.delivery.cache_path.is_some();
        SaveTargets {
            metadata: allowed && run.entries_verified && run.entries_changed && !run.projected,
            content: allowed
                && run.content_requested
                && run.content_changed
                && (run.entries_verified || run.paired_entries),
        }
    }
}

/// Operational work behind one one-shot report.
///
/// This is deliberately separate from [`Report`]: it is transient CLI telemetry, not
/// part of the stable machine-report schema or the pure query result.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PerformanceSummary {
    /// Regular files whose metadata was observed during this run.
    pub walked_files: u64,
    /// Apparent bytes represented by those walked files.
    pub walked_bytes: u64,
    /// Allocated bytes of those walked files, the figure to show beside an answer
    /// measured in allocated bytes.
    pub walked_allocated: u64,
    /// Fresh content-analysis candidates processed.
    pub fresh_files: u64,
    /// Bytes actually returned by fresh content reads.
    pub bytes_read: u64,
    /// Wall time spent processing fresh analysis candidates.
    pub analysis_ns: u64,
    /// Content-analysis records restored from the sidecar.
    pub cached_files: u64,
    /// Apparent bytes represented by restored content records.
    pub cached_bytes: u64,
    /// Metadata cache tier used for this report.
    pub source: ReportSource,
}

impl Default for PerformanceSummary {
    fn default() -> Self {
        Self {
            walked_files: 0,
            walked_bytes: 0,
            walked_allocated: 0,
            fresh_files: 0,
            bytes_read: 0,
            analysis_ns: 0,
            cached_files: 0,
            cached_bytes: 0,
            source: ReportSource::ColdScan,
        }
    }
}

impl PerformanceSummary {
    /// Total metadata throughput over the same elapsed sample as the report duration.
    /// GiB/s represents walked size, not storage read bandwidth.
    pub fn total_throughput(
        self,
        elapsed: std::time::Duration,
        size: crate::query::SizeMetric,
    ) -> String {
        let bytes = match size {
            crate::query::SizeMetric::Apparent => self.walked_bytes,
            crate::query::SizeMetric::Allocated => self.walked_allocated,
        };
        throughput_rates(self.walked_files, bytes, elapsed).map_or_else(
            || "throughput unavailable".to_owned(),
            |(files, gib)| format!("{files} files/s ({gib} GiB/s)"),
        )
    }

    fn from_open_report(report: &crate::OpenReport) -> Self {
        let analysis = report.analysis.unwrap_or_default();
        Self {
            walked_files: report.scan.files_walked,
            walked_bytes: report.scan.bytes_walked,
            walked_allocated: report.scan.allocated_walked,
            fresh_files: analysis.candidates,
            bytes_read: analysis.bytes_read,
            analysis_ns: analysis.elapsed_ns,
            cached_files: report.content_cache.hits,
            cached_bytes: report.content_cache.bytes,
            source: match report.path_taken {
                OpenPath::ColdScan => ReportSource::ColdScan,
                OpenPath::WarmRevalidate => ReportSource::WarmRevalidate,
                OpenPath::CacheOnly => ReportSource::CacheOnly,
            },
        }
    }
}

/// Cumulative walk rates over one actual elapsed sample. The byte rate is binary GiB/s,
/// rounded to three decimals; the grouped file rate counts complete files per second.
/// Returns `(files_per_second, gib_per_second)` without unit labels, or `None` when
/// elapsed time is zero. Neither rate estimates storage read bandwidth.
pub fn throughput_rates(
    files: u64,
    bytes: u64,
    elapsed: std::time::Duration,
) -> Option<(String, String)> {
    let ns = elapsed.as_nanos();
    if ns == 0 {
        return None;
    }
    let files_per_second = u128::from(files) * 1_000_000_000 / ns;
    let gib_denominator = ns * (1_u128 << 30);
    let gib_thousandths =
        (u128::from(bytes) * 1_000_000_000 * 1_000 + gib_denominator / 2) / gib_denominator;
    Some((
        crate::report_format::human_count_u128(files_per_second),
        format!("{}.{:03}", gib_thousandths / 1_000, gib_thousandths % 1_000),
    ))
}

/// Validate a request and derive the least-retention plan for its delivery and route.
///
/// A summary reducer is legal when no content analysis is requested, the sole requested
/// view is an unfiltered summary, the scan retains its whole population, and the policy
/// does not require the snapshot to participate.  [`crate::open`] and live sessions still
/// promise an index and therefore always plan full retention. Any future requirement the
/// compact tier cannot prove falls closed to `RetainedState::FullIndex`.
///
/// Control observation is the caller's decision, not this planner's: a report's rows carry
/// the ignored share of every size they show (fdu-elnn), so a scan that reads `.gitignore`
/// displays what it paid for, and one that turned it off shows no share rather than a zero.
/// The summary reducer classifies each entry against the control table the index would
/// hold and folds the ignored share without retaining the entry (fdu-1ovb), so the default
/// `fdu --view summary` takes it as well. A narrowed population (`--ignored=exclude|only`)
/// is also a selection by ignored state, which `is_unfiltered` sends to the index; the
/// planner checks the scope's population as well rather than rest on validation pairing
/// the two.
///
/// The compact tier is not gated on the cache being unavailable, because for an
/// unfiltered metadata summary the snapshot cannot save the work the scan is doing.
/// Revalidating a loaded snapshot stats every entry anyway, so the reusable index and its
/// write are additive cost with nothing to amortise them: measured on Linux/ext4 over
/// 84,539 entries, the compact tier answered in 71 ms against 161 ms for a warm
/// revalidating `Auto` run, and even a no-scan stale answer cost 81 ms because
/// deserialisation is about as expensive per record as a warm walk.  A snapshot earns its
/// keep when it avoids expensive work — re-reading file bodies for content analysis, or a
/// cold filesystem walk — not when it merely mirrors a walk that still has to happen.
///
/// Two deliveries still require the index, for reasons that are about intent rather than
/// cost.  [`Delivery::stale_ok`] must answer from the snapshot without touching the tree,
/// so it has no scan to reduce.  [`CachePolicy::On`] is an explicit request to leave a
/// current snapshot, and honouring it means materialising the index that gets written —
/// though with no cache path configured there is nothing to write, and the compact tier
/// answers it like any other summary.
///
/// Persistence follows the same reasoning as the read.  Under [`CachePolicy::Auto`] a
/// one-shot metadata report writes nothing, because no later one-shot report reads what
/// it would store: on a million-entry Linux tree the write was 0.26 s of a 1.51 s default
/// run.  Content analysis writes, because its sidecar spares re-reading unchanged files
/// and is paired with the snapshot beside it; sessions, watches, and refreshes write,
/// because they are the later reader.
pub fn plan(
    request: &Request,
    delivery: &Delivery,
    route: Route,
) -> std::result::Result<Plan, crate::query::RequestError> {
    request.validate()?;
    let mut normalized = delivery.clone();
    if route == Route::Watch {
        normalized.watch.get_or_insert_with(crate::query::WatchDelivery::default);
    }
    let delivery = &normalized;
    request.validate_delivery(delivery)?;
    if route == Route::Opened {
        if delivery.cache != CachePolicy::Off
            || delivery.watch.is_some()
            || request.basis.content.is_enabled()
        {
            return Err(crate::query::RequestError::DeliveryUnsupported {
                route: "opened",
                reason: "progressive discovery requires cache off, no content analyzers, and observation configured through OpenOptions",
            });
        }
        // An opened root runs one breadth-first producer and publishes coverage as state
        // rather than as one answer, so these fields have no effect there. Refused rather
        // than dropped: a delivery the route accepts is one it executes.
        if delivery.workers.scan.is_some()
            || delivery.order != crate::ScanOrder::default()
            || delivery.accept_partial
        {
            return Err(crate::query::RequestError::DeliveryUnsupported {
                route: "opened",
                reason: "progressive discovery schedules one breadth-first producer and reports coverage as state, so it takes no scan worker count, traversal order, or partial-answer acceptance",
            });
        }
    }
    if route == Route::Refresh && delivery.stale_ok {
        return Err(crate::query::RequestError::DeliveryUnsupported {
            route: "refresh",
            reason: "a stale answer cannot verify filesystem state",
        });
    }
    let analysis_requested = request.basis.content.is_enabled();
    let summary_is_sufficient = request.query.views.as_slice() == [ViewSpec::Summary]
        && request.query.selection.is_unfiltered()
        && request.basis.scope.population == crate::query::IgnoredEntries::Include;
    let policy_requires_index =
        delivery.stale_ok || (delivery.cache == CachePolicy::On && delivery.cache_path.is_some());
    // What a later request can reuse decides both directions. A one-shot metadata query
    // cannot amortize loading and reconciling a snapshot: both paths stat every entry. On
    // macOS/APFS (494,031 entries), warm revalidation cost 4.8 s versus 3.6 s cold. Nor
    // does a later one-shot metadata query read what it would write. Content avoids body
    // reads, and retained routes are their own later reader.
    let stored_state_pays = route != Route::OneShot || analysis_requested;
    let read_snapshot = delivery.stale_ok
        || match delivery.cache {
            CachePolicy::Off => false,
            CachePolicy::Auto | CachePolicy::On => stored_state_pays,
        };
    let persist = !delivery.stale_ok
        && match delivery.cache {
            CachePolicy::Off => false,
            CachePolicy::Auto => stored_state_pays,
            CachePolicy::On => true,
        };
    Ok(Plan {
        basis: request.basis.clone(),
        route,
        retained: if route == Route::OneShot
            && !policy_requires_index
            && !analysis_requested
            && summary_is_sufficient
        {
            RetainedState::Summary
        } else {
            RetainedState::FullIndex
        },
        load: if read_snapshot { Load::Snapshot } else { Load::None },
        verify: if delivery.stale_ok { Verify::None } else { Verify::Filesystem },
        persist,
        delivery: delivery.clone(),
    })
}

/// Execute a one-shot report, retaining the least state the request needs.
///
/// The returned report is complete as a value even while the optional save runs.  The
/// caller must join the handle before exit; dropping it also joins defensively.
///
/// This is the contract the command line has always run under, and until now the only way
/// to get it was to be the command line. `open` takes the session path: it retains an
/// index and writes a snapshot, which is right for a caller asking many questions and
/// wrong for one asking a single question -- an unfiltered summary is answered by a
/// transient tier that retains no index, and writing a snapshot for it caches state the
/// walk did not save. A Python caller therefore left
/// cache state on a tree that the same command would not have, which a later cache-only
/// read could see (fdu-4msv).
///
/// The report observes `.gitignore` control state as the request's
/// [`ScanConfig::read_controls`](crate::ScanConfig) says, on by default as for
/// [`crate::open`], so the two share one snapshot scope. Observing,
/// every tree, summary, extension, and file row carries its ignored share, and
/// [`Report::ignore_rules`](crate::query::Report::ignore_rules) names any file a control
/// limit refused, by the budget or by the line limit. Turned off, no `.gitignore` is
/// read, every share is `None`, and a selection by ignored state is refused with
/// [`Error::InvalidRequest`] before anything is scanned. Such a report reads a default
/// snapshot under every reading policy by constructing a requested-scope index from its
/// all-entry facts and discarding its classification.
///
/// The caller owns the returned [`PendingSave`] and decides when to join it, exactly as
/// the command line does, so a renderer can run while the snapshot is still being written.
pub fn prepare_report(
    request: &Request,
    delivery: &Delivery,
) -> Result<(Report, PendingSave, PerformanceSummary)> {
    prepare_report_internal(request, delivery, false, None)
        .map(|(report, pending, performance, _diagnostics)| (report, pending, performance))
}

/// Execute a one-shot report, reporting its progress through `progress` as it runs.
///
/// The same contract and the same answer as [`prepare_report`]: the handle observes the
/// run and changes nothing about it, so a report prepared with one is byte-for-byte the
/// report prepared without. The caller polls [`Progress::snapshot`] from another thread
/// while this blocks, typically to draw a wait indicator. Which phases the run passes
/// through, what the counters mean, and what holds when this returns are documented on
/// [`Progress`]; in short, the walk counters equal the returned
/// [`PerformanceSummary`]'s walked totals, and a run that requested content analysis
/// leaves `analysis` at `(fresh_files, fresh_files)`.
///
/// A run over a full index ends in [`ProgressPhase::Summarizing`](crate::ProgressPhase)
/// while it builds the answer; a save it started continues in the background, and the
/// caller decides when to join it, as with [`prepare_report`]. `Saving` is therefore
/// shown only for the moment between the save's start and the answer's, however long
/// the write takes.
pub fn prepare_report_with_progress(
    request: &Request,
    delivery: &Delivery,
    progress: &Progress,
) -> Result<(Report, PendingSave, PerformanceSummary)> {
    prepare_report_internal(request, delivery, false, Some(progress))
        .map(|(report, pending, performance, _diagnostics)| (report, pending, performance))
}

/// Execute a one-shot report and retain scan diagnostics.
///
/// Public because the command line needs it and the command line is an ordinary consumer:
/// it drives repository-controlled measurement of the installed binary. Kept separate
/// from [`prepare_report`] so callers who do not want traces pay for neither collection
/// nor serialization. The diagnostic value is present only when the report performs a
/// cold scan; cache-only opens do not scan, and warm reconciliation has a different
/// execution contract.
pub fn prepare_report_with_scan_diagnostics(
    request: &Request,
    delivery: &Delivery,
) -> Result<(Report, PendingSave, PerformanceSummary, Option<crate::scan::ScanDiagnostics>)> {
    prepare_report_internal(request, delivery, true, None)
}

fn prepare_report_internal(
    request: &Request,
    delivery: &Delivery,
    collect_scan_diagnostics: bool,
    progress: Option<&Progress>,
) -> Result<(Report, PendingSave, PerformanceSummary, Option<crate::scan::ScanDiagnostics>)> {
    // Before anything is scanned, loaded, or reduced: a request its own basis cannot answer
    // has no answer at any cost, and the compact summary tier below never reaches a reader,
    // so a check made there would not cover this route at all. A scope this build cannot
    // honour is part of that one check rather than a second one beside it, which is what
    // keeps the refusal independent of the delivery: the cache-only tier never scans and
    // the cold tier never loads, so a rule stated at either would hold for one of them.
    request.validate().map_err(Error::InvalidRequest)?;
    let scan_config = crate::ScanConfig {
        progress: progress.cloned(),
        ..request.basis.scope.scan_config(delivery)
    };
    let root = request.basis.root.as_path();
    let scan_started_at = SystemTime::now();
    let plan = plan(request, delivery, Route::OneShot).map_err(Error::InvalidRequest)?;
    match plan.retained {
        RetainedState::Summary => {
            let root = root.canonicalize().map_err(|error| Error::io(root, error))?;
            let mut fold = SummaryFold::new(&scan_config);
            let mut reduce = |observed: &crate::ObservationOp| fold.observe(observed);
            let (mut scan, scan_diagnostics) = if collect_scan_diagnostics {
                let (scan, diagnostics) = crate::scan::scan_summary_fold_with_diagnostics(
                    &root,
                    &scan_config,
                    &mut reduce,
                )?;
                (scan, Some(diagnostics))
            } else {
                (crate::scan::scan_summary_fold(&root, &scan_config, &mut reduce)?, None)
            };
            let complete = scan.is_complete();
            let generated_at = SystemTime::now();
            let (summary, ignore_rules, ignored_unverified) = fold.finish(&root, &scan.errors)?;
            let report = report_summary(
                &root,
                scan_config.scope(),
                request,
                summary,
                ignore_rules,
                ignored_unverified,
                TreeStatus::of_walk(&root, &mut scan),
                ReportProvenance::of_walk(scan_started_at, generated_at, complete),
            );
            let performance = PerformanceSummary {
                walked_files: scan.files_walked,
                walked_bytes: scan.bytes_walked,
                walked_allocated: scan.allocated_walked,
                source: ReportSource::ColdScan,
                ..PerformanceSummary::default()
            };
            Ok((report, PendingSave::none(), performance, scan_diagnostics))
        }
        RetainedState::FullIndex => {
            let (index, open_report, pending_save, scan_diagnostics) =
                execute(&plan, &request.basis, collect_scan_diagnostics, progress)?;
            let performance = PerformanceSummary::from_open_report(&open_report);
            if let Some(progress) = progress {
                progress.enter(crate::ProgressPhase::Summarizing);
            }
            let answer = report(&index, request, SystemTime::now())?;
            debug_assert_eq!(answer.scope, scan_config.scope());
            // The answer is complete and owns no part of the index, so freeing it is no
            // longer the caller's wait.
            crate::release_index(index);
            Ok((answer, pending_save, performance, scan_diagnostics))
        }
    }
}

/// The transient summary tier's reducer: the root roll-up the index would report, folded
/// from the walk's observations as they arrive, with nothing retained per entry.
///
/// Observing `.gitignore`, it also classifies every entry as the detached index builder
/// does, and tallies the entries no rule ignores beside every entry, the index's
/// `unignored` and `all` partitions; the share it reports is
/// [`IgnoredTally::between`](crate::query::IgnoredTally) the two, the index's own formula.
///
/// **Why each classification is the index's.** The builder classifies a child from two
/// inputs: its parent's classification, since nothing below an ignored directory can be
/// re-included, and the admitted control files of the directories above it, deepest
/// first. The fold has both when each entry arrives:
///
/// - A worker publishes a listing before any directory in it becomes claimable, so an
///   entry arrives after its parent's own observation, as a listing reaches the builder
///   after its parent's.
/// - A classifying walk sends each directory's control ahead of its entries
///   (`SinkMode::groups_directories` in the scanner), so it is applied before any entry
///   in the directory is classified, as the builder applies a listing's control before
///   its children. A listing that fills a batch before its `.gitignore` is listed has
///   that file read directly, only under the exact name a listing accepts, and the read
///   stands for the listing; on a tree nothing modifies during the walk it is the same
///   file with the same bytes.
/// - One consumer applies every control in arrival order, as the builder's one consumer
///   does. Which files a budget refuses when several compete for it depends on that
///   order on both routes; with one worker the order is the same on both, and with
///   several it is an order the index could also have met. Whether any file is refused
///   does not depend on it, because a cold walk only adds charges.
///
/// **Why it keeps no entries.** A parent's classification is final once its own
/// observation is folded, since every ancestor's was folded first. So the fold keeps only
/// the ignored directories whose parent is not ignored, which head every ignored subtree,
/// and a parent is ignored exactly when one of them is its ancestor or itself.
///
/// **What it withholds.** The share, where the index withholds the root's: when a control
/// file was refused, because it may have held negations as well as ignore rules, or could
/// not be read. The coverage it reports is the table's, refusals included.
struct SummaryFold {
    /// Every entry the walk retained.
    all: SummaryRow,
    /// The classifier, when the scan observes `.gitignore`.
    controls: Option<SummaryControls>,
}

/// What a classifying [`SummaryFold`] keeps: the control table, the heads of ignored
/// subtrees, and the tallies of what no rule ignores.
struct SummaryControls {
    table: crate::control::ControlTable,
    /// Ignored directories whose parent is not ignored.
    ///
    /// Unbounded by design: no head lies below another, so the set holds one path per
    /// separately ignored subtree, which is what classifying an entry needs. It grows with
    /// such subtrees, not with the entries in them: a tree of many small ignored
    /// directories, each under a directory that is not ignored, is the case that makes it
    /// large. The RSS evidence so far (exp-170, exp-171) is on trees with few heads.
    ignored_heads: std::collections::HashSet<std::path::PathBuf>,
    /// The parent last looked up, and whether it is ignored. A listing's entries mostly
    /// arrive together, so this answers nearly all of them.
    parent: Option<(std::path::PathBuf, bool)>,
    /// The controls governing the parent last classified under, resolved once for its
    /// listing (H163) with the parent's components split once for it too (H171), and
    /// dropped whenever the table changes.
    chain:
        Option<(std::path::PathBuf, crate::control::ControlChain, crate::control::SplitDirectory)>,
    /// Every entry no rule ignores, as the index's `unignored` partition.
    unignored: crate::index::RollUpScalars,
    /// The first control observation the table rejected, which fails the report as it
    /// fails the index build.
    rejected: Option<Error>,
}

impl SummaryFold {
    fn new(config: &crate::ScanConfig) -> Self {
        Self {
            all: SummaryRow::default(),
            controls: config.read_controls.then(|| SummaryControls {
                table: crate::control::ControlTable::with_limits(config.control_limits),
                ignored_heads: std::collections::HashSet::new(),
                parent: None,
                chain: None,
                unignored: crate::index::RollUpScalars::default(),
                rejected: None,
            }),
        }
    }

    fn observe(&mut self, observed: &crate::ObservationOp) {
        match &observed.op {
            crate::Op::Upsert { path, kind, attrs } => {
                match kind {
                    EntryKind::File => {
                        self.all.files += 1;
                        self.all.bytes += attrs.size;
                        self.all.allocated += attrs.allocated;
                        self.all.newest_mtime_ns = Some(
                            self.all
                                .newest_mtime_ns
                                .map_or(attrs.mtime_ns, |current| current.max(attrs.mtime_ns)),
                        );
                    }
                    EntryKind::Dir => self.all.dirs += 1,
                    EntryKind::Symlink | EntryKind::Other => {}
                }
                let Some(controls) = &mut self.controls else { return };
                if controls.classify(path, *kind) {
                    return;
                }
                // The index's contribution of an unignored entry: a file's sizes, one
                // directory, nothing for any other kind.
                match kind {
                    EntryKind::File => {
                        controls.unignored.files += 1;
                        controls.unignored.bytes += attrs.size;
                        controls.unignored.allocated += attrs.allocated;
                    }
                    EntryKind::Dir => controls.unignored.dirs += 1,
                    EntryKind::Symlink | EntryKind::Other => {}
                }
            }
            crate::Op::ControlUpsert { path, source } => {
                if let Some(controls) = &mut self.controls {
                    controls.chain = None;
                    let admitted = controls.table.upsert(path, source.clone()).map(drop);
                    controls.record(admitted);
                }
            }
            crate::Op::ControlRemove { path } => {
                if let Some(controls) = &mut self.controls {
                    controls.chain = None;
                    let removed = controls.table.remove(path).map(drop);
                    controls.record(removed);
                }
            }
            // A cold walk observes what is there; it neither removes nor invalidates.
            crate::Op::Remove { .. } | crate::Op::InvalidateSubtree { .. } => {}
        }
    }

    /// The summary row, the control coverage, and whether the row withholds its ignored
    /// share because a governing rule could not be verified.
    ///
    /// `errors` are the walk's, normalized, as the index records them.
    fn finish(
        self,
        root: &std::path::Path,
        errors: &[Error],
    ) -> Result<(SummaryRow, crate::control::ControlCoverage, bool)> {
        let Some(controls) = self.controls else {
            return Ok((
                SummaryRow { ignored: None, ..self.all },
                crate::control::ControlCoverage::NotObserved,
                false,
            ));
        };
        if let Some(error) = controls.rejected {
            return Err(error);
        }
        let unreadable =
            errors.iter().any(|error| crate::control::unreadable_control(root, error).is_some());
        let verified = controls.table.refused_len() == 0 && !unreadable;
        let all = crate::index::RollUpScalars {
            files: self.all.files,
            dirs: self.all.dirs,
            bytes: self.all.bytes,
            allocated: self.all.allocated,
            newest_mtime_ns: self.all.newest_mtime_ns.unwrap_or_default(),
        };
        let summary = SummaryRow {
            ignored: verified.then(|| crate::query::IgnoredTally::between(all, controls.unignored)),
            ..self.all
        };
        Ok((
            summary,
            crate::control::ControlCoverage::Observed(controls.table.observation()),
            !verified,
        ))
    }
}

impl SummaryControls {
    /// Whether the entry at `path` is ignored, decided as
    /// `DetachedIndexBuilder::push_directory` decides it: an entry below an ignored
    /// directory is ignored, one in a table with no rules is not, and otherwise the
    /// deepest control with an opinion decides.
    fn classify(&mut self, path: &std::path::Path, kind: EntryKind) -> bool {
        // No rule and no ignored subtree yet, as in every tree without a `.gitignore`:
        // nothing is ignored, and the parent need not even be derived.
        if self.ignored_heads.is_empty() && self.table.is_empty() {
            return false;
        }
        let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
        let parent_ignored = self.parent_ignored(parent);
        let ignored = if parent_ignored || self.table.is_empty() {
            parent_ignored
        } else if let Some(name) = path.file_name() {
            if !matches!(&self.chain, Some((cached, ..)) if cached == parent) {
                self.chain = Some((
                    parent.to_path_buf(),
                    self.table.chain_for(parent),
                    crate::control::SplitDirectory::new(parent),
                ));
            }
            let (_, chain, split) = self.chain.as_ref().expect("the chain was just resolved");
            !chain.is_empty()
                && split.with_components(|directory| {
                    chain.is_ignored_within(directory, name.as_encoded_bytes(), kind.is_dir())
                })
        } else {
            self.table.matcher_for(path).is_ignored(kind.is_dir())
        };
        if ignored && !parent_ignored && kind.is_dir() {
            self.ignored_heads.insert(path.to_path_buf());
        }
        ignored
    }

    fn parent_ignored(&mut self, parent: &std::path::Path) -> bool {
        if self.ignored_heads.is_empty() {
            return false;
        }
        if let Some((cached, ignored)) = &self.parent {
            if cached == parent {
                return *ignored;
            }
        }
        let ignored = parent.ancestors().any(|ancestor| self.ignored_heads.contains(ancestor));
        self.parent = Some((parent.to_path_buf(), ignored));
        ignored
    }

    fn record(&mut self, applied: Result<()>) {
        if let Err(error) = applied {
            self.rejected.get_or_insert(error);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::query::{IgnoredEntries, Pattern, Query, Section};
    use crate::{OpenFixture, ScanConfig};

    #[test]
    fn total_throughput_uses_one_elapsed_sample_and_selected_size() {
        use crate::query::SizeMetric;
        let work = PerformanceSummary {
            walked_files: 200,
            walked_bytes: 4_000_000_000,
            walked_allocated: 1_000_000_000,
            ..PerformanceSummary::default()
        };
        assert_eq!(
            work.total_throughput(std::time::Duration::from_secs(2), SizeMetric::Apparent),
            "100 files/s (1.863 GiB/s)"
        );
        assert_eq!(
            work.total_throughput(std::time::Duration::from_secs(2), SizeMetric::Allocated),
            "100 files/s (0.466 GiB/s)"
        );
        assert_eq!(
            work.total_throughput(std::time::Duration::ZERO, SizeMetric::Apparent),
            "throughput unavailable"
        );
        assert_eq!(
            PerformanceSummary::default()
                .total_throughput(std::time::Duration::from_secs(1), SizeMetric::Apparent),
            "0 files/s (0.000 GiB/s)"
        );
        assert_eq!(
            throughput_rates(12_345, 3 * (1_u64 << 30), std::time::Duration::from_secs(2)),
            Some(("6,172".to_owned(), "1.500".to_owned()))
        );
    }

    #[test]
    #[cfg(unix)]
    fn an_unreadable_stored_header_never_authorizes_live_replacement() {
        use std::os::unix::fs::PermissionsExt;
        if !crate::test_support::require_permission_bits() {
            return;
        }
        let root = tempfile::tempdir().expect("root");
        let cache = tempfile::tempdir().expect("cache");
        let snapshot = cache.path().join("snapshot.fdu");
        fs::write(root.path().join(".gitignore"), b"ignored\n").expect("control");
        let observed = crate::query::Basis {
            root: root.path().into(),
            scope: crate::query::Scope::default(),
            content: crate::content::AnalysisSet::NONE,
        };
        let delivery = Delivery::new(CachePolicy::Auto, Some(snapshot.clone()));
        crate::open(&observed, &delivery).expect("stronger snapshot");
        let original = fs::read(&snapshot).expect("original image");
        let basis = crate::query::Basis {
            scope: crate::query::Scope { read_controls: false, ..Default::default() },
            ..observed
        };
        let (mut index, _) =
            crate::open(&basis, &Delivery::new(CachePolicy::Off, None)).expect("fresh blind index");
        let request = Request::new(basis, Query::default(), SystemTime::now());
        let plan = plan(&request, &delivery, Route::Refresh).expect("plan");
        fs::set_permissions(&snapshot, fs::Permissions::from_mode(0o000))
            .expect("deny header read");
        let nonwriting = Delivery { cache: CachePolicy::Off, ..delivery.clone() };
        let nonwriting_plan =
            super::plan(&request, &nonwriting, Route::Refresh).expect("nonwriting plan");
        assert!(
            !crate::persist_index_changes(&index, &nonwriting_plan, true, true)
                .expect("nonwriting policy never reads the header")
        );
        fs::write(root.path().join("fresh.txt"), b"fresh").expect("mutation");
        crate::refresh(
            &mut index,
            &request.basis,
            &Delivery { cache: CachePolicy::Off, ..delivery.clone() },
        )
        .expect("off refresh does not inspect cache state");
        let result = crate::persist_index_changes(&index, &plan, true, true);
        fs::set_permissions(&snapshot, fs::Permissions::from_mode(0o600)).expect("restore");
        assert!(
            result.is_err(),
            "a writable parent must not let unknown identity authorize replacement"
        );
        assert_eq!(fs::read(&snapshot).expect("retained image"), original);
    }

    #[test]
    fn refresh_rejects_another_root_before_mutating_or_persisting() {
        let a = tempfile::tempdir().expect("root a");
        let b = tempfile::tempdir().expect("root b");
        let cache = tempfile::tempdir().expect("cache");
        let basis = crate::query::Basis {
            root: a.path().into(),
            scope: crate::query::Scope::default(),
            content: crate::content::AnalysisSet::NONE,
        };
        let (mut index, _) =
            crate::open(&basis, &Delivery::new(CachePolicy::Off, None)).expect("open a");
        fs::write(a.path().join("new"), b"new facts").expect("mutation a");
        let before = index.clock();
        let snapshot = cache.path().join("snapshot.fdu");
        let wrong = crate::query::Basis { root: b.path().into(), ..basis.clone() };
        let delivery = Delivery::new(CachePolicy::Auto, Some(snapshot.clone()));
        let error = crate::refresh(&mut index, &wrong, &delivery).expect_err("different root");
        assert!(matches!(
            error,
            Error::InvalidRequest(crate::query::RequestError::RootMismatch { .. })
        ));
        assert_eq!(index.clock(), before);
        assert!(!snapshot.exists());
        let alias = crate::query::Basis { root: a.path().join("."), ..basis };
        crate::refresh(&mut index, &alias, &delivery).expect("same root spelling");
        assert_eq!(index.total().files, 1);
        #[cfg(unix)]
        {
            let link = cache.path().join("root-alias");
            std::os::unix::fs::symlink(a.path(), &link).expect("root alias");
            let symlink_basis = crate::query::Basis { root: link, ..alias };
            crate::refresh(&mut index, &symlink_basis, &delivery)
                .expect("same canonical root through symlink");
        }
    }

    #[test]
    fn unchanged_refresh_replaces_an_incompatible_stored_baseline() {
        let root = tempfile::tempdir().expect("root");
        let other = tempfile::tempdir().expect("other root");
        let cache = tempfile::tempdir().expect("cache");
        fs::write(root.path().join("file"), b"retained").expect("file");
        let basis = crate::query::Basis {
            root: root.path().into(),
            scope: crate::query::Scope::default(),
            content: crate::content::AnalysisSet::NONE,
        };
        let delivery = Delivery::new(CachePolicy::Auto, Some(cache.path().join("snapshot.fdu")));
        for wrong_root in [false, true] {
            let wrong = if wrong_root {
                crate::query::Basis { root: other.path().into(), ..basis.clone() }
            } else {
                crate::query::Basis {
                    scope: crate::query::Scope { max_depth: Some(0), ..basis.scope.clone() },
                    ..basis.clone()
                }
            };
            crate::open(&wrong, &Delivery { cache: CachePolicy::On, ..delivery.clone() })
                .expect("incompatible snapshot");
            let (mut index, _) = crate::open(&basis, &Delivery::new(CachePolicy::Off, None))
                .expect("retained index");
            let refreshed =
                crate::refresh(&mut index, &basis, &delivery).expect("refresh reseeds cache");
            assert!(!refreshed.apply.mutated(), "the existing index was already current");
            let (cached, _) = crate::open(&basis, &Delivery { stale_ok: true, ..delivery.clone() })
                .expect("cache-only can now answer");
            assert_eq!(cached.total().bytes, 8);
        }
    }

    #[test]
    fn refreshed_metadata_and_content_are_visible_to_a_later_cache_only_open() {
        let root = tempfile::tempdir().expect("root");
        let cache = tempfile::tempdir().expect("cache");
        let path = root.path().join("note.txt");
        fs::write(&path, b"old\n").expect("old file");
        let basis = crate::query::Basis {
            root: root.path().into(),
            scope: crate::query::Scope::default(),
            content: crate::content::AnalysisSet::NONE.with_lines(),
        };
        let delivery = Delivery::new(CachePolicy::Auto, Some(cache.path().join("snapshot.fdu")));
        let (mut index, _) = crate::open(&basis, &delivery).expect("initial open");
        fs::write(&path, b"new longer text\nsecond line\n").expect("mutation");
        let refreshed = crate::refresh(&mut index, &basis, &delivery).expect("refresh");
        assert!(refreshed.is_complete());
        let (cached, report) = crate::open(&basis, &Delivery { stale_ok: true, ..delivery })
            .expect("cache-only sees refreshed tiers");
        assert_eq!(report.path_taken, OpenPath::CacheOnly);
        assert_eq!(cached.total(), index.total());
        assert_eq!(
            cached.total().bytes,
            u64::try_from(b"new longer text\nsecond line\n".len()).expect("length")
        );
        assert_eq!(report.content_cache.hits, 1);
        let original = index
            .content()
            .expect("fresh content")
            .file(Path::new("note.txt"))
            .expect("fresh file");
        let restored = cached
            .content()
            .expect("restored content")
            .file(Path::new("note.txt"))
            .expect("restored file");
        assert_eq!(restored, original);
    }

    /// One root with one file and one empty directory, and a writing delivery whose
    /// snapshot lives in its own directory so a test can make that directory read-only.
    #[cfg(unix)]
    fn owed_persistence_fixture()
    -> (tempfile::TempDir, tempfile::TempDir, crate::query::Basis, Delivery) {
        let root = tempfile::tempdir().expect("root");
        let cache = tempfile::tempdir().expect("cache");
        fs::create_dir(root.path().join("locked")).expect("locked dir");
        fs::write(root.path().join("first"), b"first").expect("first file");
        let basis = crate::query::Basis {
            root: root.path().into(),
            scope: crate::query::Scope::default(),
            content: crate::content::AnalysisSet::NONE,
        };
        let delivery = Delivery::new(CachePolicy::Auto, Some(cache.path().join("snapshot.fdu")));
        (root, cache, basis, delivery)
    }

    /// The files a cache-only open of `delivery`'s snapshot answers with.
    #[cfg(unix)]
    fn cached_files(basis: &crate::query::Basis, delivery: &Delivery) -> u64 {
        let cache_only = Delivery { stale_ok: true, ..delivery.clone() };
        crate::open(basis, &cache_only).expect("cache-only open").0.total().files
    }

    /// A refresh whose metadata write failed leaves the index holding facts the snapshot
    /// lacks; the next refresh must write them even though it changes nothing itself.
    ///
    /// The metadata write used to be keyed to the pass that ran it: a later pass that
    /// mutated nothing wrote nothing, so a snapshot that missed one write missed the
    /// facts for good, and cache-only reads answered older facts than the index held.
    #[test]
    #[cfg(unix)]
    fn an_unchanged_refresh_repeats_the_metadata_write_a_failed_refresh_owed() {
        use std::os::unix::fs::PermissionsExt;
        if !crate::test_support::require_permission_bits() {
            return;
        }
        let (root, cache, basis, delivery) = owed_persistence_fixture();
        let (mut index, _) = crate::open(&basis, &delivery).expect("initial open");
        assert_eq!(cached_files(&basis, &delivery), 1);

        fs::write(root.path().join("second"), b"second").expect("second file");
        fs::set_permissions(cache.path(), fs::Permissions::from_mode(0o555)).expect("deny write");
        let failed = crate::refresh(&mut index, &basis, &delivery);
        fs::set_permissions(cache.path(), fs::Permissions::from_mode(0o755)).expect("restore");
        assert!(failed.is_err(), "a read-only cache directory fails the write");
        assert_eq!(index.total().files, 2, "the index advanced before the write");
        assert_eq!(cached_files(&basis, &delivery), 1, "the failed write left the old image");

        let unchanged = crate::refresh(&mut index, &basis, &delivery).expect("unchanged refresh");
        assert!(unchanged.is_complete());
        assert!(!unchanged.apply.mutated(), "nothing changed between the passes");
        assert_eq!(cached_files(&basis, &delivery), 2, "the owed write ran");

        // Paid once: the next unchanged pass has nothing to write.
        let snapshot = delivery.cache_path.as_deref().expect("path");
        let written = fs::metadata(snapshot).expect("snapshot").modified().expect("mtime");
        crate::refresh(&mut index, &basis, &delivery).expect("settled refresh");
        assert_eq!(fs::metadata(snapshot).expect("snapshot").modified().expect("mtime"), written);
    }

    /// The same debt when the failed write is the one a warm `open` started: a caller
    /// keeping the index through [`crate::open_with_pending_save`] keeps the debt too.
    #[test]
    #[cfg(unix)]
    fn an_unchanged_refresh_repeats_the_metadata_write_a_failed_open_owed() {
        use std::os::unix::fs::PermissionsExt;
        if !crate::test_support::require_permission_bits() {
            return;
        }
        let (root, cache, basis, delivery) = owed_persistence_fixture();
        crate::open(&basis, &delivery).expect("complete open writes the snapshot");
        fs::write(root.path().join("second"), b"second").expect("second file");
        fs::set_permissions(cache.path(), fs::Permissions::from_mode(0o555)).expect("deny write");
        let opened = crate::open_with_pending_save(&basis, &delivery);
        // Joined before the directory is writable again: the write runs in the background.
        let outcome = opened.map(|(index, report, pending)| (index, report, pending.join()));
        fs::set_permissions(cache.path(), fs::Permissions::from_mode(0o755)).expect("restore");
        let (index, report, joined) = outcome.expect("the open itself succeeds");
        assert_eq!(report.path_taken, OpenPath::WarmRevalidate);
        assert!(joined.is_err(), "the startup write failed");
        let mut index = std::sync::Arc::into_inner(index).expect("the writer released the index");
        assert_eq!(cached_files(&basis, &delivery), 1);

        let unchanged = crate::refresh(&mut index, &basis, &delivery).expect("unchanged refresh");
        assert!(!unchanged.apply.mutated(), "nothing changed between the passes");
        assert_eq!(cached_files(&basis, &delivery), 2, "the owed write ran");
    }

    /// A partial refresh cannot write the entry tier; once the tree is readable again a
    /// complete refresh delivers the partial pass's facts to the snapshot.
    ///
    /// Restoring the directory's permissions updates its change time, so on a POSIX host
    /// the recovering pass reports that directory as updated and would write on its own
    /// account. The failed-write tests above are the ones that prove the debt is carried;
    /// this one guards that a partial pass's verified facts reach the snapshot at all.
    #[test]
    #[cfg(unix)]
    fn a_complete_refresh_persists_the_facts_a_partial_refresh_could_not() {
        use std::os::unix::fs::PermissionsExt;
        if !crate::test_support::require_permission_bits() {
            return;
        }
        let (root, _cache, basis, delivery) = owed_persistence_fixture();
        let locked = root.path().join("locked");
        let (mut index, _) = crate::open(&basis, &delivery).expect("initial open");
        assert_eq!(index.total().files, 1);

        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("deny read");
        fs::write(root.path().join("second"), b"second").expect("second file");
        let partial = crate::refresh(&mut index, &basis, &delivery);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).expect("restore");
        let partial = partial.expect("partial refresh");
        assert!(!partial.is_complete(), "the locked directory made the pass partial");
        assert!(partial.apply.mutated(), "the second file was inserted");
        assert_eq!(index.total().files, 2);
        assert_eq!(cached_files(&basis, &delivery), 1, "a partial pass never writes entries");

        let complete = crate::refresh(&mut index, &basis, &delivery).expect("complete refresh");
        assert!(complete.is_complete(), "{:?}", complete.scan.errors);
        assert_eq!(cached_files(&basis, &delivery), 2, "the complete pass wrote the facts");
    }

    #[test]
    fn cache_only_refusals_name_location_root_and_absence_separately() {
        let root = tempfile::tempdir().expect("root");
        let other = tempfile::tempdir().expect("other root");
        let cache = tempfile::tempdir().expect("cache");
        let basis = crate::query::Basis {
            root: root.path().into(),
            scope: crate::query::Scope::default(),
            content: crate::content::AnalysisSet::NONE,
        };
        let snapshot = cache.path().join("snapshot.fdu");
        let message = |delivery: &Delivery| {
            crate::open(&basis, delivery).expect_err("cache-only refusal").to_string()
        };
        let no_location = message(&Delivery::stale_ok(None));
        assert!(no_location.contains("no cache location"), "{no_location}");
        assert!(!no_location.contains("`on`"), "no write can succeed without a location");
        let missing = message(&Delivery::stale_ok(Some(snapshot.clone())));
        assert!(missing.contains("no usable snapshot"), "{missing}");
        assert!(missing.contains("with the `on` cache policy"), "{missing}");
        let other_basis = crate::query::Basis { root: other.path().into(), ..basis.clone() };
        crate::open(&other_basis, &Delivery::new(CachePolicy::Auto, Some(snapshot.clone())))
            .expect("other snapshot");
        let wrong_root = message(&Delivery::stale_ok(Some(snapshot)));
        assert!(wrong_root.contains("different root"), "{wrong_root}");
    }

    #[test]
    fn route_delivery_matrix_rejects_contracts_the_route_cannot_execute() {
        let basis = crate::query::Basis {
            root: ".".into(),
            scope: crate::query::Scope::default(),
            content: crate::content::AnalysisSet::NONE,
        };
        let request = Request::new(basis, Query::default(), SystemTime::now());
        for delivery in Delivery::enumerate() {
            for route in
                [Route::OneShot, Route::Retained, Route::Refresh, Route::Watch, Route::Opened]
            {
                let result = plan(&request, &delivery, route);
                let forbidden = delivery.stale_ok
                    && (delivery.watch.is_some()
                        || matches!(route, Route::Watch | Route::Refresh)
                        || delivery.cache == CachePolicy::Off)
                    || route == Route::Opened
                        && (delivery.cache != CachePolicy::Off
                            || delivery.watch.is_some()
                            || delivery.accept_partial);
                assert_eq!(result.is_err(), forbidden, "{route:?} {delivery:?}");
                if let Ok(plan) = result {
                    if route == Route::Opened {
                        assert_eq!(plan.load(), Load::None);
                        assert_eq!(plan.verify(), Verify::Filesystem);
                    }
                }
            }
        }
    }

    #[test]
    fn an_opened_root_refuses_the_scheduling_it_would_otherwise_drop() {
        // `OpenOptions::into_parts` runs one breadth-first producer whatever the delivery
        // says, and an opened root has no single answer for `accept_partial` to classify.
        // A value the route would silently ignore is refused at planning instead, and the
        // same values plan on a route that executes them.
        let basis = crate::query::Basis {
            root: ".".into(),
            scope: crate::query::Scope::default(),
            content: crate::content::AnalysisSet::NONE,
        };
        let request = Request::new(basis, Query::default(), SystemTime::now());
        let default = Delivery::new(CachePolicy::Off, None);
        let plan_opened = plan(&request, &default, Route::Opened).expect("defaults plan");
        assert_eq!(plan_opened.delivery().batch_size, default.batch_size);
        let unhonored = [
            (
                "scan workers",
                Delivery {
                    workers: crate::query::Workers { scan: Some(4), ..default.workers },
                    ..default.clone()
                },
            ),
            (
                "depth-first order",
                Delivery { order: crate::ScanOrder::DepthFirst, ..default.clone() },
            ),
            ("accept partial", Delivery { accept_partial: true, ..default.clone() }),
        ];
        for (case, delivery) in unhonored {
            let refused = plan(&request, &delivery, Route::Opened).expect_err(case);
            assert!(
                matches!(
                    refused,
                    crate::query::RequestError::DeliveryUnsupported { route: "opened", .. }
                ),
                "{case}: {refused}"
            );
            plan(&request, &delivery, Route::Retained)
                .unwrap_or_else(|error| panic!("{case} executes on a retained route: {error}"));
        }
        // A larger batch is honored, so it is not refused.
        let batched = Delivery { batch_size: default.batch_size * 2, ..default };
        let plan_batched = plan(&request, &batched, Route::Opened).expect("batch size plans");
        assert_eq!(plan_batched.delivery().batch_size, batched.batch_size);
    }

    #[test]
    fn tier_writes_depend_only_on_authorization_and_observed_facts() {
        // Which routes are authorized is `plan`'s decision, pinned by
        // `auto_persists_where_a_later_request_reads_what_it_stores`; this pins what each
        // tier does with the authorization it was given.
        let routes = [Route::OneShot, Route::Retained, Route::Refresh, Route::Watch, Route::Opened];
        for (delivery, persist) in
            Delivery::enumerate().flat_map(|delivery| [(delivery.clone(), false), (delivery, true)])
        {
            for bits in 0_u8..64 {
                let facts = RunFacts {
                    entries_verified: bits & 1 != 0,
                    entries_changed: bits & 2 != 0,
                    content_changed: bits & 4 != 0,
                    content_requested: bits & 8 != 0,
                    projected: bits & 16 != 0,
                    paired_entries: bits & 32 != 0,
                };
                let allowed = persist;
                let expected = SaveTargets {
                    metadata: allowed
                        && facts.entries_verified
                        && facts.entries_changed
                        && !facts.projected,
                    content: allowed
                        && facts.content_requested
                        && facts.content_changed
                        && (facts.entries_verified || facts.paired_entries),
                };
                for route in routes {
                    let plan = Plan {
                        basis: crate::query::Basis {
                            root: ".".into(),
                            scope: crate::query::Scope::default(),
                            content: crate::content::AnalysisSet::NONE,
                        },
                        route,
                        retained: RetainedState::FullIndex,
                        load: Load::Snapshot,
                        verify: Verify::Filesystem,
                        persist,
                        delivery: delivery.clone(),
                    };
                    assert_eq!(plan.writes(facts), expected, "{route:?} {delivery:?} {facts:?}");
                    let unavailable = Plan {
                        delivery: Delivery { cache_path: None, ..delivery.clone() },
                        ..plan
                    };
                    assert!(unavailable.writes(facts).none());
                }
            }
        }
    }

    fn planned(config: &OpenFixture, query: &Query) -> Plan {
        let (request, delivery) = split(Path::new("."), config, query);
        plan(&request, &delivery, Route::OneShot).expect("valid plan")
    }

    fn summary_query() -> Query {
        Query { views: vec![ViewSpec::Summary], ..Query::default() }
    }

    /// The request and the delivery a test's `OpenFixture` spells, split the way the two
    /// models now divide it: what the answer says, and how it is carried out.
    fn split(root: &Path, config: &OpenFixture, query: &Query) -> (Request, Delivery) {
        let (basis, delivery) = config.split(root);
        (Request::new(basis, query.clone(), std::time::UNIX_EPOCH), delivery)
    }

    /// [`prepare_report`] as these tests ask for it: one configuration, one query.
    fn prepared(
        root: &Path,
        config: &OpenFixture,
        query: &Query,
    ) -> Result<(Report, PendingSave, PerformanceSummary)> {
        let (request, delivery) = split(root, config, query);
        prepare_report(&request, &delivery)
    }

    /// [`prepared`], keeping the scan diagnostics.
    fn prepared_with_diagnostics(
        root: &Path,
        config: &OpenFixture,
        query: &Query,
    ) -> Result<(Report, PendingSave, PerformanceSummary, Option<crate::scan::ScanDiagnostics>)>
    {
        let (request, delivery) = split(root, config, query);
        prepare_report_with_scan_diagnostics(&request, &delivery)
    }

    fn config(policy: CachePolicy, cache_path: Option<PathBuf>) -> OpenFixture {
        OpenFixture { scan: ScanConfig::default(), cache_path, policy, ..OpenFixture::default() }
    }

    /// [`config`] with `.gitignore` observation turned off.
    fn blind(policy: CachePolicy, cache_path: Option<PathBuf>) -> OpenFixture {
        OpenFixture {
            scan: ScanConfig { read_controls: false, ..ScanConfig::default() },
            ..config(policy, cache_path)
        }
    }

    fn controls_config(
        policy: CachePolicy,
        cache_path: PathBuf,
        read_controls: bool,
    ) -> OpenFixture {
        OpenFixture {
            scan: ScanConfig { read_controls, ..ScanConfig::default() },
            cache_path: Some(cache_path),
            policy,
            ..OpenFixture::default()
        }
    }

    /// `fixture`, answered from its snapshot alone.
    fn stale(fixture: OpenFixture) -> OpenFixture {
        OpenFixture { stale_ok: true, ..fixture }
    }

    fn seed_controls_snapshot(root: &Path, cache_path: PathBuf) {
        fs::write(root.join(".gitignore"), b"ignored.log\n").expect("control file");
        fs::write(root.join("ignored.log"), b"ignored").expect("ignored file");
        crate::open_fixture(root, &controls_config(CachePolicy::Auto, cache_path, true))
            .expect("seed controls-on snapshot");
    }

    #[test]
    fn planner_uses_compact_state_only_when_the_request_proves_it_is_sufficient() {
        let off = blind(CachePolicy::Off, Some(PathBuf::from("unused.fdu")));
        assert_eq!(planned(&off, &summary_query()).retained, RetainedState::Summary);

        for policy in [CachePolicy::Auto, CachePolicy::On] {
            let unavailable = blind(policy, None);
            assert_eq!(planned(&unavailable, &summary_query()).retained, RetainedState::Summary);
        }

        let mut several_views = summary_query();
        several_views.views.push(ViewSpec::Types);
        assert_eq!(planned(&off, &several_views).retained, RetainedState::FullIndex);

        let mut filtered = summary_query();
        filtered.selection.include.push(Pattern::parse("*.rs").expect("pattern"));
        assert_eq!(planned(&off, &filtered).retained, RetainedState::FullIndex);

        // Changed deliberately by fdu-1ovb. The reducer classifies each entry with the
        // control table the index would hold, so the default summary, whose row carries an
        // ignored share, no longer needs the index; this assertion used to expect
        // `FullIndex`. `compact_summary_equals_the_indexed_summary_under_every_control_case`
        // is what licenses the change.
        let observing = config(CachePolicy::Off, None);
        assert!(observing.scan.read_controls, "observation is the default");
        assert_eq!(planned(&observing, &summary_query()).retained, RetainedState::Summary);
        // Selecting by ignored state is a filter, and a narrowed population is one retained
        // in the scope as well: both still need the index, whose traversal answers them.
        let mut by_ignored = summary_query();
        by_ignored.selection.ignored = IgnoredEntries::Exclude;
        assert_eq!(planned(&observing, &by_ignored).retained, RetainedState::FullIndex);
        for population in [IgnoredEntries::Exclude, IgnoredEntries::Only] {
            let narrowed = OpenFixture {
                scan: ScanConfig { population, ..ScanConfig::default() },
                ..config(CachePolicy::Off, None)
            };
            let mut query = summary_query();
            query.selection.ignored = population;
            assert_eq!(planned(&narrowed, &query).retained, RetainedState::FullIndex);
        }
    }

    #[test]
    fn an_available_snapshot_does_not_force_the_index_for_a_metadata_summary() {
        // A loaded snapshot cannot save the work an unfiltered metadata summary is
        // already doing: revalidation stats every entry regardless, so retaining the
        // index and writing it back is additive cost with nothing to amortise it. The
        // compact tier stays selected so the common one-shot totals request pays for a
        // walk and nothing else.
        let cached = blind(CachePolicy::Auto, Some(PathBuf::from("cache.fdu")));
        assert_eq!(
            planned(&cached, &summary_query()).retained,
            RetainedState::Summary,
            "a present snapshot must not force the index"
        );
    }

    #[test]
    fn deliveries_whose_intent_is_the_snapshot_itself_still_retain_the_index() {
        // These two are not cost decisions. A stale answer must come without touching the
        // tree, so it has no scan to reduce; `On` is an explicit request to leave a
        // current snapshot, which means materialising the index that gets written.
        let cached = || blind(CachePolicy::Auto, Some(PathBuf::from("cache.fdu")));
        for (name, delivery) in [
            ("stale", stale(cached())),
            ("on", OpenFixture { policy: CachePolicy::On, ..cached() }),
        ] {
            assert_eq!(
                planned(&delivery, &summary_query()).retained,
                RetainedState::FullIndex,
                "{name} needs the index to honour its contract"
            );
        }
    }

    #[test]
    fn a_one_shot_metadata_query_does_not_read_the_snapshot_it_cannot_use() {
        // Revalidation stats every entry regardless of what the snapshot holds, so for
        // a metadata query the load and the reconciliation against it are additive cost:
        // measured on macOS/APFS over 494,031 entries, warm revalidation cost 4.8 s
        // against 3.6 s for the cold path. This holds for every view, not just the
        // compact summary — the tree default was the measured case.
        let mut tree_query = summary_query();
        tree_query.views = vec![ViewSpec::Tree];
        for policy in [CachePolicy::Auto, CachePolicy::On] {
            let cached = config(policy, Some(PathBuf::from("cache.fdu")));
            assert!(
                planned(&cached, &tree_query).load != Load::Snapshot,
                "{policy:?} must not pay for a read that saves no work"
            );
        }
    }

    #[test]
    fn the_snapshot_is_read_where_reading_pays_or_is_the_contract() {
        // A stale answer comes from the snapshot; reading it is the request itself.
        let only = stale(config(CachePolicy::Auto, Some(PathBuf::from("cache.fdu"))));
        assert_eq!(planned(&only, &summary_query()).load, Load::Snapshot);

        // Analysis reuses the content sidecar, which avoids re-reading file bodies —
        // the one measured case where a warm read wins (639 ms to 325 ms).
        let analyzed = OpenFixture {
            analysis: crate::content::AnalysisRequest {
                profile: crate::content::AnalysisSet::NONE.with_code(),
                ..Default::default()
            },
            ..config(CachePolicy::Auto, Some(PathBuf::from("cache.fdu")))
        };
        assert_eq!(planned(&analyzed, &summary_query()).load, Load::Snapshot);

        // `Off` never reads by definition.
        let never = config(CachePolicy::Off, Some(PathBuf::from("cache.fdu")));
        assert_eq!(planned(&never, &summary_query()).load, Load::None);
    }

    #[test]
    fn auto_persists_where_a_later_request_reads_what_it_stores() {
        // The policy table in one place. A one-shot metadata report under `Auto` leaves
        // nothing: no later one-shot report reads it, and it cost 0.26 s of a 1.51 s
        // default run on a million-entry Linux tree. Analysis keeps its sidecar and the
        // snapshot it pairs with; retained routes are their own later reader. `On` writes
        // everywhere a verified answer is produced, `Off` and a stale answer nowhere.
        let cache = Some(PathBuf::from("cache.fdu"));
        let tree = Query { views: vec![ViewSpec::Tree], ..Query::default() };
        let persists = |fixture: &OpenFixture, query: &Query, route: Route| {
            let (request, delivery) = split(Path::new("."), fixture, query);
            plan(&request, &delivery, route).expect("valid plan").persists()
        };
        for (policy, one_shot, analysis, retained) in [
            (CachePolicy::Auto, false, true, true),
            (CachePolicy::On, true, true, true),
            (CachePolicy::Off, false, false, false),
        ] {
            let metadata = config(policy, cache.clone());
            assert_eq!(persists(&metadata, &tree, Route::OneShot), one_shot, "{policy:?}");
            assert_eq!(
                persists(&analyzing(metadata.clone()), &tree, Route::OneShot),
                analysis,
                "{policy:?} with analysis"
            );
            for route in [Route::Retained, Route::Refresh, Route::Watch] {
                assert_eq!(persists(&metadata, &tree, route), retained, "{policy:?} {route:?}");
            }
        }
        let stale_answer = stale(config(CachePolicy::On, cache));
        assert!(!persists(&stale_answer, &tree, Route::OneShot), "a stale answer writes nothing");
        assert!(!persists(&stale_answer, &tree, Route::Retained), "a stale answer writes nothing");
    }

    #[test]
    fn an_analysis_request_never_selects_the_compact_summary_tier() {
        // Analysis reads file contents keyed by retained entries and writes its own
        // sidecar, so the aggregate-only tier cannot answer it even though the request
        // otherwise looks like the uncached unfiltered summary the planner compacts.
        let off = blind(CachePolicy::Off, None);
        assert_eq!(planned(&off, &summary_query()).retained, RetainedState::Summary);

        for profile in [
            crate::content::AnalysisSet::NONE.with_lines(),
            crate::content::AnalysisSet::NONE.with_code(),
            crate::content::AnalysisSet::NONE.with_words(),
            crate::content::AnalysisSet::ALL,
        ] {
            let analyzed = OpenFixture {
                analysis: crate::content::AnalysisRequest { profile, ..Default::default() },
                ..blind(CachePolicy::Off, None)
            };
            assert_eq!(
                planned(&analyzed, &summary_query()).retained,
                RetainedState::FullIndex,
                "{profile:?} must retain the index"
            );
        }
    }

    #[test]
    fn a_repeated_one_shot_report_scans_cold_while_open_still_revalidates() {
        // The same snapshot, two consumers, two right answers. A one-shot report cannot
        // amortise a snapshot load, so its second run scans cold again; a caller holding
        // the index through `open` amortises it across everything that follows, so its
        // second open still takes the warm path. Both report truthfully.
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("file.txt"), b"contents").expect("file");
        let cache = tempfile::tempdir().expect("cache dir");
        let auto = config(CachePolicy::Auto, Some(cache.path().join("cache.fdu")));
        let on = OpenFixture { policy: CachePolicy::On, ..auto.clone() };
        let mut tree_query = summary_query();
        tree_query.views = vec![ViewSpec::Tree];

        let (first, pending, _) = prepared(root.path(), &on, &tree_query).expect("first report");
        pending.join().expect("first save");
        assert_eq!(first.provenance.source, ReportSource::ColdScan);
        assert!(on.cache_path.as_deref().expect("path").exists(), "`on` persists");

        let (second, pending, performance) =
            prepared(root.path(), &auto, &tree_query).expect("second report");
        pending.join().expect("second save");
        assert_eq!(
            second.provenance.source,
            ReportSource::ColdScan,
            "a repeated one-shot must not pay for a read that saves no work"
        );
        assert_eq!(performance.walked_files, 1, "the walk still happened");

        // A default report and a default `open` both observe control state, so they share
        // one snapshot scope and the `open` starts from the report's snapshot.
        let (_, open_report) = crate::open_fixture(root.path(), &auto).expect("library open");
        assert_eq!(
            open_report.path_taken,
            OpenPath::WarmRevalidate,
            "a caller holding the index still amortises the load"
        );
    }

    #[test]
    fn a_stale_answer_reads_what_an_on_report_leaves_and_auto_leaves_nothing() {
        // `auto` skips the write a one-shot metadata report cannot use, so a stale answer
        // after it has nothing to read and says how to leave something; `on` is that way,
        // and what it leaves is answered from without touching the tree.
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("file.txt"), b"contents").expect("file");
        let cache = tempfile::tempdir().expect("cache dir");
        let auto = config(CachePolicy::Auto, Some(cache.path().join("cache.fdu")));
        let mut tree_query = summary_query();
        tree_query.views = vec![ViewSpec::Tree];

        let (_, pending, _) = prepared(root.path(), &auto, &tree_query).expect("report");
        assert!(!pending.writes_metadata(), "`auto` starts no metadata write");
        pending.join().expect("nothing to save");
        assert!(!cache.path().join("cache.fdu").exists(), "`auto` leaves nothing");
        let only = stale(config(CachePolicy::Auto, Some(cache.path().join("cache.fdu"))));
        let missing = prepared(root.path(), &only, &tree_query).expect_err("nothing to read");
        assert!(missing.to_string().contains("with the `on` cache policy"), "{missing}");

        let on = OpenFixture { policy: CachePolicy::On, ..auto };
        let (_, pending, _) = prepared(root.path(), &on, &tree_query).expect("report");
        pending.join().expect("save");

        let (from_cache, pending, performance, diagnostics) =
            prepared_with_diagnostics(root.path(), &only, &tree_query).expect("cache-only report");
        pending.join().expect("no save");
        assert_eq!(from_cache.provenance.source, ReportSource::CacheOnly);
        assert_eq!(performance.walked_files, 0, "cache-only never touches the tree");
        assert!(diagnostics.is_none(), "a cache-only open has no scan trace");
    }

    #[test]
    fn controls_on_snapshot_projects_to_an_equivalent_controls_off_cache_only_report() {
        let root = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let cache_path = cache.path().join("cache.fdu");
        fs::create_dir(root.path().join("src")).expect("source dir");
        fs::write(root.path().join("src/lib.rs"), b"library").expect("source file");
        seed_controls_snapshot(root.path(), cache_path.clone());

        let controls_off = stale(controls_config(CachePolicy::Auto, cache_path, false));
        let query = Query {
            views: vec![
                ViewSpec::Summary,
                ViewSpec::Tree,
                ViewSpec::Families,
                ViewSpec::Types,
                ViewSpec::Extensions,
                ViewSpec::Languages,
                ViewSpec::Largest,
                ViewSpec::Recent,
                ViewSpec::Files,
            ],
            ..Query::default()
        };
        let (projected, pending, performance) =
            prepared(root.path(), &controls_off, &query).expect("projected report");
        pending.join().expect("no cache-only save");

        let cold = OpenFixture {
            policy: CachePolicy::Off,
            stale_ok: false,
            cache_path: None,
            ..controls_off
        };
        let (mut expected, pending, _) =
            prepared(root.path(), &cold, &query).expect("controls-off cold report");
        pending.join().expect("no cold save");
        expected.provenance = projected.provenance.clone();

        assert_eq!(performance.source, ReportSource::CacheOnly);
        assert_eq!(projected.scope, cold.scan.scope());
        assert_eq!(projected.ignore_rules, crate::control::ControlCoverage::NotObserved);
        assert_eq!(
            crate::report_format::render(&projected, crate::report_format::Format::Json, false,)
                .expect("compatible report format"),
            crate::report_format::render(&expected, crate::report_format::Format::Json, false,)
                .expect("compatible report format"),
        );
    }

    #[test]
    fn controls_on_snapshot_projects_to_controls_off_auto_report() {
        let root = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let cache_path = cache.path().join("cache.fdu");
        seed_controls_snapshot(root.path(), cache_path.clone());

        let controls_off = OpenFixture {
            analysis: crate::content::AnalysisRequest {
                profile: crate::content::AnalysisSet::NONE.with_lines(),
                ..Default::default()
            },
            ..controls_config(CachePolicy::Auto, cache_path, false)
        };
        let (report, pending, performance) = prepared(root.path(), &controls_off, &summary_query())
            .expect("controls-off warm projection");
        pending.join().expect("save content only");

        assert_eq!(report.provenance.source, ReportSource::WarmRevalidate);
        assert_eq!(report.scope, controls_off.scan.scope());
        assert_eq!(performance.source, ReportSource::WarmRevalidate);
    }

    /// Control sources past both limits, so no scan can observe them without saying so.
    ///
    /// The root rule is longer than the line limit and the nested source is past the
    /// table budget. An observing scan refuses both and records it in its control coverage;
    /// a report whose scope observes no control state read neither.
    fn write_unobservable_controls(root: &Path) {
        let mut rule = vec![b'a'; crate::control::DEFAULT_CONTROL_LINE_LIMIT + 1];
        rule.push(b'\n');
        fs::write(root.join(".gitignore"), rule).expect("oversized rule");
        fs::create_dir(root.join("vendored")).expect("nested directory");
        fs::write(
            root.join("vendored/.gitignore"),
            b"x\n".repeat(crate::control::DEFAULT_CONTROL_BUDGET / 2),
        )
        .expect("oversized source");
    }

    #[test]
    fn a_one_shot_report_observes_control_state_as_its_caller_configures() {
        // A default report reads every `.gitignore`, and a file past a bound is refused and
        // named without ending the report or its snapshot. A report that turns observation
        // off reads neither file, and says so in its report and in any snapshot it writes;
        // `on` makes each one write.
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("file.txt"), b"contents").expect("file");
        write_unobservable_controls(root.path());
        let mut tree_query = summary_query();
        tree_query.views = vec![ViewSpec::Tree];

        for read_controls in [true, false] {
            let cache = tempfile::tempdir().expect("cache dir");
            let cache_path = cache.path().join("cache.fdu");
            let caller = controls_config(CachePolicy::On, cache_path.clone(), read_controls);
            for query in [summary_query(), tree_query.clone()] {
                let (report, pending, _) = prepared(root.path(), &caller, &query)
                    .expect("a refused control file ends nothing");
                pending.join().expect("save");
                assert!(
                    report.status.complete,
                    "a refusal is not a partial: {:?}",
                    report.status.errors
                );
                assert_eq!(report.scope, caller.scan.scope());
                match &report.ignore_rules {
                    crate::control::ControlCoverage::Observed(coverage) => {
                        assert!(read_controls, "observed only when asked");
                        assert_eq!(coverage.refused, 2, "{coverage:?}");
                        assert_eq!(report.notes.len(), 2, "{:?}", report.notes);
                        assert!(
                            report.notes[0].contains("2 ignore files not applied"),
                            "{:?}",
                            report.notes
                        );
                        assert_eq!(
                            report.notes[1],
                            "note: gitignored subtotals are unavailable where governing rules could not be verified"
                        );
                    }
                    crate::control::ControlCoverage::NotObserved => {
                        assert!(!read_controls, "unobserved only when turned off");
                        assert!(report.notes.is_empty(), "{:?}", report.notes);
                    }
                }
            }

            let saved = crate::snapshot::load(&cache_path)
                .expect("load the snapshot")
                .expect("the index tier persisted");
            assert_eq!(saved.scope(), caller.scan.scope());
            assert_eq!(saved.controls().is_ok(), read_controls);
        }
    }

    /// Which failure a run names, and what kind of failure it is, must not depend on how it
    /// was delivered.
    ///
    /// A scope this build cannot honour is refused by every policy, and by a stale answer,
    /// which never scans: under `--stale-ok` the scan that would have refused it never runs,
    /// so the run used to report a snapshot miss instead -- the same request naming two
    /// different failures depending on its delivery, which the path-independence registry
    /// records as `refusal-order` for `--one-filesystem` on Windows. `follow_symlinks` is
    /// the same rule on every platform, so this test runs where the Windows case cannot.
    ///
    /// The refusal is the request model's typed one, not an engine error the surfaces then
    /// classify differently: reporting it as an engine error made the command line exit 1
    /// where Python raised `ValueError`, one request with two kinds of outcome.
    #[test]
    fn a_scope_this_build_cannot_honour_is_refused_before_any_snapshot_is_read() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("file.txt"), b"contents").expect("file");
        let cache = tempfile::tempdir().expect("cache dir");
        let cache_path = cache.path().join("cache.fdu");

        // A usable snapshot exists, so a stale read of a scope this build supports answers
        // from it.
        let warm = config(CachePolicy::On, Some(cache_path.clone()));
        let (_, pending, _) = prepared(root.path(), &warm, &summary_query()).expect("warm");
        pending.join().expect("save");
        let (_, pending, _) = prepared(
            root.path(),
            &stale(config(CachePolicy::Auto, Some(cache_path.clone()))),
            &summary_query(),
        )
        .expect("the snapshot answers a supported scope");
        pending.join().expect("no save");

        let unsupported = ScanConfig { follow_symlinks: true, ..ScanConfig::default() };
        for (policy, stale_ok) in [
            (CachePolicy::Auto, true),
            (CachePolicy::Off, false),
            (CachePolicy::Auto, false),
            (CachePolicy::On, false),
        ] {
            let asked = OpenFixture {
                scan: unsupported.clone(),
                stale_ok,
                ..config(policy, Some(cache_path.clone()))
            };
            let refused = prepared(root.path(), &asked, &summary_query())
                .expect_err("a scope this build cannot honour has no answer at any policy");
            assert!(
                matches!(
                    refused,
                    Error::InvalidRequest(crate::query::RequestError::ScopeUnsupported {
                        axis: crate::query::ScopeAxis::FollowSymlinks,
                        ..
                    })
                ),
                "{policy:?} must refuse the request rather than fail the operation: {refused}"
            );
            assert_eq!(
                refused.to_string(),
                "unsupported scan configuration: follow_symlinks requires cycle, root-boundary, \
                 and filesystem-boundary semantics",
                "{policy:?} must name the scope it cannot honour"
            );
        }
    }

    #[test]
    fn a_report_that_reads_no_gitignore_refuses_to_select_by_ignored_state() {
        // No entry of a scan that read no rule can be shown to be ignored or not, so the
        // request is refused rather than answered with every entry or none.
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join(".gitignore"), b"*.log\n").expect("control file");
        fs::write(root.path().join("debug.log"), b"ignored").expect("ignored file");
        let mut only = summary_query();
        only.selection.ignored = IgnoredEntries::Only;

        // Refused by the request model before anything is scanned: the compact summary
        // tier this request would take reaches no reader, so a check made there would not
        // cover this route at all.
        assert!(matches!(
            prepared(root.path(), &blind(CachePolicy::Off, None), &only),
            Err(Error::InvalidRequest(crate::query::RequestError::IgnoredWithoutObservation(
                IgnoredEntries::Only
            )))
        ));

        let (report, pending, _) =
            prepared(root.path(), &config(CachePolicy::Off, None), &only).expect("observed");
        pending.join().expect("no save");
        let Section::Summary(row) = report.sections[0] else { panic!("a summary") };
        assert_eq!((row.files, row.bytes), (1, 7), "only the ignored file is selected");
    }

    #[test]
    fn an_on_reports_snapshot_serves_either_cache_only_report_but_not_the_reverse() {
        // Every surface reaches this planner observing control state by default, so a
        // snapshot a default-scope report wrote under `on` serves the next one. A cache-only report that
        // turns observation off also answers from it, reading only the all-entry facts. An
        // opted-out snapshot holds no classification, so it cannot serve a default report,
        // and the refusal says why and what recovers.
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("file.txt"), b"contents").expect("file");
        let mut tree_query = summary_query();
        tree_query.views = vec![ViewSpec::Tree];

        for (writer, reader) in [(true, true), (true, false), (false, false), (false, true)] {
            let cache = tempfile::tempdir().expect("cache dir");
            let cache_path = cache.path().join("cache.fdu");
            let write = controls_config(CachePolicy::On, cache_path.clone(), writer);
            let (_, pending, _) =
                prepared(root.path(), &write, &tree_query).expect("writing report");
            pending.join().expect("save");

            let read = stale(controls_config(CachePolicy::Auto, cache_path, reader));
            match prepared(root.path(), &read, &tree_query) {
                Ok((report, pending, _)) => {
                    pending.join().expect("no save");
                    assert!(writer || !reader, "writer {writer} served reader {reader}");
                    assert_eq!(report.provenance.source, ReportSource::CacheOnly);
                    assert_eq!(
                        matches!(report.ignore_rules, crate::control::ControlCoverage::Observed(_)),
                        reader,
                        "a report describes the scope it asked for"
                    );
                }
                Err(Error::Snapshot(message)) => {
                    assert!(!writer && reader, "writer {writer}, reader {reader}: {message}");
                    assert!(message.contains(".gitignore state"), "names the cause: {message}");
                    assert!(message.contains("verified answer"), "names the remedy: {message}");
                }
                Err(other) => panic!("writer {writer}, reader {reader}: {other}"),
            }
        }
    }

    #[test]
    fn a_cache_only_open_answers_from_an_on_reports_snapshot() {
        // A default-scope report and a default `open` share one scope, so the one delivery
        // that forbids a scan answers the `open` from the snapshot the report left under
        // `on`, with the classification the report observed.
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join(".gitignore"), b"*.log\n").expect("control file");
        fs::write(root.path().join("debug.log"), b"ignored").expect("ignored file");
        let cache = tempfile::tempdir().expect("cache dir");
        let cache_path = cache.path().join("cache.fdu");
        let mut tree_query = summary_query();
        tree_query.views = vec![ViewSpec::Tree];

        let on = config(CachePolicy::On, Some(cache_path.clone()));
        let (_, pending, _) = prepared(root.path(), &on, &tree_query).expect("report");
        pending.join().expect("save");

        let only = stale(config(CachePolicy::Auto, Some(cache_path)));
        let (index, report) = crate::open_fixture(root.path(), &only).expect("the shared snapshot");
        assert_eq!(report.path_taken, OpenPath::CacheOnly);
        assert_eq!(index.is_ignored(Path::new("debug.log")).ok(), Some(Some(true)));
    }

    #[test]
    fn an_open_that_opts_out_of_control_state_shares_an_opted_out_reports_snapshot() {
        // A report and an `open` that both turn observation off write and want one scope,
        // so that open answers from the snapshot the report left under `on` without
        // touching the tree, as the one delivery that forbids a scan, and says it cannot
        // classify ignored entries.
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("file.txt"), b"contents").expect("file");
        let cache = tempfile::tempdir().expect("cache dir");
        let cache_path = cache.path().join("cache.fdu");
        let mut tree_query = summary_query();
        tree_query.views = vec![ViewSpec::Tree];

        let on = blind(CachePolicy::On, Some(cache_path.clone()));
        let (_, pending, _) = prepared(root.path(), &on, &tree_query).expect("report");
        pending.join().expect("save");
        assert!(cache_path.exists(), "the report left a snapshot");

        let only = stale(blind(CachePolicy::Auto, Some(cache_path)));
        let (index, report) = crate::open_fixture(root.path(), &only).expect("the shared snapshot");
        assert_eq!(report.path_taken, OpenPath::CacheOnly);
        assert!(matches!(
            index.is_ignored(Path::new("file.txt")),
            Err(Error::ControlStateNotObserved)
        ));
    }

    #[test]
    fn compact_summary_matches_the_indexed_summary_exactly() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::create_dir(root.path().join("src")).expect("directory");
        fs::write(root.path().join("src/lib.rs"), b"library").expect("file");
        fs::write(root.path().join("README.md"), b"read me").expect("file");
        #[cfg(unix)]
        std::os::unix::fs::symlink("README.md", root.path().join("readme-link")).expect("symlink");

        let query = summary_query();
        // Two workers so the compact fold exercises StreamingEmission recycle even on
        // a one-vCPU runner (`threads: None` would take the serial walker there).
        let off = OpenFixture {
            scan: ScanConfig { read_controls: false, threads: Some(2), ..ScanConfig::default() },
            ..blind(CachePolicy::Off, None)
        };
        let (compact, pending, performance) =
            prepared(root.path(), &off, &query).expect("compact report");
        pending.join().expect("no pending compact save");
        assert_eq!(performance.walked_files, 2);
        assert_eq!(performance.walked_bytes, 14);

        // This report turns control observation off, so the index it must match exactly is
        // opened under that scope too.
        let (index, _open_report) = crate::open_fixture(root.path(), &off).expect("indexed scan");
        let indexed = report(
            &index,
            &crate::test_support::read_of(&index, query.clone()),
            compact.provenance.generated_at,
        )
        .expect("report");

        let Section::Summary(compact_row) = compact.sections[0] else {
            panic!("compact plan did not return a summary")
        };
        let Section::Summary(indexed_row) = indexed.sections[0] else {
            panic!("indexed plan did not return a summary")
        };
        assert_eq!(compact_row.files, indexed_row.files);
        assert_eq!(compact_row.dirs, indexed_row.dirs);
        assert_eq!(compact_row.bytes, indexed_row.bytes);
        assert_eq!(compact_row.allocated, indexed_row.allocated);
        assert_eq!(compact_row.newest_mtime_ns, indexed_row.newest_mtime_ns);
        assert_eq!(compact.root, indexed.root);
        assert_eq!(compact.scope, indexed.scope);
        assert_eq!(compact.status.complete, indexed.status.complete);
        assert_eq!(compact.provenance.freshness, indexed.provenance.freshness);
    }

    /// One tree for the transient-versus-indexed differential, the scope it is summarized
    /// under, and what the index must say of it, so a case that stopped exercising what it
    /// names fails instead of agreeing vacuously.
    struct ControlCase {
        name: &'static str,
        root: tempfile::TempDir,
        scan: ScanConfig,
        /// Whether the row carries an ignored share, which a refused or unreadable control
        /// file withholds.
        share: bool,
        /// Control files refused.
        refused: u64,
        /// Which files a budget refuses depends on the order controls arrive in, so the two
        /// routes are compared only where that order is fixed: with one worker.
        order_dependent: bool,
        /// Walk errors the case induces.
        errors: bool,
        /// How control lookups under the case's root resolve a case variant of the name.
        lookups: crate::test_support::CaseLookups,
        /// For a tree whose rules sit in a case variant of the name: whether they govern,
        /// which is whether a lookup of `.gitignore` resolves to the variant.
        variant_governs: Option<bool>,
        /// A file the case made unreadable, readable again when the case is dropped so its
        /// tree can be removed even after a failed assertion. Only Unix can make one.
        #[cfg(unix)]
        denied: Option<PathBuf>,
    }

    impl Drop for ControlCase {
        fn drop(&mut self) {
            #[cfg(unix)]
            if let Some(path) = &self.denied {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
            }
        }
    }

    fn put(root: &Path, path: &str, contents: &[u8]) {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("a parent")).expect("parent directories");
        fs::write(path, contents).expect("fixture file");
    }

    /// Negation, nested files, directory-only and anchored rules, rules below an ignored
    /// directory that try to re-include, a re-included directory, a control file that
    /// ignores itself, and a listing long enough to split across small batches.
    fn rules_tree() -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("tempdir");
        let root_path = root.path();
        put(
            root_path,
            ".gitignore",
            b"*.log\n!keep.log\n/anchored.txt\ncache/\nnode_modules/\nvendor/*\n!vendor/keep/\n",
        );
        put(root_path, "a.log", b"alog");
        put(root_path, "keep.log", b"keeplog");
        put(root_path, "anchored.txt", b"anchored");
        put(root_path, "cache", b"a file named like a directory rule");
        put(root_path, "README.md", b"readme!");
        put(root_path, "src/anchored.txt", b"not anchored here");
        put(root_path, "src/x.log", b"xlog-");
        put(root_path, "src/keep.log", b"kept everywhere");
        put(root_path, "src/main.rs", b"fn main() {}");
        put(root_path, "sub/.gitignore", b"!*.log\n*.tmp\n");
        put(root_path, "sub/y.log", b"re-included");
        put(root_path, "sub/z.tmp", b"tmp");
        put(root_path, "sub/cache/data.bin", b"cached bytes");
        put(root_path, "sub/deep/.gitignore", b"*\n!.gitignore\n");
        put(root_path, "sub/deep/f.txt", b"deep");
        put(root_path, "sub/deep/inner/g.txt", b"deeper");
        put(root_path, "node_modules/pkg/.gitignore", b"!*\n");
        put(root_path, "node_modules/pkg/index.js", b"module.exports = 1;");
        put(root_path, "node_modules/pkg/lib/a.js", b"a");
        put(root_path, "vendor/a.c", b"int a;");
        put(root_path, "vendor/keep/k.c", b"int k;");
        put(root_path, "vendor/drop/d.c", b"int d;");
        put(root_path, "selfish/.gitignore", b".gitignore\n*.bak\n");
        put(root_path, "selfish/x.bak", b"backup");
        put(root_path, "selfish/y.txt", b"kept");
        put(root_path, "many/.gitignore", b"*[02468].dat\n");
        for file in 0..40 {
            put(root_path, &format!("many/f{file:02}.dat"), &vec![b'.'; file + 1]);
        }
        root
    }

    /// The control case whose only control-like file is `.GITIGNORE`.
    const CASE_VARIANT: &str = "a case-variant control name";

    /// The control case whose directory lists `.gitignore` beside `.GITIGNORE`.
    const CASE_BOTH: &str = "both spellings of the control name";

    fn control_cases() -> Vec<ControlCase> {
        let case = |name, root, scan, share, refused| ControlCase {
            name,
            root,
            scan,
            share,
            refused,
            order_dependent: false,
            errors: false,
            lookups: crate::test_support::CaseLookups::Host,
            variant_governs: None,
            #[cfg(unix)]
            denied: None,
        };
        let mut cases = vec![
            case("rules", rules_tree(), ScanConfig::default(), true, 0),
            case(
                "rules with hidden entries pruned",
                rules_tree(),
                ScanConfig {
                    hidden: Some(std::sync::Arc::new(crate::HiddenPolicy::prune_hidden(Vec::<
                        std::ffi::OsString,
                    >::new(
                    )))),
                    ..ScanConfig::default()
                },
                true,
                0,
            ),
            case(
                "rules under a depth bound",
                rules_tree(),
                ScanConfig { max_depth: Some(1), ..ScanConfig::default() },
                true,
                0,
            ),
        ];

        let empty = tempfile::tempdir().expect("tempdir");
        put(empty.path(), "only.txt", b"no rules anywhere");
        cases.push(case("no control file", empty, ScanConfig::default(), true, 0));

        // A line over the limit refuses its whole file, whatever order it arrives in.
        let long = tempfile::tempdir().expect("tempdir");
        put(long.path(), ".gitignore", b"*.log\n");
        put(long.path(), "x.log", b"ignored");
        let mut line = vec![b'x'; crate::control::DEFAULT_CONTROL_LINE_LIMIT + 1];
        line.push(b'\n');
        put(long.path(), "long/.gitignore", &line);
        put(long.path(), "long/kept.txt", b"kept");
        put(long.path(), "long/y.log", b"unknown");
        cases.push(case("line limit", long, ScanConfig::default(), false, 1));

        // Each nested file alone exceeds what the root leaves of the budget, so all seventy
        // are refused in any order: more than a note names and more than a report retains.
        let over = tempfile::tempdir().expect("tempdir");
        put(over.path(), ".gitignore", b"*.log\n");
        let mut oversized = vec![b'x'; 200];
        oversized.push(b'\n');
        for directory in 0..70 {
            put(over.path(), &format!("d{directory:02}/.gitignore"), &oversized);
            put(over.path(), &format!("d{directory:02}/f.log"), b"log");
        }
        let limits = crate::control::ControlLimits { budget: Some(256), ..Default::default() };
        cases.push(case(
            "every nested file over the budget",
            over,
            ScanConfig { control_limits: limits, ..ScanConfig::default() },
            false,
            70,
        ));

        // Any two of four fit the budget and no third does: which two is arrival order.
        let competing = tempfile::tempdir().expect("tempdir");
        for (position, directory) in ["a", "b", "c", "d"].into_iter().enumerate() {
            let mut rules = format!("r{position}").into_bytes();
            rules.extend(std::iter::repeat_n(b'y', 100));
            rules.push(b'\n');
            put(competing.path(), &format!("{directory}/.gitignore"), &rules);
            put(competing.path(), &format!("{directory}/file.txt"), b"file");
        }
        let limits = crate::control::ControlLimits { budget: Some(1000), ..Default::default() };
        let mut competing = case(
            "competing for the budget",
            competing,
            ScanConfig { control_limits: limits, ..ScanConfig::default() },
            false,
            2,
        );
        competing.order_dependent = true;
        cases.push(competing);

        // A case variant of the control name governs exactly where a lookup of `.gitignore`
        // resolves to it, as the open git makes does (fdu-0w1b): on a case-insensitive
        // volume, and through folded lookups on a case-sensitive host. Enough entries fill
        // a default batch before the listing reaches `.GITIGNORE` (enumeration order
        // permitting), so the transient fold's probe must find what the listed variant's
        // read and the index find. Hidden pruning drops the variant's row, not its rules.
        let probe = tempfile::tempdir().expect("tempdir");
        for (lookups, governs) in crate::test_support::CaseLookups::on_this_host(probe.path()) {
            for hidden in [None, Some(Vec::<std::ffi::OsString>::new())] {
                let variant = tempfile::tempdir().expect("tempdir");
                put(variant.path(), ".GITIGNORE", b"*.log\n");
                for file in 0..1_200 {
                    put(variant.path(), &format!("f{file:04}.log"), b"log");
                }
                put(variant.path(), "kept.txt", b"kept");
                let scan = ScanConfig {
                    hidden: hidden
                        .map(|allow| std::sync::Arc::new(crate::HiddenPolicy::prune_hidden(allow))),
                    ..ScanConfig::default()
                };
                let mut variant = case(CASE_VARIANT, variant, scan, true, 0);
                variant.lookups = lookups;
                variant.variant_governs = Some(governs);
                cases.push(variant);
            }
        }

        // A case-sensitive directory can list both spellings, and only the exact name
        // governs there, whatever the lookups; a case-insensitive one cannot hold both, and
        // writing the second spelling there rewrites the first file.
        for lookups in
            [crate::test_support::CaseLookups::Host, crate::test_support::CaseLookups::Folded]
        {
            let both = tempfile::tempdir().expect("tempdir");
            put(both.path(), ".gitignore", b"*.log\n");
            put(both.path(), ".GITIGNORE", b"*.tmp\n");
            if fs::read(both.path().join(".gitignore")).expect("read") != b"*.log\n" {
                eprintln!("skipped {CASE_BOTH:?}: the temporary directory is case-insensitive");
                break;
            }
            for file in 0..1_200 {
                put(both.path(), &format!("f{file:04}.tmp"), b"tmp");
            }
            put(both.path(), "x.log", b"log");
            let mut both = case(CASE_BOTH, both, ScanConfig::default(), true, 0);
            both.lookups = lookups;
            cases.push(both);
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            // A control file that is a directory or a symlink applies no rules.
            let shapes = tempfile::tempdir().expect("tempdir");
            put(shapes.path(), ".gitignore", b"*.tmp\n");
            put(shapes.path(), "weird/.gitignore/inner.tmp", b"inner");
            put(shapes.path(), "weird/kept.txt", b"kept");
            put(shapes.path(), "rules.txt", b"*.txt\n");
            fs::create_dir(shapes.path().join("linked")).expect("directory");
            std::os::unix::fs::symlink("../rules.txt", shapes.path().join("linked/.gitignore"))
                .expect("symlink");
            put(shapes.path(), "linked/still.txt", b"still counted");
            cases.push(case(
                "control files that are not files",
                shapes,
                ScanConfig::default(),
                true,
                0,
            ));

            if crate::test_support::require_permission_bits() {
                let unreadable = tempfile::tempdir().expect("tempdir");
                put(unreadable.path(), ".gitignore", b"*.log\n");
                put(unreadable.path(), "x.log", b"ignored");
                put(unreadable.path(), "sub/.gitignore", b"!*.log\n");
                put(unreadable.path(), "sub/y.log", b"unknown");
                let control = unreadable.path().join("sub/.gitignore");
                let mut denied =
                    case("unreadable control file", unreadable, ScanConfig::default(), false, 0);
                fs::set_permissions(&control, fs::Permissions::from_mode(0o000))
                    .expect("deny the control file");
                denied.denied = Some(control);
                denied.errors = true;
                cases.push(denied);
            }
        }
        cases
    }

    /// The default summary of `root` under `scan`, from the transient tier and from the
    /// index, with the transient report's provenance, which describes the delivery rather
    /// than the tree, set to the index's once the parts both routes share agree.
    fn transient_and_indexed(root: &Path, scan: &ScanConfig, label: &str) -> (Report, Report) {
        let query = summary_query();
        let transient = OpenFixture { scan: scan.clone(), ..config(CachePolicy::Off, None) };
        assert_eq!(planned(&transient, &query).retained, RetainedState::Summary, "{label}");
        let (mut compact, pending, compact_performance) =
            prepared(root, &transient, &query).expect("transient report");
        pending.join().expect("the transient tier saves nothing");

        // `on` with a snapshot path is a delivery about the snapshot, so the same request
        // takes the index: the route every default summary took before fdu-1ovb.
        let cache = tempfile::tempdir().expect("cache dir");
        let indexed = OpenFixture {
            scan: scan.clone(),
            ..config(CachePolicy::On, Some(cache.path().join("snapshot.fdu")))
        };
        assert_eq!(planned(&indexed, &query).retained, RetainedState::FullIndex, "{label}");
        let (indexed, pending, indexed_performance) =
            prepared(root, &indexed, &query).expect("indexed report");
        pending.join().expect("save");

        assert_eq!(
            (
                compact_performance.walked_files,
                compact_performance.walked_bytes,
                compact_performance.walked_allocated,
                compact_performance.source,
            ),
            (
                indexed_performance.walked_files,
                indexed_performance.walked_bytes,
                indexed_performance.walked_allocated,
                indexed_performance.source,
            ),
            "{label}: walked totals"
        );
        assert_eq!(compact.provenance.source, indexed.provenance.source, "{label}");
        assert_eq!(compact.provenance.freshness, indexed.provenance.freshness, "{label}");
        assert_eq!(
            (compact.provenance.tiers.entries.source, compact.provenance.tiers.entries.freshness),
            (indexed.provenance.tiers.entries.source, indexed.provenance.tiers.entries.freshness),
            "{label}"
        );
        compact.provenance = indexed.provenance.clone();
        (compact, indexed)
    }

    /// The transient summary is the indexed summary, whole, for every control case the
    /// index classifies: negations, nested and self-ignoring control files, rules below an
    /// ignored directory, control files that are not files, refusals by the line limit
    /// and by the budget, an unreadable control file, a case-variant control name where
    /// the volume is case-insensitive, and the notes and coverage each produces. Across
    /// worker counts, batch sizes small enough to split every listing, and both traversal
    /// orders (fdu-1ovb).
    #[test]
    fn compact_summary_equals_the_indexed_summary_under_every_control_case() {
        use crate::report_format::{Format, render};

        let default_batch = ScanConfig::default().batch_size;
        for case in control_cases() {
            let _lookups = case.lookups.install(case.root.path());
            let mut compared = 0;
            for threads in [Some(1), Some(2), Some(4), None] {
                if case.order_dependent && threads != Some(1) {
                    continue;
                }
                for batch_size in [default_batch, 1, 3] {
                    for order in [crate::ScanOrder::BreadthFirst, crate::ScanOrder::DepthFirst] {
                        let scan = ScanConfig { batch_size, threads, order, ..case.scan.clone() };
                        let label = format!(
                            "{} ({:?} lookups, {threads:?} workers, batch {batch_size}, {order:?})",
                            case.name, case.lookups
                        );
                        let (compact, indexed) =
                            transient_and_indexed(case.root.path(), &scan, &label);

                        assert_eq!(format!("{compact:#?}"), format!("{indexed:#?}"), "{label}");
                        for format in [Format::Text, Format::Json, Format::Yaml] {
                            assert_eq!(
                                render(&compact, format, false).expect("render"),
                                render(&indexed, format, false).expect("render"),
                                "{label}: {format:?}"
                            );
                        }

                        // What each case exists to exercise.
                        let Section::Summary(row) = indexed.sections[0] else {
                            panic!("{label}: a summary")
                        };
                        let crate::control::ControlCoverage::Observed(coverage) =
                            &indexed.ignore_rules
                        else {
                            panic!("{label}: the default scope observes .gitignore")
                        };
                        assert_eq!(coverage.refused, case.refused, "{label}");
                        assert_eq!(row.ignored.is_some(), case.share, "{label}");
                        assert_eq!(!indexed.status.errors.is_empty(), case.errors, "{label}");
                        if case.share && case.name.starts_with("rules") {
                            let ignored = row.ignored.expect("a share");
                            assert!(
                                ignored.files > 0 && ignored.files < row.files && ignored.dirs > 0,
                                "{label}: {ignored:?} of {row:?}"
                            );
                        }
                        if let Some(governs) = case.variant_governs {
                            assert_eq!(coverage.applied, u64::from(governs), "{label}");
                            assert_eq!(
                                row.ignored.map(|ignored| ignored.files),
                                Some(if governs { 1_200 } else { 0 }),
                                "{label}"
                            );
                        }
                        if case.name == CASE_BOTH {
                            assert_eq!(coverage.applied, 1, "{label}: the exact name alone");
                            assert_eq!(
                                row.ignored.map(|ignored| ignored.files),
                                Some(1),
                                "{label}: only `x.log` is ignored"
                            );
                        }
                        if !case.share {
                            assert!(
                                indexed
                                    .notes
                                    .iter()
                                    .any(|note| note.contains("could not be verified")),
                                "{label}: {:?}",
                                indexed.notes
                            );
                        }
                        compared += 1;
                    }
                }
            }
            assert!(compared > 0, "{} was compared", case.name);
        }
    }

    #[test]
    fn compact_summary_never_creates_the_configured_snapshot() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("payload"), b"payload").expect("file");
        let cache = root.path().join("must-not-exist.fdu");

        let (report, pending, _) =
            prepared(root.path(), &blind(CachePolicy::Off, Some(cache.clone())), &summary_query())
                .expect("compact report");
        pending.join().expect("no pending compact save");

        assert!(report.status.complete);
        assert!(!cache.exists());
    }

    #[test]
    fn full_index_report_exposes_scan_diagnostics_when_requested() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::create_dir(root.path().join("nested")).expect("directory");
        fs::write(root.path().join("nested/file.txt"), b"trace me").expect("file");
        let query = Query { views: vec![ViewSpec::Tree], ..Query::default() };

        let (report, pending, performance, diagnostics) =
            prepared_with_diagnostics(root.path(), &config(CachePolicy::Off, None), &query)
                .expect("full-index report");
        pending.join().expect("no pending save");

        assert!(report.status.complete);
        assert_eq!(performance.walked_files, 1);
        let diagnostics = diagnostics.expect("full-index scan diagnostics");
        assert_eq!(diagnostics.schema, crate::scan::SCAN_DIAGNOSTICS_SCHEMA);
        assert_eq!(diagnostics.worker_policy.ready_directories_at_finish, 0);
        assert_eq!(diagnostics.worker_policy.in_flight_directories_at_finish, 0);
    }

    /// A tree of `dirs` directories under the root, each holding `files` files of
    /// distinct sizes, with its file count and byte total.
    fn wide_tree(dirs: usize, files: usize) -> (tempfile::TempDir, u64, u64) {
        let root = tempfile::tempdir().expect("tempdir");
        let mut bytes = 0;
        for directory in 0..dirs {
            let path = root.path().join(format!("d{directory:03}"));
            fs::create_dir(&path).expect("directory");
            for file in 0..files {
                let size = directory * files + file + 1;
                fs::write(path.join(format!("f{file}.txt")), vec![b'.'; size]).expect("file");
                bytes += size as u64;
            }
        }
        (root, (dirs * files) as u64, bytes)
    }

    /// [`prepared`], reporting through `progress`.
    fn prepared_with_progress(
        root: &Path,
        config: &OpenFixture,
        query: &Query,
        progress: &Progress,
    ) -> Result<(Report, PendingSave, PerformanceSummary)> {
        let (request, delivery) = split(root, config, query);
        prepare_report_with_progress(&request, &delivery, progress)
    }

    fn analyzing(fixture: OpenFixture) -> OpenFixture {
        OpenFixture {
            analysis: crate::content::AnalysisRequest {
                profile: crate::content::AnalysisSet::LINES_ONLY,
                workers: 0,
            },
            ..fixture
        }
    }

    /// The invariant the plan makes testable: when a route completes, the handle's
    /// files, bytes, and allocated bytes equal the walked totals the route's own
    /// performance summary reports, and its directories equal the directories the route
    /// read. The allocated figure is also the answer's own total, which is what lets a
    /// display put it beside the answer. Every one-shot route: the cold full index, the
    /// transient summary fold, a cold run with content analysis, and a warm revalidation
    /// of the snapshot that run left.
    #[test]
    fn progress_ends_at_the_walked_totals_of_every_one_shot_route() {
        use crate::ProgressPhase::{Scanning, Summarizing};
        let (root, files, bytes) = wide_tree(6, 4);
        let tree = Query { views: vec![ViewSpec::Tree], ..Query::default() };

        let progress = Progress::new();
        let (_, pending, performance) =
            prepared_with_progress(root.path(), &config(CachePolicy::Off, None), &tree, &progress)
                .expect("cold full-index report");
        pending.join().expect("no save");
        let snapshot = progress.snapshot();
        assert_eq!(performance.source, ReportSource::ColdScan);
        assert_eq!((performance.walked_files, performance.walked_bytes), (files, bytes));
        assert_eq!((snapshot.files, snapshot.bytes), (files, bytes), "cold full index");
        assert_eq!(snapshot.allocated, performance.walked_allocated);
        assert!(snapshot.allocated > 0, "files with content occupy blocks");
        assert_eq!(snapshot.directories, 7, "the root and its six children");
        assert_eq!(
            (snapshot.phase, snapshot.analysis),
            (Summarizing, None),
            "the walk ended, the index was assembled, then the answer was built"
        );

        let progress = Progress::new();
        let (report, pending, performance) = prepared_with_progress(
            root.path(),
            &blind(CachePolicy::Off, None),
            &summary_query(),
            &progress,
        )
        .expect("compact summary report");
        pending.join().expect("no save");
        let Section::Summary(row) = report.sections[0] else { panic!("summary section") };
        let snapshot = progress.snapshot();
        assert_eq!((performance.walked_files, performance.walked_bytes), (files, bytes));
        assert_eq!((snapshot.files, snapshot.bytes), (files, bytes), "summary fold");
        assert_eq!(snapshot.allocated, performance.walked_allocated);
        assert_eq!(snapshot.allocated, row.allocated, "the progress figure is the answer's");
        assert_eq!(snapshot.directories, row.dirs + 1, "the row's directories and the root");
        assert_eq!((snapshot.phase, snapshot.analysis), (Scanning, None));

        // Outside the tree: a cache inside it is two more files for the warm walk.
        let cache_dir = tempfile::tempdir().expect("cache dir");
        let cache = cache_dir.path().join("snapshot.fdu");
        let progress = Progress::new();
        let (_, pending, performance) = prepared_with_progress(
            root.path(),
            &analyzing(config(CachePolicy::Auto, Some(cache.clone()))),
            &tree,
            &progress,
        )
        .expect("cold analyzed report");
        let snapshot = progress.snapshot();
        assert_eq!(snapshot.phase, Summarizing, "the answer is built while the save runs");
        pending.join().expect("save");
        assert_eq!(performance.source, ReportSource::ColdScan);
        assert_eq!((snapshot.files, snapshot.bytes), (files, bytes), "cold with analysis");
        assert_eq!(snapshot.allocated, performance.walked_allocated);
        assert_eq!(snapshot.directories, 7);
        assert_eq!(performance.fresh_files, files, "every file is a lines candidate");
        assert_eq!(snapshot.analysis, Some((files, files)));

        let progress = Progress::new();
        let (_, pending, performance) = prepared_with_progress(
            root.path(),
            &analyzing(config(CachePolicy::Auto, Some(cache))),
            &tree,
            &progress,
        )
        .expect("warm analyzed report");
        pending.join().expect("nothing to save");
        let snapshot = progress.snapshot();
        assert_eq!(performance.source, ReportSource::WarmRevalidate);
        assert_eq!((performance.walked_files, performance.walked_bytes), (files, bytes));
        assert_eq!((snapshot.files, snapshot.bytes), (files, bytes), "warm revalidation");
        assert_eq!(snapshot.allocated, performance.walked_allocated);
        assert_eq!(snapshot.directories, 7);
        assert_eq!(performance.fresh_files, 0, "the sidecar answered every candidate");
        assert_eq!((snapshot.phase, snapshot.analysis), (Summarizing, Some((0, 0))));
    }

    /// A cache-only report walks nothing: it ends building the answer, and its walk
    /// counters stay at zero.
    #[test]
    fn a_cache_only_report_ends_summarizing_and_walks_nothing() {
        use crate::ProgressPhase::Summarizing;
        let (root, _, _) = wide_tree(3, 2);
        let tree = Query { views: vec![ViewSpec::Tree], ..Query::default() };
        let cache_dir = tempfile::tempdir().expect("cache dir");
        let cache = cache_dir.path().join("snapshot.fdu");
        let (_, pending, _) =
            prepared(root.path(), &config(CachePolicy::On, Some(cache.clone())), &tree)
                .expect("a report that writes the snapshot");
        pending.join().expect("save");

        let progress = Progress::new();
        let (_, pending, performance) = prepared_with_progress(
            root.path(),
            &stale(config(CachePolicy::Auto, Some(cache))),
            &tree,
            &progress,
        )
        .expect("cache-only report");
        pending.join().expect("nothing to save");
        let snapshot = progress.snapshot();
        assert_eq!(performance.source, ReportSource::CacheOnly);
        assert_eq!(snapshot.phase, Summarizing);
        assert_eq!((snapshot.directories, snapshot.files, snapshot.bytes), (0, 0, 0));
    }

    /// The position of `phase` in `order`, so a poller can assert phases never go back.
    fn rank(phase: crate::ProgressPhase, order: &[crate::ProgressPhase]) -> usize {
        order
            .iter()
            .position(|expected| *expected == phase)
            .unwrap_or_else(|| panic!("{phase:?} is not a phase of this route"))
    }

    /// What a ticker thread sees: every counter non-decreasing from one snapshot to the
    /// next, and the phase moving only forward through the route's order. Deterministic
    /// without a timing assumption, because each claim is about consecutive reads of one
    /// monotonic cell, whatever the interleaving; the poller just reads until the run is
    /// over. The tree spans many worker chunks and many small batches, so the counters
    /// are added to from several threads while the poller reads.
    #[test]
    fn progress_is_monotonic_and_phases_advance_in_order_while_a_report_runs() {
        use crate::ProgressPhase::{
            Analyzing, Indexing, Loading, Revalidating, Saving, Scanning, Starting, Summarizing,
        };
        let (root, files, bytes) = wide_tree(48, 6);
        let cache_dir = tempfile::tempdir().expect("cache dir");
        let cache = cache_dir.path().join("snapshot.fdu");
        let fixture = OpenFixture {
            scan: ScanConfig { threads: Some(3), batch_size: 4, ..ScanConfig::default() },
            ..analyzing(config(CachePolicy::Auto, Some(cache)))
        };
        let tree = Query { views: vec![ViewSpec::Tree], ..Query::default() };

        let routes: [(&str, &[crate::ProgressPhase]); 2] = [
            ("cold", &[Starting, Loading, Scanning, Indexing, Analyzing, Saving, Summarizing]),
            ("warm", &[Starting, Loading, Revalidating, Analyzing, Saving, Summarizing]),
        ];
        for (route, order) in routes {
            let progress = Progress::new();
            // Read before the run can begin, so the first phase seen is the handle's
            // initial one whatever the scheduler does with the poller.
            let initial = progress.snapshot();
            let done = std::sync::atomic::AtomicBool::new(false);
            let (performance, seen) = std::thread::scope(|scope| {
                let poller = scope.spawn(|| {
                    let polled = progress.clone();
                    let mut previous = initial;
                    let mut seen = vec![previous.phase];
                    loop {
                        let finished = done.load(std::sync::atomic::Ordering::Acquire);
                        let current = polled.snapshot();
                        assert!(current.directories >= previous.directories, "{route}");
                        assert!(current.files >= previous.files, "{route}");
                        assert!(current.bytes >= previous.bytes, "{route}");
                        assert!(
                            rank(current.phase, order) >= rank(previous.phase, order),
                            "{route}: {:?} after {:?}",
                            current.phase,
                            previous.phase
                        );
                        if let (Some(before), Some(after)) = (previous.analysis, current.analysis) {
                            assert!(after.0 >= before.0 && after.0 <= after.1, "{route}");
                            assert_eq!(after.1, before.1, "{route}: the total is fixed");
                        }
                        if current.phase != previous.phase {
                            seen.push(current.phase);
                        }
                        previous = current;
                        // Read once more after the run reports done, so the final state
                        // is checked against the last mid-run read.
                        if finished {
                            break;
                        }
                        std::thread::yield_now();
                    }
                    seen
                });
                let (_, pending, performance) =
                    prepared_with_progress(root.path(), &fixture, &tree, &progress)
                        .expect("report");
                pending.join().expect("save");
                done.store(true, std::sync::atomic::Ordering::Release);
                (performance, poller.join().expect("poller"))
            });
            let snapshot = progress.snapshot();
            assert_eq!(
                (snapshot.files, snapshot.bytes),
                (performance.walked_files, performance.walked_bytes),
                "{route}"
            );
            assert_eq!((snapshot.files, snapshot.bytes), (files, bytes), "{route}");
            assert_eq!(snapshot.directories, 49, "{route}");
            assert_eq!(seen.first(), Some(&Starting), "{route}: {seen:?}");
            assert_eq!(
                seen.last().copied(),
                Some(snapshot.phase),
                "{route}: the poller saw the final phase"
            );
            assert_eq!(snapshot.phase, Summarizing, "{route}: the answer is built last");
            if route == "cold" {
                assert_eq!(snapshot.analysis, Some((files, files)));
            } else {
                assert_eq!(snapshot.analysis, Some((0, 0)));
                assert!(!seen.contains(&Indexing), "{route}: a warm run assembles no index");
            }
        }
    }

    /// The handle observes the run and changes nothing about it: the report prepared
    /// with one renders to the bytes of the report prepared without, and the
    /// performance summary is the same value, on the indexed and the compact routes.
    #[test]
    fn a_report_prepared_with_a_handle_is_the_report_prepared_without() {
        let (root, _, _) = wide_tree(5, 3);
        let tree = Query { views: vec![ViewSpec::Tree, ViewSpec::Files], ..Query::default() };
        let cases = [
            ("full index", config(CachePolicy::Off, None), tree),
            ("compact summary", blind(CachePolicy::Off, None), summary_query()),
        ];
        for (route, fixture, query) in cases {
            let (mut plain, pending, plain_performance) =
                prepared(root.path(), &fixture, &query).expect("plain report");
            pending.join().expect("no save");
            let progress = Progress::new();
            let (observed, pending, observed_performance) =
                prepared_with_progress(root.path(), &fixture, &query, &progress)
                    .expect("observed report");
            pending.join().expect("no save");

            assert_eq!(plain_performance, observed_performance, "{route}");
            plain.provenance = observed.provenance.clone();
            let json = |report: &Report| {
                crate::report_format::render(report, crate::report_format::Format::Json, false)
                    .expect("render")
            };
            assert_eq!(json(&plain), json(&observed), "{route}");
            assert!(progress.snapshot().files > 0, "{route}: the handle did observe the run");
        }
    }
}
