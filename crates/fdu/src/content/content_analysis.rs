//! Parallel streaming file reads and conditional analysis commits.

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
    MetricValues, TextAdmission, content_markdown_metrics::analyze_markdown,
};

const READ_CHUNK_BYTES: usize = 64 * 1024;
const CLASSIFICATION_PREFIX_BYTES: usize = 16 * 1024;
const MAX_ERROR_BYTES: usize = 512;

/// Operational counters from one content-analysis pass.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct AnalysisReport {
    /// Regular files considered by the requested profile.
    pub candidates: u64,
    /// Bytes actually returned by fresh file reads, including partial binary probes.
    pub bytes_read: u64,
    /// Wall time spent processing fresh candidates.
    pub elapsed_ns: u64,
    /// Results accepted by the index's conditional mutation boundary.
    pub applied: u64,
    /// Results discarded because indexed metadata changed while workers ran.
    pub stale: u64,
    /// Files for which metrics were produced.
    pub analyzed: u64,
    /// Known or observed binary files.
    pub binary: u64,
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
    /// Whether content analysis completed without an operational failure.
    ///
    /// Binary data, invalid UTF-8, and unsupported analyzers are coverage outcomes. They
    /// remain visible in roll-ups but do not mean the filesystem operation failed.
    pub const fn is_complete(&self) -> bool {
        self.stale == 0 && self.changed_during_read == 0 && self.io_errors == 0
    }

    /// Explain operational failures without presenting expected coverage as an error.
    pub fn failure_message(&self) -> Option<String> {
        (!self.is_complete()).then(|| {
            format!(
                "content analysis had operational failures (I/O errors: {}; changed during read: {}; stale results: {}). File and byte totals remain complete; content metrics omit affected files",
                self.io_errors, self.changed_during_read, self.stale
            )
        })
    }
}

/// Analyze all regular files selected by `request` with a fixed-size worker pool.
///
/// Workers own immutable candidates and never retain an index borrow during I/O. The
/// caller thread applies observations afterward, so metadata changes remain serialized
/// through [`Index::apply_analysis`].
pub fn analyze_index(index: &mut Index, request: AnalysisRequest) -> AnalysisReport {
    if !request.profile.is_enabled() {
        return AnalysisReport::default();
    }
    index.prepare_content_analysis(request);
    let candidates = index.pending_analysis_candidates(request);
    let mut report = AnalysisReport {
        candidates: u64::try_from(candidates.len()).unwrap_or(u64::MAX),
        ..AnalysisReport::default()
    };
    if candidates.is_empty() {
        return report;
    }

    let started = std::time::Instant::now();
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
                let _counter_guard = crate::counters::thread_flush_guard();
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
        for (observation, bytes_read) in receiver {
            report.bytes_read = report.bytes_read.saturating_add(bytes_read);
            count_coverage(&mut report, observation.analysis.coverage);
            match index.apply_analysis(observation) {
                AnalysisApplyOutcome::Applied => report.applied = report.applied.saturating_add(1),
                AnalysisApplyOutcome::Stale => report.stale = report.stale.saturating_add(1),
            }
        }
    });
    report.elapsed_ns = elapsed_ns(started);
    report
}

fn elapsed_ns(started: std::time::Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn worker_count(requested: usize, candidates: usize) -> usize {
    let available = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
    let requested = if requested == 0 { available } else { requested };
    requested.clamp(1, candidates.max(1))
}

fn analyze_candidate(
    candidate: AnalysisCandidate,
    request: AnalysisRequest,
) -> (AnalysisObservation, u64) {
    let (analysis, bytes_read) = if candidate.classification.family == ContentFamily::Binary {
        (
            record(
                &candidate,
                request,
                candidate.classification.clone(),
                CoverageReason::Binary,
                None,
            ),
            0,
        )
    } else {
        analyze_open_file(&candidate, request)
    };
    (AnalysisObservation { candidate, analysis }, bytes_read)
}

fn analyze_open_file(
    candidate: &AnalysisCandidate,
    request: AnalysisRequest,
) -> (FileAnalysis, u64) {
    crate::counters::bump(|c| c.file_opens += 1);
    let mut file = match File::open(&candidate.absolute_path) {
        Ok(file) => file,
        Err(error) => return (io_record(candidate, request, &error), 0),
    };
    let before = match file.metadata() {
        Ok(metadata) => crate::scan::attrs_from(&metadata).fingerprint(),
        Err(error) => return (io_record(candidate, request, &error), 0),
    };
    if before != candidate.attrs.fingerprint() {
        return (
            record(
                candidate,
                request,
                candidate.classification.clone(),
                CoverageReason::ChangedDuringRead,
                None,
            ),
            0,
        );
    }

    let mut accumulator = BasicAccumulator::with_logical_metrics(request.profile.includes_words());
    let mut code_accumulator = request
        .profile
        .includes_code()
        .then(|| CodeAccumulator::for_type(candidate.classification.file_type.as_str()))
        .flatten();
    let mut deferred_code = (request.profile.includes_code()
        && candidate.classification.family == ContentFamily::Unknown)
        .then(Vec::new);
    let mut markdown_source = (request.profile.includes_words()
        && (candidate.classification.file_type.as_str() == "markdown"
            || candidate.classification.family == ContentFamily::Unknown))
        .then(Vec::new);
    let mut prefix = Vec::with_capacity(CLASSIFICATION_PREFIX_BYTES);
    let mut chunk = vec![0_u8; READ_CHUNK_BYTES];
    let mut read_failure = None;
    let mut early_binary = None;
    let mut bytes_read = 0_u64;
    loop {
        crate::counters::bump(|c| c.file_reads += 1);
        match file.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => {
                crate::counters::bump(|c| c.bytes_read += count as u64);
                bytes_read = bytes_read.saturating_add(u64::try_from(count).unwrap_or(u64::MAX));
                if prefix.len() < CLASSIFICATION_PREFIX_BYTES {
                    let take = count.min(CLASSIFICATION_PREFIX_BYTES - prefix.len());
                    prefix.extend_from_slice(&chunk[..take]);
                }
                if prefix.len() >= 8 && candidate.classification.family == ContentFamily::Unknown {
                    let classification =
                        classify_path_with_prefix(&candidate.relative_path, Some(&prefix));
                    if classification.family == ContentFamily::Binary {
                        early_binary = Some(classification);
                        break;
                    }
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
        Err(error) => return (io_record(candidate, request, &error), bytes_read),
    };
    if before != after {
        return (
            record(
                candidate,
                request,
                candidate.classification.clone(),
                CoverageReason::ChangedDuringRead,
                None,
            ),
            bytes_read,
        );
    }
    if let Some(error) = read_failure {
        return (io_record(candidate, request, &error), bytes_read);
    }
    if let Some(classification) = early_binary {
        return (
            record(candidate, request, classification, CoverageReason::Binary, None),
            bytes_read,
        );
    }
    let classification = classify_path_with_prefix(&candidate.relative_path, Some(&prefix));
    if classification.family == ContentFamily::Binary {
        return (
            record(candidate, request, classification, CoverageReason::Binary, None),
            bytes_read,
        );
    }
    let analysis = match accumulator.finish() {
        TextAdmission::Accepted(mut metrics) => {
            // Word volume is meaningful for any text, and the accumulator has already
            // counted it during the streaming read — zeroing it outside prose and markup
            // discarded finished work and, with it, the answer to "how much text is in
            // this tree", which is the cheap proxy for context-window sizing that agent
            // consumers ask for.
            //
            // Paragraphs stay a prose concept. A blank-line-separated block of code is
            // not a paragraph, and counting it as one would make the prose rows lie
            // rather than merely say less.
            if !matches!(classification.family, ContentFamily::Prose | ContentFamily::Markup) {
                metrics.paragraphs = 0;
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
                    return (
                        record(
                            candidate,
                            request,
                            classification,
                            CoverageReason::Unsupported,
                            None,
                        ),
                        bytes_read,
                    );
                };
                let code_metrics = code.finish();
                debug_assert_eq!(metrics.physical_lines, code_metrics.physical_lines);
                metrics.code_lines = code_metrics.code_lines;
                metrics.comment_lines = code_metrics.comment_lines;
                metrics.code_blank_lines = code_metrics.code_blank_lines;
            }
            if request.profile.includes_words() && classification.file_type.as_str() == "markdown" {
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
    };
    (analysis, bytes_read)
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

    /// Exercises many streaming chunks with a realistically large generated source file.
    const LARGE_CODE_FILE_BYTES: usize = 17 * 1024 * 1024;

    #[test]
    fn expected_coverage_gaps_are_not_operational_failures() {
        let report =
            AnalysisReport { invalid_utf8: 1, unsupported: 3, ..AnalysisReport::default() };

        assert!(report.is_complete());
        assert_eq!(report.failure_message(), None);

        let failed = AnalysisReport {
            io_errors: 1,
            changed_during_read: 2,
            stale: 3,
            ..AnalysisReport::default()
        };
        assert!(!failed.is_complete());
        assert_eq!(
            failed.failure_message().as_deref(),
            Some(
                "content analysis had operational failures (I/O errors: 1; changed during read: 2; stale results: 3). File and byte totals remain complete; content metrics omit affected files"
            )
        );
    }

    /// Word volume is counted during the streaming read for every admitted text file, so
    /// it is reported wherever it was measured. Paragraphs are not: a blank-line-separated
    /// block of code is not a paragraph, and counting it as one would make prose rows lie.
    #[test]
    fn code_carries_word_volume_but_never_paragraphs() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("main.rs"), b"fn main() {\n\n    let x = 1;\n}\n")
            .expect("write");
        std::fs::write(root.path().join("notes.md"), b"one two\n\nthree four\n").expect("write");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        analyze_index(
            &mut index,
            AnalysisRequest { profile: super::super::AnalysisSet::ALL, workers: 2 },
        );

        let content = index.content().expect("content");
        let code = content.file(std::path::Path::new("main.rs")).expect("code record");
        assert!(code.metrics.raw_words > 0, "code keeps the words already counted for it");
        assert_eq!(code.metrics.paragraphs, 0, "code has no paragraphs");

        let prose = content.file(std::path::Path::new("notes.md")).expect("prose record");
        assert!(prose.metrics.raw_words > 0);
        assert!(prose.metrics.paragraphs > 0, "prose still counts paragraphs");
    }

    #[test]
    fn pool_analyzes_text_and_skips_known_binary_files() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("notes.md"), "one two\n\nthree\n").expect("write text");
        fs::write(root.path().join("image.png"), b"not opened as text").expect("write binary");
        let (mut index, scan) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        assert!(scan.is_complete());

        let report = analyze_index(
            &mut index,
            AnalysisRequest { profile: super::super::AnalysisSet::NONE.with_lines(), workers: 2 },
        );

        assert_eq!(report.candidates, 2);
        assert_eq!(report.analyzed, 1);
        assert_eq!(report.binary, 1);
        assert_eq!(report.bytes_read, 15, "known binary files must not be opened");
        assert!(report.elapsed_ns > 0);
        let root_rollup = index.content_rollup(std::path::Path::new("")).expect("content root");
        assert_eq!(root_rollup.total.files, 2);
        assert_eq!(root_rollup.total.metrics.physical_lines, 3);
        assert_eq!(root_rollup.total.metrics.raw_words, 3);
    }

    #[test]
    fn content_workers_publish_their_file_io_counters() {
        let _serial = crate::counters::test_serial();
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("notes.md"), "one two\n\nthree\n").expect("write text");
        let (mut index, scan) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        assert!(scan.is_complete());

        crate::counters::enable(true);
        crate::counters::reset();
        let report = analyze_index(
            &mut index,
            AnalysisRequest { profile: super::super::AnalysisSet::NONE.with_lines(), workers: 2 },
        );
        crate::counters::flush_thread();
        let counts = crate::counters::snapshot();
        crate::counters::reset();
        crate::counters::enable(false);

        assert_eq!(report.analyzed, 1);
        assert!(counts.file_opens >= 1, "content worker file open was folded: {counts:?}");
        assert!(counts.file_reads >= 2, "data and EOF reads were folded: {counts:?}");
        assert!(counts.bytes_read >= 15, "content bytes were folded: {counts:?}");
    }

    #[test]
    fn large_generated_code_is_analyzed_through_eof() {
        let root = tempfile::tempdir().expect("tempdir");
        let mut source = b"/* generated */\n".to_vec();
        source.resize(LARGE_CODE_FILE_BYTES - 1, b'x');
        source.push(b'\n');
        fs::write(root.path().join("generated.c"), source).expect("write generated C");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");

        let report = analyze_index(
            &mut index,
            AnalysisRequest { profile: super::super::AnalysisSet::NONE.with_code(), workers: 1 },
        );

        assert_eq!(report.analyzed, 1);
        assert_eq!(report.bytes_read, LARGE_CODE_FILE_BYTES as u64);
        assert!(report.elapsed_ns > 0);
        assert!(report.is_complete());
        let metrics = &index.content_rollup(std::path::Path::new("")).expect("content root").total;
        assert_eq!(metrics.bytes, LARGE_CODE_FILE_BYTES as u64);
        assert_eq!(metrics.metrics.physical_lines, 2);
        assert_eq!(metrics.metrics.code_lines, 1);
        assert_eq!(metrics.metrics.comment_lines, 1);
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
                profile: super::super::AnalysisSet::NONE.with_lines(),
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
    fn deep_detection_drives_named_consumers_and_report_evidence() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(root.path().join("vendor/docs")).expect("directories");
        fs::write(
            root.path().join("vendor/docs/generated.h"),
            "// Code generated by fixture; DO NOT EDIT.\nnamespace demo { int value; }\n",
        )
        .expect("write header");
        fs::write(
            root.path().join("script.inc"),
            "# vim: set filetype=rust:\n// comment\nfn main() {}\n",
        )
        .expect("write modeline");
        fs::write(root.path().join("download"), b"%PDF-1.7\nfixture payload")
            .expect("write signature");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");

        let report = analyze_index(
            &mut index,
            AnalysisRequest {
                profile: super::super::AnalysisSet::NONE.with_code(),
                ..AnalysisRequest::default()
            },
        );
        assert_eq!(report.analyzed, 2);
        assert_eq!(report.binary, 1);

        let content = index.content().expect("content");
        let header = content.file(std::path::Path::new("vendor/docs/generated.h")).expect("header");
        assert_eq!(header.classification.file_type.as_str(), "cpp");
        assert_eq!(
            header.classification.source,
            crate::classify::DetectionSource::AmbiguousContent
        );
        assert!(header.classification.flags.generated);
        assert!(header.classification.flags.vendored);
        assert!(header.classification.flags.documentation);

        let script = content.file(std::path::Path::new("script.inc")).expect("script");
        assert_eq!(script.classification.file_type.as_str(), "rust");
        assert_eq!(script.metrics.comment_lines, 1);
        assert_eq!(script.metrics.code_lines, 2);

        let pdf = content.file(std::path::Path::new("download")).expect("pdf");
        assert_eq!(pdf.classification.file_type.as_str(), "pdf");
        assert_eq!(pdf.coverage, CoverageReason::Binary);

        let query = crate::query::Query {
            views: vec![crate::query::ViewSpec::Types],
            ..crate::query::Query::default()
        };
        let report = crate::query::report(
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
        let crate::query::Section::Metrics { summary, .. } = &report.sections[0] else {
            panic!("expected metric summary")
        };
        assert_eq!(summary.total.generated_files, 1);
        assert_eq!(summary.total.vendored_files, 1);
        assert_eq!(summary.total.documentation_files, 1);
        assert_eq!(
            summary.total.detection_sources.get(&crate::classify::DetectionSource::FormatSignature),
            Some(&1)
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
                profile: super::super::AnalysisSet::NONE.with_code(),
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
                profile: super::super::AnalysisSet::NONE.with_words(),
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

    #[test]
    fn an_empty_analysis_still_retains_the_requested_identity() {
        let root = tempfile::tempdir().expect("tempdir");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        let request = AnalysisRequest {
            profile: super::super::AnalysisSet::NONE.with_lines(),
            ..AnalysisRequest::default()
        };

        let analysis = analyze_index(&mut index, request);

        assert_eq!(analysis.candidates, 0);
        let content = index.content().expect("the requested derived tier remains explicit");
        assert_eq!(content.profile(), Some(request.profile));
        assert_eq!(content.provenance(), Some(&ContentProvenance::for_request(request)));
    }

    #[cfg(feature = "cli")]
    #[test]
    fn content_reports_preserve_empty_profiles_and_unavailable_shares() {
        let empty = tempfile::tempdir().expect("empty tempdir");
        let (mut empty_index, _) =
            crate::scan::scan_into_index(empty.path(), &ScanConfig::default()).expect("scan");
        analyze_index(
            &mut empty_index,
            AnalysisRequest {
                profile: super::super::AnalysisSet::NONE.with_lines(),
                ..AnalysisRequest::default()
            },
        );
        let summary = crate::query::report(
            &empty_index,
            &crate::query::Query {
                views: vec![crate::query::ViewSpec::Summary],
                ..crate::query::Query::default()
            },
            &crate::query::Provenance {
                scan_started_at: None,
                generated_at: std::time::UNIX_EPOCH,
                source: crate::query::ReportSource::ColdScan,
                complete: true,
                errors: Vec::new(),
            },
        );
        let json =
            crate::report_format::render(&summary, crate::report_format::Format::Json, false);
        assert!(json.contains("\"schema\": \"fdu.report/5\""), "{json}");
        assert!(json.contains("\"analyze\": [\"lines\"]"), "{json}");

        let unsupported = tempfile::tempdir().expect("unsupported tempdir");
        fs::write(unsupported.path().join("Main.hs"), "main = pure ()\n").expect("write");
        let (mut unsupported_index, _) =
            crate::scan::scan_into_index(unsupported.path(), &ScanConfig::default()).expect("scan");
        analyze_index(
            &mut unsupported_index,
            AnalysisRequest {
                profile: super::super::AnalysisSet::NONE.with_code(),
                ..AnalysisRequest::default()
            },
        );
        let languages = crate::query::report(
            &unsupported_index,
            &crate::query::Query {
                views: vec![crate::query::ViewSpec::Languages],
                ..crate::query::Query::default()
            },
            &crate::query::Provenance {
                scan_started_at: None,
                generated_at: std::time::UNIX_EPOCH,
                source: crate::query::ReportSource::ColdScan,
                complete: false,
                errors: Vec::new(),
            },
        );
        let text =
            crate::report_format::render(&languages, crate::report_format::Format::Text, false);
        assert!(text.contains("—"), "an unavailable 0/0 share needs a distinct marker: {text}");
        assert!(text.contains("1 unsupported"), "coverage must remain visible: {text}");
        assert!(!text.contains("0.0%"), "unmeasured is not a zero percentage: {text}");
    }
}
