//! Low-distortion performance instrumentation for fdu.
//!
//! The subsystem has two complementary tiers:
//!
//! - application counters attribute logical work to the filesystem, memory, and index
//!   layers;
//! - [`process`] samples kernel counters at phase boundaries and exposes only metrics
//!   supported by the current platform.
//!
//! Recording is compiled in, off by default, and enabled with `FDU_COUNTERS=1` by the
//! shipped CLI and the repository performance probe. Per-event updates stay thread-local;
//! workers fold their counts on exit, while a drop fallback preserves counts from callers
//! that do not install an explicit worker guard.

use std::cell::Cell;
use std::ffi::OsStr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub mod alloc;
pub mod process;

static ENABLED: AtomicBool = AtomicBool::new(false);

/// Per-layer tallies for a walk, index build, and content pass.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    /// Logical directory-open operations.
    pub dir_opens: u64,
    /// Directory entries yielded by enumeration.
    pub dir_entries: u64,
    /// Logical metadata-stat operations.
    pub stats: u64,
    /// File-open operations performed by content analysis.
    pub file_opens: u64,
    /// File read calls performed by content analysis.
    pub file_reads: u64,
    /// Bytes read from files by content analysis.
    pub bytes_read: u64,
    /// Allocation requests observed by the installed counting allocator.
    pub allocs: u64,
    /// Reallocation requests observed by the installed counting allocator.
    pub reallocs: u64,
    /// Deallocations observed by the installed counting allocator.
    pub frees: u64,
    /// Bytes requested by allocations and allocation growth.
    pub bytes_allocated: u64,
    /// Index upserts applied.
    pub upserts: u64,
    /// Parent-path resolutions performed for index updates.
    pub parent_resolutions: u64,
    /// Parent resolutions served by the adjacent-parent memo.
    pub parent_memo_hits: u64,
    /// Roll-up merge operations.
    pub rollup_merges: u64,
    /// Index entries allocated.
    pub entries_allocated: u64,
}

impl Counts {
    const ZERO: Self = Self {
        dir_opens: 0,
        dir_entries: 0,
        stats: 0,
        file_opens: 0,
        file_reads: 0,
        bytes_read: 0,
        allocs: 0,
        reallocs: 0,
        frees: 0,
        bytes_allocated: 0,
        upserts: 0,
        parent_resolutions: 0,
        parent_memo_hits: 0,
        rollup_merges: 0,
        entries_allocated: 0,
    };

    /// Every counter as `(group, label, value)`, in report order.
    #[must_use]
    pub fn rows(&self) -> Vec<(&'static str, &'static str, u64)> {
        vec![
            ("filesystem operations", "directory opens", self.dir_opens),
            ("filesystem operations", "directory entries enumerated", self.dir_entries),
            ("filesystem operations", "metadata stats", self.stats),
            ("filesystem operations", "file opens", self.file_opens),
            ("filesystem operations", "file read calls", self.file_reads),
            ("filesystem operations", "bytes read from files", self.bytes_read),
            ("memory", "allocations", self.allocs),
            ("memory", "reallocations", self.reallocs),
            ("memory", "frees", self.frees),
            ("memory", "bytes allocated", self.bytes_allocated),
            ("index", "upserts applied", self.upserts),
            ("index", "parent path resolutions", self.parent_resolutions),
            ("index", "parent memo hits", self.parent_memo_hits),
            ("index", "roll-up merges", self.rollup_merges),
            ("index", "index entries allocated", self.entries_allocated),
        ]
    }

    /// Whether every counter is zero.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Fold another set into this one without panicking on overflow.
    fn add(&mut self, other: &Self) {
        self.dir_opens = self.dir_opens.saturating_add(other.dir_opens);
        self.dir_entries = self.dir_entries.saturating_add(other.dir_entries);
        self.stats = self.stats.saturating_add(other.stats);
        self.file_opens = self.file_opens.saturating_add(other.file_opens);
        self.file_reads = self.file_reads.saturating_add(other.file_reads);
        self.bytes_read = self.bytes_read.saturating_add(other.bytes_read);
        self.allocs = self.allocs.saturating_add(other.allocs);
        self.reallocs = self.reallocs.saturating_add(other.reallocs);
        self.frees = self.frees.saturating_add(other.frees);
        self.bytes_allocated = self.bytes_allocated.saturating_add(other.bytes_allocated);
        self.upserts = self.upserts.saturating_add(other.upserts);
        self.parent_resolutions = self.parent_resolutions.saturating_add(other.parent_resolutions);
        self.parent_memo_hits = self.parent_memo_hits.saturating_add(other.parent_memo_hits);
        self.rollup_merges = self.rollup_merges.saturating_add(other.rollup_merges);
        self.entries_allocated = self.entries_allocated.saturating_add(other.entries_allocated);
    }

    /// Per-event ratios against a denominator, in report order.
    #[must_use]
    pub fn per(&self, denominator: u64) -> Vec<(&'static str, &'static str, f64)> {
        self.rows()
            .into_iter()
            .map(|(group, label, value)| (group, label, ratio(value, denominator)))
            .collect()
    }
}

struct GlobalCounts {
    dir_opens: AtomicU64,
    dir_entries: AtomicU64,
    stats: AtomicU64,
    file_opens: AtomicU64,
    file_reads: AtomicU64,
    bytes_read: AtomicU64,
    allocs: AtomicU64,
    reallocs: AtomicU64,
    frees: AtomicU64,
    bytes_allocated: AtomicU64,
    upserts: AtomicU64,
    parent_resolutions: AtomicU64,
    parent_memo_hits: AtomicU64,
    rollup_merges: AtomicU64,
    entries_allocated: AtomicU64,
}

impl GlobalCounts {
    const fn new() -> Self {
        Self {
            dir_opens: AtomicU64::new(0),
            dir_entries: AtomicU64::new(0),
            stats: AtomicU64::new(0),
            file_opens: AtomicU64::new(0),
            file_reads: AtomicU64::new(0),
            bytes_read: AtomicU64::new(0),
            allocs: AtomicU64::new(0),
            reallocs: AtomicU64::new(0),
            frees: AtomicU64::new(0),
            bytes_allocated: AtomicU64::new(0),
            upserts: AtomicU64::new(0),
            parent_resolutions: AtomicU64::new(0),
            parent_memo_hits: AtomicU64::new(0),
            rollup_merges: AtomicU64::new(0),
            entries_allocated: AtomicU64::new(0),
        }
    }

    fn add(&self, counts: &Counts) {
        atomic_saturating_add(&self.dir_opens, counts.dir_opens);
        atomic_saturating_add(&self.dir_entries, counts.dir_entries);
        atomic_saturating_add(&self.stats, counts.stats);
        atomic_saturating_add(&self.file_opens, counts.file_opens);
        atomic_saturating_add(&self.file_reads, counts.file_reads);
        atomic_saturating_add(&self.bytes_read, counts.bytes_read);
        atomic_saturating_add(&self.allocs, counts.allocs);
        atomic_saturating_add(&self.reallocs, counts.reallocs);
        atomic_saturating_add(&self.frees, counts.frees);
        atomic_saturating_add(&self.bytes_allocated, counts.bytes_allocated);
        atomic_saturating_add(&self.upserts, counts.upserts);
        atomic_saturating_add(&self.parent_resolutions, counts.parent_resolutions);
        atomic_saturating_add(&self.parent_memo_hits, counts.parent_memo_hits);
        atomic_saturating_add(&self.rollup_merges, counts.rollup_merges);
        atomic_saturating_add(&self.entries_allocated, counts.entries_allocated);
    }

    fn snapshot(&self) -> Counts {
        Counts {
            dir_opens: self.dir_opens.load(Ordering::Relaxed),
            dir_entries: self.dir_entries.load(Ordering::Relaxed),
            stats: self.stats.load(Ordering::Relaxed),
            file_opens: self.file_opens.load(Ordering::Relaxed),
            file_reads: self.file_reads.load(Ordering::Relaxed),
            bytes_read: self.bytes_read.load(Ordering::Relaxed),
            allocs: self.allocs.load(Ordering::Relaxed),
            reallocs: self.reallocs.load(Ordering::Relaxed),
            frees: self.frees.load(Ordering::Relaxed),
            bytes_allocated: self.bytes_allocated.load(Ordering::Relaxed),
            upserts: self.upserts.load(Ordering::Relaxed),
            parent_resolutions: self.parent_resolutions.load(Ordering::Relaxed),
            parent_memo_hits: self.parent_memo_hits.load(Ordering::Relaxed),
            rollup_merges: self.rollup_merges.load(Ordering::Relaxed),
            entries_allocated: self.entries_allocated.load(Ordering::Relaxed),
        }
    }

    fn reset(&self) {
        self.dir_opens.store(0, Ordering::Relaxed);
        self.dir_entries.store(0, Ordering::Relaxed);
        self.stats.store(0, Ordering::Relaxed);
        self.file_opens.store(0, Ordering::Relaxed);
        self.file_reads.store(0, Ordering::Relaxed);
        self.bytes_read.store(0, Ordering::Relaxed);
        self.allocs.store(0, Ordering::Relaxed);
        self.reallocs.store(0, Ordering::Relaxed);
        self.frees.store(0, Ordering::Relaxed);
        self.bytes_allocated.store(0, Ordering::Relaxed);
        self.upserts.store(0, Ordering::Relaxed);
        self.parent_resolutions.store(0, Ordering::Relaxed);
        self.parent_memo_hits.store(0, Ordering::Relaxed);
        self.rollup_merges.store(0, Ordering::Relaxed);
        self.entries_allocated.store(0, Ordering::Relaxed);
    }
}

static GLOBAL: GlobalCounts = GlobalCounts::new();

struct LocalCounts {
    counts: Cell<Counts>,
}

impl LocalCounts {
    const fn new() -> Self {
        Self { counts: Cell::new(Counts::ZERO) }
    }
}

impl Drop for LocalCounts {
    fn drop(&mut self) {
        GLOBAL.add(&self.counts.replace(Counts::default()));
    }
}

std::thread_local! {
    // This non-dropping, const-initialized guard is accessed before `LOCAL`. If first
    // access to a dropping TLS key causes an allocation on a platform, the allocator's
    // nested callback sees `true` and takes the atomic fallback instead of recursing.
    static IN_COUNTER: Cell<bool> = const { Cell::new(false) };
    static LOCAL: LocalCounts = const { LocalCounts::new() };
}

struct ReentryGuard<'a>(&'a Cell<bool>);

impl<'a> ReentryGuard<'a> {
    fn enter(cell: &'a Cell<bool>) -> Option<Self> {
        if cell.replace(true) { None } else { Some(Self(cell)) }
    }
}

impl Drop for ReentryGuard<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

/// A guard that deterministically folds one worker's counters before it returns.
pub(crate) struct ThreadFlushGuard;

impl Drop for ThreadFlushGuard {
    fn drop(&mut self) {
        flush_thread();
    }
}

/// Begin a worker lifetime whose thread-local counters must be visible after join.
pub(crate) const fn thread_flush_guard() -> ThreadFlushGuard {
    ThreadFlushGuard
}

/// One opt-in instrumentation interval for a binary invocation.
///
/// Construct this before the measured operation and call [`Self::finish`] afterward.
/// When recording is off, construction performs no process sampling and `finish`
/// returns `None`.
#[must_use]
pub struct Measurement {
    process_before: Option<process::Snapshot>,
}

impl Measurement {
    /// Read `FDU_COUNTERS` and begin an enabled interval when it is truthy.
    pub fn from_env() -> Self {
        if !enable_from_env() {
            return Self { process_before: None };
        }

        // The boundary sample can allocate on Linux while parsing `/proc`. Take it first
        // and reset afterward so the application tier describes fdu's work, not its
        // measuring instrument.
        let process_before = process::Snapshot::now();
        reset();
        Self { process_before: Some(process_before) }
    }

    /// Finish the interval and render application and supported process counters.
    #[must_use]
    pub fn finish(mut self) -> Option<String> {
        let before = self.process_before.take()?;
        flush_thread();
        let counts = snapshot();
        let process = process::Snapshot::now().since(&before);
        enable(false);
        Some(render(&counts, &process))
    }
}

impl Drop for Measurement {
    fn drop(&mut self) {
        if self.process_before.is_some() {
            enable(false);
        }
    }
}

/// Turn recording on or off for the whole process.
pub fn enable(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

/// Turn recording on when `FDU_COUNTERS` contains a truthy value.
///
/// Empty strings, `0`, `false`, `no`, and `off` (case-insensitive) disable recording.
/// The parsed state is always applied, including a transition from enabled to disabled.
pub fn enable_from_env() -> bool {
    enable_from_value(std::env::var_os("FDU_COUNTERS").as_deref())
}

fn enable_from_value(value: Option<&OsStr>) -> bool {
    let on = value.is_some_and(|value| {
        value.to_str().is_none_or(|text| {
            let text = text.trim();
            !(text.is_empty()
                || text == "0"
                || text.eq_ignore_ascii_case("false")
                || text.eq_ignore_ascii_case("no")
                || text.eq_ignore_ascii_case("off"))
        })
    });
    enable(on);
    on
}

/// Whether recording is currently on.
#[inline]
#[must_use]
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Add to one or more counters on the calling thread.
///
/// The fallback path is important for allocation callbacks: it avoids re-entering a TLS
/// initializer and remains usable while a thread's local counter has begun destruction.
#[inline]
pub fn bump(f: impl FnOnce(&mut Counts)) {
    if !enabled() {
        return;
    }

    let mut f = Some(f);
    let stored_locally = IN_COUNTER
        .try_with(|reentry| {
            let Some(_guard) = ReentryGuard::enter(reentry) else { return false };
            LOCAL
                .try_with(|local| {
                    if let Some(update) = f.take() {
                        let mut counts = local.counts.get();
                        update(&mut counts);
                        local.counts.set(counts);
                    }
                })
                .is_ok()
        })
        .unwrap_or(false);

    if !stored_locally {
        let mut delta = Counts::default();
        if let Some(update) = f {
            update(&mut delta);
            GLOBAL.add(&delta);
        }
    }
}

/// Fold this thread's counts into the process total and clear them.
pub fn flush_thread() {
    let _ = LOCAL.try_with(|local| GLOBAL.add(&local.counts.replace(Counts::default())));
}

/// Process-wide totals, including the calling thread's unflushed counts.
#[must_use]
pub fn snapshot() -> Counts {
    let mut total = GLOBAL.snapshot();
    let _ = LOCAL.try_with(|local| total.add(&local.counts.get()));
    total
}

/// Clear process totals and the calling thread's local counts.
///
/// Call only when other instrumented workers are quiescent: another thread may still
/// hold local counts that a process-global reset cannot reach.
pub fn reset() {
    let _ = LOCAL.try_with(|local| local.counts.set(Counts::default()));
    GLOBAL.reset();
}

/// Create fdu's certified system-allocator wrapper for a binary's global allocator.
#[must_use]
pub const fn system_allocator() -> alloc::CountingAlloc<std::alloc::System> {
    alloc::CountingAlloc::system(alloc::fdu_sinks())
}

/// Render application counters followed by the supported kernel process counters.
#[must_use]
pub fn render(counts: &Counts, process: &process::Snapshot) -> String {
    let mut out = render_rows(&counts.rows());
    out.push('\n');
    out.push_str(&process.render());
    out
}

fn record_alloc(size: u64) {
    bump(|counts| {
        counts.allocs = counts.allocs.saturating_add(1);
        counts.bytes_allocated = counts.bytes_allocated.saturating_add(size);
    });
}

fn record_realloc(growth: u64) {
    bump(|counts| {
        counts.reallocs = counts.reallocs.saturating_add(1);
        counts.bytes_allocated = counts.bytes_allocated.saturating_add(growth);
    });
}

fn record_dealloc() {
    bump(|counts| counts.frees = counts.frees.saturating_add(1));
}

fn atomic_saturating_add(target: &AtomicU64, value: u64) {
    if value == 0 {
        return;
    }
    let _ = target.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_add(value))
    });
}

#[allow(clippy::cast_precision_loss)]
fn ratio(numerator: u64, denominator: u64) -> f64 {
    let denominator = if denominator == 0 { 1.0 } else { denominator as f64 };
    numerator as f64 / denominator
}

fn render_rows(rows: &[(&str, &str, u64)]) -> String {
    use std::fmt::Write as _;

    let width = rows.iter().map(|(_, label, _)| label.len()).max().unwrap_or(0);
    let mut output = String::new();
    let mut previous = None;
    for (group, label, value) in rows {
        if previous != Some(*group) {
            if previous.is_some() {
                output.push('\n');
            }
            let _ = writeln!(output, "[{group}]");
            previous = Some(*group);
        }
        let _ = writeln!(output, "  {label:<width$}  {value:>13}");
    }
    output
}

/// Serialize tests that touch process-global counter state.
#[cfg(test)]
pub(crate) fn test_serial() -> std::sync::MutexGuard<'static, ()> {
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
    SERIAL.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_exiting_thread_folds_without_a_manual_flush() {
        let _serial = test_serial();
        enable(true);
        reset();

        std::thread::spawn(|| bump(|counts| counts.dir_opens = 7)).join().expect("counting worker");

        // Other ordinary tests can perform instrumented work while this process-wide
        // toggle is on, so only the worker's lower bound is deterministic here.
        assert!(snapshot().dir_opens >= 7);
        reset();
        enable(false);
    }

    #[test]
    fn a_falsey_value_disables_a_previously_enabled_process() {
        let _serial = test_serial();
        enable(true);

        assert!(!enable_from_value(Some(OsStr::new("FALSE"))));
        assert!(!enabled());
    }

    #[test]
    fn ratios_treat_a_zero_denominator_as_one() {
        let counts = Counts { allocs: 10, ..Counts::default() };
        let allocations = counts
            .per(0)
            .into_iter()
            .find(|(_, label, _)| *label == "allocations")
            .expect("allocation row");
        assert!((allocations.2 - 10.0).abs() < f64::EPSILON);
    }
}
