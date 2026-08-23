//! Planning and executing one-shot reports with the least retained state they require.
//!
//! The command surface stays composable: callers describe cache policy and a query, not
//! an implementation strategy.  This module derives that strategy.  Most reports need
//! the complete [`Index`](crate::Index), either because another view needs hierarchy or
//! paths, or because the cache must retain reusable state.  An uncached, unfiltered
//! summary needs only five aggregate values, so that one plan reduces the scan's
//! observations directly and never builds an index.

use std::path::Path;
use std::time::SystemTime;

use crate::query::{
    Provenance, Query, Report, ReportSource, SummaryRow, ViewSpec, report, report_summary,
};
use crate::{
    CachePolicy, EntryKind, Error, Freshness, OpenConfig, OpenPath, PendingSave, Result,
    open_for_report,
};

/// The minimum state a one-shot report plan retains while scanning.
///
/// This is deliberately a small closed set.  Add another tier only when a measured view
/// can be answered exactly from materially less state than an index; callers should not
/// have to select it themselves.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum RetainedState {
    /// One aggregate row; no path or hierarchy records survive the scan.
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
/// view is an unfiltered summary, and the policy does not require the snapshot to
/// participate.  [`crate::open`] and live sessions still promise an index and therefore
/// never use this planner.  Any future requirement the compact tier cannot prove falls
/// closed to [`RetainedState::FullIndex`].
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
    let summary_is_sufficient =
        query.views.as_slice() == [ViewSpec::Summary] && query.selection.is_unfiltered();
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
/// wrong for one asking a single question -- an unfiltered summary is answered by a
/// transient tier that retains nothing, and writing a snapshot for it caches state the
/// walk did not save. A Python caller therefore left cache state on a tree that the same
/// command would not have, which a later cache-only read could see (fdu-4msv).
///
/// The caller owns the returned [`PendingSave`] and decides when to join it, exactly as
/// the command line does, so a renderer can run while the snapshot is still being written.
pub fn prepare_report(
    root: &Path,
    config: &OpenConfig,
    query: &Query,
) -> Result<(Report, PendingSave, PerformanceSummary)> {
    prepare_report_internal(root, config, query, false)
        .map(|(report, pending, performance, _diagnostics)| (report, pending, performance))
}

/// Execute a one-shot report and retain scan diagnostics.
///
/// Public because the command line needs it and the command line is an ordinary consumer:
/// it drives repository-controlled measurement of the installed binary. Kept separate
/// from [`prepare_report`] so callers who do not want traces pay for neither collection
/// nor serialization.
pub fn prepare_report_with_scan_diagnostics(
    root: &Path,
    config: &OpenConfig,
    query: &Query,
) -> Result<(Report, PendingSave, PerformanceSummary, Option<crate::scan::ScanDiagnostics>)> {
    prepare_report_internal(root, config, query, true)
}

fn prepare_report_internal(
    root: &Path,
    config: &OpenConfig,
    query: &Query,
    collect_scan_diagnostics: bool,
) -> Result<(Report, PendingSave, PerformanceSummary, Option<crate::scan::ScanDiagnostics>)> {
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
            let (index, open_report, pending_save) =
                open_for_report(root, config, plan.read_snapshot)?;
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
            Ok((report(&index, query, &provenance), pending_save, performance, None))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;
    use crate::ScanConfig;
    use crate::query::{Pattern, Section};

    fn summary_query() -> Query {
        Query { views: vec![ViewSpec::Summary], ..Query::default() }
    }

    fn config(policy: CachePolicy, cache_path: Option<PathBuf>) -> OpenConfig {
        OpenConfig { scan: ScanConfig::default(), cache_path, policy, ..OpenConfig::default() }
    }

    #[test]
    fn planner_uses_compact_state_only_when_the_request_proves_it_is_sufficient() {
        let off = config(CachePolicy::Off, Some(PathBuf::from("unused.fdu")));
        assert_eq!(plan_report(&off, &summary_query()).retained_state, RetainedState::Summary);

        for policy in [CachePolicy::Auto, CachePolicy::Refresh, CachePolicy::ReadOnly] {
            let unavailable = config(policy, None);
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
    }

    #[test]
    fn an_available_snapshot_does_not_force_the_index_for_a_metadata_summary() {
        // A loaded snapshot cannot save the work an unfiltered metadata summary is
        // already doing: revalidation stats every entry regardless, so retaining the
        // index and writing it back is additive cost with nothing to amortise it. The
        // compact tier stays selected so the common one-shot totals request pays for a
        // walk and nothing else.
        for policy in [CachePolicy::Auto, CachePolicy::ReadOnly] {
            let cached = config(policy, Some(PathBuf::from("cache.fdu")));
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
        let off = config(CachePolicy::Off, None);
        assert_eq!(plan_report(&off, &summary_query()).retained_state, RetainedState::Summary);

        for profile in [
            crate::content::AnalysisSet::NONE.with_lines(),
            crate::content::AnalysisSet::NONE.with_code(),
            crate::content::AnalysisSet::NONE.with_words(),
            crate::content::AnalysisSet::ALL,
        ] {
            let analyzed = OpenConfig {
                analysis: crate::content::AnalysisRequest { profile, ..Default::default() },
                ..config(CachePolicy::Off, None)
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

        let (first, pending, _) =
            prepare_report(root.path(), &auto, &tree_query).expect("first report");
        pending.join().expect("first save");
        assert_eq!(first.source, ReportSource::ColdScan);
        assert!(auto.cache_path.as_deref().expect("path").exists(), "first run persists");

        let (second, pending, performance) =
            prepare_report(root.path(), &auto, &tree_query).expect("second report");
        pending.join().expect("second save");
        assert_eq!(
            second.source,
            ReportSource::ColdScan,
            "a repeated one-shot must not pay for a read that saves no work"
        );
        assert_eq!(performance.walked_files, 1, "the walk still happened");

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

        let (_, pending, _) = prepare_report(root.path(), &auto, &tree_query).expect("report");
        pending.join().expect("save");

        let only = config(CachePolicy::Only, Some(cache.path().join("cache.fdu")));
        let (from_cache, pending, performance) =
            prepare_report(root.path(), &only, &tree_query).expect("cache-only report");
        pending.join().expect("no save");
        assert_eq!(from_cache.source, ReportSource::CacheOnly);
        assert_eq!(performance.walked_files, 0, "cache-only never touches the tree");
    }

    #[test]
    fn compact_summary_matches_the_indexed_summary_exactly() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::create_dir(root.path().join("src")).expect("directory");
        fs::write(root.path().join("src/lib.rs"), b"library").expect("file");
        fs::write(root.path().join("README.md"), b"read me").expect("file");

        let query = summary_query();
        let off = config(CachePolicy::Off, None);
        let (compact, pending, performance) =
            prepare_report(root.path(), &off, &query).expect("compact report");
        pending.join().expect("no pending compact save");
        assert_eq!(performance.walked_files, 2);
        assert_eq!(performance.walked_bytes, 14);

        let (index, open_report) = crate::open(root.path(), &off).expect("indexed scan");
        let indexed = report(
            &index,
            &query,
            &Provenance {
                scan_started_at: compact.scan_started_at,
                generated_at: compact.generated_at,
                source: ReportSource::ColdScan,
                complete: open_report.is_complete(),
                errors: Vec::new(),
            },
        );

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

        let (report, pending, _) = prepare_report(
            root.path(),
            &config(CachePolicy::Off, Some(cache.clone())),
            &summary_query(),
        )
        .expect("compact report");
        pending.join().expect("no pending compact save");

        assert!(report.complete);
        assert!(!cache.exists());
    }
}
