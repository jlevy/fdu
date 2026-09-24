//! Planning and executing one-shot reports with the least retained state they require.
//!
//! The command surface stays composable: callers describe cache policy and a query, not
//! an implementation strategy.  This module derives that strategy.  Most reports need
//! the complete [`Index`](crate::Index), either because another view needs hierarchy or
//! paths, or because the cache must retain reusable state.  An unfiltered summary that
//! observes no `.gitignore` needs only five aggregate values, so that one plan reduces the
//! scan's observations directly and never builds an index.

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
    /// Without hierarchy or a control table this tier cannot tell whether an entry is
    /// ignored, so a scan that observes control state never selects it.
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
        let allowed = self.delivery.cache.writes() && self.delivery.cache_path.is_some();
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
    fn from_open_report(report: &crate::OpenReport) -> Self {
        let analysis = report.analysis.unwrap_or_default();
        Self {
            walked_files: report.scan.files_walked,
            walked_bytes: report.scan.bytes_walked,
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

/// Validate a request and derive the least-retention plan for its delivery and route.
///
/// A summary reducer is legal when no content analysis is requested, the sole requested
/// view is an unfiltered summary, the scan observes no control state, and the policy does
/// not require the snapshot to participate.  [`crate::open`] and live sessions still
/// promise an index and therefore always plan full retention. Any future requirement the
/// compact tier cannot prove falls closed to `RetainedState::FullIndex`.
///
/// Control observation is the caller's decision, not this planner's: a report's rows carry
/// the ignored share of every size they show (fdu-elnn), so a scan that reads `.gitignore`
/// displays what it paid for, and one that turned it off shows no share rather than a zero.
/// The summary reducer keeps no table to classify with, so an observing summary falls
/// closed to the index. That trades the reducer's small footprint for the ignored share in
/// the default `fdu --view summary`; the performance ledger records what it costs.
///
/// The compact tier is not gated on the cache being unavailable, because for an
/// unfiltered metadata summary the snapshot cannot save the work the scan is doing.
/// Revalidating a loaded snapshot stats every entry anyway, so the reusable index and its
/// write are additive cost with nothing to amortise them: measured on Linux/ext4 over
/// 84,539 entries, the compact tier answered in 71 ms against 161 ms for a warm
/// revalidating `Auto` run, and even a no-scan `Only` read cost 81 ms because
/// deserialisation is about as expensive per record as a warm walk.  A snapshot earns its
/// keep when it avoids expensive work — re-reading file bodies for content analysis, or a
/// cold filesystem walk — not when it merely mirrors a walk that still has to happen.
///
/// Two policies still require the index, for reasons that are about intent rather than
/// cost.  [`CachePolicy::Only`] must answer from the snapshot without touching the tree,
/// so it has no scan to reduce.  [`CachePolicy::Refresh`] is an explicit request to
/// rewrite the snapshot, and honouring it means materialising the index that gets
/// written — though with no cache path configured there is nothing to rewrite, and the
/// compact tier answers it like any other summary.
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
    if route == Route::Refresh && delivery.cache == CachePolicy::Only {
        return Err(crate::query::RequestError::DeliveryUnsupported {
            route: "refresh",
            reason: "the only cache policy cannot verify filesystem state",
        });
    }
    let analysis_requested = request.basis.content.is_enabled();
    let summary_is_sufficient = request.query.views.as_slice() == [ViewSpec::Summary]
        && request.query.selection.is_unfiltered()
        && !request.basis.scope.read_controls;
    let policy_requires_index = match delivery.cache {
        CachePolicy::Only => true,
        CachePolicy::Refresh => delivery.cache_path.is_some(),
        CachePolicy::Off | CachePolicy::Auto | CachePolicy::ReadOnly => false,
    };
    // A one-shot metadata query cannot amortize loading and reconciling a snapshot:
    // both paths stat every entry. On macOS/APFS (494,031 entries), warm revalidation
    // cost 4.8 s versus 3.6 s cold, while the write it might avoid cost only ~50 ms.
    // Content avoids body reads, and retained routes amortize their reusable index.
    let read_snapshot = match delivery.cache {
        CachePolicy::Only => true,
        CachePolicy::Off | CachePolicy::Refresh => false,
        CachePolicy::Auto | CachePolicy::ReadOnly => route != Route::OneShot || analysis_requested,
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
        verify: if delivery.cache == CachePolicy::Only { Verify::None } else { Verify::Filesystem },
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
/// wrong for one asking a single question -- an unfiltered summary that reads no
/// `.gitignore` is answered by a transient tier that retains nothing, and writing a
/// snapshot for it caches state the walk did not save. A Python caller therefore left
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
/// A run that returns with a save still pending has entered
/// [`ProgressPhase::Saving`](crate::ProgressPhase); the caller decides when to join it,
/// as with [`prepare_report`].
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
            let mut summary = SummaryRow::default();
            let mut reduce = |observed: &crate::ObservationOp| {
                let crate::Op::Upsert { kind, attrs, .. } = &observed.op else {
                    return;
                };
                match kind {
                    EntryKind::File => {
                        summary.files += 1;
                        summary.bytes += attrs.size;
                        summary.allocated += attrs.allocated;
                        summary.newest_mtime_ns = Some(
                            summary
                                .newest_mtime_ns
                                .map_or(attrs.mtime_ns, |current| current.max(attrs.mtime_ns)),
                        );
                    }
                    EntryKind::Dir => summary.dirs += 1,
                    EntryKind::Symlink | EntryKind::Other => {}
                }
            };
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
            let report = report_summary(
                &root,
                scan_config.scope(),
                request,
                summary,
                TreeStatus::of_walk(&root, &mut scan),
                ReportProvenance::of_walk(scan_started_at, generated_at, complete),
            );
            let performance = PerformanceSummary {
                walked_files: scan.files_walked,
                walked_bytes: scan.bytes_walked,
                source: ReportSource::ColdScan,
                ..PerformanceSummary::default()
            };
            Ok((report, PendingSave::none(), performance, scan_diagnostics))
        }
        RetainedState::FullIndex => {
            let (index, open_report, pending_save, scan_diagnostics) =
                execute(&plan, &request.basis, collect_scan_diagnostics, progress)?;
            let performance = PerformanceSummary::from_open_report(&open_report);
            let answer = report(&index, request, SystemTime::now())?;
            debug_assert_eq!(answer.scope, scan_config.scope());
            Ok((answer, pending_save, performance, scan_diagnostics))
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
        for policy in [CachePolicy::Off, CachePolicy::ReadOnly] {
            let nonwriting = Delivery { cache: policy, ..delivery.clone() };
            let nonwriting_plan =
                super::plan(&request, &nonwriting, Route::Refresh).expect("nonwriting plan");
            assert!(
                !crate::persist_index_changes(&index, &nonwriting_plan, true, true)
                    .expect("nonwriting policy never reads the header")
            );
        }
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
            crate::open(&wrong, &Delivery { cache: CachePolicy::Refresh, ..delivery.clone() })
                .expect("incompatible snapshot");
            let (mut index, _) = crate::open(&basis, &Delivery::new(CachePolicy::Off, None))
                .expect("retained index");
            let refreshed =
                crate::refresh(&mut index, &basis, &delivery).expect("refresh reseeds cache");
            assert!(!refreshed.apply.mutated(), "the existing index was already current");
            let (cached, _) =
                crate::open(&basis, &Delivery { cache: CachePolicy::Only, ..delivery.clone() })
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
        let (cached, report) =
            crate::open(&basis, &Delivery { cache: CachePolicy::Only, ..delivery })
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
        let cache_only = Delivery { cache: CachePolicy::Only, ..delivery.clone() };
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
        let no_location = message(&Delivery::new(CachePolicy::Only, None));
        assert!(no_location.contains("no cache location"), "{no_location}");
        assert!(!no_location.contains("use auto"), "no write can succeed without a location");
        let missing = message(&Delivery::new(CachePolicy::Only, Some(snapshot.clone())));
        assert!(missing.contains("no usable snapshot"), "{missing}");
        assert!(missing.contains("auto"), "{missing}");
        let other_basis = crate::query::Basis { root: other.path().into(), ..basis.clone() };
        crate::open(&other_basis, &Delivery::new(CachePolicy::Auto, Some(snapshot.clone())))
            .expect("other snapshot");
        let wrong_root = message(&Delivery::new(CachePolicy::Only, Some(snapshot)));
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
                let forbidden = (delivery.watch.is_some()
                    || matches!(route, Route::Watch | Route::Refresh))
                    && delivery.cache == CachePolicy::Only
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
    fn write_policy_depends_only_on_delivery_and_observed_facts() {
        let routes = [Route::OneShot, Route::Retained, Route::Refresh, Route::Watch, Route::Opened];
        for delivery in Delivery::enumerate() {
            for bits in 0_u8..64 {
                let facts = RunFacts {
                    entries_verified: bits & 1 != 0,
                    entries_changed: bits & 2 != 0,
                    content_changed: bits & 4 != 0,
                    content_requested: bits & 8 != 0,
                    projected: bits & 16 != 0,
                    paired_entries: bits & 32 != 0,
                };
                let allowed = delivery.cache.writes();
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

    /// [`config`] with `.gitignore` observation turned off, the one scan the compact
    /// summary tier can answer.
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

        for policy in [CachePolicy::Auto, CachePolicy::Refresh, CachePolicy::ReadOnly] {
            let unavailable = blind(policy, None);
            assert_eq!(planned(&unavailable, &summary_query()).retained, RetainedState::Summary);
        }

        let mut several_views = summary_query();
        several_views.views.push(ViewSpec::Types);
        assert_eq!(planned(&off, &several_views).retained, RetainedState::FullIndex);

        let mut filtered = summary_query();
        filtered.selection.include.push(Pattern::parse("*.rs").expect("pattern"));
        assert_eq!(planned(&off, &filtered).retained, RetainedState::FullIndex);

        // The reducer keeps no control table, so a summary whose row carries an ignored
        // share, or selects by one, needs the index.
        let observing = config(CachePolicy::Off, None);
        assert!(observing.scan.read_controls, "observation is the default");
        assert_eq!(planned(&observing, &summary_query()).retained, RetainedState::FullIndex);
        let mut by_ignored = summary_query();
        by_ignored.selection.ignored = IgnoredEntries::Exclude;
        assert_eq!(planned(&observing, &by_ignored).retained, RetainedState::FullIndex);
    }

    #[test]
    fn an_available_snapshot_does_not_force_the_index_for_a_metadata_summary() {
        // A loaded snapshot cannot save the work an unfiltered metadata summary is
        // already doing: revalidation stats every entry regardless, so retaining the
        // index and writing it back is additive cost with nothing to amortise it. The
        // compact tier stays selected so the common one-shot totals request pays for a
        // walk and nothing else.
        for policy in [CachePolicy::Auto, CachePolicy::ReadOnly] {
            let cached = blind(policy, Some(PathBuf::from("cache.fdu")));
            assert_eq!(
                planned(&cached, &summary_query()).retained,
                RetainedState::Summary,
                "{policy:?} must not be forced onto the index by a present snapshot"
            );
        }
    }

    #[test]
    fn policies_whose_intent_is_the_snapshot_itself_still_retain_the_index() {
        // These two are not cost decisions. `Only` must answer without touching the tree,
        // so it has no scan to reduce; `Refresh` is an explicit request to rewrite the
        // snapshot, which means materialising the index that gets written.
        for policy in [CachePolicy::Only, CachePolicy::Refresh] {
            let cached = config(policy, Some(PathBuf::from("cache.fdu")));
            assert_eq!(
                planned(&cached, &summary_query()).retained,
                RetainedState::FullIndex,
                "{policy:?} needs the index to honour its contract"
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
        for policy in [CachePolicy::Auto, CachePolicy::ReadOnly] {
            let cached = config(policy, Some(PathBuf::from("cache.fdu")));
            assert!(
                planned(&cached, &tree_query).load != Load::Snapshot,
                "{policy:?} must not pay for a read that saves no work"
            );
        }
    }

    #[test]
    fn the_snapshot_is_read_where_reading_pays_or_is_the_contract() {
        // `Only` answers from the snapshot; reading it is the request itself.
        let only = config(CachePolicy::Only, Some(PathBuf::from("cache.fdu")));
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

        // `Off` and `Refresh` never read by definition.
        for policy in [CachePolicy::Off, CachePolicy::Refresh] {
            let never = config(policy, Some(PathBuf::from("cache.fdu")));
            assert_eq!(planned(&never, &summary_query()).load, Load::None, "{policy:?}");
        }
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
        let mut tree_query = summary_query();
        tree_query.views = vec![ViewSpec::Tree];

        let (first, pending, _) = prepared(root.path(), &auto, &tree_query).expect("first report");
        pending.join().expect("first save");
        assert_eq!(first.provenance.source, ReportSource::ColdScan);
        assert!(auto.cache_path.as_deref().expect("path").exists(), "first run persists");

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
    fn cache_only_still_answers_from_a_snapshot_left_by_a_one_shot_report() {
        // Skipping the read must not skip the write: the snapshot a default run leaves
        // behind is what `--cache only` answers from without touching the tree.
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("file.txt"), b"contents").expect("file");
        let cache = tempfile::tempdir().expect("cache dir");
        let auto = config(CachePolicy::Auto, Some(cache.path().join("cache.fdu")));
        let mut tree_query = summary_query();
        tree_query.views = vec![ViewSpec::Tree];

        let (_, pending, _) = prepared(root.path(), &auto, &tree_query).expect("report");
        pending.join().expect("save");

        let only = config(CachePolicy::Only, Some(cache.path().join("cache.fdu")));
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

        let controls_off = controls_config(CachePolicy::Only, cache_path, false);
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

        let cold = OpenFixture { policy: CachePolicy::Off, cache_path: None, ..controls_off };
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
        // off reads neither file, and says so in its report and in any snapshot it writes.
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("file.txt"), b"contents").expect("file");
        write_unobservable_controls(root.path());
        let mut tree_query = summary_query();
        tree_query.views = vec![ViewSpec::Tree];

        for read_controls in [true, false] {
            let cache = tempfile::tempdir().expect("cache dir");
            let cache_path = cache.path().join("cache.fdu");
            let caller = controls_config(CachePolicy::Auto, cache_path.clone(), read_controls);
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
                        assert_eq!(report.notes.len(), 1, "{:?}", report.notes);
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
    /// A scope this build cannot honour is refused by every policy, including the one that
    /// never scans: under `--cache only` the scan that would have refused it never runs, so
    /// the run used to report a snapshot miss instead -- the same request naming two
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

        // A usable snapshot exists, so a cache-only read of a scope this build supports
        // answers from it.
        let warm = config(CachePolicy::Auto, Some(cache_path.clone()));
        let (_, pending, _) = prepared(root.path(), &warm, &summary_query()).expect("warm");
        pending.join().expect("save");
        let (_, pending, _) = prepared(
            root.path(),
            &config(CachePolicy::Only, Some(cache_path.clone())),
            &summary_query(),
        )
        .expect("the snapshot answers a supported scope");
        pending.join().expect("no save");

        let unsupported = ScanConfig { follow_symlinks: true, ..ScanConfig::default() };
        for policy in
            [CachePolicy::Only, CachePolicy::Off, CachePolicy::Auto, CachePolicy::ReadOnly]
        {
            let asked = OpenFixture {
                scan: unsupported.clone(),
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
    fn a_default_reports_snapshot_serves_either_cache_only_report_but_not_the_reverse() {
        // Every surface reaches this planner observing control state by default, so a
        // snapshot any default report wrote serves the next one. A cache-only report that
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
            let write = controls_config(CachePolicy::Auto, cache_path.clone(), writer);
            let (_, pending, _) =
                prepared(root.path(), &write, &tree_query).expect("writing report");
            pending.join().expect("save");

            let read = controls_config(CachePolicy::Only, cache_path, reader);
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
                    assert!(message.contains("`auto`"), "names the remedy: {message}");
                }
                Err(other) => panic!("writer {writer}, reader {reader}: {other}"),
            }
        }
    }

    #[test]
    fn a_cache_only_open_answers_from_a_default_reports_snapshot() {
        // A default report and a default `open` share one scope, so the one policy that
        // forbids a scan answers the `open` from the report's snapshot, with the
        // classification the report observed.
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join(".gitignore"), b"*.log\n").expect("control file");
        fs::write(root.path().join("debug.log"), b"ignored").expect("ignored file");
        let cache = tempfile::tempdir().expect("cache dir");
        let cache_path = cache.path().join("cache.fdu");
        let mut tree_query = summary_query();
        tree_query.views = vec![ViewSpec::Tree];

        let auto = config(CachePolicy::Auto, Some(cache_path.clone()));
        let (_, pending, _) = prepared(root.path(), &auto, &tree_query).expect("report");
        pending.join().expect("save");

        let only = config(CachePolicy::Only, Some(cache_path));
        let (index, report) = crate::open_fixture(root.path(), &only).expect("the shared snapshot");
        assert_eq!(report.path_taken, OpenPath::CacheOnly);
        assert_eq!(index.is_ignored(Path::new("debug.log")).ok(), Some(Some(true)));
    }

    #[test]
    fn an_open_that_opts_out_of_control_state_shares_an_opted_out_reports_snapshot() {
        // A report and an `open` that both turn observation off write and want one scope,
        // so that open answers from the report's snapshot without touching the tree, under
        // the one policy that forbids a scan, and says it cannot classify ignored entries.
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("file.txt"), b"contents").expect("file");
        let cache = tempfile::tempdir().expect("cache dir");
        let cache_path = cache.path().join("cache.fdu");
        let mut tree_query = summary_query();
        tree_query.views = vec![ViewSpec::Tree];

        let auto = blind(CachePolicy::Auto, Some(cache_path.clone()));
        let (_, pending, _) = prepared(root.path(), &auto, &tree_query).expect("report");
        pending.join().expect("save");
        assert!(cache_path.exists(), "the report left a snapshot");

        let only = blind(CachePolicy::Only, Some(cache_path));
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

        // Only a report that turns control observation off takes the compact tier, so the
        // index it must match exactly is opened under that scope too.
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
    /// files and bytes equal the walked totals the route's own performance summary
    /// reports, and its directories equal the directories the route read. Every
    /// one-shot route: the cold full index, the transient summary fold, a cold run with
    /// content analysis, and a warm revalidation of the snapshot that run left.
    #[test]
    fn progress_ends_at_the_walked_totals_of_every_one_shot_route() {
        use crate::ProgressPhase::{Analyzing, Indexing, Saving, Scanning};
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
        assert_eq!(snapshot.directories, 7, "the root and its six children");
        assert_eq!(
            (snapshot.phase, snapshot.analysis),
            (Indexing, None),
            "the walk ended, then the index was assembled"
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
        assert_eq!(snapshot.phase, Saving, "returned with a save pending");
        pending.join().expect("save");
        assert_eq!(performance.source, ReportSource::ColdScan);
        assert_eq!((snapshot.files, snapshot.bytes), (files, bytes), "cold with analysis");
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
        assert_eq!(snapshot.directories, 7);
        assert_eq!(performance.fresh_files, 0, "the sidecar answered every candidate");
        assert_eq!(
            (snapshot.phase, snapshot.analysis),
            (Analyzing, Some((0, 0))),
            "an unchanged tree writes nothing, so analysis is the last phase"
        );
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
            Analyzing, Indexing, Loading, Revalidating, Saving, Scanning, Starting,
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
            ("cold", &[Starting, Loading, Scanning, Indexing, Analyzing, Saving]),
            ("warm", &[Starting, Loading, Revalidating, Analyzing, Saving]),
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
            if route == "cold" {
                assert_eq!(snapshot.analysis, Some((files, files)));
                assert_eq!(snapshot.phase, Saving, "a cold run writes the snapshot");
                assert!(seen.contains(&Scanning), "{route}: {seen:?}");
            } else {
                assert_eq!(snapshot.analysis, Some((0, 0)));
                assert_eq!(snapshot.phase, Analyzing, "an unchanged tree writes nothing");
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
