//! Completeness and provenance for one coherent report snapshot.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::{Coverage, CoverageReason, Freshness, Index, Source};

use super::{ReportSource, Request};

/// Completeness of the facts used to answer one report.
#[derive(Clone, Debug)]
pub struct TreeStatus {
    pub complete: bool,
    pub coverage: Coverage,
    pub errors: Vec<String>,
    pub errors_omitted: u64,
}

impl TreeStatus {
    /// Derive status from the same immutable index snapshot used to build report rows.
    pub fn of(index: &Index, request: &Request) -> Self {
        let state = index.state();
        let mut details: Vec<(PathBuf, String)> = index
            .issues()
            .iter()
            .map(|issue| (issue.path.clone().unwrap_or_default(), issue.message.clone()))
            .collect();
        let mut content_failures = 0_u64;
        if request.basis.content.is_enabled() {
            if let Some(content) = index.content() {
                for (path, analysis) in content.records() {
                    let Some(reason) = analysis.operational_failure() else {
                        continue;
                    };
                    content_failures = content_failures.saturating_add(1);
                    let detail = analysis.error.clone().unwrap_or_else(|| match reason {
                        crate::content::CoverageReason::IoError => {
                            "content analysis could not read the file".to_string()
                        }
                        crate::content::CoverageReason::ChangedDuringRead => {
                            "file changed during content analysis".to_string()
                        }
                        _ => unreachable!("operational_failure returns only operational reasons"),
                    });
                    details.push((path.to_path_buf(), format!("{}: {detail}", path.display())));
                }
            }
        }
        details.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        let retained = details.len().min(crate::MAX_RETAINED_ISSUES);
        let omitted = details.len().saturating_sub(retained) as u64;
        let errors = details.into_iter().take(retained).map(|(_, message)| message).collect();
        let complete = state.coverage == Coverage::Complete && content_failures == 0;
        Self {
            complete,
            coverage: if content_failures == 0 {
                state.coverage
            } else {
                Coverage::Partial(CoverageReason::Failed)
            },
            errors,
            errors_omitted: state.issues.omitted.saturating_add(omitted),
        }
    }

    pub(crate) fn of_walk(scan: &crate::ScanReport) -> Self {
        let complete = scan.is_complete();
        let mut errors: Vec<String> = scan.errors.iter().map(ToString::to_string).collect();
        errors.sort();
        let omitted = errors.len().saturating_sub(crate::MAX_RETAINED_ISSUES) as u64;
        errors.truncate(crate::MAX_RETAINED_ISSUES);
        Self {
            complete,
            coverage: if complete {
                Coverage::Complete
            } else {
                Coverage::Partial(CoverageReason::Inaccessible)
            },
            errors,
            errors_omitted: omitted,
        }
    }
}

/// Source and currency of one retained tier.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TierState {
    pub source: Source,
    pub freshness: Freshness,
    pub observed_at_ns: i64,
}

/// Provenance of each tier contributing to a report.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TierProvenance {
    pub entries: TierState,
    pub content: Option<TierState>,
}

/// How and when the coherent answer was produced.
#[derive(Clone, Debug)]
pub struct ReportProvenance {
    pub source: ReportSource,
    pub freshness: Freshness,
    pub scan_started_at: Option<SystemTime>,
    pub generated_at: SystemTime,
    pub tiers: TierProvenance,
}

impl ReportProvenance {
    pub fn of(index: &Index, generated_at: SystemTime) -> Self {
        let state = index.state();
        let entries = TierState {
            source: state.source,
            freshness: state.freshness,
            observed_at_ns: index.scanned_at_ns(),
        };
        let scan_started_at = u64::try_from(index.writing_pass_started_at_ns())
            .ok()
            .map(|nanos| SystemTime::UNIX_EPOCH + Duration::from_nanos(nanos));
        let content = index.content().and_then(|content| content.state()).map(|state| TierState {
            source: state.source,
            freshness: state.freshness,
            observed_at_ns: state.observed_at_ns,
        });
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
        let observed_at_ns = crate::query::system_time_to_nanos(started).unwrap_or(0);
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
