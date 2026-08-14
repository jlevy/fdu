//! Per-layer tallies of the operations that decide how long a run takes.
//!
//! # Why this exists
//!
//! The experiment ledger is full of results that were hard to interpret because the
//! measurement said *how long* something took without saying *what it did*. Two
//! examples, both recent enough to still sting:
//!
//! - The allocator was about 35% of the engine work in a cold scan, and the only way
//!   that became visible was a callgrind run — a fifty-times-slower tool whose profile
//!   then had to have the probe's own oracle digest subtracted from it by hand.
//! - `exp-051` predicted 15% on a job and delivered 7.35%, because the prediction
//!   described the index-build *component* while the accept rule reads *wall*. Nothing
//!   in the measurement made that distinction visible until the harness happened to
//!   report a component number alongside the wall.
//!
//! Both are visibility problems, not measurement problems.
//! A counter that says "this run made 47,318 `statx` calls and 1.4M allocations" turns
//! the next such question into arithmetic instead of a profiling session.
//!
//! # The cost rule
//!
//! Instrumentation that distorts what it measures is worse than none, because it is
//! believed. Three things keep this cheap enough to leave on:
//!
//! 1. **Counters are thread-local and non-atomic.** Incrementing is a load, an add, and
//!    a store to a thread's own cache line. There is no contention to measure, which
//!    matters because the walk is the parallel part and a shared atomic counter there
//!    would be measuring the counter.
//! 2. **Counts, never timers, on per-entry paths.** Reading a clock costs an order of
//!    magnitude more than incrementing a `u64`, so per-entry work is counted and only
//!    per-batch or per-phase spans are timed. This is the same chunk-amortization rule
//!    [`crate::scan::WalkAttribution`] already follows.
//! 3. **The overhead is measured, not asserted.** See
//!    `docs/project/experiments/exp-052-*`: the paired cost of leaving these on is
//!    recorded there, and if it ever exceeds what the accept rule can tolerate, the
//!    `perf-counters` feature turns the whole module into nothing.
//!
//! # Reading them
//!
//! Counters accumulate per thread and are folded into a process-wide total by
//! [`snapshot`]. A walk's worker threads fold their own totals as they finish, so a
//! snapshot taken after a scan sees every thread's work.
//!
//! Nothing here is part of the engine's contract: counters describe an implementation,
//! and a change that makes a number go away is free to do so.

use std::cell::Cell;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

/// One tally, named so that the report reads as a sentence about the run.
///
/// Ordering matters only for the rendered report; adding a field is additive.
macro_rules! define_counters {
    ($($field:ident: $label:literal, $group:literal;)+) => {
        /// A set of counts, either one thread's or the whole process's.
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
        pub struct Counts {
            $(
                #[doc = $label]
                pub $field: u64,
            )+
        }

        impl Counts {
            /// Every counter as `(group, label, value)`, for rendering.
            pub fn rows(&self) -> Vec<(&'static str, &'static str, u64)> {
                vec![$(($group, $label, self.$field),)+]
            }

            /// Whether anything was counted at all, which distinguishes "this run did no
            /// syscalls" from "this build has counters compiled out".
            pub fn is_empty(&self) -> bool {
                $(self.$field == 0 &&)+ true
            }

            fn add(&mut self, other: &Self) {
                $(self.$field += other.$field;)+
            }
        }

        /// The process-wide fold of every thread that has reported.
        mod global {
            use super::AtomicU64;
            $(
                #[allow(non_upper_case_globals)]
                pub(super) static $field: AtomicU64 = AtomicU64::new(0);
            )+
        }

        fn flush_into_global(local: &Counts) {
            $(
                if local.$field != 0 {
                    global::$field.fetch_add(local.$field, Ordering::Relaxed);
                }
            )+
        }

        fn read_global() -> Counts {
            Counts {
                $($field: global::$field.load(Ordering::Relaxed),)+
            }
        }

        fn reset_global() {
            $(global::$field.store(0, Ordering::Relaxed);)+
        }
    };
}

define_counters! {
    // The filesystem layer: what the walk asked the kernel to do. These are the
    // numbers that decide whether a scan is syscall-bound, and the ones a peer
    // comparison turns on — fdu and dut reading the same tree should differ here
    // before they differ anywhere else.
    // `dir_opens` counts `read_dir` calls, and `dir_entries` counts what they
    // yielded. The underlying `getdents64` batch count is deliberately absent: std's
    // `ReadDir` does not expose it, and a counter that always reads zero misleads
    // worse than a missing one. `strace -c` remains the way to see batches.
    dir_opens: "directory opens", "syscalls";
    dir_entries: "directory entries enumerated", "syscalls";
    stats: "metadata stats", "syscalls";
    file_opens: "file opens", "syscalls";
    file_reads: "file read calls", "syscalls";
    bytes_read: "bytes read from files", "syscalls";

    // The memory layer: the cost that dominated a cold scan's profile and was
    // invisible without callgrind. Counting allocations is cheap next to performing
    // them, which is what makes always-on affordable here.
    allocs: "allocations", "memory";
    reallocs: "reallocations", "memory";
    frees: "frees", "memory";
    bytes_allocated: "bytes allocated", "memory";

    // The index layer: work the consumer did, as opposed to time it took. A parent
    // resolution that descends from the root and one that hits the memo cost very
    // differently, and only the split makes that legible.
    upserts: "upserts applied", "index";
    parent_resolutions: "parent path resolutions", "index";
    parent_memo_hits: "parent memo hits", "index";
    rollup_merges: "roll-up merges", "index";
    entries_allocated: "index entries allocated", "index";
}

thread_local! {
    static LOCAL: Cell<Counts> = const { Cell::new(Counts {
        dir_opens: 0, dir_entries: 0, stats: 0, file_opens: 0, file_reads: 0,
        bytes_read: 0, allocs: 0, reallocs: 0, frees: 0, bytes_allocated: 0,
        upserts: 0, parent_resolutions: 0, parent_memo_hits: 0, rollup_merges: 0,
        entries_allocated: 0,
    }) };
}

/// Add to one counter on the calling thread.
///
/// Takes a closure rather than a field name so the call site reads as the operation it
/// describes. `#[inline]` rather than `inline(always)`: the body is small enough that
/// the ordinary heuristic takes it, and `exp-052` measured the result at no detectable
/// cost, so forcing the compiler's hand would be an assertion the evidence does not
/// need.
#[inline]
pub fn bump(f: impl FnOnce(&mut Counts)) {
    if cfg!(feature = "perf-counters") {
        LOCAL.with(|cell| {
            let mut counts = cell.get();
            f(&mut counts);
            cell.set(counts);
        });
    }
}

/// Fold this thread's counts into the process total and clear them.
///
/// A worker calls this as it finishes so its work is visible to a later [`snapshot`].
/// Forgetting to call it loses that thread's counts rather than corrupting anything.
pub fn flush_thread() {
    if cfg!(feature = "perf-counters") {
        LOCAL.with(|cell| {
            let counts = cell.replace(Counts::default());
            flush_into_global(&counts);
        });
    }
}

/// The process-wide totals, including the calling thread's unflushed counts.
pub fn snapshot() -> Counts {
    if !cfg!(feature = "perf-counters") {
        return Counts::default();
    }
    let mut total = read_global();
    LOCAL.with(|cell| total.add(&cell.get()));
    total
}

/// Clear every counter, process-wide and on the calling thread.
///
/// Used between phases of a probe run so that "the load's counts" and "the scan's
/// counts" are separable. It cannot clear *other* threads' unflushed counts, which is
/// why workers flush as they exit.
pub fn reset() {
    if cfg!(feature = "perf-counters") {
        LOCAL.with(|cell| cell.set(Counts::default()));
        reset_global();
    }
}

/// Whether this build counts anything, so a report can say "compiled out" rather than
/// showing a page of zeroes and inviting the wrong conclusion.
pub const fn enabled() -> bool {
    cfg!(feature = "perf-counters")
}

/// Render counts as aligned `group.label = value` lines, grouped in declaration order.
pub fn render(counts: &Counts) -> String {
    if !enabled() {
        return "counters: compiled out (build with --features perf-counters)\n".to_string();
    }
    let rows = counts.rows();
    let width = rows.iter().map(|(_, label, _)| label.len()).max().unwrap_or(0);
    let mut out = String::new();
    let mut group = "";
    for (row_group, label, value) in rows {
        if row_group != group {
            if !group.is_empty() {
                out.push('\n');
            }
            out.push_str(row_group);
            out.push_str(":\n");
            group = row_group;
        }
        let _ = writeln!(out, "  {label:<width$}  {value:>14}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Counters are process-wide, so tests that read totals must not run beside tests
    /// that write them. One test covers the whole lifecycle for that reason.
    #[test]
    fn counts_accumulate_flush_and_reset() {
        reset();
        bump(|c| c.stats += 3);
        bump(|c| c.allocs += 1);

        let before_flush = snapshot();
        if !enabled() {
            assert!(before_flush.is_empty(), "counters compiled out must stay zero");
            return;
        }
        assert_eq!(before_flush.stats, 3, "unflushed local counts are visible");

        flush_thread();
        let after_flush = snapshot();
        assert_eq!(after_flush.stats, 3, "flushing moves counts rather than losing them");
        assert_eq!(after_flush.allocs, 1);

        // A second thread's counts reach the total only once it flushes, which is the
        // contract every walk worker relies on.
        std::thread::spawn(|| {
            bump(|c| c.stats += 10);
            flush_thread();
        })
        .join()
        .expect("counting thread");
        assert_eq!(snapshot().stats, 13, "another thread's flushed counts fold in");

        reset();
        assert!(snapshot().is_empty(), "reset clears process and local counts");
    }

    #[test]
    fn rendering_names_every_counter() {
        let counts = Counts { stats: 7, ..Counts::default() };
        let text = render(&counts);
        if enabled() {
            assert!(text.contains("metadata stats"), "labels appear: {text}");
            assert!(text.contains("syscalls:"), "groups appear: {text}");
        } else {
            assert!(text.contains("compiled out"), "a disabled build says so: {text}");
        }
    }
}
