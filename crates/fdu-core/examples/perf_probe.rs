//! Repository-only component probe for the performance evidence harness.
//!
//! This example is excluded from published packages. It uses only supported public
//! APIs and emits one compact, versioned JSON object after the measured component has
//! completed. The parent harness owns authoritative process timing and exact corpus
//! validation.

// Measurement scaffolding, kept out of the library so the engine's unsafe-free
// guarantee stands: counting allocations needs `unsafe impl GlobalAlloc`, and the
// probe is the right place to pay for that.
use std::env;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::hint::black_box;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use fdu_core::content::{AnalysisRequest, AnalysisSet, CoverageReason};
use fdu_core::query::{Provenance, Query, ReportSource, ViewSpec};
use fdu_core::{
    Attrs, CachePolicy, ChangeOutcome, ChangeRequest, Clock, Commit, Coverage, EffectiveChange,
    EntryId, EntryKind, Index, LifecyclePhase, Observation, Op, OpenConfig, OpenOptions,
    OpenedIndex, ReadRequest, ScanConfig, ScanOrder,
};

const PROBE_SCHEMA: &str = "fdu-perf-probe-v1";
const DIGEST_ALGORITHM: &str = "fdu-index-record-v1/sha256-multiset-v1";
const COMMIT_DIGEST_ALGORITHM: &str = "fdu-commit-debug-v1/sha256-sequence-v1";
const OPENED_PROBE_JOURNAL_CAPACITY: usize = 4 * 1024 * 1024;

/// Count what the run allocates.
///
/// Always installed, and inert until recording is enabled: the wrapper checks one
/// relaxed atomic the branch predictor always gets right, which is nothing next to an
/// allocation. One binary that can be asked for numbers beats two that differ in
/// whether they have any. exp-052 measured the whole arrangement at no detectable cost.
#[global_allocator]
static ALLOCATOR: fdu_core::counters::alloc::CountingAlloc<std::alloc::System> =
    fdu_core::counters::system_allocator();

fn main() -> ExitCode {
    let arguments: Vec<_> = env::args_os().skip(1).collect();
    if arguments.len() == 1 && arguments[0] == "--version" {
        println!("fdu-perf-probe {}", env!("FDU_BUILD_VERSION"));
        return ExitCode::SUCCESS;
    }
    // `FDU_COUNTERS=1` turns recording on. Off by default so a probe run measured
    // against a control is not measuring the instrument, and on by one environment
    // variable when a run is meant to explain itself.
    let measurement = fdu_core::counters::Measurement::from_env();
    let exit = match Arguments::parse(arguments.into_iter())
        .and_then(|arguments| execute_repeated(&arguments))
    {
        Ok(output) => {
            println!("{}", output.render());
            // stderr, not the JSON: the harness parses stdout against a versioned
            // schema, and per-layer tallies describe an implementation rather than the
            // measurement contract. A schema bump can follow if the harness ever wants
            // to store them.
            if output.summary.complete { ExitCode::SUCCESS } else { ExitCode::from(2) }
        }
        Err(error) => {
            eprintln!("fdu perf probe: {}", error.0);
            ExitCode::from(1)
        }
    };
    if let Some(report) = measurement.finish() {
        eprint!("{report}");
    }
    exit
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    CodeSloc,
    CodeSlocCacheHit,
    CodeSlocSeed,
    ContentBasic,
    ContentBinaryGate,
    ContentCacheHit,
    ContentDisabled,
    ContentOpen,
    ContentQuery,
    ContentSeed,
    DetectAmbiguous,
    DetectResolved,
    DocumentCacheHit,
    DocumentSeed,
    DeltaApply,
    DeltaApplyBatched,
    DeltaApplyLarge,
    MarkdownProse,
    OpenedDiscovery,
    Query,
    ColdOpenSave,
    DefaultTree,
    Revalidate,
    ScanIndex,
    ScanProducer,
    SnapshotLoad,
    Summary,
    SnapshotSave,
    TextProse,
    ValidateIndex,
}

impl Mode {
    fn parse(value: &str) -> ProbeResult<Self> {
        match value {
            "code-sloc" => Ok(Self::CodeSloc),
            "code-sloc-cache-hit" => Ok(Self::CodeSlocCacheHit),
            "code-sloc-seed" => Ok(Self::CodeSlocSeed),
            "content-basic" => Ok(Self::ContentBasic),
            "content-binary-gate" => Ok(Self::ContentBinaryGate),
            "content-cache-hit" => Ok(Self::ContentCacheHit),
            "content-disabled" => Ok(Self::ContentDisabled),
            "content-open" => Ok(Self::ContentOpen),
            "content-query" => Ok(Self::ContentQuery),
            "content-seed" => Ok(Self::ContentSeed),
            "detect-ambiguous" => Ok(Self::DetectAmbiguous),
            "detect-resolved" => Ok(Self::DetectResolved),
            "document-cache-hit" => Ok(Self::DocumentCacheHit),
            "document-seed" => Ok(Self::DocumentSeed),
            "delta-apply" => Ok(Self::DeltaApply),
            "delta-apply-batched" => Ok(Self::DeltaApplyBatched),
            "delta-apply-large" => Ok(Self::DeltaApplyLarge),
            "markdown-prose" => Ok(Self::MarkdownProse),
            "opened-discovery" => Ok(Self::OpenedDiscovery),
            "query" => Ok(Self::Query),
            "revalidate" => Ok(Self::Revalidate),
            "cold-open-save" => Ok(Self::ColdOpenSave),
            "default-tree" => Ok(Self::DefaultTree),
            "scan-index" => Ok(Self::ScanIndex),
            "scan-producer" => Ok(Self::ScanProducer),
            "snapshot-load" => Ok(Self::SnapshotLoad),
            "snapshot-save" => Ok(Self::SnapshotSave),
            "summary" => Ok(Self::Summary),
            "text-prose" => Ok(Self::TextProse),
            "validate-index" => Ok(Self::ValidateIndex),
            _ => Err(ProbeError(format!("unknown mode {value:?}"))),
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::CodeSloc => "code-sloc",
            Self::CodeSlocCacheHit => "code-sloc-cache-hit",
            Self::CodeSlocSeed => "code-sloc-seed",
            Self::ContentBasic => "content-basic",
            Self::ContentBinaryGate => "content-binary-gate",
            Self::ContentCacheHit => "content-cache-hit",
            Self::ContentDisabled => "content-disabled",
            Self::ContentOpen => "content-open",
            Self::ContentQuery => "content-query",
            Self::ContentSeed => "content-seed",
            Self::DetectAmbiguous => "detect-ambiguous",
            Self::DetectResolved => "detect-resolved",
            Self::DocumentCacheHit => "document-cache-hit",
            Self::DocumentSeed => "document-seed",
            Self::DeltaApply => "delta-apply",
            Self::DeltaApplyBatched => "delta-apply-batched",
            Self::DeltaApplyLarge => "delta-apply-large",
            Self::MarkdownProse => "markdown-prose",
            Self::OpenedDiscovery => "opened-discovery",
            Self::Query => "query",
            Self::Revalidate => "revalidate",
            Self::ColdOpenSave => "cold-open-save",
            Self::DefaultTree => "default-tree",
            Self::ScanIndex => "scan-index",
            Self::ScanProducer => "scan-producer",
            Self::SnapshotLoad => "snapshot-load",
            Self::SnapshotSave => "snapshot-save",
            Self::Summary => "summary",
            Self::TextProse => "text-prose",
            Self::ValidateIndex => "validate-index",
        }
    }
}

#[derive(Debug)]
struct Arguments {
    mode: Mode,
    root: PathBuf,
    snapshot: Option<PathBuf>,
    operations: usize,
    queries: usize,
    repeat: usize,
    oracle_enabled: bool,
    diagnostics: bool,
    worker_policy: fdu_core::scan::WorkerPolicyExperiment,
    scan: ScanConfig,
}

impl Arguments {
    fn parse(arguments: impl Iterator<Item = OsString>) -> ProbeResult<Self> {
        let mut arguments = arguments;
        let mode = arguments
            .next()
            .ok_or_else(|| ProbeError("missing probe mode".into()))?
            .into_string()
            .map_err(|_| ProbeError("probe mode must be Unicode".into()))?;
        let mut root = None;
        let mut snapshot = None;
        let mut operations = 1_000_usize;
        let mut queries = 1_000_usize;
        let mut repeat = 1_usize;
        let mut oracle_enabled = true;
        let mut diagnostics = false;
        let mut worker_policy = fdu_core::scan::WorkerPolicyExperiment::ShippedOneShot;
        let mut scan = ScanConfig::default();
        while let Some(flag) = arguments.next() {
            match flag.to_str() {
                Some("--root") => root = Some(next_path(&mut arguments, "--root")?),
                Some("--snapshot") => {
                    snapshot = Some(next_path(&mut arguments, "--snapshot")?);
                }
                Some("--operations") => {
                    operations = next_usize(&mut arguments, "--operations")?;
                }
                Some("--queries") => {
                    queries = next_usize(&mut arguments, "--queries")?;
                }
                Some("--repeat") => {
                    repeat = next_usize(&mut arguments, "--repeat")?;
                }
                Some("--no-oracle") => oracle_enabled = false,
                Some("--diagnostics") => diagnostics = true,
                Some("--worker-policy") => {
                    let value = arguments
                        .next()
                        .ok_or_else(|| ProbeError("--worker-policy requires a value".into()))?;
                    worker_policy = match value.to_str() {
                        Some("shipped") => fdu_core::scan::WorkerPolicyExperiment::ShippedOneShot,
                        Some("repeated") => fdu_core::scan::WorkerPolicyExperiment::RepeatedWindows,
                        Some("staged-gated") => {
                            fdu_core::scan::WorkerPolicyExperiment::StagedGatedWindows
                        }
                        _ => return Err(ProbeError(format!("unknown worker policy {value:?}"))),
                    };
                    diagnostics = true;
                }
                Some("--batch-size") => {
                    scan.batch_size = next_usize(&mut arguments, "--batch-size")?;
                }
                Some("--order") => {
                    let value = arguments
                        .next()
                        .ok_or_else(|| ProbeError("--order requires a value".into()))?;
                    scan.order = match value.to_str() {
                        Some("breadth-first") => ScanOrder::BreadthFirst,
                        Some("depth-first") => ScanOrder::DepthFirst,
                        _ => return Err(ProbeError(format!("unknown order {value:?}"))),
                    };
                }
                Some("--threads") => {
                    scan.threads = Some(next_usize(&mut arguments, "--threads")?);
                }
                Some("--max-depth") => {
                    scan.max_depth = Some(next_usize(&mut arguments, "--max-depth")?);
                }
                _ => return Err(ProbeError(format!("unknown argument {flag:?}"))),
            }
        }
        let root = root.ok_or_else(|| ProbeError("--root is required".into()))?;
        if operations == 0 || queries == 0 {
            return Err(ProbeError("--operations and --queries must be nonzero".into()));
        }
        if repeat == 0 {
            return Err(ProbeError("--repeat must be nonzero".into()));
        }
        Ok(Self {
            mode: Mode::parse(&mode)?,
            root,
            snapshot,
            operations,
            queries,
            repeat,
            oracle_enabled,
            diagnostics,
            worker_policy,
            scan,
        })
    }

    fn snapshot(&self) -> ProbeResult<&Path> {
        self.snapshot
            .as_deref()
            .ok_or_else(|| ProbeError("--snapshot is required for this mode".into()))
    }
}

fn next_path(arguments: &mut impl Iterator<Item = OsString>, flag: &str) -> ProbeResult<PathBuf> {
    arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| ProbeError(format!("{flag} requires a value")))
}

fn next_usize(arguments: &mut impl Iterator<Item = OsString>, flag: &str) -> ProbeResult<usize> {
    let value = arguments.next().ok_or_else(|| ProbeError(format!("{flag} requires a value")))?;
    let value = value.to_str().ok_or_else(|| ProbeError(format!("{flag} must be Unicode")))?;
    value.parse().map_err(|_| ProbeError(format!("{flag} must be a positive integer")))
}

/// Run the measured component `--repeat` times and report the last iteration.
///
/// This exists for sampling profilers, not for timing. A single scan of a large tree
/// finishes in a few hundred milliseconds, which at a 1 ms sampling interval is a few
/// hundred stacks — enough to see the top frame and not much else. Repeating the work
/// in one process buys resolution without changing what the code does.
///
/// Timing evidence never comes from a repeated run: by the second iteration the page
/// cache, the allocator, and the branch predictors are all warmer than any real
/// invocation would find them. The harness measures `--repeat 1` and profiles
/// separately, and the two are reported separately.
fn execute_repeated(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    for _ in 1..arguments.repeat {
        black_box(execute(arguments)?);
    }
    let mut output = execute(arguments)?;
    output.oracle_enabled = arguments.oracle_enabled;
    Ok(output)
}

fn execute(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    match arguments.mode {
        Mode::CodeSloc => content_analysis(arguments, code_request()),
        Mode::CodeSlocCacheHit => content_open(arguments, CachePolicy::Only, code_request()),
        Mode::CodeSlocSeed => {
            let mut output = content_open(arguments, CachePolicy::Auto, code_request())?;
            output.summary.complete = true;
            Ok(output)
        }
        Mode::ContentBasic | Mode::ContentBinaryGate => {
            content_analysis(arguments, basic_request())
        }
        Mode::ContentCacheHit => content_open(arguments, CachePolicy::Only, basic_request()),
        Mode::ContentDisabled => content_analysis(arguments, AnalysisRequest::default()),
        Mode::ContentOpen => content_open(arguments, CachePolicy::Auto, basic_request()),
        Mode::ContentQuery => content_query(arguments),
        Mode::ContentSeed => {
            let mut output = content_open(arguments, CachePolicy::Auto, basic_request())?;
            output.summary.complete = true;
            Ok(output)
        }
        Mode::DetectAmbiguous => classification_probe(arguments, true),
        Mode::DetectResolved => classification_probe(arguments, false),
        Mode::DocumentCacheHit => content_open(arguments, CachePolicy::Only, document_request()),
        Mode::DocumentSeed => {
            let mut output = content_open(arguments, CachePolicy::Auto, document_request())?;
            output.summary.complete = true;
            Ok(output)
        }
        Mode::ScanProducer => scan_producer(arguments),
        Mode::ScanIndex | Mode::ValidateIndex => scan_index(arguments),
        Mode::SnapshotSave => snapshot_save(arguments),
        Mode::ColdOpenSave => cold_open_save(arguments),
        Mode::DefaultTree => default_tree(arguments),
        Mode::SnapshotLoad => snapshot_load(arguments),
        Mode::Summary => summary_tier(arguments),
        Mode::Revalidate => revalidate(arguments),
        Mode::DeltaApply | Mode::DeltaApplyLarge => delta_apply(arguments),
        Mode::DeltaApplyBatched => delta_apply_batched(arguments),
        Mode::OpenedDiscovery => opened_discovery(arguments),
        Mode::MarkdownProse | Mode::TextProse => content_analysis(arguments, document_request()),
        Mode::Query => query(arguments),
    }
}

fn classification_probe(arguments: &Arguments, ambiguous: bool) -> ProbeResult<ProbeOutput> {
    let (index, scan) = fdu_core::scan::scan_into_index(&arguments.root, &arguments.scan)?;
    if !scan.is_complete() {
        return Err(ProbeError("classification probe setup scan was partial".into()));
    }
    let resolved = [
        "src/main.rs",
        "src/lib.py",
        "web/app.ts",
        "docs/guide.md",
        "data/config.json",
        "assets/photo.png",
        "archive.tar.zst",
        "Makefile",
    ];
    let ambiguous_cases = [
        ("include/value.h", b"namespace demo { constexpr int value = 1; }\n" as &[u8]),
        ("script.inc", b"# vim: set filetype=rust:\nfn main() {}\n"),
        ("manual.1", b".TH FDU 1\n.SH NAME\nfdu - disk usage\n"),
        ("document.unknown", b"<?xml version=\"1.0\"?><root/>"),
        ("download", b"%PDF-1.7\nfixture"),
        ("program", b"#!/usr/bin/env python3\nprint('ok')\n"),
    ];
    let started = Instant::now();
    let mut observed = 0_u64;
    for iteration in 0..arguments.operations {
        let classification = if ambiguous {
            let (path, prefix) = ambiguous_cases[iteration % ambiguous_cases.len()];
            fdu_core::classify::classify_path_with_prefix(Path::new(path), Some(prefix))
        } else {
            fdu_core::classify::classify_path(Path::new(resolved[iteration % resolved.len()]))
        };
        observed = observed.saturating_add(
            u64::try_from(classification.file_type.as_str().len()).unwrap_or(u64::MAX),
        );
        black_box(classification);
    }
    let component = started.elapsed();
    let mut summary = summarize_index(arguments, &index)?;
    summary.query_iterations = u64::try_from(arguments.operations).unwrap_or(u64::MAX);
    summary.query_observations = observed;
    Ok(ProbeOutput::new(arguments.mode, "synthetic", component, summary))
}

fn basic_request() -> AnalysisRequest {
    AnalysisRequest { profile: AnalysisSet::NONE.with_lines(), ..AnalysisRequest::default() }
}

fn code_request() -> AnalysisRequest {
    AnalysisRequest { profile: AnalysisSet::NONE.with_code(), ..AnalysisRequest::default() }
}

fn document_request() -> AnalysisRequest {
    AnalysisRequest { profile: AnalysisSet::NONE.with_words(), ..AnalysisRequest::default() }
}

fn content_analysis(arguments: &Arguments, request: AnalysisRequest) -> ProbeResult<ProbeOutput> {
    let (mut index, scan) = fdu_core::scan::scan_into_index(&arguments.root, &arguments.scan)?;
    if !scan.is_complete() {
        return Err(ProbeError("content-analysis setup scan was partial".into()));
    }
    let enabled = request.profile.is_enabled();
    let started = Instant::now();
    let report = fdu_core::content::analyze_index(&mut index, request);
    black_box(report);
    let component = started.elapsed();
    let mut summary = summarize_index(arguments, &index)?;
    attach_content_summary(&mut summary, &index);
    summary.content_candidates = report.candidates;
    summary.content_applied = report.applied;
    summary.complete = scan.is_complete() && (!enabled || report.is_complete());
    Ok(ProbeOutput::new(arguments.mode, "scan", component, summary))
}

fn content_open(
    arguments: &Arguments,
    policy: CachePolicy,
    analysis: AnalysisRequest,
) -> ProbeResult<ProbeOutput> {
    let snapshot = arguments.snapshot()?.to_path_buf();
    let config = OpenConfig {
        scan: arguments.scan.clone(),
        cache_path: Some(snapshot.clone()),
        policy,
        analysis,
    };
    let started = Instant::now();
    let (index, report) = fdu_core::open(&arguments.root, &config)?;
    let component = started.elapsed();
    let mut summary = summarize_index(arguments, &index)?;
    attach_content_summary(&mut summary, &index);
    summary.content_cache_hits = report.content_cache.hits;
    if let Some(analysis) = report.analysis {
        summary.content_candidates = analysis.candidates;
        summary.content_applied = analysis.applied;
    }
    summary.complete = report.is_complete();
    summary.errors = u64::try_from(report.scan.errors.len()).unwrap_or(u64::MAX);
    summary.snapshot_bytes = snapshot.metadata().ok().map(|metadata| metadata.len());
    let source = match report.path_taken {
        fdu_core::OpenPath::ColdScan => "cold-scan",
        fdu_core::OpenPath::WarmRevalidate => "warm-revalidate",
        fdu_core::OpenPath::CacheOnly => "content-cache",
    };
    Ok(ProbeOutput::new(arguments.mode, source, component, summary))
}

fn content_query(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let (mut index, scan) = fdu_core::scan::scan_into_index(&arguments.root, &arguments.scan)?;
    if !scan.is_complete() {
        return Err(ProbeError("content-query setup scan was partial".into()));
    }
    let analysis = fdu_core::content::analyze_index(&mut index, basic_request());
    let query = Query {
        views: vec![ViewSpec::Types, ViewSpec::Families, ViewSpec::Languages, ViewSpec::Documents],
        ..Query::default()
    };
    let provenance = Provenance {
        scan_started_at: None,
        generated_at: std::time::UNIX_EPOCH,
        source: ReportSource::ColdScan,
        complete: analysis.is_complete(),
        errors: Vec::new(),
    };
    let started = Instant::now();
    for _ in 0..arguments.queries {
        black_box(fdu_core::query::report(&index, &query, &provenance));
    }
    let component = started.elapsed();
    let mut summary = summarize_index(arguments, &index)?;
    attach_content_summary(&mut summary, &index);
    summary.content_candidates = analysis.candidates;
    summary.content_applied = analysis.applied;
    summary.query_iterations = u64::try_from(arguments.queries).unwrap_or(u64::MAX);
    summary.complete = analysis.is_complete();
    Ok(ProbeOutput::new(arguments.mode, "scan", component, summary))
}

fn scan_producer(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let mut summary = Summary::default();
    let started = Instant::now();
    let report = if arguments.diagnostics {
        let (report, diagnostics) = fdu_core::scan::scan_with_policy_diagnostics(
            &arguments.root,
            &arguments.scan,
            &mut |observation| summary.observe(&observation),
            arguments.worker_policy,
        )?;
        summary.scan_diagnostics = Some(diagnostics);
        report
    } else {
        fdu_core::scan::scan(&arguments.root, &arguments.scan, &mut |observation| {
            summary.observe(&observation);
        })?
    };
    let component = started.elapsed();
    summary.entries = summary.entries.saturating_add(1);
    summary.dirs = summary.dirs.saturating_add(1);
    summary.dirs_read = report.dirs_read;
    summary.attribution = Some(report.attribution);
    summary.errors = u64::try_from(report.errors.len()).unwrap_or(u64::MAX);
    summary.complete = report.is_complete();
    if summary.complete && arguments.oracle_enabled {
        // The exact oracle is deliberately outside the component timer. A producer
        // summary that is faster because it skipped, duplicated, or misclassified an
        // entry must never become an accepted performance sample.
        let (validation_index, validation_report) =
            fdu_core::scan::scan_into_index(&arguments.root, &arguments.scan)?;
        if !validation_report.is_complete() {
            return Err(ProbeError("scan-producer validation scan was partial".into()));
        }
        let validation = summarize_index(arguments, &validation_index)?;
        validate_producer_summary(&summary, &validation)?;
        summary.engine_digest = validation.engine_digest;
        summary.index_len = validation.index_len;
    }
    Ok(ProbeOutput::new(arguments.mode, "scan", component, summary))
}

fn validate_producer_summary(producer: &Summary, validation: &Summary) -> ProbeResult<()> {
    let matches = producer.entries == validation.entries
        && producer.files == validation.files
        && producer.dirs == validation.dirs
        && producer.symlinks == validation.symlinks
        && producer.other == validation.other
        && producer.apparent_bytes == validation.apparent_bytes
        && producer.allocated_bytes == validation.allocated_bytes
        && producer.newest_file_mtime_ns == validation.newest_file_mtime_ns;
    if !matches {
        return Err(ProbeError(
            "scan-producer compact summary disagreed with exact validation".into(),
        ));
    }
    Ok(())
}

fn scan_index(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let counters = begin_component_counters();
    let started = Instant::now();
    let (index, report, diagnostics) = if arguments.diagnostics {
        let (index, report, diagnostics) = fdu_core::scan::scan_into_index_with_policy_diagnostics(
            &arguments.root,
            &arguments.scan,
            arguments.worker_policy,
        )?;
        (index, report, Some(diagnostics))
    } else {
        let (index, report) = fdu_core::scan::scan_into_index(&arguments.root, &arguments.scan)?;
        (index, report, None)
    };
    let component = started.elapsed();
    let counters = finish_component_counters(counters.as_ref());
    let mut summary = summarize_index(arguments, &index)?;
    summary.counters = counters;
    summary.scan_diagnostics = diagnostics;
    summary.dirs_read = report.dirs_read;
    summary.attribution = Some(report.attribution);
    summary.errors = u64::try_from(report.errors.len()).unwrap_or(u64::MAX);
    summary.complete = report.is_complete();
    Ok(ProbeOutput::new(arguments.mode, "scan", component, summary))
}

/// The aggregate tier: five exact tallies, no retained index, no snapshot.
///
/// This is `fdu --view summary` -- the plan `plan_report` selects when a caller asks one
/// unfiltered question and keeps nothing. It is the tier closest to the machine floor
/// (1.20x on the primary synthetic subject, 1.59x on `/usr`) and it was the only tier
/// with no probe mode, so every number about it came from the command line and carried
/// process spawn, argument parsing, canonicalization and rendering. exp-043 and exp-044
/// both resolved on wall changes of +0.67% and -1.15% while user CPU fell 40% and 50%,
/// with no component timer available to tell dilution from a real effect.
///
/// The blocker recorded against this in `fdu-tyjx` was that the planner was
/// `pub(crate)`, so an example could not reach the tier at all. `fdu-z7sp` has since
/// exported `prepare_report` for an unrelated reason -- a library caller wanting one
/// report without retaining an index -- and that is the whole API this needs.
///
/// **The oracle is different here, and it has to be.** Every other mode returns
/// `engine_digest`, a multiset hash over every entry the index retained. This tier
/// retains nothing to hash. What it can prove is that its five tallies agree with an
/// independent walk, which is the same check the tool comparison already makes against
/// third-party walkers; `Job.oracle` selects it. A tier that reported no oracle at all
/// would be a tier whose speed nobody could trust.
fn summary_tier(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    // `cache_path: None` with `CachePolicy::Off` is what keeps the planner on the
    // transient tier: `Refresh` would demand an index to write, and `Only` would demand
    // a snapshot to read. Off is also what the measured invocation uses.
    let config = OpenConfig {
        scan: arguments.scan.clone(),
        cache_path: None,
        policy: CachePolicy::Off,
        analysis: AnalysisRequest::default(),
    };
    let query = Query { views: vec![ViewSpec::Summary], ..Query::default() };

    let started = Instant::now();
    // `_performance` is what the command line prints in its footer; this tier's tallies
    // come out of the report itself, so it is deliberately unused here.
    let (report, pending, _performance) =
        fdu_core::prepare_report(&arguments.root, &config, &query)?;
    let component = started.elapsed();
    // Nothing to join on this tier -- it writes no cache -- but joining is what the
    // command line does, and a mode that skipped it would stop measuring the same thing
    // the moment the tier ever gained a write.
    pending.join()?;

    let row = report
        .sections
        .iter()
        .find_map(|section| match section {
            fdu_core::query::Section::Summary(row) => Some(*row),
            _ => None,
        })
        .ok_or_else(|| ProbeError("summary plan returned no summary section".to_string()))?;

    let mut summary = Summary {
        files: row.files,
        dirs: row.dirs,
        apparent_bytes: u128::from(row.bytes),
        allocated_bytes: u128::from(row.allocated),
        newest_file_mtime_ns: row.newest_mtime_ns,
        // Deliberately absent rather than zero: this tier never observed an index, and a
        // zero digest would read as "no entries" to an oracle rather than "not offered".
        engine_digest: None,
        index_len: None,
        ..Summary::default()
    };
    summary.errors = u64::try_from(report.errors.len()).unwrap_or(u64::MAX);
    summary.complete = report.complete;
    // The walk counted more than the summary reports -- symlinks and other kinds are
    // observed and then deliberately not tallied -- so entries is the honest total of
    // what this tier can speak for, not of what it touched.
    summary.entries = row.files + row.dirs;
    Ok(ProbeOutput::new(arguments.mode, "scan", component, summary))
}

/// The default command, `fdu <dir>`: scan, index, rendered tree, snapshot write.
///
/// This is the path a user takes by typing nothing else, and no ledger job measured it
/// before this mode existed: `cold-scan-index`, the proxy every cumulative checkpoint
/// used, is the walk plus the index build and excludes both the render and the write,
/// which the cache-layers plan priced at about a third of a default run. Two defects found
/// in the PR #38 review live in exactly that blind spot (`fdu-2um8`, `fdu-n75m`).
///
/// Faithful to the command line rather than to the cheapest probe-able shape: cache
/// policy `Auto` with a real cache path, the tree view at its default depth, the text
/// renderer run to completion, and the save joined before returning -- which is what the
/// command line does before it exits. `prepare_report` never reads the snapshot for a
/// metadata query (revalidation would stat every entry anyway), so a repeated run over an
/// unchanged tree scans cold and writes again; `snapshot_written` says whether it did.
///
/// The oracle is `tallies`, read off the tree's root node, because the index is consumed
/// inside `prepare_report` and never returned -- the same reason the aggregate tier has no
/// digest. The five numbers the user sees at the top of the tree are what is checked.
fn default_tree(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let snapshot = arguments.snapshot()?.to_path_buf();
    let identity_before = snapshot_identity(&snapshot);
    let config = OpenConfig {
        scan: arguments.scan.clone(),
        cache_path: Some(snapshot.clone()),
        policy: CachePolicy::Auto,
        analysis: AnalysisRequest::default(),
    };
    let query = Query { views: vec![ViewSpec::Tree], ..Query::default() };

    let counters = begin_component_counters();
    let started = Instant::now();
    let (report, pending, _performance, scan_diagnostics) = if arguments.diagnostics {
        fdu_core::prepare_report_with_scan_diagnostics(&arguments.root, &config, &query)?
    } else {
        let (report, pending, performance) =
            fdu_core::prepare_report(&arguments.root, &config, &query)?;
        (report, pending, performance, None)
    };
    // Rendered to a string the way the command line renders into its writer; the bytes
    // are not printed because stdout carries this probe's JSON, and `black_box` keeps the
    // render from being optimised away as an unused value.
    let rendered =
        fdu_core::report_format::render(&report, fdu_core::report_format::Format::Text, false);
    black_box(rendered.len());
    pending.join()?;
    let component = started.elapsed();
    let counters = finish_component_counters(counters.as_ref());

    let root = report
        .sections
        .iter()
        .find_map(|section| match section {
            fdu_core::query::Section::Tree(node) => Some(node),
            _ => None,
        })
        .ok_or_else(|| ProbeError("default tree returned no tree section".to_string()))?;
    let identity_after = snapshot_identity(&snapshot);

    let mut summary = Summary {
        files: root.files,
        dirs: root.dirs,
        apparent_bytes: u128::from(root.bytes),
        allocated_bytes: u128::from(root.allocated),
        newest_file_mtime_ns: root.newest_mtime_ns,
        // Absent rather than zero, as for the aggregate tier: the index was consumed
        // inside `prepare_report`, so this mode has no digest to offer and says so.
        engine_digest: None,
        index_len: None,
        ..Summary::default()
    };
    summary.errors = u64::try_from(report.errors.len()).unwrap_or(u64::MAX);
    summary.counters = counters;
    summary.scan_diagnostics = scan_diagnostics;
    summary.complete = report.complete;
    summary.entries = root.files + root.dirs;
    summary.snapshot_bytes = snapshot.metadata().ok().map(|metadata| metadata.len());
    // A rewrite lands a fresh temporary and renames it over the path, so the file's
    // identity changes; a run that found the bytes identical leaves the file in place
    // and only moves its mtime. Identity, not mtime, is therefore the signal, and it is
    // the number a reader of this job wants beside the wall time.
    summary.snapshot_written = Some(identity_after.is_some() && identity_after != identity_before);
    Ok(ProbeOutput::new(arguments.mode, "scan", component, summary))
}

/// Something that changes when a file is replaced and survives when it is left alone.
///
/// The inode where the platform has one; elsewhere the creation time, which a rename of
/// a fresh temporary also moves. `None` when there is no file.
fn snapshot_identity(path: &Path) -> Option<(u64, Option<std::time::SystemTime>)> {
    let metadata = path.metadata().ok()?;
    #[cfg(unix)]
    let inode = {
        use std::os::unix::fs::MetadataExt;
        metadata.ino()
    };
    #[cfg(not(unix))]
    let inode = 0u64;
    Some((inode, metadata.created().ok()))
}

/// A cold scan that also writes its cache, through the real `open` path.
///
/// `snapshot-save` calls `snapshot::save` directly, so it never exercises what a
/// cache-writing run actually costs: `spawn_save`'s hand-off to the writer thread and
/// the join that a one-shot caller performs before exiting. That is the shape of
/// `fdu --cache refresh`, the default first run against a tree, and it was
/// unmeasurable under the accept rule until this job existed.
fn cold_open_save(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let snapshot = arguments.snapshot()?.to_path_buf();
    let config = OpenConfig {
        scan: arguments.scan.clone(),
        cache_path: Some(snapshot.clone()),
        // Refresh rather than Auto: the job must always walk and always write, or a
        // stray snapshot would silently turn one trial into a warm open.
        policy: CachePolicy::Refresh,
        analysis: AnalysisRequest::default(),
    };
    let started = Instant::now();
    let (index, report, pending) = fdu_core::open_with_pending_save(&arguments.root, &config)?;
    pending.join()?;
    let component = started.elapsed();
    if !report.is_complete() {
        return Err(ProbeError("cold-open-save scan was partial".into()));
    }
    let mut summary = summarize_index(arguments, &index)?;
    summary.complete = report.is_complete();
    summary.errors = u64::try_from(report.scan.errors.len()).unwrap_or(u64::MAX);
    summary.dirs_read = report.scan.dirs_read;
    summary.snapshot_bytes = snapshot.metadata().ok().map(|metadata| metadata.len());
    Ok(ProbeOutput::new(arguments.mode, "scan", component, summary))
}

fn snapshot_save(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let (index, report) = fdu_core::scan::scan_into_index(&arguments.root, &arguments.scan)?;
    if !report.is_complete() {
        return Err(ProbeError("snapshot setup scan was partial".into()));
    }
    let snapshot = arguments.snapshot()?;
    let started = Instant::now();
    fdu_core::snapshot::save(&index, snapshot)?;
    let component = started.elapsed();
    let mut summary = summarize_index(arguments, &index)?;
    summary.snapshot_bytes = Some(snapshot.metadata()?.len());
    Ok(ProbeOutput::new(arguments.mode, "scan", component, summary))
}

fn snapshot_load(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let snapshot = arguments.snapshot()?;
    let started = Instant::now();
    let index = load_snapshot(snapshot)?;
    let component = started.elapsed();
    let mut summary = summarize_index(arguments, &index)?;
    summary.snapshot_bytes = Some(snapshot.metadata()?.len());
    Ok(ProbeOutput::new(arguments.mode, "snapshot", component, summary))
}

fn revalidate(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let snapshot = arguments.snapshot()?;
    let mut index = load_snapshot(snapshot)?;
    let started = Instant::now();
    let report = fdu_core::scan::reconcile(&mut index, &arguments.scan, &mut |_| {})?;
    let component = started.elapsed();
    let mut summary = summarize_index(arguments, &index)?;
    summary.dirs_read = report.scan.dirs_read;
    // Deliberately left None: neither reconciliation path has complete attribution yet
    // (tracked in fdu-78wr). Null says "not instrumented"; zeros would lie.
    summary.errors = u64::try_from(report.scan.errors.len()).unwrap_or(u64::MAX);
    summary.complete = report.is_complete();
    summary.apply = ApplySummary {
        inserted: report.apply.inserted,
        invalidated: report.apply.invalidated,
        removed: report.apply.removed,
        stale: report.apply.stale,
        unchanged: report.apply.unchanged,
        updated: report.apply.updated,
    };
    summary.snapshot_bytes = Some(snapshot.metadata()?.len());
    Ok(ProbeOutput::new(arguments.mode, "snapshot", component, summary))
}

fn delta_apply(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let observation = Observation::new(synthetic_operations(arguments.operations));
    let mut index = Index::new(&arguments.root);
    let counters = begin_component_counters();
    let started = Instant::now();
    let outcome = index.apply(&observation)?;
    let component = started.elapsed();
    let counters = finish_component_counters(counters.as_ref());
    let mut summary = summarize_index(arguments, &index)?;
    summary.counters = counters;
    summary.apply.add(outcome.stats);
    summary.commit = Some(summarize_commits(&outcome.commit.into_iter().collect::<Vec<_>>()));
    Ok(ProbeOutput::new(arguments.mode, "synthetic", component, summary))
}

fn delta_apply_batched(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let observations: Vec<_> = synthetic_operations(arguments.operations)
        .chunks(arguments.scan.batch_size)
        .map(|operations| Observation::new(operations.to_vec()))
        .collect();
    let mut index = Index::new(&arguments.root);
    let mut apply = ApplySummary::default();
    let mut commits = Vec::with_capacity(observations.len());
    let counters = begin_component_counters();
    let started = Instant::now();
    for observation in &observations {
        let outcome = index.apply(observation)?;
        apply.add(outcome.stats);
        if let Some(commit) = outcome.commit {
            commits.push(commit);
        }
    }
    let component = started.elapsed();
    let counters = finish_component_counters(counters.as_ref());
    let mut summary = summarize_index(arguments, &index)?;
    summary.counters = counters;
    summary.apply = apply;
    summary.commit = Some(summarize_commits(&commits));
    Ok(ProbeOutput::new(arguments.mode, "synthetic", component, summary))
}

fn synthetic_operations(operations: usize) -> Vec<Op> {
    let mut generated = Vec::with_capacity(operations.saturating_add(1));
    generated.push(Op::Upsert {
        path: PathBuf::from("synthetic"),
        kind: EntryKind::Dir,
        attrs: Attrs::default(),
    });
    generated.extend((0..operations).map(|index| Op::Upsert {
        path: PathBuf::from(format!("synthetic/entry-{index:09}.dat")),
        kind: EntryKind::File,
        attrs: Attrs {
            size: u64::try_from(index % 4096).unwrap_or(0),
            allocated: 4096,
            mtime_ns: i64::try_from(index).unwrap_or(i64::MAX),
            ctime_ns: i64::try_from(index).unwrap_or(i64::MAX),
            inode: u64::try_from(index).unwrap_or(u64::MAX),
            dev: 1,
        },
    }));
    generated
}

fn opened_discovery(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let options = OpenOptions {
        batch_size: arguments.scan.batch_size,
        follow_symlinks: arguments.scan.follow_symlinks,
        one_filesystem: arguments.scan.one_filesystem,
        hidden: arguments.scan.hidden.clone(),
        exclude_special: arguments.scan.exclude_special,
        types: arguments.scan.types.clone(),
        journal_capacity: OPENED_PROBE_JOURNAL_CAPACITY,
        ..OpenOptions::default()
    };
    let counters = begin_component_counters();
    let started = Instant::now();
    let opened = OpenedIndex::open(&arguments.root, options)?;
    let initial = opened.read(ReadRequest::default())?;
    let mut cursor = fdu_core::EngineVersion { sequence: Clock::ZERO, ..initial.version };
    let mut commits = Vec::new();
    let terminal = loop {
        let poll =
            opened.changes(ChangeRequest { after: cursor, timeout: Duration::from_secs(30) })?;
        cursor = poll.cursor;
        match poll.outcome {
            ChangeOutcome::Changes { commits: next, .. } => commits.extend(next),
            ChangeOutcome::Idle => {
                return Err(ProbeError("opened discovery did not settle before timeout".into()));
            }
            ChangeOutcome::Reset { .. } => {
                return Err(ProbeError(
                    "opened discovery outran the probe's exact journal capacity".into(),
                ));
            }
        }
        if matches!(
            poll.state.phase,
            LifecyclePhase::Ready
                | LifecyclePhase::Watching
                | LifecyclePhase::Stopped
                | LifecyclePhase::Failed
        ) {
            break poll.state;
        }
    };
    opened.close()?;
    let component = started.elapsed();
    let counters = finish_component_counters(counters.as_ref());

    // The independent exact tree oracle is deliberately outside the component timer.
    // The timed opened path is held to both this final digest and the exact public
    // commit sequence it returned while discovery progressed.
    let (mut summary, validation_complete) = if arguments.oracle_enabled {
        let validation_scan = ScanConfig {
            max_depth: None,
            threads: Some(1),
            order: ScanOrder::BreadthFirst,
            read_controls: true,
            ..arguments.scan.clone()
        };
        let (validation, report) =
            fdu_core::scan::scan_into_index(&arguments.root, &validation_scan)?;
        (summarize_index(arguments, &validation)?, report.is_complete())
    } else {
        (Summary::default(), true)
    };
    summary.counters = counters;
    summary.dirs_read = terminal.progress.directories_complete;
    summary.errors = terminal.issues.retained.saturating_add(terminal.issues.omitted);
    summary.complete = terminal.coverage == Coverage::Complete && validation_complete;
    for commit in &commits {
        for change in &commit.changes {
            match change {
                EffectiveChange::Inserted { .. } => {
                    summary.apply.inserted = summary.apply.inserted.saturating_add(1);
                }
                EffectiveChange::Updated { .. } => {
                    summary.apply.updated = summary.apply.updated.saturating_add(1);
                }
                EffectiveChange::Removed { .. } => {
                    summary.apply.removed = summary.apply.removed.saturating_add(1);
                }
                EffectiveChange::Invalidated { .. } => {
                    summary.apply.invalidated = summary.apply.invalidated.saturating_add(1);
                }
                EffectiveChange::ControlUpdated { .. } | EffectiveChange::Reclassified { .. } => {}
            }
        }
    }
    summary.commit = Some(summarize_commits(&commits));
    Ok(ProbeOutput::new(arguments.mode, "opened", component, summary))
}

fn summarize_commits(commits: &[Commit]) -> CommitSummary {
    let mut digest = Sha256::new();
    digest.update(COMMIT_DIGEST_ALGORITHM.as_bytes());
    digest.update(&[0]);
    let mut summary = CommitSummary::default();
    for commit in commits {
        let record = format!("{commit:?}");
        digest.update(&u64::try_from(record.len()).unwrap_or(u64::MAX).to_be_bytes());
        digest.update(record.as_bytes());
        summary.commits = summary.commits.saturating_add(1);
        summary.changes =
            summary.changes.saturating_add(u64::try_from(commit.changes.len()).unwrap_or(u64::MAX));
        summary.state_transitions = summary
            .state_transitions
            .saturating_add(u64::try_from(commit.state.len()).unwrap_or(u64::MAX));
        summary.observations = summary.observations.saturating_add(commit.work.observations);
        summary.dirty_paths = summary
            .dirty_paths
            .saturating_add(u64::try_from(commit.impact.dirty_paths.len()).unwrap_or(u64::MAX));
        summary.all_dirty_commits =
            summary.all_dirty_commits.saturating_add(u64::from(commit.impact.all_dirty));
        summary.first_clock.get_or_insert(commit.clock.0);
        summary.last_clock = Some(commit.clock.0);
    }
    summary.digest = hex(&digest.finalize());
    summary
}

fn query(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let index = if let Some(snapshot) = arguments.snapshot.as_deref() {
        load_snapshot(snapshot)?
    } else {
        let (index, report) = fdu_core::scan::scan_into_index(&arguments.root, &arguments.scan)?;
        if !report.is_complete() {
            return Err(ProbeError("query setup scan was partial".into()));
        }
        index
    };
    let started = Instant::now();
    let mut observed = 0_u64;
    for _ in 0..arguments.queries {
        let child_count = index.children(Path::new("")).map_or(0, |children| children.len());
        observed = observed.saturating_add(u64::try_from(child_count).unwrap_or(u64::MAX));
        black_box(index.total());
    }
    black_box(observed);
    let component = started.elapsed();
    let mut summary = summarize_index(arguments, &index)?;
    summary.query_iterations = u64::try_from(arguments.queries).unwrap_or(u64::MAX);
    summary.query_observations = observed;
    Ok(ProbeOutput::new(
        arguments.mode,
        if arguments.snapshot.is_some() { "snapshot" } else { "scan" },
        component,
        summary,
    ))
}

fn load_snapshot(path: &Path) -> ProbeResult<Index> {
    fdu_core::snapshot::load(path)?.ok_or_else(|| ProbeError("snapshot was not usable".into()))
}

#[derive(Debug, Default)]
struct ApplySummary {
    inserted: u64,
    updated: u64,
    removed: u64,
    unchanged: u64,
    invalidated: u64,
    stale: u64,
}

impl ApplySummary {
    fn add(&mut self, stats: fdu_core::ApplyStats) {
        self.inserted = self.inserted.saturating_add(stats.inserted);
        self.updated = self.updated.saturating_add(stats.updated);
        self.removed = self.removed.saturating_add(stats.removed);
        self.unchanged = self.unchanged.saturating_add(stats.unchanged);
        self.invalidated = self.invalidated.saturating_add(stats.invalidated);
        self.stale = self.stale.saturating_add(stats.stale);
    }
}

#[derive(Debug, Default)]
struct CommitSummary {
    commits: u64,
    changes: u64,
    state_transitions: u64,
    observations: u64,
    dirty_paths: u64,
    all_dirty_commits: u64,
    first_clock: Option<u64>,
    last_clock: Option<u64>,
    digest: String,
}

#[derive(Debug)]
struct CounterSummary {
    allocs: u64,
    reallocs: u64,
    frees: u64,
    bytes_allocated: u64,
    baseline_batches: u64,
    baseline_accepted_ops: u64,
    opened_batches: u64,
    opened_accepted_ops: u64,
    public_batches: u64,
    public_accepted_ops: u64,
    ancestry_overlay_inserts: u64,
    ancestry_path_comparisons: u64,
    ancestry_parent_proofs: u64,
    effect_paths: u64,
    effect_path_bytes: u64,
    impact_candidates: u64,
    impact_ancestor_visits: u64,
    impact_retained_dirty_paths: u64,
    impact_all_dirty: u64,
    journal_retained_commits: u64,
    journal_cloned_commits: u64,
    journal_oversized_commits: u64,
    journal_dropped_commits: u64,
}

impl CounterSummary {
    fn since(before: &fdu_core::counters::Counts) -> Self {
        let after = fdu_core::counters::snapshot();
        macro_rules! delta {
            ($field:ident) => {
                after.$field.saturating_sub(before.$field)
            };
        }
        Self {
            allocs: delta!(allocs),
            reallocs: delta!(reallocs),
            frees: delta!(frees),
            bytes_allocated: delta!(bytes_allocated),
            baseline_batches: delta!(baseline_batches),
            baseline_accepted_ops: delta!(baseline_accepted_ops),
            opened_batches: delta!(opened_batches),
            opened_accepted_ops: delta!(opened_accepted_ops),
            public_batches: delta!(public_batches),
            public_accepted_ops: delta!(public_accepted_ops),
            ancestry_overlay_inserts: delta!(ancestry_overlay_inserts),
            ancestry_path_comparisons: delta!(ancestry_path_comparisons),
            ancestry_parent_proofs: delta!(ancestry_parent_proofs),
            effect_paths: delta!(effect_paths),
            effect_path_bytes: delta!(effect_path_bytes),
            impact_candidates: delta!(impact_candidates),
            impact_ancestor_visits: delta!(impact_ancestor_visits),
            impact_retained_dirty_paths: delta!(impact_retained_dirty_paths),
            impact_all_dirty: delta!(impact_all_dirty),
            journal_retained_commits: delta!(journal_retained_commits),
            journal_cloned_commits: delta!(journal_cloned_commits),
            journal_oversized_commits: delta!(journal_oversized_commits),
            journal_dropped_commits: delta!(journal_dropped_commits),
        }
    }
}

fn begin_component_counters() -> Option<fdu_core::counters::Counts> {
    fdu_core::counters::enabled().then(fdu_core::counters::snapshot)
}

fn finish_component_counters(
    before: Option<&fdu_core::counters::Counts>,
) -> Option<CounterSummary> {
    before.map(CounterSummary::since)
}

#[derive(Debug)]
struct Summary {
    allocated_bytes: u128,
    attribution: Option<fdu_core::scan::WalkAttribution>,
    apparent_bytes: u128,
    apply: ApplySummary,
    complete: bool,
    content_analyzed: u64,
    content_applied: u64,
    content_binary: u64,
    content_cache_hits: u64,
    content_candidates: u64,
    content_digest: Option<String>,
    content_invalid_utf8: u64,
    content_records: u64,
    commit: Option<CommitSummary>,
    counters: Option<CounterSummary>,
    dirs: u64,
    dirs_read: u64,
    engine_digest: Option<String>,
    entries: u64,
    errors: u64,
    files: u64,
    index_len: Option<u64>,
    newest_file_mtime_ns: Option<i64>,
    other: u64,
    query_iterations: u64,
    query_observations: u64,
    scan_diagnostics: Option<fdu_core::scan::ScanDiagnostics>,
    snapshot_bytes: Option<u64>,
    /// Whether the run replaced the snapshot file; `None` for modes that have no snapshot.
    snapshot_written: Option<bool>,
    symlinks: u64,
}

impl Default for Summary {
    fn default() -> Self {
        Self {
            allocated_bytes: 0,
            attribution: None,
            apparent_bytes: 0,
            apply: ApplySummary::default(),
            complete: true,
            content_analyzed: 0,
            content_applied: 0,
            content_binary: 0,
            content_cache_hits: 0,
            content_candidates: 0,
            content_digest: None,
            content_invalid_utf8: 0,
            content_records: 0,
            commit: None,
            counters: None,
            dirs: 0,
            dirs_read: 0,
            engine_digest: None,
            entries: 0,
            errors: 0,
            files: 0,
            index_len: None,
            newest_file_mtime_ns: None,
            other: 0,
            query_iterations: 0,
            query_observations: 0,
            scan_diagnostics: None,
            snapshot_bytes: None,
            snapshot_written: None,
            symlinks: 0,
        }
    }
}

fn attach_content_summary(summary: &mut Summary, index: &Index) {
    let Some(content) = index.content() else {
        summary.content_digest = Some(hex(&Sha256::digest(b"fdu-content-summary-v1\0disabled")));
        return;
    };
    let Some(root) = content.rollup(Path::new("")) else {
        summary.content_digest = Some(hex(&Sha256::digest(b"fdu-content-summary-v1\0empty")));
        return;
    };
    summary.content_records = root.total.files;
    summary.content_analyzed = root.total.analyzed_files;
    summary.content_binary = root.coverage.get(&CoverageReason::Binary).copied().unwrap_or(0);
    summary.content_invalid_utf8 =
        root.coverage.get(&CoverageReason::InvalidUtf8).copied().unwrap_or(0);
    let metrics = root.total.metrics;
    let record = format!(
        concat!(
            "fdu-content-summary-v1\0records={}\0analyzed={}\0binary={}\0invalid_utf8={}\0",
            "physical={}\0blank={}\0nonblank={}\0code={}\0comment={}\0",
            "code_blank={}\0raw_words={}\0logical_words={}\0paragraphs={}\0visible_words={}\0",
            "visible_logical_words={}"
        ),
        summary.content_records,
        summary.content_analyzed,
        summary.content_binary,
        summary.content_invalid_utf8,
        metrics.physical_lines,
        metrics.blank_lines,
        metrics.nonblank_lines,
        metrics.code_lines,
        metrics.comment_lines,
        metrics.code_blank_lines,
        metrics.raw_words,
        metrics.logical_word_stats.logical_words(),
        metrics.paragraphs,
        metrics.visible_words,
        metrics.visible_logical_word_stats.logical_words(),
    );
    summary.content_digest = Some(hex(&Sha256::digest(record.as_bytes())));
}

impl Summary {
    fn observe(&mut self, observation: &Observation) {
        for observed in &observation.ops {
            let Op::Upsert { kind, attrs, .. } = &observed.op else {
                continue;
            };
            self.entries = self.entries.saturating_add(1);
            match kind {
                EntryKind::File => {
                    self.files = self.files.saturating_add(1);
                    self.apparent_bytes = self.apparent_bytes.saturating_add(attrs.size.into());
                    self.allocated_bytes =
                        self.allocated_bytes.saturating_add(attrs.allocated.into());
                    self.newest_file_mtime_ns = Some(
                        self.newest_file_mtime_ns
                            .map_or(attrs.mtime_ns, |newest| newest.max(attrs.mtime_ns)),
                    );
                }
                EntryKind::Dir => self.dirs = self.dirs.saturating_add(1),
                EntryKind::Symlink => self.symlinks = self.symlinks.saturating_add(1),
                EntryKind::Other => self.other = self.other.saturating_add(1),
            }
        }
    }
}

fn summarize_index(arguments: &Arguments, index: &Index) -> ProbeResult<Summary> {
    if !arguments.oracle_enabled {
        let total = index.total();
        let entries = index.len();
        let dirs = total.dirs.saturating_add(1);
        return Ok(Summary {
            allocated_bytes: total.allocated.into(),
            apparent_bytes: total.bytes.into(),
            dirs,
            entries,
            files: total.files,
            index_len: Some(entries),
            newest_file_mtime_ns: (total.files > 0).then_some(total.newest_mtime_ns),
            other: entries.saturating_sub(total.files).saturating_sub(dirs),
            ..Summary::default()
        });
    }
    let mut summary = Summary::default();
    let mut digest = MultisetDigest::default();
    let mut stack = vec![EntryId::ROOT];
    while let Some(id) = stack.pop() {
        let kind = index
            .kind_of(id)
            .ok_or_else(|| ProbeError("index contained a stale kind handle".into()))?;
        let attrs = index
            .attrs_of(id)
            .ok_or_else(|| ProbeError("index contained a stale attribute handle".into()))?;
        let relative = index
            .path_of(id)
            .ok_or_else(|| ProbeError("index contained a stale path handle".into()))?;
        let normalized = normalized_path(&relative)?;
        digest.add(&engine_record(&normalized, kind, attrs)?);
        summary.entries = summary.entries.saturating_add(1);
        match kind {
            EntryKind::File => summary.files = summary.files.saturating_add(1),
            EntryKind::Dir => summary.dirs = summary.dirs.saturating_add(1),
            EntryKind::Symlink => summary.symlinks = summary.symlinks.saturating_add(1),
            EntryKind::Other => summary.other = summary.other.saturating_add(1),
        }
        let children = index
            .children_of(id)
            .ok_or_else(|| ProbeError("index contained a stale child handle".into()))?;
        stack.extend(children.map(|(_, child)| child));
    }
    let total = index.total();
    summary.apparent_bytes = total.bytes.into();
    summary.allocated_bytes = total.allocated.into();
    summary.newest_file_mtime_ns = (total.files > 0).then_some(total.newest_mtime_ns);
    summary.index_len = Some(index.len());
    summary.engine_digest = Some(digest.finish());
    Ok(summary)
}

fn normalized_path(path: &Path) -> ProbeResult<String> {
    if path.as_os_str().is_empty() {
        return Ok(".".into());
    }
    let mut normalized = String::new();
    for component in path.components() {
        let Component::Normal(name) = component else {
            return Err(ProbeError("index path was not relative and normalized".into()));
        };
        let name = name
            .to_str()
            .ok_or_else(|| ProbeError("benchmark corpus path was not Unicode".into()))?;
        if !normalized.is_empty() {
            normalized.push('/');
        }
        normalized.push_str(name);
    }
    Ok(normalized)
}

fn engine_record(path: &str, kind: EntryKind, attrs: &Attrs) -> ProbeResult<Vec<u8>> {
    let path_len = u32::try_from(path.len())
        .map_err(|_| ProbeError("engine digest path exceeds u32".into()))?;
    let mut record = Vec::with_capacity(path.len().saturating_add(53));
    record.extend_from_slice(&path_len.to_be_bytes());
    record.extend_from_slice(path.as_bytes());
    record.push(kind as u8);
    record.extend_from_slice(&attrs.size.to_be_bytes());
    record.extend_from_slice(&attrs.allocated.to_be_bytes());
    record.extend_from_slice(&attrs.mtime_ns.to_be_bytes());
    record.extend_from_slice(&attrs.ctime_ns.to_be_bytes());
    record.extend_from_slice(&attrs.inode.to_be_bytes());
    record.extend_from_slice(&attrs.dev.to_be_bytes());
    Ok(record)
}

struct ProbeOutput {
    component_ns: u128,
    mode: Mode,
    oracle_enabled: bool,
    source: &'static str,
    summary: Summary,
}

impl ProbeOutput {
    fn new(mode: Mode, source: &'static str, component: Duration, summary: Summary) -> Self {
        Self { component_ns: component.as_nanos(), mode, oracle_enabled: true, source, summary }
    }

    fn render(&self) -> String {
        let summary = &self.summary;
        let digest = json_optional_string(summary.engine_digest.as_deref());
        let content_digest = json_optional_string(summary.content_digest.as_deref());
        let commit = json_commit_summary(summary.commit.as_ref());
        let counters = json_counter_summary(summary.counters.as_ref());
        let index_len = json_optional_u64(summary.index_len);
        let newest_file_mtime_ns = json_optional_i64(summary.newest_file_mtime_ns);
        let snapshot_bytes = json_optional_u64(summary.snapshot_bytes);
        let snapshot_written = summary
            .snapshot_written
            .map_or_else(|| "null".to_string(), |written| written.to_string());
        format!(
            concat!(
                "{{\"component_ns\":{},\"attribution\":{},\"mode\":\"{}\",",
                "\"oracle_enabled\":{},\"schema\":\"{}\",",
                "\"scan_diagnostics\":{},\"source\":\"{}\",\"summary\":{{",
                "\"allocated_bytes\":{},\"apparent_bytes\":{},",
                "\"apply\":{{\"inserted\":{},\"invalidated\":{},",
                "\"removed\":{},\"stale\":{},\"unchanged\":{},\"updated\":{}}},",
                "\"complete\":{},\"dirs\":{},\"dirs_read\":{},",
                "\"content\":{{\"analyzed\":{},\"applied\":{},\"binary\":{},",
                "\"cache_hits\":{},\"candidates\":{},\"digest\":{},",
                "\"invalid_utf8\":{},\"records\":{}}},",
                "\"commit\":{},",
                "\"counters\":{},",
                "\"engine_digest\":{},\"entries\":{},\"errors\":{},",
                "\"files\":{},\"index_len\":{},\"newest_file_mtime_ns\":{},\"other\":{},",
                "\"query_iterations\":{},\"query_observations\":{},",
                "\"snapshot_bytes\":{},\"snapshot_written\":{},",
                "\"symlinks\":{}}}}}"
            ),
            self.component_ns,
            json_attribution(self.summary.attribution.as_ref()),
            self.mode.name(),
            self.oracle_enabled,
            PROBE_SCHEMA,
            json_scan_diagnostics(self.summary.scan_diagnostics.as_ref()),
            self.source,
            summary.allocated_bytes,
            summary.apparent_bytes,
            summary.apply.inserted,
            summary.apply.invalidated,
            summary.apply.removed,
            summary.apply.stale,
            summary.apply.unchanged,
            summary.apply.updated,
            summary.complete,
            summary.dirs,
            summary.dirs_read,
            summary.content_analyzed,
            summary.content_applied,
            summary.content_binary,
            summary.content_cache_hits,
            summary.content_candidates,
            content_digest,
            summary.content_invalid_utf8,
            summary.content_records,
            commit,
            counters,
            digest,
            summary.entries,
            summary.errors,
            summary.files,
            index_len,
            newest_file_mtime_ns,
            summary.other,
            summary.query_iterations,
            summary.query_observations,
            snapshot_bytes,
            snapshot_written,
            summary.symlinks,
        )
    }
}

fn json_commit_summary(summary: Option<&CommitSummary>) -> String {
    let Some(summary) = summary else {
        return "null".into();
    };
    format!(
        concat!(
            "{{\"algorithm\":\"{}\",\"all_dirty_commits\":{},",
            "\"changes\":{},\"commits\":{},\"digest\":\"{}\",",
            "\"dirty_paths\":{},\"first_clock\":{},\"last_clock\":{},",
            "\"observations\":{},\"state_transitions\":{}}}"
        ),
        COMMIT_DIGEST_ALGORITHM,
        summary.all_dirty_commits,
        summary.changes,
        summary.commits,
        summary.digest,
        summary.dirty_paths,
        json_optional_u64(summary.first_clock),
        json_optional_u64(summary.last_clock),
        summary.observations,
        summary.state_transitions,
    )
}

fn json_counter_summary(summary: Option<&CounterSummary>) -> String {
    let Some(summary) = summary else {
        return "null".into();
    };
    format!(
        concat!(
            "{{\"allocs\":{},\"reallocs\":{},\"frees\":{},\"bytes_allocated\":{},",
            "\"baseline_batches\":{},\"baseline_accepted_ops\":{},",
            "\"opened_batches\":{},\"opened_accepted_ops\":{},",
            "\"public_batches\":{},\"public_accepted_ops\":{},",
            "\"ancestry_overlay_inserts\":{},\"ancestry_path_comparisons\":{},",
            "\"ancestry_parent_proofs\":{},\"effect_paths\":{},",
            "\"effect_path_bytes\":{},\"impact_candidates\":{},",
            "\"impact_ancestor_visits\":{},\"impact_retained_dirty_paths\":{},",
            "\"impact_all_dirty\":{},",
            "\"journal_retained_commits\":{},\"journal_cloned_commits\":{},",
            "\"journal_oversized_commits\":{},\"journal_dropped_commits\":{}}}"
        ),
        summary.allocs,
        summary.reallocs,
        summary.frees,
        summary.bytes_allocated,
        summary.baseline_batches,
        summary.baseline_accepted_ops,
        summary.opened_batches,
        summary.opened_accepted_ops,
        summary.public_batches,
        summary.public_accepted_ops,
        summary.ancestry_overlay_inserts,
        summary.ancestry_path_comparisons,
        summary.ancestry_parent_proofs,
        summary.effect_paths,
        summary.effect_path_bytes,
        summary.impact_candidates,
        summary.impact_ancestor_visits,
        summary.impact_retained_dirty_paths,
        summary.impact_all_dirty,
        summary.journal_retained_commits,
        summary.journal_cloned_commits,
        summary.journal_oversized_commits,
        summary.journal_dropped_commits,
    )
}

fn json_scan_diagnostics(value: Option<&fdu_core::scan::ScanDiagnostics>) -> String {
    value.map_or_else(|| "null".into(), fdu_core::scan::ScanDiagnostics::to_json)
}

fn json_attribution(value: Option<&fdu_core::scan::WalkAttribution>) -> String {
    value.map_or_else(
        || "null".into(),
        |a| {
            format!(
                concat!(
                    "{{\"wall_ns\":{},\"work_ns\":{},\"starved_ns\":{},",
                    "\"lock_wait_ns\":{},\"send_ns\":{},\"claims\":{},",
                    "\"lock_ops\":{},\"lock_contended\":{}}}"
                ),
                a.wall_ns,
                a.work_ns,
                a.starved_ns,
                a.lock_wait_ns,
                a.send_ns,
                a.claims,
                a.lock_ops,
                a.lock_contended,
            )
        },
    )
}

fn json_optional_string(value: Option<&str>) -> String {
    value.map_or_else(|| "null".into(), |value| format!("\"{}\"", json_escape(value)))
}

fn json_optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "null".into(), |value| value.to_string())
}

fn json_optional_i64(value: Option<i64>) -> String {
    value.map_or_else(|| "null".into(), |value| value.to_string())
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character <= '\u{1f}' => {
                let _ = write!(escaped, "\\u{:04x}", u32::from(character));
            }
            character => escaped.push(character),
        }
    }
    escaped
}

#[derive(Default)]
struct MultisetDigest {
    count: u64,
    sum: [u8; 32],
    xor: [u8; 32],
}

impl MultisetDigest {
    fn add(&mut self, record: &[u8]) {
        let leaf = Sha256::digest(record);
        let mut carry = 0_u16;
        for index in (0..32).rev() {
            self.xor[index] ^= leaf[index];
            let total = u16::from(self.sum[index]) + u16::from(leaf[index]) + carry;
            self.sum[index] = total.to_be_bytes()[1];
            carry = total >> 8;
        }
        self.count = self.count.saturating_add(1);
    }

    fn finish(self) -> String {
        let mut final_record = Vec::with_capacity(DIGEST_ALGORITHM.len() + 73);
        final_record.extend_from_slice(DIGEST_ALGORITHM.as_bytes());
        final_record.push(0);
        final_record.extend_from_slice(&self.count.to_be_bytes());
        final_record.extend_from_slice(&self.xor);
        final_record.extend_from_slice(&self.sum);
        hex(&Sha256::digest(&final_record))
    }
}

struct Sha256 {
    buffer: [u8; 64],
    buffer_len: usize,
    length_bytes: u64,
    state: [u32; 8],
}

impl Sha256 {
    const INITIAL: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];

    fn new() -> Self {
        Self { buffer: [0; 64], buffer_len: 0, length_bytes: 0, state: Self::INITIAL }
    }

    fn digest(bytes: &[u8]) -> [u8; 32] {
        let mut hash = Self::new();
        hash.update(bytes);
        hash.finalize()
    }

    fn update(&mut self, mut bytes: &[u8]) {
        self.length_bytes =
            self.length_bytes.saturating_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX));
        if self.buffer_len > 0 {
            let wanted = (64 - self.buffer_len).min(bytes.len());
            self.buffer[self.buffer_len..self.buffer_len + wanted]
                .copy_from_slice(&bytes[..wanted]);
            self.buffer_len += wanted;
            bytes = &bytes[wanted..];
            if self.buffer_len == 64 {
                Self::compress(&mut self.state, &self.buffer);
                self.buffer_len = 0;
            }
        }
        while bytes.len() >= 64 {
            let (block, remaining) = bytes.split_at(64);
            let mut block_array = [0_u8; 64];
            block_array.copy_from_slice(block);
            Self::compress(&mut self.state, &block_array);
            bytes = remaining;
        }
        self.buffer[..bytes.len()].copy_from_slice(bytes);
        self.buffer_len = bytes.len();
    }

    fn finalize(mut self) -> [u8; 32] {
        let bit_length = self.length_bytes.saturating_mul(8);
        self.buffer[self.buffer_len] = 0x80;
        self.buffer_len += 1;
        if self.buffer_len > 56 {
            self.buffer[self.buffer_len..].fill(0);
            Self::compress(&mut self.state, &self.buffer);
            self.buffer = [0; 64];
        } else {
            self.buffer[self.buffer_len..56].fill(0);
        }
        self.buffer[56..].copy_from_slice(&bit_length.to_be_bytes());
        Self::compress(&mut self.state, &self.buffer);
        let mut output = [0_u8; 32];
        for (chunk, word) in output.chunks_exact_mut(4).zip(self.state) {
            chunk.copy_from_slice(&word.to_be_bytes());
        }
        output
    }

    #[allow(clippy::many_single_char_names)]
    fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
        const K: [u32; 64] = [
            0x428a_2f98,
            0x7137_4491,
            0xb5c0_fbcf,
            0xe9b5_dba5,
            0x3956_c25b,
            0x59f1_11f1,
            0x923f_82a4,
            0xab1c_5ed5,
            0xd807_aa98,
            0x1283_5b01,
            0x2431_85be,
            0x550c_7dc3,
            0x72be_5d74,
            0x80de_b1fe,
            0x9bdc_06a7,
            0xc19b_f174,
            0xe49b_69c1,
            0xefbe_4786,
            0x0fc1_9dc6,
            0x240c_a1cc,
            0x2de9_2c6f,
            0x4a74_84aa,
            0x5cb0_a9dc,
            0x76f9_88da,
            0x983e_5152,
            0xa831_c66d,
            0xb003_27c8,
            0xbf59_7fc7,
            0xc6e0_0bf3,
            0xd5a7_9147,
            0x06ca_6351,
            0x1429_2967,
            0x27b7_0a85,
            0x2e1b_2138,
            0x4d2c_6dfc,
            0x5338_0d13,
            0x650a_7354,
            0x766a_0abb,
            0x81c2_c92e,
            0x9272_2c85,
            0xa2bf_e8a1,
            0xa81a_664b,
            0xc24b_8b70,
            0xc76c_51a3,
            0xd192_e819,
            0xd699_0624,
            0xf40e_3585,
            0x106a_a070,
            0x19a4_c116,
            0x1e37_6c08,
            0x2748_774c,
            0x34b0_bcb5,
            0x391c_0cb3,
            0x4ed8_aa4a,
            0x5b9c_ca4f,
            0x682e_6ff3,
            0x748f_82ee,
            0x78a5_636f,
            0x84c8_7814,
            0x8cc7_0208,
            0x90be_fffa,
            0xa450_6ceb,
            0xbef9_a3f7,
            0xc671_78f2,
        ];
        let mut words = [0_u32; 64];
        for (index, chunk) in block.chunks_exact(4).enumerate() {
            let mut word = [0_u8; 4];
            word.copy_from_slice(chunk);
            words[index] = u32::from_be_bytes(word);
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] =
                words[index - 16].wrapping_add(s0).wrapping_add(words[index - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
        for index in 0..64 {
            let sum1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ (!e & g);
            let temporary1 = h
                .wrapping_add(sum1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(words[index]);
            let sum0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temporary2 = sum0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temporary1);
            d = c;
            c = b;
            b = a;
            a = temporary1.wrapping_add(temporary2);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[derive(Debug)]
struct ProbeError(String);

type ProbeResult<T> = Result<T, ProbeError>;

impl From<fdu_core::Error> for ProbeError {
    fn from(error: fdu_core::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<std::io::Error> for ProbeError {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_standard_vectors() {
        assert_eq!(
            hex(&Sha256::digest(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(&Sha256::digest(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn multiset_digest_is_order_independent() {
        let mut first = MultisetDigest::default();
        first.add(b"one");
        first.add(b"two");
        let mut second = MultisetDigest::default();
        second.add(b"two");
        second.add(b"one");
        assert_eq!(first.finish(), second.finish());
    }

    #[test]
    fn json_render_is_one_complete_object() {
        let rendered =
            ProbeOutput::new(Mode::ScanIndex, "scan", Duration::from_nanos(7), Summary::default())
                .render();
        assert!(rendered.starts_with("{\"component_ns\":7,"));
        assert!(rendered.ends_with('}'));
        assert!(rendered.contains("\"schema\":\"fdu-perf-probe-v1\""));
        assert!(rendered.contains("\"oracle_enabled\":true"));
    }

    #[test]
    fn no_oracle_profile_summary_does_not_walk_the_index() {
        let arguments = Arguments::parse(
            ["scan-index", "--root", "/root", "--no-oracle"].into_iter().map(OsString::from),
        )
        .expect("profile arguments");
        let index = Index::new("/root");

        let summary = summarize_index(&arguments, &index).expect("unverified summary");

        assert!(!arguments.oracle_enabled);
        assert_eq!(summary.entries, 1);
        assert_eq!(summary.dirs, 1);
        assert!(summary.engine_digest.is_none());
    }

    #[test]
    fn default_tree_probe_can_retain_full_index_scan_diagnostics() {
        let root = tempfile::tempdir().expect("root tempdir");
        let scratch = tempfile::tempdir().expect("scratch tempdir");
        std::fs::create_dir(root.path().join("nested")).expect("nested directory");
        std::fs::write(root.path().join("nested/file.txt"), b"trace me").expect("file");
        let arguments = Arguments::parse(
            [
                OsString::from("default-tree"),
                OsString::from("--root"),
                root.path().as_os_str().to_owned(),
                OsString::from("--snapshot"),
                scratch.path().join("snapshot.fdu").into_os_string(),
                OsString::from("--diagnostics"),
            ]
            .into_iter(),
        )
        .expect("probe arguments");

        let output = default_tree(&arguments).expect("default-tree probe");

        let diagnostics = output.summary.scan_diagnostics.expect("scan diagnostics");
        assert_eq!(diagnostics.schema, fdu_core::scan::SCAN_DIAGNOSTICS_SCHEMA);
        assert_eq!(diagnostics.worker_policy.ready_directories_at_finish, 0);
        assert_eq!(diagnostics.worker_policy.in_flight_directories_at_finish, 0);
    }

    #[test]
    fn summary_reports_newest_regular_file_mtime() {
        let attrs = |mtime_ns| Attrs {
            size: 1,
            allocated: 512,
            mtime_ns,
            ctime_ns: mtime_ns,
            inode: u64::try_from(mtime_ns).unwrap_or_default(),
            dev: 1,
        };
        let observation = Observation::new(vec![
            Op::Upsert { path: PathBuf::from("older"), kind: EntryKind::File, attrs: attrs(10) },
            Op::Upsert {
                path: PathBuf::from("newer-directory"),
                kind: EntryKind::Dir,
                attrs: attrs(99),
            },
            Op::Upsert {
                path: PathBuf::from("newest-file"),
                kind: EntryKind::File,
                attrs: attrs(30),
            },
        ]);
        let mut summary = Summary::default();

        summary.observe(&observation);

        assert_eq!(summary.newest_file_mtime_ns, Some(30));
        let rendered =
            ProbeOutput::new(Mode::ScanProducer, "scan", Duration::ZERO, summary).render();
        assert!(rendered.contains("\"newest_file_mtime_ns\":30"));
    }
}
