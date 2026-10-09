//! Completeness and provenance for one coherent report snapshot.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::{Coverage, CoverageReason, Freshness, Index, Issue, Source};

use super::{IgnoredEntries, ReportSource, Request};

/// Completeness of the facts used to answer one report.
#[derive(Clone, Debug)]
pub struct TreeStatus {
    /// Whether every requested structural and content fact is represented.
    pub complete: bool,
    /// Coverage of the least complete requested tier.
    pub coverage: Coverage,
    /// Bounded operational failure details, ordered by root and then by path.
    pub errors: Vec<StatusIssue>,
    /// Further failure details omitted by the shared retention bound.
    pub errors_omitted: u64,
}

/// One operational failure in a report's status, with the root it was met under.
///
/// The issue's path stays relative to that root, as every report path does, so
/// [`Report::roots`](crate::query::Report::roots)`[root]` joined with it names the entry
/// exactly. [`Issue`] itself gains no root: it is the engine contract's, shared with opened
/// roots and commits, which have one root by construction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusIssue {
    /// Position of the issue's root among the report's roots; 0 for a report over one root.
    pub root: usize,
    /// What failed, and where under its root.
    pub issue: Issue,
}

impl StatusIssue {
    /// An issue met under a report's only root.
    pub(crate) const fn of_one_root(issue: Issue) -> Self {
        Self { root: 0, issue }
    }
}

impl TreeStatus {
    /// The status of a report over several roots, from each root's status in their order.
    ///
    /// Complete when every root is; the coverage is the first incomplete root's. Errors
    /// keep root order and, within a root, the path order each part already has, bounded by
    /// the shared retention limit; every detail that bound drops is counted beside those
    /// each part already dropped.
    pub(crate) fn merge(parts: Vec<Self>) -> Self {
        let complete = parts.iter().all(|part| part.complete);
        let coverage = parts
            .iter()
            .find(|part| !part.complete)
            .or_else(|| parts.first())
            .map_or(Coverage::Complete, |part| part.coverage);
        let mut errors = Vec::new();
        let mut errors_omitted = 0_u64;
        for (root, part) in parts.into_iter().enumerate() {
            errors_omitted = errors_omitted.saturating_add(part.errors_omitted);
            for detail in part.errors {
                if errors.len() < crate::MAX_RETAINED_ISSUES {
                    errors.push(StatusIssue { root, issue: detail.issue });
                } else {
                    errors_omitted = errors_omitted.saturating_add(1);
                }
            }
        }
        Self { complete, coverage, errors, errors_omitted }
    }

    /// Derive status from the same immutable index snapshot used to build report rows.
    pub fn of(index: &Index, request: &Request) -> Self {
        let state = index.state();
        let mut details: Vec<(PathBuf, Issue)> = Vec::with_capacity(crate::MAX_RETAINED_ISSUES);
        let mut detail_count = 0_u64;
        for issue in index.issues() {
            detail_count = detail_count.saturating_add(1);
            retain_first_detail(
                &mut details,
                (issue.path.clone().unwrap_or_default(), issue.clone()),
            );
        }
        let unknown_population = request.query.selection.ignored != IgnoredEntries::Include
            && index.control_table().refused_len() > 0;
        if unknown_population {
            for refusal in index.control_table().refusals() {
                detail_count = detail_count.saturating_add(1);
                let issue = Issue::provider_failure(
                    Some(&refusal.path),
                    format!(
                        "ignored population is unknown below this refused control ({})",
                        refusal.reason.label()
                    ),
                );
                retain_first_detail(&mut details, (refusal.path, issue));
            }
        }
        let wanted = index.content_identity(request.basis.content);
        let admitted = index.content().and_then(|content| content.admit(&wanted));
        let mut content_failures = 0_u64;
        if request.basis.content.is_enabled() {
            if let Some(content) = admitted {
                for (path, analysis) in content.records() {
                    let Some(reason) = analysis.operational_failure() else {
                        continue;
                    };
                    content_failures = content_failures.saturating_add(1);
                    detail_count = detail_count.saturating_add(1);
                    let detail = analysis.error.clone().unwrap_or_else(|| match reason {
                        crate::content::CoverageReason::IoError => {
                            "content analysis could not read the file".to_string()
                        }
                        crate::content::CoverageReason::ChangedDuringRead => {
                            "file changed during content analysis".to_string()
                        }
                        _ => unreachable!("operational_failure returns only operational reasons"),
                    });
                    retain_first_detail(
                        &mut details,
                        (path.to_path_buf(), Issue::provider_failure(Some(path), detail)),
                    );
                }
            }
        }
        let content_tier_partial = request.basis.content.is_enabled()
            && admitted
                .and_then(crate::stored_state::ContentProjection::state)
                .is_some_and(|tier| tier.freshness == Freshness::Partial);
        let content_pending = index.content_has_pending(request.basis.content);
        if content_tier_partial && content_failures == 0 {
            detail_count = detail_count.saturating_add(1);
            retain_first_detail(
                &mut details,
                (
                    PathBuf::new(),
                    Issue::provider_failure(
                        None,
                        "content analysis results became stale before they could be retained"
                            .to_string(),
                    ),
                ),
            );
        }
        let retained = u64::try_from(details.len()).unwrap_or(u64::MAX);
        let errors = details.into_iter().map(|(_, issue)| StatusIssue::of_one_root(issue)).collect();
        let complete = state.coverage == Coverage::Complete
            && content_failures == 0
            && !content_tier_partial
            && !content_pending
            && !unknown_population;
        Self {
            complete,
            coverage: if content_failures == 0
                && !content_tier_partial
                && !content_pending
                && !unknown_population
            {
                state.coverage
            } else {
                Coverage::Partial(CoverageReason::Failed)
            },
            errors,
            errors_omitted: state
                .issues
                .omitted
                .saturating_add(detail_count.saturating_sub(retained)),
        }
    }

    /// Derive status from a one-shot walk, collapsing repeated causes first.
    ///
    /// Workers can meet one unreadable path more than once; the index retains each cause
    /// once, so this path must too or the two routes disagree about `errors_omitted`.
    pub(crate) fn of_walk(root: &std::path::Path, scan: &mut crate::ScanReport) -> Self {
        crate::scan::normalize_walk_errors(root, &mut scan.errors);
        let complete = scan.is_complete();
        let mut details = Vec::with_capacity(crate::MAX_RETAINED_ISSUES);
        let mut count = 0_u64;
        for error in &scan.errors {
            count = count.saturating_add(1);
            let issue = crate::Issue::from_error_under(root, error);
            retain_first_detail(&mut details, (issue.path.clone().unwrap_or_default(), issue));
        }
        let retained = u64::try_from(details.len()).unwrap_or(u64::MAX);
        let errors = details.into_iter().map(|(_, issue)| StatusIssue::of_one_root(issue)).collect();
        Self {
            complete,
            coverage: if complete {
                Coverage::Complete
            } else {
                Coverage::Partial(CoverageReason::Inaccessible)
            },
            errors,
            errors_omitted: count.saturating_sub(retained),
        }
    }
}

fn retain_first_detail(details: &mut Vec<(PathBuf, Issue)>, detail: (PathBuf, Issue)) {
    let position = details
        .binary_search_by(|current| {
            current.0.cmp(&detail.0).then_with(|| current.1.message.cmp(&detail.1.message))
        })
        .unwrap_or_else(|position| position);
    if position >= crate::MAX_RETAINED_ISSUES {
        return;
    }
    details.insert(position, detail);
    if details.len() > crate::MAX_RETAINED_ISSUES {
        details.pop();
    }
}

/// Source and currency of one retained tier.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TierState {
    /// Weakest source represented by this tier.
    pub source: Source,
    /// Current trust state of this tier.
    pub freshness: Freshness,
    /// When this tier's facts were observed, or `None` when the stored format cannot say.
    pub observed_at_ns: Option<i64>,
}

/// Provenance of each tier contributing to a report.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TierProvenance {
    /// Retained filesystem-entry tier.
    pub entries: TierState,
    /// Sparse content-analysis tier when requested and present.
    pub content: Option<TierState>,
}

/// How and when the coherent answer was produced.
#[derive(Clone, Debug)]
pub struct ReportProvenance {
    /// Weakest source among tiers contributing to the report.
    pub source: ReportSource,
    /// Least fresh tier contributing to the report.
    pub freshness: Freshness,
    /// Conservative start watermark of the entry verification pass.
    pub scan_started_at: Option<SystemTime>,
    /// Caller-supplied instant at which this answer was generated.
    pub generated_at: SystemTime,
    /// Per-tier source, currency, and observation time.
    pub tiers: TierProvenance,
}

impl ReportProvenance {
    /// Derive report provenance for the requested tiers from one immutable index snapshot.
    pub fn of(
        index: &Index,
        content_requested: crate::content::AnalysisSet,
        generated_at: SystemTime,
    ) -> Self {
        let state = index.state();
        let entries = TierState {
            source: state.source,
            freshness: state.freshness,
            observed_at_ns: Some(index.writing_pass_started_at_ns()),
        };
        let scan_started_at = u64::try_from(index.writing_pass_started_at_ns())
            .ok()
            .map(|nanos| SystemTime::UNIX_EPOCH + Duration::from_nanos(nanos));
        let content_pending = index.content_has_pending(content_requested);
        let wanted = index.content_identity(content_requested);
        let content = if content_requested.is_enabled() {
            index.content().and_then(|content| content.admit(&wanted)).and_then(|content| {
                content.state().map(|state| TierState {
                    source: state.source,
                    freshness: if content_pending { Freshness::Partial } else { state.freshness },
                    observed_at_ns: state.observed_at_ns,
                })
            })
        } else {
            None
        };
        let source = content.map_or(entries.source, |state| entries.source.max(state.source));
        let freshness = content
            .map_or(entries.freshness, |state| least_fresh(entries.freshness, state.freshness));
        Self {
            source: report_source(source),
            freshness,
            scan_started_at,
            generated_at,
            tiers: TierProvenance { entries, content },
        }
    }

    pub(crate) fn of_walk(started: SystemTime, generated_at: SystemTime, complete: bool) -> Self {
        let observed_at_ns = crate::query::system_time_to_nanos(started);
        let freshness = if complete { Freshness::Fresh } else { Freshness::Partial };
        let entries = TierState { source: Source::Scanned, freshness, observed_at_ns };
        Self {
            source: ReportSource::ColdScan,
            freshness,
            scan_started_at: Some(started),
            generated_at,
            tiers: TierProvenance { entries, content: None },
        }
    }

    /// The provenance of a report over several roots, from each root's.
    ///
    /// The weakest source and the least fresh tier, as one report already takes them across
    /// its tiers, so one root answered from a snapshot makes the whole answer a cached one.
    /// Times are the earliest, since the answer is no newer than its oldest part, and
    /// unknown when any part's is. A content tier is reported only when every root has one.
    pub(crate) fn merge(parts: Vec<Self>) -> Self {
        let mut parts = parts.into_iter();
        let first = parts.next().expect("a report reads at least one root");
        parts.fold(first, |merged, part| Self {
            source: weaker_source(merged.source, part.source),
            freshness: least_fresh(merged.freshness, part.freshness),
            scan_started_at: merged.scan_started_at.zip(part.scan_started_at).map(|(a, b)| a.min(b)),
            generated_at: merged.generated_at.min(part.generated_at),
            tiers: TierProvenance {
                entries: merged_tier(merged.tiers.entries, part.tiers.entries),
                content: merged.tiers.content.zip(part.tiers.content).map(|(a, b)| merged_tier(a, b)),
            },
        })
    }
}

/// One tier's state over two roots: the weaker source, the less fresh state, and the
/// earlier observation, unknown when either is.
fn merged_tier(left: TierState, right: TierState) -> TierState {
    TierState {
        source: left.source.max(right.source),
        freshness: least_fresh(left.freshness, right.freshness),
        observed_at_ns: left.observed_at_ns.zip(right.observed_at_ns).map(|(a, b)| a.min(b)),
    }
}

/// The weaker of two report sources: an answer no filesystem verified is weaker than a
/// revalidated one, which is weaker than a cold walk.
fn weaker_source(left: ReportSource, right: ReportSource) -> ReportSource {
    let rank = |source| match source {
        ReportSource::ColdScan => 0,
        ReportSource::WarmRevalidate => 1,
        ReportSource::CacheOnly => 2,
    };
    if rank(left) >= rank(right) { left } else { right }
}

fn least_fresh(left: Freshness, right: Freshness) -> Freshness {
    let rank = |freshness| match freshness {
        Freshness::Fresh => 0,
        Freshness::Reconciling => 1,
        Freshness::Stale => 2,
        Freshness::Partial => 3,
    };
    if rank(left) >= rank(right) { left } else { right }
}

fn report_source(source: Source) -> ReportSource {
    match source {
        Source::Scanned => ReportSource::ColdScan,
        Source::Revalidated | Source::JournalScoped => ReportSource::WarmRevalidate,
        Source::Cached => ReportSource::CacheOnly,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::UNIX_EPOCH;

    use super::{ReportProvenance, TreeStatus};
    use crate::content::{AnalysisRequest, AnalysisSet, analyze_index};
    use crate::query::{Basis, Query, Request};
    use crate::scan::ScanConfig;
    use crate::{Freshness, Index, Source};

    #[test]
    fn partial_content_tier_keeps_status_incomplete_without_a_failure_record() {
        let mut index = Index::new("/unused");
        let analysis = AnalysisSet::NONE.with_lines();
        index.prepare_content_analysis(AnalysisRequest {
            profile: analysis,
            ..AnalysisRequest::default()
        });
        index.set_content_tier_state(Source::Scanned, Freshness::Partial, Some(1));
        let mut basis = Basis::held_by(&index);
        basis.content = analysis;
        let request = Request::new(basis, Query::default(), UNIX_EPOCH);

        let status = TreeStatus::of(&index, &request);

        assert!(!status.complete);
        assert_eq!(status.coverage, crate::Coverage::Partial(crate::CoverageReason::Failed));
        assert_eq!(
            status.errors[0].issue.message,
            "content analysis results became stale before they could be retained"
        );
    }

    #[test]
    fn metadata_only_provenance_excludes_an_unrequested_partial_content_tier() {
        let mut index = Index::new("/unused");
        index.prepare_content_analysis(AnalysisRequest {
            profile: AnalysisSet::NONE.with_lines(),
            ..AnalysisRequest::default()
        });
        index.set_content_tier_state(Source::Cached, Freshness::Partial, None);

        let provenance = ReportProvenance::of(&index, AnalysisSet::NONE, UNIX_EPOCH);

        assert_eq!(provenance.tiers.content, None);
        assert_eq!(provenance.freshness, Freshness::Fresh);
        assert_eq!(provenance.source, crate::query::ReportSource::ColdScan);
    }

    #[test]
    fn repeated_content_read_failure_stays_incomplete_until_a_verified_recovery() {
        let root = tempfile::tempdir().expect("root");
        let path = root.path().join("failed.txt");
        fs::write(&path, b"hello").expect("fixture");
        let scan = ScanConfig::default();
        let (mut index, report) = crate::scan::scan_into_index(root.path(), &scan).expect("scan");
        assert!(report.is_complete());
        fs::remove_file(&path).expect("make the retained candidate unreadable");
        let analysis = AnalysisRequest { profile: AnalysisSet::NONE.with_lines(), workers: 1 };
        let first = analyze_index(&mut index, analysis);
        assert_eq!(first.lines.io_errors, 1);
        let request = Request::new(Basis::held_by(&index), Query::default(), UNIX_EPOCH);

        let first_status = TreeStatus::of(&index, &request);
        assert!(!first_status.complete);
        assert_eq!(first_status.errors.len(), 1);
        assert_eq!(
            first_status.errors[0].issue.path.as_deref(),
            Some(std::path::Path::new("failed.txt"))
        );

        let repeated = analyze_index(&mut index, analysis);
        assert_eq!(repeated.lines.io_errors, 1, "a failed record must be retried");
        assert!(!TreeStatus::of(&index, &request).complete);

        fs::write(&path, b"recovered").expect("recover file");
        crate::scan::reconcile(&mut index, &scan, &mut |_| {}).expect("reconcile recovery");
        let recovered = analyze_index(&mut index, analysis);
        assert_eq!(recovered.lines.analyzed, 1);
        let status = TreeStatus::of(&index, &request);
        assert!(status.complete, "a successful reread clears the operational failure");
        assert!(status.errors.is_empty());
    }

    /// A walk can meet one unreadable directory from several workers. The summary fast
    /// path must collapse those repeats as the index does, or `--view summary` and
    /// `--view tree` disagree about how many errors were omitted (R113-4).
    #[test]
    fn walk_status_counts_each_unreadable_path_once() {
        let root = std::path::Path::new("/root");
        let denied = |number: usize| crate::Error::Io {
            path: root.join(format!("denied-{number:02}")),
            source: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        };
        let mut scan = crate::ScanReport::default();
        for number in (0..40).rev() {
            scan.errors.push(denied(number));
            scan.errors.push(denied(number));
        }

        let status = TreeStatus::of_walk(root, &mut scan);

        assert!(!status.complete);
        assert_eq!(status.errors.len(), 40);
        assert_eq!(status.errors_omitted, 0);
        assert_eq!(status.errors[0].issue.path.as_deref(), Some(std::path::Path::new("denied-00")));
    }

    #[test]
    fn content_failures_retain_the_first_paths_and_count_every_omission() {
        let root = tempfile::tempdir().expect("root");
        for number in (0..66).rev() {
            fs::write(root.path().join(format!("file-{number:02}.txt")), b"x").expect("fixture");
        }
        let (mut index, report) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        assert!(report.is_complete());
        for number in 0..66 {
            fs::remove_file(root.path().join(format!("file-{number:02}.txt")))
                .expect("make candidate unreadable");
        }
        analyze_index(
            &mut index,
            AnalysisRequest { profile: AnalysisSet::NONE.with_lines(), workers: 2 },
        );
        let request = Request::new(Basis::held_by(&index), Query::default(), UNIX_EPOCH);

        let status = TreeStatus::of(&index, &request);

        assert!(!status.complete);
        assert_eq!(status.errors.len(), crate::MAX_RETAINED_ISSUES);
        assert_eq!(status.errors_omitted, 2);
        let retained_paths: Vec<_> =
            status.errors.iter().map(|detail| detail.issue.path.as_deref().expect("path")).collect();
        assert_eq!(retained_paths[0], std::path::Path::new("file-00.txt"));
        assert_eq!(retained_paths[63], std::path::Path::new("file-63.txt"));
        assert!(retained_paths.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(!retained_paths.contains(&std::path::Path::new("file-64.txt")));
        assert!(!retained_paths.contains(&std::path::Path::new("file-65.txt")));
        assert_eq!(
            index
                .content()
                .expect("content")
                .records()
                .filter(|(_, record)| { record.operational_failure().is_some() })
                .count(),
            66,
            "the diagnostic bound must not discard retained failure state"
        );
    }
}
