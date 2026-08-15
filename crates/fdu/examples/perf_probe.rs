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

use fdu::content::{AnalysisProfile, AnalysisRequest, CoverageReason};
use fdu::query::{Provenance, Query, ReportSource, ViewSpec};
use fdu::{
    Attrs, CachePolicy, EntryId, EntryKind, Index, Observation, Op, OpenConfig, ScanConfig,
    ScanOrder,
};

const PROBE_SCHEMA: &str = "fdu-perf-probe-v1";
const DIGEST_ALGORITHM: &str = "fdu-index-record-v1/sha256-multiset-v1";

/// Count what the run allocates.
///
/// Always installed, and inert until recording is enabled: the wrapper checks one
/// relaxed atomic the branch predictor always gets right, which is nothing next to an
/// allocation. One binary that can be asked for numbers beats two that differ in
/// whether they have any. exp-052 measured the whole arrangement at no detectable cost.
#[global_allocator]
static ALLOCATOR: fdu::counters::alloc::CountingAlloc<std::alloc::System> =
    fdu::counters::system_allocator();

fn main() -> ExitCode {
    // `FDU_COUNTERS=1` turns recording on. Off by default so a probe run measured
    // against a control is not measuring the instrument, and on by one environment
    // variable when a run is meant to explain itself.
    let measurement = fdu::counters::Measurement::from_env();
    let exit = match Arguments::parse(env::args_os().skip(1))
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
    MarkdownProse,
    Query,
    ColdOpenSave,
    Revalidate,
    ScanIndex,
    ScanProducer,
    SnapshotLoad,
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
            "markdown-prose" => Ok(Self::MarkdownProse),
            "query" => Ok(Self::Query),
            "revalidate" => Ok(Self::Revalidate),
            "cold-open-save" => Ok(Self::ColdOpenSave),
            "scan-index" => Ok(Self::ScanIndex),
            "scan-producer" => Ok(Self::ScanProducer),
            "snapshot-load" => Ok(Self::SnapshotLoad),
            "snapshot-save" => Ok(Self::SnapshotSave),
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
            Self::MarkdownProse => "markdown-prose",
            Self::Query => "query",
            Self::Revalidate => "revalidate",
            Self::ColdOpenSave => "cold-open-save",
            Self::ScanIndex => "scan-index",
            Self::ScanProducer => "scan-producer",
            Self::SnapshotLoad => "snapshot-load",
            Self::SnapshotSave => "snapshot-save",
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
        Ok(Self { mode: Mode::parse(&mode)?, root, snapshot, operations, queries, repeat, scan })
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
    execute(arguments)
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
        Mode::SnapshotLoad => snapshot_load(arguments),
        Mode::Revalidate => revalidate(arguments),
        Mode::DeltaApply => delta_apply(arguments),
        Mode::MarkdownProse | Mode::TextProse => content_analysis(arguments, document_request()),
        Mode::Query => query(arguments),
    }
}

fn classification_probe(arguments: &Arguments, ambiguous: bool) -> ProbeResult<ProbeOutput> {
    let (index, scan) = fdu::scan::scan_into_index(&arguments.root, &arguments.scan)?;
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
            fdu::classify::classify_path_with_prefix(Path::new(path), Some(prefix))
        } else {
            fdu::classify::classify_path(Path::new(resolved[iteration % resolved.len()]))
        };
        observed = observed.saturating_add(
            u64::try_from(classification.file_type.as_str().len()).unwrap_or(u64::MAX),
        );
        black_box(classification);
    }
    let component = started.elapsed();
    let mut summary = summarize_index(&index)?;
    summary.query_iterations = u64::try_from(arguments.operations).unwrap_or(u64::MAX);
    summary.query_observations = observed;
    Ok(ProbeOutput::new(arguments.mode, "synthetic", component, summary))
}

fn basic_request() -> AnalysisRequest {
    AnalysisRequest { profile: AnalysisProfile::Basic, ..AnalysisRequest::default() }
}

fn code_request() -> AnalysisRequest {
    AnalysisRequest { profile: AnalysisProfile::Code, ..AnalysisRequest::default() }
}

fn document_request() -> AnalysisRequest {
    AnalysisRequest { profile: AnalysisProfile::Documents, ..AnalysisRequest::default() }
}

fn content_analysis(arguments: &Arguments, request: AnalysisRequest) -> ProbeResult<ProbeOutput> {
    let (mut index, scan) = fdu::scan::scan_into_index(&arguments.root, &arguments.scan)?;
    if !scan.is_complete() {
        return Err(ProbeError("content-analysis setup scan was partial".into()));
    }
    let enabled = request.profile.is_enabled();
    let started = Instant::now();
    let report = fdu::content::analyze_index(&mut index, request);
    black_box(report);
    let component = started.elapsed();
    let mut summary = summarize_index(&index)?;
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
    let (index, report) = fdu::open(&arguments.root, &config)?;
    let component = started.elapsed();
    let mut summary = summarize_index(&index)?;
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
        fdu::OpenPath::ColdScan => "cold-scan",
        fdu::OpenPath::WarmRevalidate => "warm-revalidate",
        fdu::OpenPath::CacheOnly => "content-cache",
    };
    Ok(ProbeOutput::new(arguments.mode, source, component, summary))
}

fn content_query(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let (mut index, scan) = fdu::scan::scan_into_index(&arguments.root, &arguments.scan)?;
    if !scan.is_complete() {
        return Err(ProbeError("content-query setup scan was partial".into()));
    }
    let analysis = fdu::content::analyze_index(&mut index, basic_request());
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
        black_box(fdu::query::report(&index, &query, &provenance));
    }
    let component = started.elapsed();
    let mut summary = summarize_index(&index)?;
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
    let report = fdu::scan::scan(&arguments.root, &arguments.scan, &mut |observation| {
        summary.observe(&observation);
    })?;
    let component = started.elapsed();
    summary.entries = summary.entries.saturating_add(1);
    summary.dirs = summary.dirs.saturating_add(1);
    summary.dirs_read = report.dirs_read;
    summary.attribution = Some(report.attribution);
    summary.errors = u64::try_from(report.errors.len()).unwrap_or(u64::MAX);
    summary.complete = report.is_complete();
    if summary.complete {
        // The exact oracle is deliberately outside the component timer. A producer
        // summary that is faster because it skipped, duplicated, or misclassified an
        // entry must never become an accepted performance sample.
        let (validation_index, validation_report) =
            fdu::scan::scan_into_index(&arguments.root, &arguments.scan)?;
        if !validation_report.is_complete() {
            return Err(ProbeError("scan-producer validation scan was partial".into()));
        }
        let validation = summarize_index(&validation_index)?;
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
    let started = Instant::now();
    let (index, report) = fdu::scan::scan_into_index(&arguments.root, &arguments.scan)?;
    let component = started.elapsed();
    let mut summary = summarize_index(&index)?;
    summary.dirs_read = report.dirs_read;
    summary.attribution = Some(report.attribution);
    summary.errors = u64::try_from(report.errors.len()).unwrap_or(u64::MAX);
    summary.complete = report.is_complete();
    Ok(ProbeOutput::new(arguments.mode, "scan", component, summary))
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
    let (index, report, pending) = fdu::open_with_pending_save(&arguments.root, &config)?;
    pending.join()?;
    let component = started.elapsed();
    if !report.is_complete() {
        return Err(ProbeError("cold-open-save scan was partial".into()));
    }
    let mut summary = summarize_index(&index)?;
    summary.complete = report.is_complete();
    summary.errors = u64::try_from(report.scan.errors.len()).unwrap_or(u64::MAX);
    summary.dirs_read = report.scan.dirs_read;
    summary.snapshot_bytes = snapshot.metadata().ok().map(|metadata| metadata.len());
    Ok(ProbeOutput::new(arguments.mode, "scan", component, summary))
}

fn snapshot_save(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let (index, report) = fdu::scan::scan_into_index(&arguments.root, &arguments.scan)?;
    if !report.is_complete() {
        return Err(ProbeError("snapshot setup scan was partial".into()));
    }
    let snapshot = arguments.snapshot()?;
    let started = Instant::now();
    fdu::snapshot::save(&index, snapshot)?;
    let component = started.elapsed();
    let mut summary = summarize_index(&index)?;
    summary.snapshot_bytes = Some(snapshot.metadata()?.len());
    Ok(ProbeOutput::new(arguments.mode, "scan", component, summary))
}

fn snapshot_load(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let snapshot = arguments.snapshot()?;
    let started = Instant::now();
    let index = load_snapshot(snapshot)?;
    let component = started.elapsed();
    let mut summary = summarize_index(&index)?;
    summary.snapshot_bytes = Some(snapshot.metadata()?.len());
    Ok(ProbeOutput::new(arguments.mode, "snapshot", component, summary))
}

fn revalidate(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let snapshot = arguments.snapshot()?;
    let mut index = load_snapshot(snapshot)?;
    let started = Instant::now();
    let report = fdu::scan::reconcile(&mut index, &arguments.scan, &mut |_| {})?;
    let component = started.elapsed();
    let mut summary = summarize_index(&index)?;
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
    let operations = (0..arguments.operations)
        .map(|index| Op::Upsert {
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
        })
        .collect();
    let observation = Observation::new(operations);
    let mut index = Index::new(&arguments.root);
    let started = Instant::now();
    let outcome = index.apply(&observation)?;
    let component = started.elapsed();
    let mut summary = summarize_index(&index)?;
    summary.apply = ApplySummary {
        inserted: outcome.inserted,
        invalidated: outcome.invalidated,
        removed: outcome.removed,
        stale: outcome.stale,
        unchanged: outcome.unchanged,
        updated: outcome.updated,
    };
    Ok(ProbeOutput::new(arguments.mode, "synthetic", component, summary))
}

fn query(arguments: &Arguments) -> ProbeResult<ProbeOutput> {
    let index = if let Some(snapshot) = arguments.snapshot.as_deref() {
        load_snapshot(snapshot)?
    } else {
        let (index, report) = fdu::scan::scan_into_index(&arguments.root, &arguments.scan)?;
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
    let mut summary = summarize_index(&index)?;
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
    fdu::snapshot::load(path)?.ok_or_else(|| ProbeError("snapshot was not usable".into()))
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

#[derive(Debug)]
struct Summary {
    allocated_bytes: u128,
    attribution: Option<fdu::scan::WalkAttribution>,
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
    snapshot_bytes: Option<u64>,
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
            snapshot_bytes: None,
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

fn summarize_index(index: &Index) -> ProbeResult<Summary> {
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
    source: &'static str,
    summary: Summary,
}

impl ProbeOutput {
    fn new(mode: Mode, source: &'static str, component: Duration, summary: Summary) -> Self {
        Self { component_ns: component.as_nanos(), mode, source, summary }
    }

    fn render(&self) -> String {
        let summary = &self.summary;
        let digest = json_optional_string(summary.engine_digest.as_deref());
        let content_digest = json_optional_string(summary.content_digest.as_deref());
        let index_len = json_optional_u64(summary.index_len);
        let newest_file_mtime_ns = json_optional_i64(summary.newest_file_mtime_ns);
        let snapshot_bytes = json_optional_u64(summary.snapshot_bytes);
        format!(
            concat!(
                "{{\"component_ns\":{},\"attribution\":{},\"mode\":\"{}\",\"schema\":\"{}\",",
                "\"source\":\"{}\",\"summary\":{{",
                "\"allocated_bytes\":{},\"apparent_bytes\":{},",
                "\"apply\":{{\"inserted\":{},\"invalidated\":{},",
                "\"removed\":{},\"stale\":{},\"unchanged\":{},\"updated\":{}}},",
                "\"complete\":{},\"dirs\":{},\"dirs_read\":{},",
                "\"content\":{{\"analyzed\":{},\"applied\":{},\"binary\":{},",
                "\"cache_hits\":{},\"candidates\":{},\"digest\":{},",
                "\"invalid_utf8\":{},\"records\":{}}},",
                "\"engine_digest\":{},\"entries\":{},\"errors\":{},",
                "\"files\":{},\"index_len\":{},\"newest_file_mtime_ns\":{},\"other\":{},",
                "\"query_iterations\":{},\"query_observations\":{},",
                "\"snapshot_bytes\":{},",
                "\"symlinks\":{}}}}}"
            ),
            self.component_ns,
            json_attribution(self.summary.attribution.as_ref()),
            self.mode.name(),
            PROBE_SCHEMA,
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
            summary.symlinks,
        )
    }
}

fn json_attribution(value: Option<&fdu::scan::WalkAttribution>) -> String {
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

impl From<fdu::Error> for ProbeError {
    fn from(error: fdu::Error) -> Self {
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
