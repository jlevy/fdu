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
    Delivery, Provenance, Query, Report, ReportSource, Request, SummaryRow, ViewSpec, report,
    report_summary,
};
use crate::{
    CachePolicy, EntryKind, Error, Freshness, OpenConfig, OpenPath, PendingSave, Result,
    SnapshotUse, open_for_report,
};

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

/// The execution strategy derived from cache policy and query requirements.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ReportPlan {
    /// Smallest state that can answer the request without changing its semantics.
    pub retained_state: RetainedState,
    /// Whether an existing snapshot should be loaded before the scan.
    ///
    /// Reading is a cost decision by the same rule as the tier above: revalidating a
    /// loaded snapshot stats every entry regardless, so for a one-shot metadata query
    /// the load and the reconciliation against it are additive work with nothing to
    /// amortise them — measured on macOS/APFS over 494,031 entries, a warm revalidating
    /// default run cost 4.8 s against 3.6 s to scan cold and persist, while the write
    /// the read could at best avoid cost ~50 ms. A warm path that loses to a cold scan
    /// of the same view is a defect by the project's own rule, and this flag is what
    /// removes it. Reading still pays in exactly two places: [`CachePolicy::Only`],
    /// whose contract is to answer from the snapshot, and content analysis, whose
    /// sidecar avoids re-reading file bodies.
    pub read_snapshot: bool,
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

/// Derive the least-retention plan that can answer `query` under `config`.
///
/// A summary reducer is legal when no content analysis is requested, the sole requested
/// view is an unfiltered summary, the scan observes no control state, and the policy does
/// not require the snapshot to participate.  [`crate::open`] and live sessions still
/// promise an index and therefore never use this planner.  Any future requirement the
/// compact tier cannot prove falls closed to [`RetainedState::FullIndex`].
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
pub(crate) fn plan_report(config: &OpenConfig, query: &Query) -> ReportPlan {
    // Analysis reads file contents keyed by retained entries and writes its own sidecar,
    // so the aggregate-only tier cannot answer it.
    let analysis_requested = config.analysis.profile.is_enabled();
    let summary_is_sufficient = query.views.as_slice() == [ViewSpec::Summary]
        && query.selection.is_unfiltered()
        && !config.scan.read_controls;
    let policy_requires_index = match config.policy {
        // Must answer from the snapshot without touching the tree, so there is no scan
        // to reduce in the first place.
        CachePolicy::Only => true,
        // An explicit instruction to rewrite the snapshot, which requires materialising
        // the index that gets written — but only when there is somewhere to write it.
        CachePolicy::Refresh => config.cache_path.is_some(),
        CachePolicy::Off | CachePolicy::Auto | CachePolicy::ReadOnly => false,
    };
    let read_snapshot = match config.policy {
        // The contract is to answer from the snapshot; reading it is the request.
        CachePolicy::Only => true,
        // These two never read by definition.
        CachePolicy::Off | CachePolicy::Refresh => false,
        // A cost decision, and for a one-shot report the read pays only when a content
        // sidecar can be reused: revalidation stats every entry regardless, so for a
        // metadata query the load and reconciliation are purely additive. See the field
        // doc on [`ReportPlan::read_snapshot`] for the measurement.
        CachePolicy::Auto | CachePolicy::ReadOnly => analysis_requested,
    };
    ReportPlan {
        retained_state: if !policy_requires_index && !analysis_requested && summary_is_sufficient {
            RetainedState::Summary
        } else {
            RetainedState::FullIndex
        },
        read_snapshot,
    }
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
/// snapshot under [`CachePolicy::Only`], consuming its all-entry facts and describing none
/// of its classification.
///
/// The caller owns the returned [`PendingSave`] and decides when to join it, exactly as
/// the command line does, so a renderer can run while the snapshot is still being written.
pub fn prepare_report(
    request: &Request,
    delivery: &Delivery,
) -> Result<(Report, PendingSave, PerformanceSummary)> {
    prepare_report_internal(request, delivery, false)
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
    prepare_report_internal(request, delivery, true)
}

fn prepare_report_internal(
    request: &Request,
    delivery: &Delivery,
    collect_scan_diagnostics: bool,
) -> Result<(Report, PendingSave, PerformanceSummary, Option<crate::scan::ScanDiagnostics>)> {
    // Before anything is scanned, loaded, or reduced: a request its own basis cannot answer
    // has no answer at any cost, and the compact summary tier below never reaches a reader,
    // so a check made there would not cover this route at all. A scope this build cannot
    // honour is part of that one check rather than a second one beside it, which is what
    // keeps the refusal independent of the delivery: the cache-only tier never scans and
    // the cold tier never loads, so a rule stated at either would hold for one of them.
    request.validate().map_err(Error::InvalidRequest)?;
    let config = &OpenConfig::of(request, delivery);
    let query = &request.query;
    let root = request.basis.root.as_path();
    let scan_started_at = SystemTime::now();
    let plan = plan_report(config, query);
    match plan.retained_state {
        RetainedState::Summary => {
            let root = root.canonicalize().map_err(|error| Error::io(root, error))?;
            let mut summary = SummaryRow::default();
            let mut reduce = |observation: crate::Observation| {
                for observed in observation.ops {
                    let crate::Op::Upsert { kind, attrs, .. } = observed.op else {
                        continue;
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
                }
            };
            let (scan, scan_diagnostics) = if collect_scan_diagnostics {
                let (scan, diagnostics) =
                    crate::scan::scan_with_diagnostics(&root, &config.scan, &mut reduce)?;
                (scan, Some(diagnostics))
            } else {
                (crate::scan::scan(&root, &config.scan, &mut reduce)?, None)
            };
            let complete = scan.is_complete();
            let provenance = Provenance {
                scan_started_at: Some(scan_started_at),
                generated_at: SystemTime::now(),
                source: ReportSource::ColdScan,
                complete,
                errors: scan.errors.iter().map(ToString::to_string).collect(),
            };
            let report = report_summary(
                &root,
                config.scan.scope(),
                query.selection.size,
                summary,
                if complete { Freshness::Fresh } else { Freshness::Partial },
                &provenance,
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
            let (index, open_report, pending_save, scan_diagnostics) = open_for_report(
                root,
                config,
                plan.read_snapshot,
                SnapshotUse::ReportOnly,
                collect_scan_diagnostics,
            )?;
            let provenance = Provenance {
                scan_started_at: Some(scan_started_at),
                generated_at: SystemTime::now(),
                source: match open_report.path_taken {
                    OpenPath::ColdScan => ReportSource::ColdScan,
                    OpenPath::WarmRevalidate => ReportSource::WarmRevalidate,
                    OpenPath::CacheOnly => ReportSource::CacheOnly,
                },
                complete: open_report.is_complete(),
                // error_messages() also surfaces analysis and restored content-cache
                // diagnostics, which errors() alone would drop on a warm or cache-only open.
                errors: open_report.error_messages(),
            };
            let performance = PerformanceSummary::from_open_report(&open_report);
            let mut answer = report(&index, request, &provenance)?;
            // A cache-only report may consume a controls-on snapshot for a controls-off
            // request because reporting reads only the all-entry facts. No Index escapes
            // this boundary, and the projected report must describe the requested scope
            // rather than the stronger internal snapshot it consumed, including what it
            // says about ignore rules and every row's ignored share.
            answer.scope = config.scan.scope();
            if !config.scan.control_identity().is_observed() {
                crate::query::forget_ignore_classification(&mut answer);
                answer.notes = crate::query::display_notes(query, &answer.ignore_rules);
            }
            Ok((answer, pending_save, performance, scan_diagnostics))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::ScanConfig;
    use crate::query::{IgnoredEntries, Pattern, Section};

    fn summary_query() -> Query {
        Query { views: vec![ViewSpec::Summary], ..Query::default() }
    }

    /// The request and the delivery a test's `OpenConfig` spells, split the way the two
    /// models now divide it: what the answer says, and how it is carried out.
    fn split(root: &Path, config: &OpenConfig, query: &Query) -> (Request, Delivery) {
        let (basis, delivery) = config.split(root);
        (Request::new(basis, query.clone(), SystemTime::now()), delivery)
    }

    /// [`prepare_report`] as these tests ask for it: one configuration, one query.
    fn prepared(
        root: &Path,
        config: &OpenConfig,
        query: &Query,
    ) -> Result<(Report, PendingSave, PerformanceSummary)> {
        let (request, delivery) = split(root, config, query);
        prepare_report(&request, &delivery)
    }

    /// [`prepared`], keeping the scan diagnostics.
    fn prepared_with_diagnostics(
        root: &Path,
        config: &OpenConfig,
        query: &Query,
    ) -> Result<(Report, PendingSave, PerformanceSummary, Option<crate::scan::ScanDiagnostics>)>
    {
        let (request, delivery) = split(root, config, query);
        prepare_report_with_scan_diagnostics(&request, &delivery)
    }

    fn config(policy: CachePolicy, cache_path: Option<PathBuf>) -> OpenConfig {
        OpenConfig { scan: ScanConfig::default(), cache_path, policy, ..OpenConfig::default() }
    }

    /// [`config`] with `.gitignore` observation turned off, the one scan the compact
    /// summary tier can answer.
    fn blind(policy: CachePolicy, cache_path: Option<PathBuf>) -> OpenConfig {
        OpenConfig {
            scan: ScanConfig { read_controls: false, ..ScanConfig::default() },
            ..config(policy, cache_path)
        }
    }

    fn controls_config(
        policy: CachePolicy,
        cache_path: PathBuf,
        read_controls: bool,
    ) -> OpenConfig {
        OpenConfig {
            scan: ScanConfig { read_controls, ..ScanConfig::default() },
            cache_path: Some(cache_path),
            policy,
            ..OpenConfig::default()
        }
    }

    fn seed_controls_snapshot(root: &Path, cache_path: PathBuf) {
        fs::write(root.join(".gitignore"), b"ignored.log\n").expect("control file");
        fs::write(root.join("ignored.log"), b"ignored").expect("ignored file");
        crate::open(root, &controls_config(CachePolicy::Auto, cache_path, true))
            .expect("seed controls-on snapshot");
    }

    #[test]
    fn planner_uses_compact_state_only_when_the_request_proves_it_is_sufficient() {
        let off = blind(CachePolicy::Off, Some(PathBuf::from("unused.fdu")));
        assert_eq!(plan_report(&off, &summary_query()).retained_state, RetainedState::Summary);

        for policy in [CachePolicy::Auto, CachePolicy::Refresh, CachePolicy::ReadOnly] {
            let unavailable = blind(policy, None);
            assert_eq!(
                plan_report(&unavailable, &summary_query()).retained_state,
                RetainedState::Summary
            );
        }

        let mut several_views = summary_query();
        several_views.views.push(ViewSpec::Types);
        assert_eq!(plan_report(&off, &several_views).retained_state, RetainedState::FullIndex);

        let mut filtered = summary_query();
        filtered.selection.include.push(Pattern::parse("*.rs").expect("pattern"));
        assert_eq!(plan_report(&off, &filtered).retained_state, RetainedState::FullIndex);

        // The reducer keeps no control table, so a summary whose row carries an ignored
        // share, or selects by one, needs the index.
        let observing = config(CachePolicy::Off, None);
        assert!(observing.scan.read_controls, "observation is the default");
        assert_eq!(
            plan_report(&observing, &summary_query()).retained_state,
            RetainedState::FullIndex
        );
        let mut by_ignored = summary_query();
        by_ignored.selection.ignored = IgnoredEntries::Exclude;
        assert_eq!(plan_report(&observing, &by_ignored).retained_state, RetainedState::FullIndex);
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
                plan_report(&cached, &summary_query()).retained_state,
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
                plan_report(&cached, &summary_query()).retained_state,
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
                !plan_report(&cached, &tree_query).read_snapshot,
                "{policy:?} must not pay for a read that saves no work"
            );
        }
    }

    #[test]
    fn the_snapshot_is_read_where_reading_pays_or_is_the_contract() {
        // `Only` answers from the snapshot; reading it is the request itself.
        let only = config(CachePolicy::Only, Some(PathBuf::from("cache.fdu")));
        assert!(plan_report(&only, &summary_query()).read_snapshot);

        // Analysis reuses the content sidecar, which avoids re-reading file bodies —
        // the one measured case where a warm read wins (639 ms to 325 ms).
        let analyzed = OpenConfig {
            analysis: crate::content::AnalysisRequest {
                profile: crate::content::AnalysisSet::NONE.with_code(),
                ..Default::default()
            },
            ..config(CachePolicy::Auto, Some(PathBuf::from("cache.fdu")))
        };
        assert!(plan_report(&analyzed, &summary_query()).read_snapshot);

        // `Off` and `Refresh` never read by definition.
        for policy in [CachePolicy::Off, CachePolicy::Refresh] {
            let never = config(policy, Some(PathBuf::from("cache.fdu")));
            assert!(!plan_report(&never, &summary_query()).read_snapshot, "{policy:?}");
        }
    }

    #[test]
    fn an_analysis_request_never_selects_the_compact_summary_tier() {
        // Analysis reads file contents keyed by retained entries and writes its own
        // sidecar, so the aggregate-only tier cannot answer it even though the request
        // otherwise looks like the uncached unfiltered summary the planner compacts.
        let off = blind(CachePolicy::Off, None);
        assert_eq!(plan_report(&off, &summary_query()).retained_state, RetainedState::Summary);

        for profile in [
            crate::content::AnalysisSet::NONE.with_lines(),
            crate::content::AnalysisSet::NONE.with_code(),
            crate::content::AnalysisSet::NONE.with_words(),
            crate::content::AnalysisSet::ALL,
        ] {
            let analyzed = OpenConfig {
                analysis: crate::content::AnalysisRequest { profile, ..Default::default() },
                ..blind(CachePolicy::Off, None)
            };
            assert_eq!(
                plan_report(&analyzed, &summary_query()).retained_state,
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
        assert_eq!(first.source, ReportSource::ColdScan);
        assert!(auto.cache_path.as_deref().expect("path").exists(), "first run persists");

        let (second, pending, performance) =
            prepared(root.path(), &auto, &tree_query).expect("second report");
        pending.join().expect("second save");
        assert_eq!(
            second.source,
            ReportSource::ColdScan,
            "a repeated one-shot must not pay for a read that saves no work"
        );
        assert_eq!(performance.walked_files, 1, "the walk still happened");

        // A default report and a default `open` both observe control state, so they share
        // one snapshot scope and the `open` starts from the report's snapshot.
        let (_, open_report) = crate::open(root.path(), &auto).expect("library open");
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
        assert_eq!(from_cache.source, ReportSource::CacheOnly);
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

        let cold = OpenConfig { policy: CachePolicy::Off, cache_path: None, ..controls_off };
        let (mut expected, pending, _) =
            prepared(root.path(), &cold, &query).expect("controls-off cold report");
        pending.join().expect("no cold save");
        expected.scan_started_at = projected.scan_started_at;
        expected.generated_at = projected.generated_at;
        expected.source = projected.source;
        expected.freshness = projected.freshness;

        assert_eq!(performance.source, ReportSource::CacheOnly);
        assert_eq!(projected.scope, cold.scan.scope());
        assert_eq!(projected.ignore_rules, crate::control::ControlCoverage::NotObserved);
        assert_eq!(
            crate::report_format::render(&projected, crate::report_format::Format::Json, false,),
            crate::report_format::render(&expected, crate::report_format::Format::Json, false,),
        );
    }

    #[test]
    fn controls_on_snapshot_does_not_serve_controls_off_auto_report() {
        let root = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let cache_path = cache.path().join("cache.fdu");
        seed_controls_snapshot(root.path(), cache_path.clone());

        let controls_off = OpenConfig {
            analysis: crate::content::AnalysisRequest {
                profile: crate::content::AnalysisSet::NONE.with_lines(),
                ..Default::default()
            },
            ..controls_config(CachePolicy::Auto, cache_path, false)
        };
        let (report, pending, performance) = prepared(root.path(), &controls_off, &summary_query())
            .expect("controls-off cold fallback");
        pending.join().expect("save controls-off snapshot");

        assert_eq!(report.source, ReportSource::ColdScan);
        assert_eq!(report.scope, controls_off.scan.scope());
        assert_eq!(performance.source, ReportSource::ColdScan);
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
                assert!(report.complete, "a refusal is not a partial: {:?}", report.errors);
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
            let asked = OpenConfig {
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
                    assert_eq!(report.source, ReportSource::CacheOnly);
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
        let (index, report) = crate::open(root.path(), &only).expect("the shared snapshot");
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
        let (index, report) = crate::open(root.path(), &only).expect("the shared snapshot");
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

        let query = summary_query();
        let off = blind(CachePolicy::Off, None);
        let (compact, pending, performance) =
            prepared(root.path(), &off, &query).expect("compact report");
        pending.join().expect("no pending compact save");
        assert_eq!(performance.walked_files, 2);
        assert_eq!(performance.walked_bytes, 14);

        // Only a report that turns control observation off takes the compact tier, so the
        // index it must match exactly is opened under that scope too.
        let (index, open_report) = crate::open(root.path(), &off).expect("indexed scan");
        let indexed = report(
            &index,
            &crate::test_support::read_of(&index, query.clone()),
            &Provenance {
                scan_started_at: compact.scan_started_at,
                generated_at: compact.generated_at,
                source: ReportSource::ColdScan,
                complete: open_report.is_complete(),
                errors: Vec::new(),
            },
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
        assert_eq!(compact.complete, indexed.complete);
        assert_eq!(compact.freshness, indexed.freshness);
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

        assert!(report.complete);
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

        assert!(report.complete);
        assert_eq!(performance.walked_files, 1);
        let diagnostics = diagnostics.expect("full-index scan diagnostics");
        assert_eq!(diagnostics.schema, crate::scan::SCAN_DIAGNOSTICS_SCHEMA);
        assert_eq!(diagnostics.worker_policy.ready_directories_at_finish, 0);
        assert_eq!(diagnostics.worker_policy.in_flight_directories_at_finish, 0);
    }
}
