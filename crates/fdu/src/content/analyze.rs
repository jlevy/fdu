//! Bounded parallel file reads and conditional analysis commits.

use std::fs::File;
use std::io::Read;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};

use crate::Index;
use crate::classify::{ContentFamily, classify_path_with_prefix};

use super::{
    AnalysisApplyOutcome, AnalysisCandidate, AnalysisObservation, AnalysisRequest,
    BasicAccumulator, CodeAccumulator, ContentProvenance, CoverageReason, FileAnalysis,
    LogicalWordStats, MetricValues, TextAdmission, markdown::analyze_markdown,
};

const READ_CHUNK_BYTES: usize = 64 * 1024;
const CLASSIFICATION_PREFIX_BYTES: usize = 16 * 1024;
const MAX_ERROR_BYTES: usize = 512;

/// Operational counters from one content-analysis pass.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct AnalysisReport {
    /// Regular files considered by the requested profile.
    pub candidates: u64,
    /// Results accepted by the index's conditional mutation boundary.
    pub applied: u64,
    /// Results discarded because indexed metadata changed while workers ran.
    pub stale: u64,
    /// Files for which metrics were produced.
    pub analyzed: u64,
    /// Known or observed binary files.
    pub binary: u64,
    /// Files rejected by the configured byte bound.
    pub too_large: u64,
    /// Files whose byte stream was not valid UTF-8.
    pub invalid_utf8: u64,
    /// Files that changed during their read.
    pub changed_during_read: u64,
    /// File-open, metadata, or read failures.
    pub io_errors: u64,
    /// Files for which no requested analyzer was available.
    pub unsupported: u64,
}

impl AnalysisReport {
    /// Whether every candidate reached an analyzed or known-binary terminal outcome.
    pub const fn is_complete(&self) -> bool {
        self.stale == 0
            && self.too_large == 0
            && self.invalid_utf8 == 0
            && self.changed_during_read == 0
            && self.io_errors == 0
            && self.unsupported == 0
    }
}

/// Analyze all regular files selected by `request` with a fixed-size worker pool.
///
/// Workers own immutable candidates and never retain an index borrow during I/O. The
/// caller thread applies observations afterward, so metadata changes remain serialized
/// through [`Index::apply_analysis`].
pub fn analyze_index(index: &mut Index, request: AnalysisRequest) -> AnalysisReport {
    let candidates = index.pending_analysis_candidates(request);
    let mut report = AnalysisReport {
        candidates: u64::try_from(candidates.len()).unwrap_or(u64::MAX),
        ..AnalysisReport::default()
    };
    if candidates.is_empty() {
        return report;
    }

    let workers = worker_count(request.workers, candidates.len());
    let next = AtomicUsize::new(0);
    let candidates = Arc::new(candidates);
    let (sender, receiver) = mpsc::sync_channel(workers.saturating_mul(2).max(1));

    std::thread::scope(|scope| {
        for _ in 0..workers {
            let sender = sender.clone();
            let candidates = Arc::clone(&candidates);
            let next = &next;
            scope.spawn(move || {
                loop {
                    let slot = next.fetch_add(1, Ordering::Relaxed);
                    let Some(candidate) = candidates.get(slot).cloned() else { break };
                    if sender.send(analyze_candidate(candidate, request)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(sender);
        for observation in receiver {
            count_coverage(&mut report, observation.analysis.coverage);
            match index.apply_analysis(observation) {
                AnalysisApplyOutcome::Applied => report.applied = report.applied.saturating_add(1),
                AnalysisApplyOutcome::Stale => report.stale = report.stale.saturating_add(1),
            }
        }
    });
    report
}

fn worker_count(requested: usize, candidates: usize) -> usize {
    let available = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
    let requested = if requested == 0 { available } else { requested };
    requested.clamp(1, candidates.max(1))
}

fn analyze_candidate(
    candidate: AnalysisCandidate,
    request: AnalysisRequest,
) -> AnalysisObservation {
    let analysis = if candidate.classification.family == ContentFamily::Binary {
        record(&candidate, request, candidate.classification.clone(), CoverageReason::Binary, None)
    } else if candidate.attrs.size > request.max_file_bytes {
        record(
            &candidate,
            request,
            candidate.classification.clone(),
            CoverageReason::TooLarge,
            None,
        )
    } else {
        analyze_open_file(&candidate, request)
    };
    AnalysisObservation { candidate, analysis }
}

fn analyze_open_file(candidate: &AnalysisCandidate, request: AnalysisRequest) -> FileAnalysis {
    let mut file = match File::open(&candidate.absolute_path) {
        Ok(file) => file,
        Err(error) => return io_record(candidate, request, &error),
    };
    let before = match file.metadata() {
        Ok(metadata) => crate::scan::attrs_from(&metadata).fingerprint(),
        Err(error) => return io_record(candidate, request, &error),
    };
    if before != candidate.attrs.fingerprint() {
        return record(
            candidate,
            request,
            candidate.classification.clone(),
            CoverageReason::ChangedDuringRead,
            None,
        );
    }

    let mut accumulator =
        BasicAccumulator::with_logical_metrics(request.profile.includes_documents());
    let mut code_accumulator = request
        .profile
        .includes_code()
        .then(|| CodeAccumulator::for_type(candidate.classification.file_type.as_str()))
        .flatten();
    let mut deferred_code = (request.profile.includes_code()
        && candidate.classification.family == ContentFamily::Unknown)
        .then(Vec::new);
    let mut markdown_source = (request.profile.includes_documents()
        && candidate.classification.file_type.as_str() == "markdown")
        .then(Vec::new);
    let mut prefix = Vec::with_capacity(CLASSIFICATION_PREFIX_BYTES);
    let mut chunk = vec![0_u8; READ_CHUNK_BYTES];
    let mut bytes_read = 0_u64;
    let mut read_failure = None;
    let mut exceeded = false;
    loop {
        let remaining = request.max_file_bytes.saturating_sub(bytes_read);
        let allowance = usize::try_from(remaining.saturating_add(1).min(READ_CHUNK_BYTES as u64))
            .expect("read allowance is capped at a usize constant");
        if allowance == 0 {
            exceeded = true;
            break;
        }
        match file.read(&mut chunk[..allowance]) {
            Ok(0) => break,
            Ok(count) => {
                bytes_read = bytes_read.saturating_add(count as u64);
                if prefix.len() < CLASSIFICATION_PREFIX_BYTES {
                    let take = count.min(CLASSIFICATION_PREFIX_BYTES - prefix.len());
                    prefix.extend_from_slice(&chunk[..take]);
                }
                if bytes_read > request.max_file_bytes {
                    exceeded = true;
                    break;
                }
                accumulator.push(&chunk[..count]);
                if let Some(code) = &mut code_accumulator {
                    code.push(&chunk[..count]);
                }
                if let Some(deferred) = &mut deferred_code {
                    deferred.extend_from_slice(&chunk[..count]);
                }
                if let Some(source) = &mut markdown_source {
                    source.extend_from_slice(&chunk[..count]);
                }
            }
            Err(error) => {
                read_failure = Some(error);
                break;
            }
        }
    }

    let after = match file.metadata() {
        Ok(metadata) => crate::scan::attrs_from(&metadata).fingerprint(),
        Err(error) => return io_record(candidate, request, &error),
    };
    if before != after {
        return record(
            candidate,
            request,
            candidate.classification.clone(),
            CoverageReason::ChangedDuringRead,
            None,
        );
    }
    if let Some(error) = read_failure {
        return io_record(candidate, request, &error);
    }
    let classification = classify_path_with_prefix(&candidate.relative_path, Some(&prefix));
    if exceeded {
        return record(candidate, request, classification, CoverageReason::TooLarge, None);
    }
    match accumulator.finish() {
        TextAdmission::Accepted(mut metrics) => {
            if !matches!(classification.family, ContentFamily::Prose | ContentFamily::Markup) {
                metrics.raw_words = 0;
                metrics.paragraphs = 0;
                metrics.logical_word_stats = LogicalWordStats::default();
            }
            if request.profile.includes_code() && classification.family == ContentFamily::Code {
                if code_accumulator.is_none() {
                    if let Some(deferred) = deferred_code {
                        if let Some(mut code) =
                            CodeAccumulator::for_type(classification.file_type.as_str())
                        {
                            code.push(&deferred);
                            code_accumulator = Some(code);
                        }
                    }
                }
                let Some(code) = code_accumulator else {
                    return record(
                        candidate,
                        request,
                        classification,
                        CoverageReason::Unsupported,
                        None,
                    );
                };
                let code_metrics = code.finish();
                debug_assert_eq!(metrics.physical_lines, code_metrics.physical_lines);
                metrics.code_lines = code_metrics.code_lines;
                metrics.comment_lines = code_metrics.comment_lines;
                metrics.code_blank_lines = code_metrics.code_blank_lines;
            }
            if request.profile.includes_documents()
                && classification.file_type.as_str() == "markdown"
            {
                if let Some(source) = markdown_source {
                    let source = std::str::from_utf8(&source)
                        .expect("basic admission already established valid UTF-8");
                    let visible = analyze_markdown(source);
                    metrics.visible_words = visible.visible_words;
                    metrics.visible_logical_word_stats = visible.visible_logical_word_stats;
                    metrics.paragraphs = visible.paragraphs;
                }
            }
            analyzed_record(candidate, request, classification, metrics)
        }
        TextAdmission::Binary => {
            record(candidate, request, classification, CoverageReason::Binary, None)
        }
        TextAdmission::InvalidUtf8 => {
            record(candidate, request, classification, CoverageReason::InvalidUtf8, None)
        }
    }
}

fn analyzed_record(
    candidate: &AnalysisCandidate,
    request: AnalysisRequest,
    classification: crate::classify::Classification,
    metrics: MetricValues,
) -> FileAnalysis {
    FileAnalysis {
        metrics,
        ..record(candidate, request, classification, CoverageReason::Analyzed, None)
    }
}

fn io_record(
    candidate: &AnalysisCandidate,
    request: AnalysisRequest,
    error: &std::io::Error,
) -> FileAnalysis {
    let mut detail = error.to_string();
    detail.truncate(char_boundary_at_or_before(&detail, MAX_ERROR_BYTES));
    record(
        candidate,
        request,
        candidate.classification.clone(),
        CoverageReason::IoError,
        Some(detail),
    )
}

fn char_boundary_at_or_before(value: &str, limit: usize) -> usize {
    let mut boundary = limit.min(value.len());
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    boundary
}

fn record(
    candidate: &AnalysisCandidate,
    request: AnalysisRequest,
    classification: crate::classify::Classification,
    coverage: CoverageReason,
    error: Option<String>,
) -> FileAnalysis {
    FileAnalysis {
        classification,
        fingerprint: candidate.attrs.fingerprint(),
        bytes: candidate.attrs.size,
        profile: request.profile,
        provenance: ContentProvenance::for_request(request),
        metrics: MetricValues::default(),
        coverage,
        error,
    }
}

fn count_coverage(report: &mut AnalysisReport, coverage: CoverageReason) {
    let counter = match coverage {
        CoverageReason::Analyzed => &mut report.analyzed,
        CoverageReason::Binary => &mut report.binary,
        CoverageReason::InvalidUtf8 => &mut report.invalid_utf8,
        CoverageReason::TooLarge => &mut report.too_large,
        CoverageReason::IoError => &mut report.io_errors,
        CoverageReason::ChangedDuringRead => &mut report.changed_during_read,
        CoverageReason::Unsupported => &mut report.unsupported,
    };
    *counter = counter.saturating_add(1);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use crate::scan::ScanConfig;

    use super::*;

    #[test]
    fn pool_analyzes_text_and_skips_known_binary_and_oversized_files() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("notes.md"), "one two\n\nthree\n").expect("write text");
        fs::write(root.path().join("image.png"), b"not opened as text").expect("write binary");
        fs::write(root.path().join("large.txt"), b"12345678901234567").expect("write large");
        let (mut index, scan) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        assert!(scan.is_complete());

        let report = analyze_index(
            &mut index,
            AnalysisRequest {
                profile: super::super::AnalysisProfile::Basic,
                max_file_bytes: 16,
                workers: 2,
            },
        );

        assert_eq!(report.candidates, 3);
        assert_eq!(report.analyzed, 1);
        assert_eq!(report.binary, 1);
        assert_eq!(report.too_large, 1);
        let root_rollup = index.content_rollup(std::path::Path::new("")).expect("content root");
        assert_eq!(root_rollup.total.files, 3);
        assert_eq!(root_rollup.total.metrics.physical_lines, 3);
        assert_eq!(root_rollup.total.metrics.raw_words, 3);
    }

    #[test]
    fn nul_and_invalid_utf8_are_coverage_not_provisional_metrics() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("nul.unknown"), b"line\nlate\0nul").expect("write nul");
        fs::write(root.path().join("bad.unknown"), [b'a', 0xff]).expect("write invalid");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");

        let report = analyze_index(
            &mut index,
            AnalysisRequest {
                profile: super::super::AnalysisProfile::Basic,
                ..AnalysisRequest::default()
            },
        );
        assert_eq!(report.binary, 1);
        assert_eq!(report.invalid_utf8, 1);
        assert_eq!(
            index.content_rollup(std::path::Path::new("")).expect("root").total.metrics,
            MetricValues::default()
        );
    }

    #[test]
    fn code_profile_partitions_supported_languages_and_marks_others_unsupported() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("main.rs"), "// comment\nfn main() {} // mixed\n\n")
            .expect("write rust");
        fs::write(root.path().join("Main.hs"), "-- not claimed\nmain = pure ()\n")
            .expect("write haskell");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");

        let report = analyze_index(
            &mut index,
            AnalysisRequest {
                profile: super::super::AnalysisProfile::Code,
                ..AnalysisRequest::default()
            },
        );

        assert_eq!(report.analyzed, 1);
        assert_eq!(report.unsupported, 1);
        let rust = index.content().expect("content").file(std::path::Path::new("main.rs"));
        let metrics = rust.expect("rust record").metrics;
        assert_eq!(metrics.physical_lines, 3);
        assert_eq!(metrics.code_lines, 1);
        assert_eq!(metrics.comment_lines, 1);
        assert_eq!(metrics.code_blank_lines, 1);
        assert_eq!(
            metrics.physical_lines,
            metrics.code_lines + metrics.comment_lines + metrics.code_blank_lines
        );
        let haskell = index.content().expect("content").file(std::path::Path::new("Main.hs"));
        assert_eq!(haskell.expect("haskell record").coverage, CoverageReason::Unsupported);

        let query = crate::query::Query {
            views: vec![crate::query::ViewSpec::Languages],
            ..crate::query::Query::default()
        };
        let rendered = crate::query::report(
            &index,
            &query,
            &crate::query::Provenance {
                scan_started_at: None,
                generated_at: std::time::UNIX_EPOCH,
                source: crate::query::ReportSource::ColdScan,
                complete: false,
                errors: Vec::new(),
            },
        );
        let crate::query::Section::Metrics { summary, .. } = &rendered.sections[0] else {
            panic!("expected language metrics")
        };
        assert_eq!(summary.share_metric, crate::query::ShareMetric::CodeLines);
        assert_eq!((summary.total.share.numerator, summary.total.share.denominator), (1, 1));
        assert_eq!(summary.total.coverage.get(&CoverageReason::Unsupported), Some(&1));
    }

    #[test]
    fn document_profile_uses_visible_markdown_and_logical_plain_text() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(
            root.path().join("guide.md"),
            "# Read [the label](https://example.test)\n\n`hidden code` 中文\n",
        )
        .expect("write markdown");
        fs::write(root.path().join("notes.txt"), "oneverylongtoken\n\nplain words\n")
            .expect("write text");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");

        let report = analyze_index(
            &mut index,
            AnalysisRequest {
                profile: super::super::AnalysisProfile::Documents,
                ..AnalysisRequest::default()
            },
        );
        assert!(report.is_complete());

        let query = crate::query::Query {
            views: vec![crate::query::ViewSpec::Documents],
            ..crate::query::Query::default()
        };
        let rendered = crate::query::report(
            &index,
            &query,
            &crate::query::Provenance {
                scan_started_at: None,
                generated_at: std::time::UNIX_EPOCH,
                source: crate::query::ReportSource::ColdScan,
                complete: true,
                errors: Vec::new(),
            },
        );
        let crate::query::Section::Metrics { summary, .. } = &rendered.sections[0] else {
            panic!("expected document metrics")
        };
        assert_eq!(summary.share_metric, crate::query::ShareMetric::DocumentWords);
        let rows =
            summary.rows.iter().map(|row| (row.id.as_str(), row)).collect::<BTreeMap<_, _>>();
        let markdown = rows["markdown"];
        assert!(markdown.metrics.raw_words > markdown.metrics.visible_words);
        assert_eq!(markdown.metrics.visible_words, 4);
        assert_eq!(markdown.metrics.paragraphs, 2);
        let text = rows["text"];
        assert!(text.metrics.logical_word_stats.logical_words() > text.metrics.raw_words);
        assert_eq!(crate::query::document_words(&summary.total), 7);
    }
}
