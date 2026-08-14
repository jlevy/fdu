//! fdu's per-layer operation counters.
//!
//! The mechanism — thread-local storage, the global fold, the runtime toggle, the
//! counting allocator — lives in [`perfkit`], which is domain-neutral and reusable. This
//! module supplies only the names, because what is worth counting is the one part that
//! cannot be generic.
//!
//! # Reading them
//!
//! Recording is off until something calls [`perfkit::enable`], so an ordinary `fdu` run
//! pays one predictable branch per site and nothing else. The performance probe turns it
//! on; `FDU_COUNTERS=1` turns it on for the CLI.
//!
//! # Which tier answers which question
//!
//! These are the *application* tier: they attribute work to a layer, which is what tells
//! you where to change code. They count what fdu believes it did.
//!
//! [`perfkit::process`] is the *process* tier: real kernel numbers that cannot be
//! fooled but cannot attribute. Cross-check one against the other; when they disagree,
//! or when neither can name a call site, reach for callgrind or `strace -c`.
//!
//! One coverage limit is worth repeating here because it has already misled once:
//! `/proc/self/io`'s `syscr` counts the read family only. A walk over 17,128 entries,
//! every one a `getdents64` or `statx`, moved it by 30. Enumeration and stat counts have
//! no cheap kernel source, so [`walk`]'s counters are the instrument for those and
//! `strace -c` is the periodic ground truth.

perfkit::counters! {
    /// Per-layer tallies for a walk, an index build, and a content pass.
    pub mod walk {
        // The filesystem layer. `dir_opens` counts `read_dir` calls and `dir_entries`
        // what they yielded. The underlying `getdents64` batch count is deliberately
        // absent: std's `ReadDir` does not expose it, and a counter pinned to zero
        // misleads worse than a missing one.
        dir_opens: "directory opens", "syscalls";
        dir_entries: "directory entries enumerated", "syscalls";
        stats: "metadata stats", "syscalls";
        file_opens: "file opens", "syscalls";
        file_reads: "file read calls", "syscalls";
        bytes_read: "bytes read from files", "syscalls";

        // The memory layer, filled in by the counting allocator a binary installs.
        // Zero in a library test, where the harness owns the global allocator.
        allocs: "allocations", "memory";
        reallocs: "reallocations", "memory";
        frees: "frees", "memory";
        bytes_allocated: "bytes allocated", "memory";

        // The index layer: work the consumer did, as opposed to time it took. The
        // parent-resolution split is what makes exp-051's memo legible without a
        // profiler.
        upserts: "upserts applied", "index";
        parent_resolutions: "parent path resolutions", "index";
        parent_memo_hits: "parent memo hits", "index";
        rollup_merges: "roll-up merges", "index";
        entries_allocated: "index entries allocated", "index";
    }
}

/// Turn recording on when `FDU_COUNTERS` is set to anything but a falsey spelling.
///
/// Lets a shipped binary be asked for numbers without a flag of its own, which is the
/// difference between "rebuild to see anything" and "ask the run you already have".
pub fn enable_from_env() -> bool {
    perfkit::enable_from_env("FDU_COUNTERS")
}

/// Whether recording is on.
#[must_use]
pub fn enabled() -> bool {
    perfkit::is_enabled()
}

/// Add to one or more counters on the calling thread.
pub fn bump(f: impl FnOnce(&mut walk::Counts)) {
    walk::bump(f);
}

/// Fold this thread's counts into the process total.
pub fn flush_thread() {
    walk::flush_thread();
}

/// Process-wide totals, including the calling thread's unflushed counts.
#[must_use]
pub fn snapshot() -> walk::Counts {
    walk::snapshot()
}

/// Clear every counter.
pub fn reset() {
    walk::reset();
}

/// Both tiers rendered together: application counters, then kernel process counters.
///
/// Printing them adjacent is the point — the cross-check is the discipline, and a report
/// that shows one tier invites trusting it alone.
#[must_use]
pub fn render(counts: &walk::Counts, process: &perfkit::process::Snapshot) -> String {
    let mut out = perfkit::render_rows(&counts.rows());
    out.push('\n');
    out.push_str(&process.render());
    out
}

/// Serialize tests that touch the process-global counters.
///
/// The counters and the enable flag are process-wide, which is what lets any layer
/// increment them without a context object threaded through it. The cost is that
/// concurrent tests interfere, so every test that reads absolute totals takes this
/// first.
#[cfg(test)]
pub(crate) fn test_serial() -> std::sync::MutexGuard<'static, ()> {
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
    SERIAL.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}
