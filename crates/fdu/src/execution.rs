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
    open_with_pending_save,
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
}

/// Operational work behind one one-shot report.
///
/// This is deliberately separate from [`Report`]: it is transient CLI telemetry, not
/// part of the stable machine-report schema or the pure query result.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct PerformanceSummary {
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
/// A summary reducer is legal only when no snapshot can participate, no content analysis
/// is requested, the sole requested view is an unfiltered summary, and the caller asks for
/// a one-shot report through this API.  [`crate::open`] and live sessions still promise an
/// index and therefore never use this planner.  Any future requirement the compact tier
/// cannot prove falls closed to [`RetainedState::FullIndex`].
pub(crate) fn plan_report(config: &OpenConfig, query: &Query) -> ReportPlan {
    let cache_cannot_participate = config.policy == CachePolicy::Off
        || (config.cache_path.is_none() && config.policy != CachePolicy::Only);
    // Analysis reads file contents keyed by retained entries and writes its own sidecar,
    // so the aggregate-only tier cannot answer it.
    let analysis_requested = config.analysis.profile.is_enabled();
    let summary_is_sufficient =
        query.views.as_slice() == [ViewSpec::Summary] && query.selection.is_unfiltered();
    ReportPlan {
        retained_state: if cache_cannot_participate && !analysis_requested && summary_is_sufficient
        {
            RetainedState::Summary
        } else {
            RetainedState::FullIndex
        },
    }
}

/// Execute a one-shot report, returning a snapshot write that may overlap rendering.
///
/// The returned report is complete as a value even while the optional save runs.  The
/// caller must join the handle before exit; dropping it also joins defensively.
pub(crate) fn prepare_report(
    root: &Path,
    config: &OpenConfig,
    query: &Query,
) -> Result<(Report, PendingSave, PerformanceSummary)> {
    let scan_started_at = SystemTime::now();
    match plan_report(config, query).retained_state {
        RetainedState::Summary => {
            let root = root.canonicalize().map_err(|error| Error::io(root, error))?;
            let mut summary = SummaryRow::default();
            let scan = crate::scan::scan(&root, &config.scan, &mut |observation| {
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
            })?;
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
            Ok((report, PendingSave::none(), performance))
        }
        RetainedState::FullIndex => {
            let (index, open_report, pending_save) = open_with_pending_save(root, config)?;
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
            Ok((report(&index, query, &provenance), pending_save, performance))
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

        let cached = config(CachePolicy::Auto, Some(PathBuf::from("cache.fdu")));
        assert_eq!(plan_report(&cached, &summary_query()).retained_state, RetainedState::FullIndex);

        let cache_only = config(CachePolicy::Only, None);
        assert_eq!(
            plan_report(&cache_only, &summary_query()).retained_state,
            RetainedState::FullIndex
        );

        let mut several_views = summary_query();
        several_views.views.push(ViewSpec::Types);
        assert_eq!(plan_report(&off, &several_views).retained_state, RetainedState::FullIndex);

        let mut filtered = summary_query();
        filtered.selection.include.push(Pattern::parse("*.rs").expect("pattern"));
        assert_eq!(plan_report(&off, &filtered).retained_state, RetainedState::FullIndex);
    }

    #[test]
    fn an_analysis_request_never_selects_the_compact_summary_tier() {
        // Analysis reads file contents keyed by retained entries and writes its own
        // sidecar, so the aggregate-only tier cannot answer it even though the request
        // otherwise looks like the uncached unfiltered summary the planner compacts.
        let off = config(CachePolicy::Off, None);
        assert_eq!(plan_report(&off, &summary_query()).retained_state, RetainedState::Summary);

        for profile in [
            crate::content::AnalysisProfile::Basic,
            crate::content::AnalysisProfile::Code,
            crate::content::AnalysisProfile::Documents,
            crate::content::AnalysisProfile::Full,
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
