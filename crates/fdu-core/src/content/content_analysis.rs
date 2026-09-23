//! Parallel streaming file reads and conditional analysis commits.

use std::fs::File;
use std::io::Read;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};

use crate::Index;
use crate::classify::{ContentFamily, TypeRegistry, classify_with};

use super::{
    AnalysisApplyOutcome, AnalysisCandidate, AnalysisObservation, AnalysisRequest, AnalyzerOutcome,
    BasicAccumulator, BasicMetrics, CodeAccumulator, CodeMetrics, ContentProvenance,
    CoverageReason, FileAnalysis, TextAdmission, WordMetrics,
    content_markdown_metrics::analyze_markdown,
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
    /// Coverage of the shared lines unit.
    pub lines: AnalyzerCoverage,
    /// Coverage of the code unit when requested.
    pub code: Option<AnalyzerCoverage>,
    /// Coverage of the words unit when requested.
    pub words: Option<AnalyzerCoverage>,
}

/// Operational and semantic outcomes for one requested analyzer unit.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct AnalyzerCoverage {
    /// Files for which the unit produced metrics.
    pub analyzed: u64,
    /// Known or observed binary files.
    pub binary: u64,
    /// Files whose byte stream was not valid UTF-8.
    pub invalid_utf8: u64,
    /// Files with a recognized but unsupported text encoding.
    pub unsupported_encoding: u64,
    /// Files that changed during their read.
    pub changed_during_read: u64,
    /// File-open, metadata, or read failures.
    pub io_errors: u64,
    /// Files for which this analyzer was unavailable.
    pub unsupported: u64,
}

impl AnalysisReport {
    /// Whether content analysis completed without an operational failure.
    ///
    /// Binary data, invalid UTF-8, and unsupported analyzers are coverage outcomes. They
    /// remain visible in roll-ups but do not mean the filesystem operation failed.
    pub fn is_complete(&self) -> bool {
        self.stale == 0
            && self.lines.is_complete()
            && self.code.is_none_or(AnalyzerCoverage::is_complete)
            && self.words.is_none_or(AnalyzerCoverage::is_complete)
    }

    /// Explain operational failures without presenting expected coverage as an error.
    pub fn failure_message(&self) -> Option<String> {
        (!self.is_complete()).then(|| {
            format!(
                "content analysis had operational failures (I/O errors: {}; changed during read: {}; stale results: {}). File and byte totals remain complete; content metrics omit affected files",
                self.io_errors(), self.changed_during_read(), self.stale
            )
        })
    }

    fn io_errors(&self) -> u64 {
        self.lines.io_errors
    }

    fn changed_during_read(&self) -> u64 {
        self.lines.changed_during_read
    }
}

impl AnalyzerCoverage {
    const fn is_complete(self) -> bool {
        self.changed_during_read == 0 && self.io_errors == 0
    }

    fn count(&mut self, reason: CoverageReason) {
        let counter = match reason {
            CoverageReason::Analyzed => &mut self.analyzed,
            CoverageReason::Binary => &mut self.binary,
            CoverageReason::InvalidUtf8 => &mut self.invalid_utf8,
            CoverageReason::UnsupportedEncoding => &mut self.unsupported_encoding,
            CoverageReason::IoError => &mut self.io_errors,
            CoverageReason::ChangedDuringRead => &mut self.changed_during_read,
            CoverageReason::Unsupported => &mut self.unsupported,
        };
        *counter = counter.saturating_add(1);
    }
}

/// Analyze all regular files selected by `request` with a fixed-size worker pool.
///
/// Workers own immutable candidates and never retain an index borrow during I/O. The
/// caller thread applies observations afterward, so metadata changes remain serialized
/// through the index's own apply step, which is crate-private until the request model
/// decides the public analysis surface.
pub fn analyze_index(index: &mut Index, request: AnalysisRequest) -> AnalysisReport {
    if !request.profile.is_enabled() {
        return AnalysisReport::default();
    }
    let pass_started_at_ns =
        crate::query::system_time_to_nanos(std::time::SystemTime::now()).unwrap_or(0);
    let previous_state = index.content().and_then(super::ContentIndex::state);
    index.prepare_content_analysis(request);
    let candidates = index.pending_analysis_candidates(request);
    let mut report = AnalysisReport {
        candidates: u64::try_from(candidates.len()).unwrap_or(u64::MAX),
        code: request.profile.includes_code().then(AnalyzerCoverage::default),
        words: request.profile.includes_words().then(AnalyzerCoverage::default),
        ..AnalysisReport::default()
    };
    if candidates.is_empty() {
        finish_content_tier(index, &report, previous_state, pass_started_at_ns);
        return report;
    }

    let started = std::time::Instant::now();
    // Cloned out before the scope: the workers need the index's rules while the receive
    // loop holds the index mutably, and an `Arc` is what lets both be true.
    let types = index.types_shared();
    let workers = worker_count(request.workers, candidates.len());
    let next = AtomicUsize::new(0);
    let candidates = Arc::new(candidates);
    let (sender, receiver) = mpsc::sync_channel(workers.saturating_mul(2).max(1));

    std::thread::scope(|scope| {
        for _ in 0..workers {
            let sender = sender.clone();
            let candidates = Arc::clone(&candidates);
            let next = &next;
            let types = &types;
            scope.spawn(move || {
                let _counter_guard = crate::counters::thread_flush_guard();
                loop {
                    let slot = next.fetch_add(1, Ordering::Relaxed);
                    let Some(candidate) = candidates.get(slot).cloned() else { break };
                    if sender.send(analyze_candidate(types, candidate, request)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(sender);
        for (observation, bytes_read) in receiver {
            report.bytes_read = report.bytes_read.saturating_add(bytes_read);
            count_coverage(&mut report, &observation.analysis);
            match index.apply_analysis(observation) {
                AnalysisApplyOutcome::Applied => report.applied = report.applied.saturating_add(1),
                AnalysisApplyOutcome::Stale => report.stale = report.stale.saturating_add(1),
            }
        }
    });
    report.elapsed_ns = elapsed_ns(started);
    finish_content_tier(index, &report, previous_state, pass_started_at_ns);
    report
}

fn finish_content_tier(
    index: &mut Index,
    report: &AnalysisReport,
    previous: Option<super::ContentTierState>,
    pass_started_at_ns: i64,
) {
    let entry = index.state();
    let source = match entry.source {
        crate::Source::Cached => crate::Source::Cached,
        _ if previous.is_some_and(|state| state.source == crate::Source::Cached) => {
            crate::Source::Revalidated
        }
        _ => crate::Source::Scanned,
    };
    let freshness = if report.is_complete() {
        match source {
            crate::Source::Cached => crate::Freshness::Stale,
            _ => entry.freshness,
        }
    } else {
        crate::Freshness::Partial
    };
    let observed_at_ns = if source == crate::Source::Cached {
        previous.and_then(|state| state.observed_at_ns)
    } else {
        Some(pass_started_at_ns)
    };
    index.set_content_tier_state(source, freshness, observed_at_ns);
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
    types: &TypeRegistry,
    candidate: AnalysisCandidate,
    request: AnalysisRequest,
) -> (AnalysisObservation, u64) {
    let (analysis, bytes_read) = if candidate.classification.family == ContentFamily::Binary {
        (
            record(
                types,
                &candidate,
                request,
                candidate.classification.clone(),
                CoverageReason::Binary,
                None,
            ),
            0,
        )
    } else {
        analyze_open_file(types, &candidate, request)
    };
    let provenance = ContentProvenance::for_request(request, types.fingerprint());
    (AnalysisObservation { candidate, profile: request.profile, provenance, analysis }, bytes_read)
}

fn analyze_open_file(
    types: &TypeRegistry,
    candidate: &AnalysisCandidate,
    request: AnalysisRequest,
) -> (FileAnalysis, u64) {
    crate::counters::bump(|c| c.file_opens += 1);
    let mut file = match File::open(&candidate.absolute_path) {
        Ok(file) => file,
        Err(error) => return (io_record(types, candidate, request, &error), 0),
    };
    let before = match file.metadata() {
        Ok(metadata) => match crate::scan::attrs_from_file(&file, &metadata) {
            Ok(attrs) => attrs.fingerprint(),
            Err(error) => return (io_record(types, candidate, request, &error), 0),
        },
        Err(error) => return (io_record(types, candidate, request, &error), 0),
    };
    if before != candidate.attrs.fingerprint() {
        return (
            record(
                types,
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
    let mut encoding_prefix = [0_u8; 4];
    let mut encoding_prefix_len = 0_usize;
    let mut encoding_decided = false;
    let mut unsupported_encoding = false;
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
                let mut body = &chunk[..count];
                if !encoding_decided {
                    let take = body.len().min(encoding_prefix.len() - encoding_prefix_len);
                    encoding_prefix[encoding_prefix_len..encoding_prefix_len + take]
                        .copy_from_slice(&body[..take]);
                    encoding_prefix_len += take;
                    body = &body[take..];
                    if has_unsupported_encoding_bom(&encoding_prefix[..encoding_prefix_len]) {
                        unsupported_encoding = true;
                        break;
                    }
                    if encoding_prefix_len < encoding_prefix.len() {
                        continue;
                    }
                    encoding_decided = true;
                }
                if prefix.len() >= 8 && candidate.classification.family == ContentFamily::Unknown {
                    let classification =
                        classify_with(types, &candidate.relative_path, Some(&prefix));
                    if classification.family == ContentFamily::Binary {
                        early_binary = Some(classification);
                        break;
                    }
                    if prefix.len() == CLASSIFICATION_PREFIX_BYTES {
                        if let Some(deferred) = deferred_code.take() {
                            if classification.family == ContentFamily::Code {
                                if let Some(mut code) =
                                    CodeAccumulator::for_type(classification.file_type.as_str())
                                {
                                    code.push(&deferred);
                                    code_accumulator = Some(code);
                                }
                            }
                        }
                        if classification.file_type.as_str() != "markdown" {
                            markdown_source = None;
                        }
                    }
                }
                if encoding_decided && encoding_prefix_len != 0 {
                    push_analysis_bytes(
                        &mut accumulator,
                        &mut code_accumulator,
                        &mut deferred_code,
                        &mut markdown_source,
                        &encoding_prefix[..encoding_prefix_len],
                    );
                    encoding_prefix_len = 0;
                }
                push_analysis_bytes(
                    &mut accumulator,
                    &mut code_accumulator,
                    &mut deferred_code,
                    &mut markdown_source,
                    body,
                );
            }
            Err(error) => {
                read_failure = Some(error);
                break;
            }
        }
    }

    let after = match file.metadata() {
        Ok(metadata) => match crate::scan::attrs_from_file(&file, &metadata) {
            Ok(attrs) => attrs.fingerprint(),
            Err(error) => return (io_record(types, candidate, request, &error), bytes_read),
        },
        Err(error) => return (io_record(types, candidate, request, &error), bytes_read),
    };
    if before != after {
        return (
            record(
                types,
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
        return (io_record(types, candidate, request, &error), bytes_read);
    }
    if unsupported_encoding {
        return (
            record(
                types,
                candidate,
                request,
                candidate.classification.clone(),
                CoverageReason::UnsupportedEncoding,
                None,
            ),
            bytes_read,
        );
    }
    if encoding_prefix_len != 0 {
        push_analysis_bytes(
            &mut accumulator,
            &mut code_accumulator,
            &mut deferred_code,
            &mut markdown_source,
            &encoding_prefix[..encoding_prefix_len],
        );
    }
    if let Some(classification) = early_binary {
        return (
            record(types, candidate, request, classification, CoverageReason::Binary, None),
            bytes_read,
        );
    }
    let classification = classify_with(types, &candidate.relative_path, Some(&prefix));
    if classification.family == ContentFamily::Binary {
        return (
            record(types, candidate, request, classification, CoverageReason::Binary, None),
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
            let mut code_supported = !request.profile.includes_code();
            if request.profile.includes_code() {
                if code_accumulator.is_none() && classification.family == ContentFamily::Code {
                    if let Some(deferred) = deferred_code {
                        if let Some(mut code) =
                            CodeAccumulator::for_type(classification.file_type.as_str())
                        {
                            code.push(&deferred);
                            code_accumulator = Some(code);
                        }
                    }
                }
                code_supported = code_accumulator.is_some();
                if let Some(code) = code_accumulator.take() {
                    let code_metrics = code.finish();
                    debug_assert_eq!(metrics.physical_lines, code_metrics.physical_lines);
                    metrics.code_lines = code_metrics.code_lines;
                    metrics.comment_lines = code_metrics.comment_lines;
                    metrics.code_blank_lines = code_metrics.code_blank_lines;
                }
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
            let lines = BasicMetrics {
                physical_lines: metrics.physical_lines,
                blank_lines: metrics.blank_lines,
                nonblank_lines: metrics.nonblank_lines,
                raw_words: metrics.raw_words,
            };
            let code = request.profile.includes_code().then(|| {
                if code_supported {
                    AnalyzerOutcome::analyzed(CodeMetrics {
                        code_lines: metrics.code_lines,
                        comment_lines: metrics.comment_lines,
                        code_blank_lines: metrics.code_blank_lines,
                    })
                } else {
                    AnalyzerOutcome::unavailable(CoverageReason::Unsupported)
                }
            });
            let words = request.profile.includes_words().then_some(AnalyzerOutcome::analyzed(
                WordMetrics {
                    paragraphs: metrics.paragraphs,
                    visible_words: metrics.visible_words,
                    logical_word_stats: metrics.logical_word_stats,
                    visible_logical_word_stats: metrics.visible_logical_word_stats,
                },
            ));
            analyzed_record(candidate, classification, lines, code, words)
        }
        TextAdmission::Binary => {
            record(types, candidate, request, classification, CoverageReason::Binary, None)
        }
        TextAdmission::InvalidUtf8 => {
            record(types, candidate, request, classification, CoverageReason::InvalidUtf8, None)
        }
    };
    (analysis, bytes_read)
}

fn has_unsupported_encoding_bom(prefix: &[u8]) -> bool {
    prefix.starts_with(&[0xff, 0xfe])
        || prefix.starts_with(&[0xfe, 0xff])
        || prefix.starts_with(&[0x00, 0x00, 0xfe, 0xff])
}

fn push_analysis_bytes(
    accumulator: &mut BasicAccumulator,
    code_accumulator: &mut Option<CodeAccumulator>,
    deferred_code: &mut Option<Vec<u8>>,
    markdown_source: &mut Option<Vec<u8>>,
    bytes: &[u8],
) {
    accumulator.push(bytes);
    if let Some(code) = code_accumulator {
        code.push(bytes);
    }
    if let Some(deferred) = deferred_code {
        deferred.extend_from_slice(bytes);
    }
    if let Some(source) = markdown_source {
        source.extend_from_slice(bytes);
    }
}

fn analyzed_record(
    candidate: &AnalysisCandidate,
    classification: crate::classify::Classification,
    lines: BasicMetrics,
    code: Option<AnalyzerOutcome<CodeMetrics>>,
    words: Option<AnalyzerOutcome<WordMetrics>>,
) -> FileAnalysis {
    FileAnalysis {
        fingerprint: candidate.attrs.fingerprint(),
        bytes: candidate.attrs.size,
        detection: classification.into(),
        lines: AnalyzerOutcome::analyzed(lines),
        code,
        words,
        error: None,
    }
}

fn io_record(
    types: &TypeRegistry,
    candidate: &AnalysisCandidate,
    request: AnalysisRequest,
    error: &std::io::Error,
) -> FileAnalysis {
    let mut detail = error.to_string();
    detail.truncate(char_boundary_at_or_before(&detail, MAX_ERROR_BYTES));
    record(
        types,
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
    _types: &TypeRegistry,
    candidate: &AnalysisCandidate,
    request: AnalysisRequest,
    classification: crate::classify::Classification,
    coverage: CoverageReason,
    error: Option<String>,
) -> FileAnalysis {
    FileAnalysis {
        fingerprint: candidate.attrs.fingerprint(),
        bytes: candidate.attrs.size,
        detection: classification.into(),
        lines: AnalyzerOutcome::unavailable(coverage),
        code: request.profile.includes_code().then_some(AnalyzerOutcome::unavailable(coverage)),
        words: request.profile.includes_words().then_some(AnalyzerOutcome::unavailable(coverage)),
        error,
    }
}

fn count_coverage(report: &mut AnalysisReport, analysis: &FileAnalysis) {
    report.lines.count(analysis.lines.coverage());
    if let (Some(coverage), Some(outcome)) = (&mut report.code, analysis.code) {
        coverage.count(outcome.coverage());
    }
    if let (Some(coverage), Some(outcome)) = (&mut report.words, analysis.words) {
        coverage.count(outcome.coverage());
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

    use crate::content::AnalysisSet;
    use crate::scan::ScanConfig;

    use super::*;

    /// Exercises many streaming chunks with a realistically large generated source file.
    const LARGE_CODE_FILE_BYTES: usize = 17 * 1024 * 1024;

    #[test]
    fn expected_coverage_gaps_are_not_operational_failures() {
        let report = AnalysisReport {
            lines: AnalyzerCoverage { invalid_utf8: 1, ..AnalyzerCoverage::default() },
            code: Some(AnalyzerCoverage { unsupported: 3, ..AnalyzerCoverage::default() }),
            ..AnalysisReport::default()
        };

        assert!(report.is_complete());
        assert_eq!(report.failure_message(), None);

        let failed = AnalysisReport {
            lines: AnalyzerCoverage {
                io_errors: 1,
                changed_during_read: 2,
                ..AnalyzerCoverage::default()
            },
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

    /// Every words-unit metric is independent of whether code was also requested.
    #[test]
    fn code_carries_word_volume_and_paragraphs() {
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
        assert!(code.lines.value().expect("line metrics").raw_words > 0);
        assert_eq!(code.code.expect("code outcome").coverage(), CoverageReason::Analyzed);
        assert!(code.words.and_then(AnalyzerOutcome::value).expect("word metrics").paragraphs > 0);

        let prose = content.file(std::path::Path::new("notes.md")).expect("prose record");
        assert!(prose.lines.value().expect("line metrics").raw_words > 0);
        assert_eq!(prose.code.expect("code outcome").coverage(), CoverageReason::Unsupported);
        assert!(prose.words.and_then(AnalyzerOutcome::value).expect("word metrics").paragraphs > 0);
    }

    #[test]
    fn every_metric_is_independent_of_other_requested_units_and_grouping_is_name_only() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(root.path().join("generated")).expect("generated directory");
        for (path, bytes) in [
            ("main.rs", b"// comment\nfn main() { println!(\"hello world\"); }\n".as_slice()),
            ("tool.py", b"# comment\nprint('hello world')\n"),
            ("Main.hs", b"-- comment\nmain = putStrLn \"hello world\"\n"),
            ("guide.md", b"# Hello\n\nVisible [words](https://example.test).\n"),
            ("notes.txt", b"first paragraph words\n\nsecond paragraph words\n"),
            ("ambiguous.h", b"// generated fixture\nnamespace demo { int value; }\n"),
            ("script", b"#!/usr/bin/env python3\nprint('from shebang')\n"),
            ("document", b"%PDF-1.7\nfixture"),
            ("nul.unknown", b"text before\0binary"),
            ("invalid.unknown", &[b't', b'e', b'x', b't', 0xff]),
            ("generated/output.rs", b"// Code generated; DO NOT EDIT.\nfn output() {}\n"),
        ] {
            fs::write(root.path().join(path), bytes).expect("write fixture");
        }
        let (baseline, scan) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        assert!(scan.is_complete());

        let profiles = [
            AnalysisSet::LINES_ONLY,
            AnalysisSet::CODE_ONLY,
            AnalysisSet::WORDS_ONLY,
            AnalysisSet::ALL,
        ];
        let mut observed = Vec::new();
        for profile in profiles {
            let mut index = baseline.clone();
            let analysis = analyze_index(&mut index, AnalysisRequest { profile, workers: 2 });
            assert!(analysis.is_complete(), "{profile:?}: {analysis:?}");
            let query = crate::query::Query {
                views: vec![crate::query::ViewSpec::Types],
                ..crate::query::Query::default()
            };
            let report = crate::query::report(
                &index,
                &crate::test_support::read_of(&index, query),
                std::time::UNIX_EPOCH,
            )
            .expect("report");
            let crate::query::Section::Metrics { summary, .. } = &report.sections[0] else {
                panic!("expected type metrics")
            };
            let mut groups: Vec<_> = summary.rows.iter().map(|row| row.id.clone()).collect();
            groups.sort();
            let values: std::collections::BTreeMap<_, _> = std::iter::once(&summary.total)
                .chain(&summary.rows)
                .map(|row| {
                    let metrics: Vec<_> = crate::content::METRICS
                        .iter()
                        .map(|metric| row.metric_value(metric))
                        .collect();
                    (row.id.clone(), metrics)
                })
                .collect();
            observed.push((profile, groups, values));
        }

        for (_, groups, _) in &observed[1..] {
            assert_eq!(groups, &observed[0].1, "requested analyzers must not change type groups");
        }
        for (metric_index, metric) in crate::content::METRICS.iter().enumerate() {
            let (_, _, baseline) = observed
                .iter()
                .find(|(profile, _, _)| profile.contains(metric.owner))
                .expect("an owning profile exists");
            for (profile, _, rows) in &observed {
                assert_eq!(rows.keys().collect::<Vec<_>>(), baseline.keys().collect::<Vec<_>>());
                for (id, values) in rows {
                    let expected = if profile.contains(metric.owner) {
                        Some(baseline[id][metric_index].expect("owning profile exposes metric"))
                    } else {
                        None
                    };
                    assert_eq!(
                        values[metric_index], expected,
                        "{} changed or leaked in row {id} under {profile:?}",
                        metric.name
                    );
                }
            }
        }
    }

    #[test]
    fn prefix_classification_handoff_neither_drops_nor_double_counts_large_files() {
        let root = tempfile::tempdir().expect("tempdir");
        for size in [20 * 1024, 80 * 1024] {
            let mut python = b"#!/usr/bin/env python3\n".to_vec();
            while python.len() < size {
                python.extend_from_slice(b"value = 1  # one comment\n");
            }
            fs::write(root.path().join(format!("script-{size}")), &python).expect("shebang");
            fs::write(root.path().join(format!("script-{size}.py")), &python).expect("python");

            let mut plain = Vec::new();
            while plain.len() < size {
                plain.extend_from_slice(b"ordinary words in a plain text line\n");
            }
            fs::write(root.path().join(format!("plain-{size}.unknown")), &plain)
                .expect("unknown text");
            fs::write(root.path().join(format!("plain-{size}.txt")), &plain).expect("text");
        }
        let (baseline, scan) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");
        assert!(scan.is_complete());

        for profile in [AnalysisSet::CODE_ONLY, AnalysisSet::WORDS_ONLY, AnalysisSet::ALL] {
            let mut index = baseline.clone();
            let report = analyze_index(&mut index, AnalysisRequest { profile, workers: 1 });
            assert!(report.is_complete(), "{profile:?}: {report:?}");
            let content = index.content().expect("content");
            for size in [20 * 1024, 80 * 1024] {
                let script = content
                    .file(Path::new(&format!("script-{size}")))
                    .expect("extensionless script");
                let python =
                    content.file(Path::new(&format!("script-{size}.py"))).expect("named Python");
                assert_eq!(script.lines.value(), python.lines.value(), "{profile:?}, {size} bytes");
                if profile.includes_code() {
                    assert_eq!(script.code, python.code, "code handoff at {size} bytes");
                }
                if profile.includes_words() {
                    assert_eq!(script.words, python.words, "word handoff at {size} bytes");
                }

                let unknown = content
                    .file(Path::new(&format!("plain-{size}.unknown")))
                    .expect("unknown text");
                let text =
                    content.file(Path::new(&format!("plain-{size}.txt"))).expect("named text");
                assert_eq!(unknown.lines.value(), text.lines.value(), "plain lines at {size}");
                if profile.includes_words() {
                    assert_eq!(unknown.words, text.words, "plain words at {size}");
                }
            }
        }
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
        assert_eq!(report.lines.analyzed, 1);
        assert_eq!(report.lines.binary, 1);
        assert_eq!(report.bytes_read, 15, "known binary files must not be opened");
        assert!(report.elapsed_ns > 0);
        let root_rollup = index.content_rollup(std::path::Path::new("")).expect("content root");
        assert_eq!(root_rollup.total.files, 2);
        assert_eq!(root_rollup.total.lines.metrics.physical_lines, 3);
        assert_eq!(root_rollup.total.lines.metrics.raw_words, 3);
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

        assert_eq!(report.lines.analyzed, 1);
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

        assert_eq!(report.lines.analyzed, 1);
        assert_eq!(report.bytes_read, LARGE_CODE_FILE_BYTES as u64);
        assert!(report.elapsed_ns > 0);
        assert!(report.is_complete());
        let metrics = &index.content_rollup(std::path::Path::new("")).expect("content root").total;
        assert_eq!(metrics.bytes, LARGE_CODE_FILE_BYTES as u64);
        assert_eq!(metrics.lines.metrics.physical_lines, 2);
        assert_eq!(metrics.code.metrics.code_lines, 1);
        assert_eq!(metrics.code.metrics.comment_lines, 1);
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
        assert_eq!(report.lines.binary, 1);
        assert_eq!(report.lines.invalid_utf8, 1);
        let metrics = &index.content_rollup(std::path::Path::new("")).expect("root").total.lines;
        assert_eq!(metrics.metrics, BasicMetrics::default());
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
        assert_eq!(report.lines.analyzed, 2);
        assert_eq!(report.lines.binary, 1);

        let content = index.content().expect("content");
        let header = content.file(std::path::Path::new("vendor/docs/generated.h")).expect("header");
        assert_eq!(header.detection.file_type.as_str(), "cpp");
        assert_eq!(header.detection.source, crate::classify::DetectionSource::AmbiguousContent);
        assert!(header.detection.flags.generated);
        assert!(header.detection.flags.vendored);
        assert!(header.detection.flags.documentation);

        let script = content.file(std::path::Path::new("script.inc")).expect("script");
        assert_eq!(script.detection.file_type.as_str(), "rust");
        let script_code = script.code.and_then(AnalyzerOutcome::value).expect("code metrics");
        assert_eq!(script_code.comment_lines, 1);
        assert_eq!(script_code.code_lines, 2);

        let pdf = content.file(std::path::Path::new("download")).expect("pdf");
        assert_eq!(pdf.detection.file_type.as_str(), "pdf");
        assert_eq!(pdf.lines.coverage(), CoverageReason::Binary);

        let query = crate::query::Query {
            views: vec![crate::query::ViewSpec::Types],
            ..crate::query::Query::default()
        };
        let report = crate::query::report(
            &index,
            &crate::test_support::read_of(&index, query),
            std::time::UNIX_EPOCH,
        )
        .expect("report");
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

        assert_eq!(report.lines.analyzed, 2);
        assert_eq!(report.code.expect("code coverage").unsupported, 1);
        let rust = index.content().expect("content").file(std::path::Path::new("main.rs"));
        let rust = rust.expect("rust record");
        let lines = rust.lines.value().expect("line metrics");
        let metrics = rust.code.and_then(AnalyzerOutcome::value).expect("code metrics");
        assert_eq!(lines.physical_lines, 3);
        assert_eq!(metrics.code_lines, 1);
        assert_eq!(metrics.comment_lines, 1);
        assert_eq!(metrics.code_blank_lines, 1);
        assert_eq!(
            lines.physical_lines,
            metrics.code_lines + metrics.comment_lines + metrics.code_blank_lines
        );
        let haskell = index.content().expect("content").file(std::path::Path::new("Main.hs"));
        let haskell = haskell.expect("haskell record");
        assert_eq!(haskell.lines.coverage(), CoverageReason::Analyzed);
        assert_eq!(haskell.code.expect("code outcome").coverage(), CoverageReason::Unsupported);

        let query = crate::query::Query {
            views: vec![crate::query::ViewSpec::Languages],
            ..crate::query::Query::default()
        };
        let rendered = crate::query::report(
            &index,
            &crate::test_support::read_of(&index, query),
            std::time::UNIX_EPOCH,
        )
        .expect("report");
        let crate::query::Section::Metrics { summary, .. } = &rendered.sections[0] else {
            panic!("expected language metrics")
        };
        assert_eq!(summary.share_metric, crate::query::ShareMetric::CodeLines);
        assert_eq!((summary.total.share.numerator, summary.total.share.denominator), (1, 1));
        assert_eq!(summary.total.coverage.get(&CoverageReason::Unsupported), Some(&1));
    }

    #[test]
    fn unicode_boms_are_nonoperational_unsupported_encoding_outcomes_per_unit() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::write(root.path().join("little.rs"), [0xff, 0xfe, b'f', 0, b'n', 0])
            .expect("UTF-16 LE");
        fs::write(root.path().join("big.txt"), [0xfe, 0xff, 0, b'w']).expect("UTF-16 BE");
        fs::write(root.path().join("wide"), [0, 0, 0xfe, 0xff, 0, 0, 0, b'w']).expect("UTF-32 BE");
        fs::write(root.path().join("notes.md"), b"ordinary words\n").expect("UTF-8");
        let (mut index, _) =
            crate::scan::scan_into_index(root.path(), &ScanConfig::default()).expect("scan");

        let report = analyze_index(
            &mut index,
            AnalysisRequest { profile: AnalysisSet::ALL, ..AnalysisRequest::default() },
        );

        assert!(report.is_complete(), "unsupported encodings are not operational failures");
        assert_eq!(report.lines.unsupported_encoding, 3);
        assert_eq!(report.code.expect("code coverage").unsupported_encoding, 3);
        assert_eq!(report.words.expect("word coverage").unsupported_encoding, 3);
        assert_eq!(report.lines.binary, 0);
        assert_eq!(report.lines.invalid_utf8, 0);
        let query = crate::query::Query {
            views: vec![crate::query::ViewSpec::Types],
            ..crate::query::Query::default()
        };
        let rendered = crate::query::report(
            &index,
            &crate::test_support::read_of(&index, query),
            std::time::UNIX_EPOCH,
        )
        .expect("report");
        for format in [crate::report_format::Format::Json, crate::report_format::Format::Yaml] {
            let output = crate::report_format::render(&rendered, format, false);
            let total = match format {
                crate::report_format::Format::Json => {
                    output.split("\"rows\":").next().expect("metrics total")
                }
                crate::report_format::Format::Yaml => {
                    output.split("\n      rows:").next().expect("metrics total")
                }
                _ => unreachable!("only machine document formats are tested"),
            };
            assert_eq!(
                total.matches("unsupported_encoding").count(),
                3,
                "each requested analyzer unit has its own total coverage map: {output}"
            );
        }
        let text =
            crate::report_format::render(&rendered, crate::report_format::Format::Text, false);
        assert!(
            text.contains("1 lines (1 nonblank, 0 blank), 2 words"),
            "unsupported code coverage cannot turn prose lines into a zero code partition: {text}"
        );
        let content = index.content().expect("content");
        for path in ["little.rs", "big.txt", "wide"] {
            let record = content.file(Path::new(path)).expect("encoding record");
            assert_eq!(record.lines.coverage(), CoverageReason::UnsupportedEncoding);
            assert_eq!(
                record.code.expect("code outcome").coverage(),
                CoverageReason::UnsupportedEncoding
            );
            assert_eq!(
                record.words.expect("word outcome").coverage(),
                CoverageReason::UnsupportedEncoding
            );
            assert!(record.error.is_none());
        }
        assert_eq!(
            content
                .file(Path::new("notes.md"))
                .expect("markdown record")
                .code
                .expect("code outcome")
                .coverage(),
            CoverageReason::Unsupported,
            "an unsupported analyzer remains distinct from an unsupported encoding"
        );
    }

    #[test]
    fn unsupported_encoding_boms_are_recognized_from_progressive_prefixes() {
        for (bom, recognized_at) in [
            (&[0xff, 0xfe][..], 2_usize),
            (&[0xfe, 0xff][..], 2_usize),
            (&[0xff, 0xfe, 0, 0][..], 2_usize),
            (&[0, 0, 0xfe, 0xff][..], 4_usize),
        ] {
            for length in 0..=bom.len() {
                assert_eq!(
                    has_unsupported_encoding_bom(&bom[..length]),
                    length >= recognized_at,
                    "{length} byte prefix of {bom:?}"
                );
            }
        }
        assert!(!has_unsupported_encoding_bom(&[0xef, 0xbb, 0xbf, b'x']));
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
            &crate::test_support::read_of(&index, query),
            std::time::UNIX_EPOCH,
        )
        .expect("report");
        let crate::query::Section::Metrics { summary, .. } = &rendered.sections[0] else {
            panic!("expected document metrics")
        };
        assert_eq!(summary.share_metric, crate::query::ShareMetric::DocumentWords);
        let rows =
            summary.rows.iter().map(|row| (row.id.as_str(), row)).collect::<BTreeMap<_, _>>();
        let markdown = rows["markdown"];
        assert!(markdown.metrics.raw_words > markdown.metrics.visible_words);
        assert_eq!(markdown.metrics.visible_words, Some(4));
        assert_eq!(markdown.metrics.paragraphs, Some(2));
        let text = rows["text"];
        assert!(text.metrics.logical_words > text.metrics.raw_words);
        assert_eq!(crate::query::document_words(&summary.total), Some(7));
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
        assert_eq!(
            content.provenance(),
            Some(ContentProvenance::for_request(request, crate::classify::type_rule_fingerprint()))
        );
    }

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
            &crate::test_support::read_of(
                &empty_index,
                crate::query::Query {
                    views: vec![crate::query::ViewSpec::Summary],
                    ..crate::query::Query::default()
                },
            ),
            std::time::UNIX_EPOCH,
        )
        .expect("report");
        let json =
            crate::report_format::render(&summary, crate::report_format::Format::Json, false);
        assert!(json.contains("\"schema\": \"fdu.report/7\""), "{json}");
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
            &crate::test_support::read_of(
                &unsupported_index,
                crate::query::Query {
                    views: vec![crate::query::ViewSpec::Languages],
                    ..crate::query::Query::default()
                },
            ),
            std::time::UNIX_EPOCH,
        )
        .expect("report");
        let text =
            crate::report_format::render(&languages, crate::report_format::Format::Text, false);
        assert!(text.contains("—"), "an unavailable 0/0 share needs a distinct marker: {text}");
        assert!(
            text.contains("1 lines (1 nonblank, 0 blank), 1 unsupported"),
            "unsupported code falls back to the valid line partition: {text}"
        );
        assert!(!text.contains("0.0%"), "unmeasured is not a zero percentage: {text}");
    }
}
