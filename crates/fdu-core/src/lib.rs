//! **fdu** — a fast, incremental file roll-up engine.
//!
//! fdu answers, for any directory in a tree: how big is it, how many files does it hold,
//! what changed most recently, and what kinds of files live in it — hierarchically, for
//! every directory at once, from a single walk.
//!
//! # The shape: three artifacts, one contract
//!
//! 1. **The index** ([`Index`]) — the in-memory hierarchical structure: entry
//!    records plus per-directory roll-up state.
//! 2. **The snapshot** ([`snapshot`]) — that index, serialized.
//! 3. **The change contract** ([`Observation`] and [`AppliedDelta`]) —
//!    producers submit verified observations; the index commits clocked effective
//!    changes.
//!
//! Everything else is a producer of observations or a consumer of applied deltas. The
//! walker establishes a baseline from upsert observations; the reconciler submits the
//! conditional diff between indexed state and reality; the watch layer submits verified,
//! coalesced observations. The index arbitrates them and re-rolls its reducers; a change
//! feed consumes the effective committed deltas.
//!
//! A deliberate consequence: **watching is not tied to the roll-up logic.** The index
//! knows `apply(Observation)` and nothing about filesystem events, so a batch scan, a test
//! feeding synthetic observations, and a live watcher are indistinguishable to it.
//!
//! # Freshness is a ladder, not a set of alternatives
//!
//! [`open`] is the conservative, blocking entry point: it loads a compatible snapshot,
//! reconciles the configured filesystem scope, and only then returns. It does not serve
//! the loaded baseline concurrently. Applications that want that model can own an
//! [`IndexHandle`], call the applying reconciliation APIs, and inspect [`Freshness`]
//! while readers continue between short write batches. With the `watch` feature,
//! [`watch::Watcher::apply_next`] verifies event hints and closes invalidations through
//! subtree reconciliation; neither `open` nor the Python binding starts it implicitly.
//!
//! ```no_run
//! use fdu_core::{OpenConfig, open};
//! use std::path::Path;
//!
//! let (index, report) = open(Path::new("."), &OpenConfig::default())?;
//! let total = index.total();
//! println!("{} files, {} bytes ({:?})", total.files, total.bytes, report.path_taken);
//! # Ok::<(), fdu_core::Error>(())
//! ```
//!
//! # Feature flags
//!
//! - `cli` *(default)* — the `fdu` binary and its dependencies. Library consumers should
//!   take `default-features = false`.
//! - `watch` *(default)* — the OS-native watch layer. Strictly additive: without it
//!   everything else works, just without live updates.

pub mod cache;
pub mod classify;
pub mod content;
pub mod counters;
mod engine_contract;
mod execution;
mod index;
mod platform_tuning;
pub mod query;
pub mod scan;
pub mod snapshot;
pub mod tags;
#[cfg(test)]
mod test_support;

// Ungated: rendering is not a command-line concern. It was behind `cli` only because it
// took its ANSI colour types from clap, so the library could produce a report and not
// print it -- and a display note added elsewhere on this branch called into here and
// broke the no-default-features build, which is that gap showing itself.
pub mod report_format;

#[cfg(feature = "watch")]
pub mod watch_session;

#[cfg(feature = "watch")]
pub mod watch;

#[cfg(feature = "watch")]
pub use crate::watch_session as session;

pub use crate::cache::{
    CacheStatus, SnapshotInfo, cache_status, clear_all_caches, clear_cache, list_caches,
};
pub use crate::engine_contract::{
    AppliedDelta, Attrs, Bound, Clock, CommittedState, CoverageReason, Cursor, EntryKind, Error,
    Expectation, Fingerprint, Freshness, InvalidateReason, Issue, IssueKind, Observation,
    ObservationOp, Op, PathExpectation, PathState, Phase, Provenance, Result, ScanScope, SessionId,
    Source, StateChange, Status,
};
pub use crate::index::{
    ApplyOutcome, ApplyStats, ChildPage, ChildPageRequest, ChildRemainder, ChildSnapshot, EntryId,
    ExtRemainder, ExtTally, Index, IndexHandle, ProjectionWork, ReadBundle, ReadRequest,
    ReportRequest, RollUp, RollUpScalars, Since, Work,
};
// Ungated with report_format, for the same reason: one-shot planning is an execution
// strategy, not a front end. A caller wanting one report without retaining an index was
// previously required to compile the command line to get it (fdu-z7sp).
pub use crate::execution::{
    PerformanceSummary, prepare_report, prepare_report_with_scan_diagnostics,
};
pub use crate::scan::{ReconcileReport, ScanConfig, ScanOrder, ScanReport};
#[cfg(feature = "watch")]
pub use crate::watch_session::{Batch, Change, ChangeKind, Session};

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// How to open a tree.
#[derive(Clone, Debug, Default)]
pub struct OpenConfig {
    /// Walk settings.
    pub scan: ScanConfig,
    /// Where the snapshot for this root lives.
    ///
    /// `None` disables the cache regardless of policy, which is what a caller with no
    /// writable cache directory gets.
    pub cache_path: Option<PathBuf>,
    /// How the snapshot may be used.
    pub policy: CachePolicy,
    /// Optional streaming content analysis. Disabled preserves metadata-only behavior.
    pub analysis: content::AnalysisRequest,
}

/// How an [`open`] may use the snapshot cache.
///
/// One explicit axis rather than a pair of booleans, because "did this answer touch the
/// filesystem" and "did it leave a trace" are the two questions a caller actually has,
/// and a boolean pair can express combinations that have no meaning.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CachePolicy {
    /// Read the snapshot, revalidate it, and write it back when the scan is complete.
    #[default]
    Auto,
    /// Ignore any snapshot, scan cold, and rewrite it. The benchmark control.
    Refresh,
    /// Read and revalidate, but never write. A warm answer that leaves no trace.
    ReadOnly,
    /// Answer from the snapshot alone, without touching the tree.
    ///
    /// Fails when no usable snapshot exists: there is no data to answer with, and
    /// silently falling back to a scan would make the fast path unpredictable.
    Only,
    /// Ignore the snapshot entirely and leave nothing behind.
    Off,
}

impl CachePolicy {
    /// Whether this policy may read an existing snapshot.
    fn reads(self) -> bool {
        matches!(self, Self::Auto | Self::ReadOnly | Self::Only)
    }

    /// Whether this policy may write a snapshot back.
    ///
    /// Public because a caller deciding whether to prepare a cache directory, or to warn
    /// that a run will leave nothing behind, is asking about the policy it was handed --
    /// and the alternative is matching on the variants, which is the same knowledge
    /// copied into every caller.
    pub fn writes(self) -> bool {
        matches!(self, Self::Auto | Self::Refresh)
    }

    /// Whether this policy may touch the filesystem at all.
    fn scans(self) -> bool {
        !matches!(self, Self::Only)
    }
}

/// Which tier of the freshness ladder an [`open`] actually used.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpenPath {
    /// No usable snapshot: the tree was walked from scratch.
    ColdScan,
    /// A snapshot was loaded and reconciled against the filesystem.
    WarmRevalidate,
    /// A snapshot answered on its own; the filesystem was never consulted.
    ///
    /// The only tier that can be stale, and it says so rather than implying currency.
    CacheOnly,
}

/// A snapshot write running alongside rendering.
///
/// The index is read-only by the time this starts, so the writer and the renderer are
/// two readers of the same data. The handle exists so the process can join before it
/// exits: an abandoned write would leave a half-written snapshot for the next run to
/// reject, turning a warm start into a cold one for no reason.
#[derive(Debug)]
#[must_use = "join the save before exiting or the snapshot may be abandoned"]
pub struct PendingSave {
    workers: Vec<(&'static str, std::thread::JoinHandle<Result<()>>)>,
}

impl PendingSave {
    /// Nothing to wait for.
    pub(crate) fn none() -> Self {
        Self { workers: Vec::new() }
    }

    /// Wait for the write to finish, returning its result.
    ///
    /// A failed save is the caller's to report, not to die on: the answer already
    /// rendered is still correct, and only the next run's warmth is lost.
    pub fn join(mut self) -> Result<()> {
        let mut first_error = None;
        for (name, worker) in self.workers.drain(..) {
            let outcome = worker
                .join()
                .unwrap_or_else(|_| Err(Error::Snapshot(format!("{name} cache writer panicked"))));
            if first_error.is_none() {
                first_error = outcome.err();
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}

impl Drop for PendingSave {
    fn drop(&mut self) {
        // A dropped handle still waits: losing the write silently would be worse than
        // the brief delay, and this only happens on a path that forgot to join.
        for (_, worker) in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

/// What [`open`] did.
#[derive(Debug)]
pub struct OpenReport {
    /// Cache tier used to produce the returned index.
    pub path_taken: OpenPath,
    /// Filesystem walk results, including any partial errors.
    pub scan: ScanReport,
    /// Content-analysis work performed after metadata reconciliation.
    pub analysis: Option<content::AnalysisReport>,
    /// Reusable records restored from the independently versioned content sidecar.
    pub content_cache: content::ContentCacheLoad,
}

impl OpenReport {
    /// Whether every path in the requested scan scope was read successfully.
    pub fn is_complete(&self) -> bool {
        self.scan.is_complete()
            && self.analysis.as_ref().is_none_or(content::AnalysisReport::is_complete)
    }

    /// Per-path errors that make this result partial.
    pub fn errors(&self) -> &[Error] {
        &self.scan.errors
    }

    /// Human-readable diagnostics for every operational condition that makes this result partial.
    pub fn error_messages(&self) -> Vec<String> {
        let mut errors = self.scan.errors.iter().map(ToString::to_string).collect::<Vec<_>>();
        if let Some(message) =
            self.analysis.as_ref().and_then(content::AnalysisReport::failure_message)
        {
            errors.push(message);
        }
        errors
    }
}

/// Open a tree, using the snapshot cache when one is usable.
///
/// On the warm path the snapshot is loaded and then reconciled against the filesystem
/// before being returned. Errors are represented as partial freshness and the previous
/// complete snapshot is left untouched; callers must inspect [`OpenReport::is_complete`]
/// or [`Index::freshness`] before treating totals as complete.
pub fn open(root: &Path, config: &OpenConfig) -> Result<(Index, OpenReport)> {
    let (index, report, pending) = open_with_pending_save(root, config)?;
    // Joining first is what makes the unwrap infallible: the writer held the only other
    // reference, and this is the blocking entry point, so by here it has finished and
    // dropped it. `try_unwrap` rather than a clone keeps the owned-`Index` signature
    // honest — a fallback clone here would quietly reintroduce the copy the shared
    // writer exists to avoid.
    pending.join()?;
    let index = std::sync::Arc::into_inner(index)
        .expect("the joined writer released the only other reference");
    Ok((index, report))
}

/// Open a tree, returning the snapshot write for the caller to join.
///
/// The blocking [`open`] is the right default; a caller that renders its own output can
/// use this to overlap the write with rendering and join before exiting.
///
/// This path always loads a usable snapshot, because its callers — live sessions and
/// library consumers holding the index — amortise the load across everything they do
/// with it. A one-shot report cannot; the internal report planner decides per request
/// whether the read pays and routes through the gated variant below.
pub fn open_with_pending_save(
    root: &Path,
    config: &OpenConfig,
) -> Result<(std::sync::Arc<Index>, OpenReport, PendingSave)> {
    open_for_report(root, config, true)
}

/// [`open_with_pending_save`] with the snapshot read under the caller's control.
///
/// `read_snapshot: false` skips loading an existing snapshot and takes the cold-scan
/// path: for a one-shot metadata query, revalidation stats every entry regardless, so
/// the load and the reconciliation against it are additive cost with nothing to
/// amortise them — measured on macOS/APFS over 494,031 entries, warm revalidation cost
/// 4.8 s against 3.6 s for the cold path, whose write-behind the read could at best
/// have saved ~50 ms of. Persistence is unaffected: the cold path still writes per
/// [`cold_scan_save_targets`], so the snapshot stays fresh for [`CachePolicy::Only`]
/// and for content-analysis reuse.
///
/// A policy that cannot scan reads regardless of the flag — for [`CachePolicy::Only`]
/// the snapshot is the contract, not a cost choice.
pub(crate) fn open_for_report(
    root: &Path,
    config: &OpenConfig,
    read_snapshot: bool,
) -> Result<(std::sync::Arc<Index>, OpenReport, PendingSave)> {
    let root = root.canonicalize().map_err(|e| Error::io(root, e))?;
    let policy = config.policy;
    // The *start* of the operation, not its end. A file modified mid-walk may have been
    // observed before the modification, so only the start bound is conservative for an
    // incremental follow-up query -- which is what this watermark is for.
    let started_at = Some(std::time::SystemTime::now());

    let loaded = match ((read_snapshot || !policy.scans()) && policy.reads(), &config.cache_path) {
        // The guard and the rules go down together. A snapshot describing another root or
        // a different scan scope is not this tree's answer, and it is refused from the
        // header rather than after a full parse; one that does answer is built under the
        // caller's own registry and tag rules, so every derived value is right the first
        // time instead of being relabelled afterwards.
        (true, Some(cache_path)) => snapshot::load_for(
            cache_path,
            &snapshot::LoadRequest {
                root: &root,
                scope: config.scan.scope(),
                types: config.scan.types().clone(),
                tags: config.scan.tags(),
            },
        )?,
        _ => None,
    };

    if !policy.scans() {
        let Some(mut index) = loaded else {
            return Err(Error::Snapshot(
                "no usable snapshot for this root and scan scope".to_string(),
            ));
        };
        // Deliberately no reconciliation: this tier never touches the tree. The index is
        // marked unverified so the answer cannot claim a currency it has not earned — a
        // snapshot records the freshness it was written with, which was true then.
        index.mark_unverified();
        bind_path_tags(&mut index, config);
        // A cache-only open ran no walk, so there is nothing that could have failed: the
        // snapshot covered its scope when it was written, and `mark_unverified` above is
        // what says the tree has not been checked since.
        record_run(&mut index, crate::query::ReportSource::CacheOnly, &ScanReport::default(), None);
        let content_cache = load_content(&mut index, config)?;
        if config.analysis.profile.is_enabled()
            && (!content_cache.usable
                || content_cache.hits
                    != u64::try_from(index.analysis_candidates(config.analysis.profile).len())
                        .unwrap_or(u64::MAX))
        {
            return Err(Error::Snapshot(
                "no complete usable content sidecar for this root and analysis profile".into(),
            ));
        }
        return Ok((
            std::sync::Arc::new(index),
            OpenReport {
                path_taken: OpenPath::CacheOnly,
                scan: ScanReport::default(),
                analysis: None,
                content_cache,
            },
            PendingSave::none(),
        ));
    }

    if let Some(mut index) = loaded {
        let reconciled = scan::reconcile(&mut index, &config.scan, &mut |_| {})?;
        let scan_report = reconciled.scan;
        bind_path_tags(&mut index, config);
        record_run(
            &mut index,
            crate::query::ReportSource::WarmRevalidate,
            &scan_report,
            started_at,
        );
        index.establish_baseline();
        let content_cache = load_content(&mut index, config)?;
        let analysis = config
            .analysis
            .profile
            .is_enabled()
            .then(|| content::analyze_index(&mut index, config.analysis));
        // A reconciliation that mutated nothing leaves an index that serializes to the
        // bytes already on disk, so rewriting it is pure cost: the clone, the encode,
        // and the write all produce a file identical to the one just read. Each artifact
        // is judged separately because content and metadata are invalidated separately.
        let writes = SaveTargets {
            metadata: reconciled.apply.mutated(),
            // Stale sidecar records no longer match a live candidate; rewriting is what
            // drops them, so a load that saw any is a reason to write even when the
            // analysis added nothing.
            content: analysis.as_ref().is_some_and(|report| report.applied > 0)
                || content_cache.stale > 0,
        };
        let index = std::sync::Arc::new(index);
        let pending = spawn_save(&index, config, scan_report.is_complete(), writes);
        return Ok((
            index,
            OpenReport {
                path_taken: OpenPath::WarmRevalidate,
                scan: scan_report,
                analysis,
                content_cache,
            },
            pending,
        ));
    }

    let (mut index, scan_report) = scan::scan_into_index(&root, &config.scan)?;
    bind_path_tags(&mut index, config);
    record_run(&mut index, crate::query::ReportSource::ColdScan, &scan_report, started_at);
    let content_cache = load_content(&mut index, config)?;
    let analysis = config
        .analysis
        .profile
        .is_enabled()
        .then(|| content::analyze_index(&mut index, config.analysis));
    let index = std::sync::Arc::new(index);
    let pending = spawn_save(
        &index,
        config,
        scan_report.is_complete(),
        cold_scan_save_targets(&index, config),
    );
    Ok((
        index,
        OpenReport { path_taken: OpenPath::ColdScan, scan: scan_report, analysis, content_cache },
        pending,
    ))
}

/// Record what the operation that just finished did, on the index it produced.
///
/// One place, so the three open paths cannot disagree about what they claim. These facts
/// used to live beside the index in the caller -- which meant a read sampled them from a
/// different lock than the rows, and a refresh landing between the two acquisitions
/// paired an old projection with new status. Committing them with the tree makes that
/// pairing impossible rather than unlikely.
fn record_run(
    index: &mut Index,
    source: crate::query::ReportSource,
    scan: &ScanReport,
    started_at: Option<std::time::SystemTime>,
) {
    index.install_run_facts(crate::query::RunFacts {
        scan_started_at: started_at,
        source,
        complete: scan.is_complete(),
        errors: scan.errors.iter().map(crate::Issue::from_error).collect(),
    });
}

/// Bind Path-tier tag rules against the index that now exists, and re-tag under them.
///
/// Called once on each of the three open paths, at the point the tree stops changing. A
/// Path-tier rule reads control files, and where those files are is something only an
/// index knows; binding earlier meant finding them by walking the tree, which cost a cold
/// scan a second full traversal, cost every `.gitignore` save another, and on the
/// cache-only path broke that path's one promise -- it opened by walking the tree it is
/// defined not to touch.
///
/// So the rules arrive unbound, entries are tagged as they land with whatever the
/// Name-tier rules decide, and this closes the gap: it reads exactly the control files the
/// index lists, then re-tags. The re-tag is an in-memory traversal, which is the trade --
/// one pass over entries already in hand instead of a second pass over the filesystem.
///
/// A no-op when no enabled rule reads a path, which is the default.
fn bind_path_tags(index: &mut Index, config: &OpenConfig) {
    let tags = config.scan.tags();
    if !tags.needs_path() {
        return;
    }
    let root = index.root_path().to_path_buf();
    let directories = index.control_file_directories();
    index.adopt_tag_rules(std::sync::Arc::new(
        tags.bound_to(&root, directories.iter().map(std::path::PathBuf::as_path)),
    ));
}

/// Which cache artifacts a completed open still needs to write.
///
/// A cold scan established both from nothing and writes both.  A warm open writes only
/// what its own pass changed, which on an unchanged tree is neither.
#[derive(Clone, Copy, Debug)]
struct SaveTargets {
    metadata: bool,
    content: bool,
}

impl SaveTargets {
    const fn all() -> Self {
        Self { metadata: true, content: true }
    }

    const fn nothing() -> Self {
        Self { metadata: false, content: false }
    }

    const fn none(self) -> bool {
        !self.metadata && !self.content
    }
}

/// Entry count at or above which a first scan's metadata snapshot is worth writing.
///
/// `None` persists unconditionally, which is what every release has shipped and what
/// this build still does. It stays unset because APFS measurement argued against a
/// threshold, not because the measurement is outstanding.
///
/// A snapshot repays its write only if a later run reads it *and* that read beats
/// rescanning. For metadata neither half is free. Revalidating a loaded snapshot stats
/// every entry regardless, so a warm `Auto` run saves nothing; measured on Linux/ext4
/// over 84,539 entries, a warm round-trip cost 162 ms against 132 ms to rebuild the index
/// outright. Only the no-scan `Only` tier avoids the walk, and warm it still loses, 81 ms
/// against 71 ms, because deserialisation costs about what a warm walk costs.
///
/// It wins decisively in exactly one metadata regime: a cold operating-system cache,
/// where the same read took 118 ms against 277 ms to scan. So the question a threshold
/// really answers is whether the next run will find this tree's metadata evicted, and
/// tree size is the honest proxy for that — a tree that fits comfortably in the page
/// cache will be warm again and the snapshot will never pay, while one that does not
/// will be cold and it always will.
///
/// That reasoning is ext4's, and it did not survive the crossing. Measured on APFS over
/// 175,128 entries, warm, the `Only` read took 146 ms against 521 ms to scan — a win
/// rather than ext4's loss, because deserialisation costs about the same on both while an
/// APFS metadata walk costs roughly three and a half times as much per entry. The 90 ms
/// write repays about fourfold on the first later `Only` read, at any size, so the
/// premise that metadata pays back only under a cold cache is false here.
///
/// A threshold would therefore give up real value on exactly the trees it gated, and it
/// would also introduce a visible cliff: below it, `fdu PATH` would stop leaving a
/// snapshot for a later `--cache only`. `fdu-hvs5` carries the evidence, and
/// `docs/project/guides/platform-tuning.md` carries why this one is a constant worth
/// not having.
const SNAPSHOT_MIN_ENTRIES: Option<u64> = None;

/// Which artifacts a first, cold scan should persist.
///
/// Separate from the warm-revalidation decision above, which asks whether a *loaded*
/// snapshot changed. This one asks whether a snapshot is worth creating at all.
fn cold_scan_save_targets(index: &Index, config: &OpenConfig) -> SaveTargets {
    cold_scan_save_targets_with(index.len(), config, SNAPSHOT_MIN_ENTRIES)
}

/// The decision itself, with the threshold injected so both sides stay provable while
/// [`SNAPSHOT_MIN_ENTRIES`] is unset.
fn cold_scan_save_targets_with(
    entries: u64,
    config: &OpenConfig,
    minimum: Option<u64>,
) -> SaveTargets {
    // Content sidecars are the clearest case for persisting: re-reading file bodies is
    // the expensive half of analysis, and reusing them measured 639 ms down to 325 ms.
    if config.analysis.profile.is_enabled() {
        return SaveTargets::all();
    }
    // An explicit instruction to rewrite the snapshot outranks any cost estimate.
    if config.policy == CachePolicy::Refresh {
        return SaveTargets::all();
    }
    match minimum {
        Some(minimum) if entries < minimum => SaveTargets::nothing(),
        _ => SaveTargets::all(),
    }
}

fn load_content(index: &mut Index, config: &OpenConfig) -> Result<content::ContentCacheLoad> {
    let (true, Some(snapshot_path)) = (config.policy.reads(), config.cache_path.as_deref()) else {
        return Ok(content::ContentCacheLoad::default());
    };
    content::load_content_cache(index, config.analysis, &content::content_cache_path(snapshot_path))
}

/// Start a snapshot write, when policy and completeness allow one.
///
/// Only a complete scan is written: a snapshot recording a partial view would be served
/// as fact on the next run, and the existing complete snapshot is better than that.
fn spawn_save(
    index: &std::sync::Arc<Index>,
    config: &OpenConfig,
    complete: bool,
    writes: SaveTargets,
) -> PendingSave {
    let (Some(cache_path), true, true, false) =
        (config.cache_path.clone(), config.policy.writes(), complete, writes.none())
    else {
        return PendingSave::none();
    };

    // The index is read-only from here, so the writer and the caller's rendering are two
    // readers of one index rather than of two copies. This used to deep-clone — every
    // boxed entry, both stored copies of every name, and every `BTreeMap` — on the
    // caller's thread, before rendering could start, on every cache-writing run.
    // Sharing is what buys the independence a clone was buying; a run with nothing to
    // write still returns above rather than reaching this point.
    let snapshot_source = std::sync::Arc::clone(index);
    let analysis = config.analysis;
    let mut workers = Vec::with_capacity(2);
    if writes.metadata {
        let metadata_source = std::sync::Arc::clone(&snapshot_source);
        let metadata_path = cache_path.clone();
        if let Ok(worker) =
            std::thread::Builder::new().name("fdu-snapshot".to_string()).spawn(move || {
                let _counter_guard = counters::thread_flush_guard();
                snapshot::save(&metadata_source, &metadata_path)
            })
        {
            workers.push(("metadata", worker));
        }
    }
    if writes.content && analysis.profile.is_enabled() {
        let content_path = content::content_cache_path(&cache_path);
        if let Ok(worker) =
            std::thread::Builder::new().name("fdu-content-cache".to_string()).spawn(move || {
                let _counter_guard = counters::thread_flush_guard();
                content::save_content_cache(&snapshot_source, analysis, &content_path)
            })
        {
            workers.push(("content", worker));
        }
    }
    // A machine that cannot spawn either thread can still answer; it just answers cold
    // next time.
    PendingSave { workers }
}

/// The conventional snapshot location for a root.
///
/// Snapshots are keyed by a hash of the canonical root path under the user cache
/// directory, so two roots never collide and a moved tree simply misses rather than
/// reading another tree's data.
pub fn default_cache_path(root: &Path) -> Option<PathBuf> {
    let canonical = root.canonicalize().ok()?;
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in canonical.as_os_str().as_encoded_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    Some(user_cache_dir()?.join("fdu").join(format!("{hash:016x}.fdu")))
}

fn user_cache_dir() -> Option<PathBuf> {
    if let Some(xdg) = nonempty_env("XDG_CACHE_HOME") {
        return Some(PathBuf::from(xdg));
    }
    platform_cache_dir()
}

fn nonempty_env(name: &str) -> Option<OsString> {
    std::env::var_os(name).filter(|value| !value.is_empty())
}

#[cfg(target_os = "windows")]
fn platform_cache_dir() -> Option<PathBuf> {
    windows_cache_dir(
        nonempty_env("LOCALAPPDATA"),
        nonempty_env("USERPROFILE"),
        nonempty_env("HOME"),
    )
}

#[cfg(target_os = "windows")]
fn windows_cache_dir(
    local_app_data: Option<OsString>,
    user_profile: Option<OsString>,
    home: Option<OsString>,
) -> Option<PathBuf> {
    local_app_data
        .map(PathBuf::from)
        .or_else(|| user_profile.map(|path| PathBuf::from(path).join("AppData").join("Local")))
        .or_else(|| home.map(|path| PathBuf::from(path).join(".cache")))
}

#[cfg(target_os = "macos")]
fn platform_cache_dir() -> Option<PathBuf> {
    Some(PathBuf::from(nonempty_env("HOME")?).join("Library").join("Caches"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn platform_cache_dir() -> Option<PathBuf> {
    Some(PathBuf::from(nonempty_env("HOME")?).join(".cache"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_file(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write");
    }

    /// Supplied rules reach the answer, and invalidate a snapshot taken under others.
    ///
    /// The end-to-end property behind the registry: a consumer whose taxonomy differs
    /// from this repository's classifies its own way without rebuilding the crate, and a
    /// snapshot written under one taxonomy is never served under another. The second half
    /// is the one that would fail silently -- the entry counts and byte totals are
    /// identical either way, so a stale snapshot looks entirely correct.
    /// A cold scan with gitignore rules visits each directory once, not twice.
    ///
    /// Binding used to walk the tree looking for control files, and then the metadata walk
    /// visited every one of those directories again. With the rule default-on, that
    /// doubled directory I/O for every run on a git tree -- a tagging option quietly
    /// paying for a second traversal of the filesystem. Binding from the index costs an
    /// in-memory re-tag instead.
    ///
    /// One scan inside the measured window, and a bound rather than an equality. The
    /// counters are process-global and `test_serial` only serializes the tests that take
    /// it, so concurrent work can inflate these -- never deflate them. That asymmetry is
    /// what the assertion is built on: the interesting failure is a *second* traversal,
    /// which cannot hide under an upper bound of one.
    #[cfg(feature = "gitignore")]
    #[test]
    fn a_cold_scan_with_gitignore_rules_opens_each_directory_once() {
        const DIRECTORIES: u64 = 4; // the root, `src`, `docs`, `docs/deep`

        let _serial = crate::counters::test_serial();
        let dir = tempfile::tempdir().expect("tempdir");
        let tree = dir.path().join("tree");
        for (relative, contents) in [
            (".gitignore", "*.log\n"),
            ("src/.gitignore", "!keep.log\n"),
            ("src/main.rs", "fn main() {}"),
            ("docs/notes.md", "# notes"),
            ("docs/deep/more.md", "# more"),
        ] {
            let path = tree.join(relative);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
            std::fs::write(&path, contents).expect("write");
        }

        let config = OpenConfig {
            policy: CachePolicy::Refresh,
            cache_path: None,
            scan: ScanConfig {
                // Single-threaded, so the walk's own counts are on this thread and land in
                // the window rather than trailing out of a pool after it closes.
                threads: Some(1),
                tags: Some(std::sync::Arc::new(
                    crate::tags::TagRules::from_names(["gitignore"]).expect("enables"),
                )),
                ..ScanConfig::default()
            },
            ..OpenConfig::default()
        };

        crate::counters::enable(true);
        let before = crate::counters::snapshot();
        let opened = open_for_report(&tree, &config, false);
        crate::counters::flush_thread();
        let after = crate::counters::snapshot();
        crate::counters::enable(false);

        let (index, _report, pending) = opened.expect("scan");
        pending.join().expect("no cache path is configured, so there is nothing to write");
        let opens = after.dir_opens - before.dir_opens;

        assert!(opens >= DIRECTORIES, "the walk must have happened at all: {opens} opens");
        assert!(
            opens < DIRECTORIES * 2,
            "a Path-tier tag added a second traversal: {opens} opens for {DIRECTORIES} \
             directories"
        );
        // And the rule actually decided something, so the bound above is about a scan that
        // did the work rather than one that skipped it.
        assert!(index.tags_of(Path::new("src/keep.log")).is_empty());
    }

    /// The cache-only tier must not touch the tree, and gitignore used to make it.
    ///
    /// Binding a Path-tier rule meant finding its control files, and finding them meant
    /// walking the root -- so `--cache only`, whose entire contract is that it answers
    /// from the snapshot without going to the filesystem, opened by traversing the very
    /// tree it promises not to look at. The answer was still right; the promise was
    /// silently broken, and the cost was the whole point of the tier.
    ///
    /// The counters are the assertion, because the promise is about work rather than about
    /// output: a test that only compared rows would have passed throughout.
    #[cfg(feature = "gitignore")]
    #[test]
    fn a_cache_only_open_with_gitignore_rules_does_not_walk_the_tree() {
        let _serial = crate::counters::test_serial();
        let dir = tempfile::tempdir().expect("tempdir");
        let tree = dir.path().join("tree");
        let cache = dir.path().join("cache").join("snap.fdu");
        for (relative, contents) in [
            (".gitignore", "*.log\n"),
            ("src/.gitignore", "!keep.log\n"),
            ("src/main.rs", "fn main() {}"),
            ("src/keep.log", "kept"),
            ("debug.log", "dropped"),
        ] {
            let path = tree.join(relative);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
            std::fs::write(&path, contents).expect("write");
        }

        let tags =
            std::sync::Arc::new(crate::tags::TagRules::from_names(["gitignore"]).expect("enables"));
        let config = |policy| OpenConfig {
            policy,
            cache_path: Some(cache.clone()),
            scan: ScanConfig { tags: Some(tags.clone()), ..ScanConfig::default() },
            ..OpenConfig::default()
        };

        // Cold, to leave a snapshot behind.
        let (cold, _, pending) =
            open_for_report(&tree, &config(CachePolicy::Auto), false).expect("cold open");
        pending.join().expect("the snapshot must land before the warm open reads it");
        assert_eq!(cold.tags_of(Path::new("debug.log")), vec!["gitignore"]);
        assert!(
            cold.tags_of(Path::new("src/keep.log")).is_empty(),
            "the nested negation must win, or the fixture is not exercising precedence"
        );

        // Warm, from the snapshot alone. A delta rather than an absolute, for the reason
        // the scan counters use one: these are process-global and a test that does not
        // take `test_serial` can add to them. Concurrency inflates and never deflates, so
        // zero is the one value it cannot manufacture.
        crate::counters::enable(true);
        let before = crate::counters::snapshot();
        let opened = open_for_report(&tree, &config(CachePolicy::Only), true);
        crate::counters::flush_thread();
        let after = crate::counters::snapshot();
        crate::counters::enable(false);

        let (warm, report, _) = opened.expect("cache-only open");
        assert_eq!(report.path_taken, OpenPath::CacheOnly);
        assert_eq!(
            after.dir_opens - before.dir_opens,
            0,
            "the cache-only tier read a directory: {} opens, {} entries",
            after.dir_opens - before.dir_opens,
            after.dir_entries - before.dir_entries
        );

        // And it still answers the same, which is what makes the zero above a saving
        // rather than a regression.
        assert_eq!(warm.tags_of(Path::new("debug.log")), vec!["gitignore"]);
        assert!(warm.tags_of(Path::new("src/keep.log")).is_empty());
        assert!(warm.tags_of(Path::new("src/main.rs")).is_empty());
    }

    #[test]
    fn supplied_type_rules_change_the_answer_and_invalidate_the_snapshot() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("main.rs"), b"fn main() {}\n");

        let default_config = OpenConfig {
            cache_path: Some(snapshot_path.clone()),
            analysis: content::AnalysisRequest {
                profile: content::AnalysisSet::NONE.with_lines(),
                ..content::AnalysisRequest::default()
            },
            ..OpenConfig::default()
        };
        let (index, _) = open(dir.path(), &default_config).expect("default open");
        assert_eq!(index.classify(Path::new("main.rs")).file_type.as_str(), "rust");
        drop(index);

        let mine = std::sync::Arc::new(
            classify::TypeRegistry::from_manifest(
                "[[kind]]\nid = \"notes\"\nfamily = \"prose\"\nextensions = [\"rs\"]\n",
            )
            .expect("a minimal manifest"),
        );
        let custom_config = OpenConfig {
            scan: scan::ScanConfig { types: Some(mine.clone()), ..scan::ScanConfig::default() },
            ..default_config.clone()
        };

        assert_ne!(
            custom_config.scan.scope(),
            default_config.scan.scope(),
            "different rules are a different scan scope"
        );

        let (index, report) = open(dir.path(), &custom_config).expect("custom open");
        assert_eq!(
            report.path_taken,
            OpenPath::ColdScan,
            "the snapshot was written under other rules and must not be reused"
        );
        assert_eq!(index.classify(Path::new("main.rs")).file_type.as_str(), "notes");
        assert_eq!(index.types().fingerprint(), mine.fingerprint());

        // And the snapshot the custom run wrote is reusable by a run under the same rules.
        let (_, report) = open(dir.path(), &custom_config).expect("second custom open");
        assert_eq!(report.path_taken, OpenPath::WarmRevalidate, "same rules, same snapshot");
    }

    /// The behaviour table from the design, asserted rather than described.
    #[test]
    fn each_cache_policy_reads_scans_and_writes_as_documented() {
        for (policy, expect_write) in [
            (CachePolicy::Auto, true),
            (CachePolicy::Refresh, true),
            (CachePolicy::ReadOnly, false),
            (CachePolicy::Off, false),
        ] {
            let dir = tempfile::tempdir().expect("tempdir");
            let cache = tempfile::tempdir().expect("cache dir");
            let snapshot_path = cache.path().join("snap.fdu");
            write_file(&dir.path().join("a.txt"), b"hello");

            let config = OpenConfig {
                cache_path: Some(snapshot_path.clone()),
                policy,
                ..OpenConfig::default()
            };
            let (index, report) = open(dir.path(), &config).expect("open");

            assert_eq!(index.total().files, 1, "{policy:?} lost an entry");
            assert_eq!(report.path_taken, OpenPath::ColdScan, "{policy:?} without a snapshot");
            assert_eq!(
                snapshot_path.exists(),
                expect_write,
                "{policy:?} wrote a snapshot: {}",
                snapshot_path.exists()
            );
        }
    }

    #[test]
    fn read_only_takes_the_warm_path_without_rewriting_the_snapshot() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        // Seed a snapshot with auto, then read it without leaving a trace.
        let seed = OpenConfig {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenConfig::default()
        };
        open(dir.path(), &seed).expect("seed");
        let before = fs::metadata(&snapshot_path).expect("snapshot exists").len();

        let read_only = OpenConfig { policy: CachePolicy::ReadOnly, ..seed };
        let (index, report) = open(dir.path(), &read_only).expect("warm open");
        assert_eq!(report.path_taken, OpenPath::WarmRevalidate);
        assert_eq!(index.total().files, 1);
        assert_eq!(fs::metadata(&snapshot_path).expect("still there").len(), before);
    }

    /// A verified warm open over an unchanged tree rewrote a byte-identical snapshot on
    /// every run, paying a full index clone, encode, and write to reproduce the file it
    /// had just read.  The bytes are the assertion: if a future change makes an
    /// unchanged reconciliation produce different serialized state, this fails loudly
    /// rather than letting the skip silently drop it.
    #[test]
    fn an_unchanged_warm_open_leaves_the_snapshot_alone_and_a_changed_one_rewrites_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");
        write_file(&dir.path().join("sub/b.txt"), b"world");

        let auto = OpenConfig {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenConfig::default()
        };
        open(dir.path(), &auto).expect("seed");
        let seeded = fs::read(&snapshot_path).expect("seeded snapshot");

        let (index, report) = open(dir.path(), &auto).expect("warm open");
        assert_eq!(report.path_taken, OpenPath::WarmRevalidate);
        assert_eq!(index.total().files, 2);
        assert_eq!(
            fs::read(&snapshot_path).expect("snapshot still there"),
            seeded,
            "an unchanged warm open must not rewrite the snapshot"
        );

        write_file(&dir.path().join("sub/c.txt"), b"new file");
        let (index, report) = open(dir.path(), &auto).expect("warm open after a change");
        assert_eq!(report.path_taken, OpenPath::WarmRevalidate);
        assert_eq!(index.total().files, 3);
        let after_add = fs::read(&snapshot_path).expect("rewritten snapshot");
        assert_ne!(after_add, seeded, "a warm open that found a new file must persist it");

        fs::remove_file(dir.path().join("sub/c.txt")).expect("remove");
        let (index, _) = open(dir.path(), &auto).expect("warm open after a removal");
        assert_eq!(index.total().files, 2);
        assert_ne!(
            fs::read(&snapshot_path).expect("rewritten snapshot"),
            after_add,
            "a warm open that found a removal must persist it"
        );

        // The skip must leave a snapshot a later cache-only open can still serve.
        let cache_only = OpenConfig { policy: CachePolicy::Only, ..auto };
        let (restored, _) = open(dir.path(), &cache_only).expect("cache-only open");
        assert_eq!(restored.total().files, 2);
    }

    #[test]
    fn refresh_ignores_an_existing_snapshot_and_rewrites_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        let auto = OpenConfig {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenConfig::default()
        };
        open(dir.path(), &auto).expect("seed");

        // A second auto open would be warm; refresh must scan cold anyway, which is what
        // makes it usable as a benchmark control.
        let refresh = OpenConfig { policy: CachePolicy::Refresh, ..auto };
        let (_, report) = open(dir.path(), &refresh).expect("refresh open");
        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert!(snapshot_path.exists());
    }

    #[test]
    fn cache_only_answers_from_the_snapshot_without_touching_the_tree() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        let auto = OpenConfig {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenConfig::default()
        };
        open(dir.path(), &auto).expect("seed");

        // Change the tree after the snapshot was taken. A cache-only answer must report
        // what it has, not what is there now — and its freshness must say so.
        write_file(&dir.path().join("b.txt"), b"new file");

        let only = OpenConfig { policy: CachePolicy::Only, ..auto };
        let (index, report) = open(dir.path(), &only).expect("cache-only open");
        assert_eq!(report.path_taken, OpenPath::CacheOnly);
        assert_eq!(index.total().files, 1, "the new file must not appear");
        assert_ne!(index.freshness(), Freshness::Fresh, "a stale answer must not claim currency");
    }

    #[test]
    fn cache_only_fails_closed_when_no_snapshot_is_usable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&dir.path().join("a.txt"), b"hello");

        // Guessing a scan here would make the fast path unpredictable: sometimes instant,
        // sometimes a full walk, with nothing in the output to say which happened.
        let only = OpenConfig {
            cache_path: Some(cache.path().join("absent.fdu")),
            policy: CachePolicy::Only,
            ..OpenConfig::default()
        };
        assert!(matches!(open(dir.path(), &only), Err(Error::Snapshot(_))));
    }

    #[test]
    fn content_sidecar_skips_unchanged_reads_and_serves_cache_only() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("notes.md"), b"one two\n");
        let analysis = content::AnalysisRequest {
            profile: content::AnalysisSet::NONE.with_lines(),
            ..content::AnalysisRequest::default()
        };
        let auto = OpenConfig {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            analysis,
            ..OpenConfig::default()
        };

        let (first, first_report) = open(dir.path(), &auto).expect("cold analyzed open");
        assert_eq!(first_report.analysis.expect("analysis").analyzed, 1);
        assert_eq!(
            first.content_rollup(Path::new("")).expect("content").total.metrics.raw_words,
            2
        );
        assert!(content::content_cache_path(&snapshot_path).exists());

        let (_, warm_report) = open(dir.path(), &auto).expect("warm analyzed open");
        assert_eq!(warm_report.content_cache.hits, 1);
        assert_eq!(warm_report.content_cache.bytes, 8);
        assert_eq!(warm_report.analysis.expect("analysis").candidates, 0);

        fs::remove_file(dir.path().join("notes.md")).expect("remove source");
        let only = OpenConfig { policy: CachePolicy::Only, ..auto };
        let (cached, cached_report) = open(dir.path(), &only).expect("cache-only content");
        assert_eq!(cached_report.content_cache.hits, 1);
        assert_eq!(cached_report.content_cache.bytes, 8);
        assert_eq!(
            cached.content_rollup(Path::new("")).expect("content").total.metrics.raw_words,
            2
        );
    }

    #[test]
    fn cached_coverage_exclusions_remain_visible_without_making_the_run_partial() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("invalid.txt"), b"valid prefix\xff");
        let auto = OpenConfig {
            cache_path: Some(snapshot_path),
            policy: CachePolicy::Auto,
            analysis: content::AnalysisRequest {
                profile: content::AnalysisSet::NONE.with_lines(),
                ..content::AnalysisRequest::default()
            },
            ..OpenConfig::default()
        };

        let (_, cold_report) = open(dir.path(), &auto).expect("cold analyzed open");
        assert!(cold_report.is_complete());
        assert_eq!(cold_report.analysis.expect("analysis").invalid_utf8, 1);
        assert!(cold_report.error_messages().is_empty());

        let (_, warm_report) = open(dir.path(), &auto).expect("warm analyzed open");
        assert_eq!(warm_report.content_cache.hits, 1);
        assert_eq!(warm_report.content_cache.coverage_exclusions, 1);
        assert_eq!(warm_report.analysis.expect("analysis").candidates, 0);
        assert!(warm_report.is_complete());
        assert!(warm_report.error_messages().is_empty());

        let only = OpenConfig { policy: CachePolicy::Only, ..auto };
        let (_, cached_report) = open(dir.path(), &only).expect("cache-only analyzed open");
        assert_eq!(cached_report.content_cache.coverage_exclusions, 1);
        assert!(cached_report.is_complete());
        assert!(cached_report.error_messages().is_empty());
    }

    #[test]
    fn cache_only_analysis_fails_closed_without_its_sidecar() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("notes.md"), b"one two\n");
        let metadata_only = OpenConfig {
            cache_path: Some(snapshot_path),
            policy: CachePolicy::Auto,
            ..OpenConfig::default()
        };
        open(dir.path(), &metadata_only).expect("seed metadata");

        let only = OpenConfig {
            policy: CachePolicy::Only,
            analysis: content::AnalysisRequest {
                profile: content::AnalysisSet::NONE.with_lines(),
                ..content::AnalysisRequest::default()
            },
            ..metadata_only
        };
        assert!(matches!(open(dir.path(), &only), Err(Error::Snapshot(_))));

        let analyzed = OpenConfig { policy: CachePolicy::Auto, ..only.clone() };
        open(dir.path(), &analyzed).expect("write an explicit empty content sidecar");
        let (cached, report) = open(dir.path(), &only).expect("restore empty analyzed state");
        assert!(report.content_cache.usable);
        assert_eq!(
            cached.content().and_then(content::ContentIndex::profile),
            Some(content::AnalysisSet::NONE.with_lines())
        );
    }

    #[test]
    fn cache_only_empty_analysis_still_requires_a_usable_sidecar() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        let metadata_only = OpenConfig {
            cache_path: Some(snapshot_path),
            policy: CachePolicy::Auto,
            ..OpenConfig::default()
        };
        open(dir.path(), &metadata_only).expect("seed empty metadata");

        let only = OpenConfig {
            policy: CachePolicy::Only,
            analysis: content::AnalysisRequest {
                profile: content::AnalysisSet::NONE.with_lines(),
                ..content::AnalysisRequest::default()
            },
            ..metadata_only
        };
        assert!(matches!(open(dir.path(), &only), Err(Error::Snapshot(_))));
    }

    #[test]
    fn a_snapshot_for_another_root_is_treated_as_absent() {
        let one = tempfile::tempdir().expect("tempdir");
        let two = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&one.path().join("a.txt"), b"hello");
        write_file(&two.path().join("b.txt"), b"other tree");

        let config = OpenConfig {
            cache_path: Some(snapshot_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenConfig::default()
        };
        open(one.path(), &config).expect("seed from the first root");

        // Reading another tree's snapshot would be worse than a cache miss.
        let (index, report) = open(two.path(), &config).expect("second root");
        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert_eq!(index.total().files, 1);
        assert_eq!(index.root_path(), two.path().canonicalize().expect("canonical").as_path());
    }

    #[test]
    fn open_without_a_cache_always_scans_cold() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_file(&dir.path().join("a.txt"), b"hello");

        let (index, report) = open(dir.path(), &OpenConfig::default()).expect("open");
        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert_eq!(index.total().files, 1);
    }

    #[test]
    fn second_open_takes_the_warm_path_and_stays_correct() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&dir.path().join("a.txt"), b"hello");
        write_file(&dir.path().join("src/main.rs"), b"fn main() {}");

        let config = OpenConfig {
            cache_path: Some(cache.path().join("snap.fdu")),
            policy: CachePolicy::Auto,
            ..OpenConfig::default()
        };

        let (first, first_report) = open(dir.path(), &config).expect("cold open");
        assert_eq!(first_report.path_taken, OpenPath::ColdScan);
        assert_eq!(first.total().files, 2);

        // Change the tree between opens: the warm path must notice.
        write_file(&dir.path().join("added.md"), b"new");
        fs::remove_file(dir.path().join("a.txt")).expect("remove");

        let (second, second_report) = open(dir.path(), &config).expect("warm open");
        assert_eq!(second_report.path_taken, OpenPath::WarmRevalidate);
        assert_eq!(second.total().files, 2);
        assert!(second.lookup(Path::new("added.md")).is_some());
        assert!(second.lookup(Path::new("a.txt")).is_none());
    }

    #[test]
    fn a_snapshot_from_another_root_is_ignored() {
        let a = tempfile::tempdir().expect("tempdir a");
        let b = tempfile::tempdir().expect("tempdir b");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&a.path().join("only-in-a.txt"), b"x");
        write_file(&b.path().join("only-in-b.txt"), b"y");

        let cache_path = cache.path().join("snap.fdu");
        let config = OpenConfig {
            cache_path: Some(cache_path),
            policy: CachePolicy::Auto,
            ..OpenConfig::default()
        };

        open(a.path(), &config).expect("open a");
        let (index, report) = open(b.path(), &config).expect("open b");

        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert!(index.lookup(Path::new("only-in-b.txt")).is_some());
        assert!(index.lookup(Path::new("only-in-a.txt")).is_none());
    }

    #[test]
    fn snapshot_scope_mismatch_forces_a_cold_scan() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&dir.path().join("top.txt"), b"top");
        write_file(&dir.path().join("deep/nested.txt"), b"nested");

        let cache_path = cache.path().join("snap.fdu");
        let full = OpenConfig {
            cache_path: Some(cache_path.clone()),
            policy: CachePolicy::Auto,
            ..OpenConfig::default()
        };
        open(dir.path(), &full).expect("full open");

        let shallow = OpenConfig {
            scan: ScanConfig { max_depth: Some(1), ..ScanConfig::default() },
            cache_path: Some(cache_path),
            policy: CachePolicy::ReadOnly,
            analysis: content::AnalysisRequest::default(),
        };
        let (index, report) = open(dir.path(), &shallow).expect("shallow open");

        assert_eq!(report.path_taken, OpenPath::ColdScan);
        assert!(index.lookup(Path::new("deep")).is_some());
        assert!(index.lookup(Path::new("deep/nested.txt")).is_none());
    }

    #[test]
    fn operational_batch_size_does_not_invalidate_a_snapshot() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        write_file(&dir.path().join("a.txt"), b"a");
        let cache_path = cache.path().join("snap.fdu");

        let first = OpenConfig {
            scan: ScanConfig { batch_size: 1, ..ScanConfig::default() },
            cache_path: Some(cache_path.clone()),
            policy: CachePolicy::Auto,
            analysis: content::AnalysisRequest::default(),
        };
        open(dir.path(), &first).expect("first open");

        let second = OpenConfig {
            scan: ScanConfig { batch_size: 17, ..ScanConfig::default() },
            cache_path: Some(cache_path),
            policy: CachePolicy::ReadOnly,
            analysis: content::AnalysisRequest::default(),
        };
        let (_, report) = open(dir.path(), &second).expect("second open");
        assert_eq!(report.path_taken, OpenPath::WarmRevalidate);
    }

    #[test]
    fn cache_paths_differ_per_root() {
        let a = tempfile::tempdir().expect("tempdir a");
        let b = tempfile::tempdir().expect("tempdir b");
        let (Some(pa), Some(pb)) = (default_cache_path(a.path()), default_cache_path(b.path()))
        else {
            return; // No HOME in this environment; nothing to assert.
        };
        assert_ne!(pa, pb);
        assert_eq!(pa, default_cache_path(a.path()).expect("stable"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_cache_discovery_prefers_native_locations() {
        let local = OsString::from(r"C:\Users\tester\AppData\Local");
        let profile = OsString::from(r"D:\Profile");
        let home = OsString::from(r"E:\Home");

        assert_eq!(
            windows_cache_dir(Some(local.clone()), Some(profile.clone()), Some(home.clone())),
            Some(PathBuf::from(local))
        );
        assert_eq!(
            windows_cache_dir(None, Some(profile.clone()), Some(home.clone())),
            Some(PathBuf::from(profile).join("AppData").join("Local"))
        );
        assert_eq!(
            windows_cache_dir(None, None, Some(home.clone())),
            Some(PathBuf::from(home).join(".cache"))
        );
        assert_eq!(windows_cache_dir(None, None, None), None);
    }
}

#[cfg(test)]
mod save_tests {
    use super::*;
    use std::fs;

    fn write_file(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write");
    }

    fn config(snapshot_path: &Path, policy: CachePolicy) -> OpenConfig {
        OpenConfig {
            cache_path: Some(snapshot_path.to_path_buf()),
            policy,
            ..OpenConfig::default()
        }
    }

    #[test]
    fn a_pending_save_completes_when_joined() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        let (_index, _report, pending) =
            open_with_pending_save(dir.path(), &config(&snapshot_path, CachePolicy::Auto))
                .expect("open");
        pending.join().expect("save succeeds");
        assert!(snapshot_path.exists(), "a joined save must have landed");
    }

    #[test]
    fn a_dropped_save_still_lands() {
        // Dropping without joining is a caller mistake, not a reason to lose the write:
        // the next run would otherwise pay for a cold scan this one already did.
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        {
            let (_index, _report, _pending) =
                open_with_pending_save(dir.path(), &config(&snapshot_path, CachePolicy::Auto))
                    .expect("open");
        }
        assert!(snapshot_path.exists());
    }

    #[test]
    #[cfg(unix)]
    fn a_partial_scan_leaves_the_previous_snapshot_alone() {
        // Writing a partial view would serve it as fact on the next run, and the
        // existing complete snapshot is better than that.
        use std::os::unix::fs::PermissionsExt;

        if !crate::test_support::permission_bits_are_enforced() {
            // A privileged process reads the denied directory anyway, so the fixture
            // cannot produce the partial scan this asserts on.
            eprintln!("skipped: this process is not subject to Unix permission bits");
            return;
        }

        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        let settings = config(&snapshot_path, CachePolicy::Auto);
        open(dir.path(), &settings).expect("seed a complete snapshot");
        let complete_len = fs::metadata(&snapshot_path).expect("exists").len();

        let denied = dir.path().join("denied");
        fs::create_dir(&denied).expect("create");
        write_file(&denied.join("hidden.txt"), b"hidden");
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).expect("deny");

        let opened = open(dir.path(), &settings);
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o700)).expect("restore");
        let (_index, report) = opened.expect("partial open still returns a result");

        assert!(!report.is_complete(), "the scan should be partial");
        assert_eq!(
            fs::metadata(&snapshot_path).expect("still there").len(),
            complete_len,
            "a partial scan must not overwrite a complete snapshot"
        );
    }

    #[test]
    fn no_snapshot_is_written_when_the_policy_forbids_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot_path = cache.path().join("snap.fdu");
        write_file(&dir.path().join("a.txt"), b"hello");

        for policy in [CachePolicy::ReadOnly, CachePolicy::Off] {
            let (_index, _report, pending) =
                open_with_pending_save(dir.path(), &config(&snapshot_path, policy)).expect("open");
            pending.join().expect("nothing to join");
            assert!(!snapshot_path.exists(), "{policy:?} wrote a snapshot");
        }
    }
}

#[cfg(test)]
mod cold_scan_persistence_tests {
    use super::*;

    fn config(policy: CachePolicy, profile: content::AnalysisSet) -> OpenConfig {
        OpenConfig {
            policy,
            analysis: content::AnalysisRequest { profile, ..Default::default() },
            ..OpenConfig::default()
        }
    }

    #[test]
    fn analysis_always_persists_because_rereading_bodies_is_the_expensive_half() {
        // The measured case for the cache: 639 ms to 325 ms warm. Size is irrelevant
        // here, so even a tiny tree under a large threshold still writes.
        let analyzed = config(CachePolicy::Auto, content::AnalysisSet::NONE.with_code());
        let targets = cold_scan_save_targets_with(1, &analyzed, Some(250_000));
        assert!(targets.metadata && targets.content, "analysis must persist its sidecar");
    }

    #[test]
    fn refresh_persists_because_the_caller_asked_for_it_outright() {
        let refresh = config(CachePolicy::Refresh, content::AnalysisSet::NONE);
        let targets = cold_scan_save_targets_with(1, &refresh, Some(250_000));
        assert!(targets.metadata, "refresh is an explicit instruction, not a cost estimate");
    }

    #[test]
    fn a_metadata_scan_persists_on_either_side_of_an_enabled_threshold() {
        // Both branches are asserted here rather than through the shipped constant, so
        // the gate stays proven while SNAPSHOT_MIN_ENTRIES is still unset.
        let plain = config(CachePolicy::Auto, content::AnalysisSet::NONE);
        let below = cold_scan_save_targets_with(9, &plain, Some(10));
        assert!(below.none(), "a tree below the threshold should not create a snapshot");

        let at = cold_scan_save_targets_with(10, &plain, Some(10));
        assert!(at.metadata, "the threshold is inclusive");
    }

    #[test]
    fn an_unset_threshold_preserves_the_behaviour_every_release_has_shipped() {
        // SNAPSHOT_MIN_ENTRIES stays None because APFS measurement argued against a
        // threshold rather than supplying one (fdu-hvs5): there the snapshot repays its
        // write on the first later `Only` read at any size. A cold metadata scan must
        // persist exactly as before, so `fdu PATH` keeps leaving a snapshot behind.
        let plain = config(CachePolicy::Auto, content::AnalysisSet::NONE);
        assert!(
            cold_scan_save_targets_with(1, &plain, None).metadata,
            "an unset threshold must not change persistence"
        );
        assert_eq!(
            SNAPSHOT_MIN_ENTRIES, None,
            "enabling this is a product decision, not a default"
        );
    }
}
