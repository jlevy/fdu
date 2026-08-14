//! Low-distortion performance instrumentation for iterative optimization loops.
//!
//! # What this is for
//!
//! Optimizing a system means running the same loop over and over: profile, form a
//! hypothesis, change one thing, measure it paired against a control, keep it or throw
//! it away, write down what happened. The measurement half of that loop is where most
//! of the mistakes live, and almost all of them come from the same root cause — the
//! measurement said *how long* something took without saying *what it did*.
//!
//! This crate provides the "what it did" half, in a form cheap enough to leave switched
//! on. It is deliberately domain-neutral: an application declares its own counters with
//! [`counters`], and everything else here works the same whether the thing being
//! measured is a filesystem walker, a parser, or a request handler.
//!
//! # The three tiers, and why you need all of them
//!
//! Instrumentation lives at three levels, and each answers a question the others cannot.
//! Using only one is the most common way to reach a confident wrong conclusion.
//!
//! | Tier | Source | Cost | Answers |
//! | --- | --- | --- | --- |
//! | Application | [`counters`] at call sites | ~1ns per event | *which layer* did the work |
//! | Process | [`process`] from the kernel | one file read per phase | what the process *really* did |
//! | External | `strace -c`, `perf`, callgrind | 2× to 50× slowdown | ground truth, and call sites |
//!
//! The application tier attributes cost to a layer, which is what you actually need to
//! decide where to change code — but it counts what *your* code thinks it did. The
//! process tier is real kernel data and cannot be fooled, but it cannot attribute. The
//! external tier is authoritative about both and far too slow to leave on.
//!
//! The discipline that makes this work is **cross-checking the application tier against
//! the process tier**, and reaching for the external tier only when they disagree or
//! when neither can attribute a cost to a call site. [`process::Snapshot`] exists for
//! exactly that check.
//!
//! # Coverage is not uniform, and pretending otherwise causes errors
//!
//! `/proc/self/io` reports `syscr` and `syscw`, which look like syscall counts and are
//! not. They count the read and write families only. A directory walk over 17,128
//! entries — every one of them a `getdents64` or a `statx` — moved `syscr` by **30**.
//!
//! So there is no cheap in-process source for enumeration or stat syscall counts on
//! Linux, and an application counter at the call site is the correct instrument for
//! those, with `strace -c` as the periodic ground-truth check. Knowing *which* facts
//! each tier can supply is most of the skill.
//!
//! # Keeping the instrument from distorting the measurement
//!
//! Instrumentation that changes what it measures is worse than none, because it is
//! believed. Four rules, in descending order of how much they matter:
//!
//! 1. **Thread-local and non-atomic.** A shared atomic counter in a parallel section
//!    measures the counter's contention, not the work. Threads fold into the global
//!    total when they finish.
//! 2. **Count, do not time, on per-event paths.** A clock read costs an order of
//!    magnitude more than an integer increment. Time whole phases; count events.
//! 3. **Toggle at runtime, not only at build time.** A build flag means keeping two
//!    binaries and rebuilding to see anything, which is friction in exactly the loop
//!    that should be frictionless. See [`enable`].
//! 4. **Measure the overhead and record it.** Not "this should be cheap" — an A/B
//!    against an uninstrumented control, in the ledger, re-run when the instrumentation
//!    grows.
//!
//! # Using it
//!
//! ```
//! perfkit::counters! {
//!     pub mod metrics {
//!         dir_opens: "directory opens", "syscalls";
//!         allocs: "allocations", "memory";
//!     }
//! }
//!
//! perfkit::enable(true);
//! metrics::bump(|c| c.dir_opens += 1);
//! assert_eq!(metrics::snapshot().dir_opens, 1);
//! ```

#![doc(test(attr(deny(warnings))))]

use std::sync::atomic::{AtomicBool, Ordering};

pub mod alloc;
pub mod process;

mod macros;

/// Whether instrumentation is currently recording.
///
/// Checked before every increment. A relaxed load of a `static` the branch predictor
/// sees the same answer from every time costs approximately nothing next to the work
/// being counted; what it buys is one binary that can be asked for numbers instead of
/// two that differ in whether they have any.
static ENABLED: AtomicBool = AtomicBool::new(false);

/// Turn recording on or off for the whole process.
///
/// Off by default so that a program that links this crate and never calls it pays a
/// predictable branch and nothing else. Turning it on mid-run is allowed and means
/// exactly what it looks like: counts describe the period recording was on.
pub fn enable(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

/// Whether recording is on.
#[inline]
#[must_use]
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Turn recording on when an environment variable asks for it.
///
/// Returns whether recording ended up on. Lets a binary opt in without a flag of its
/// own — useful for a probe run under a harness that only controls the environment.
#[must_use]
pub fn enable_from_env(var: &str) -> bool {
    let on = std::env::var_os(var)
        .is_some_and(|value| !matches!(value.to_str(), Some("" | "0" | "false" | "no")));
    if on {
        enable(true);
    }
    on
}

/// Divide one count by another for a report, treating a zero denominator as one.
///
/// Counters are `u64` and ratios are `f64`, so this narrows. The precision loss is
/// unreachable in practice — `f64` is exact to 2^53, and a counter that high would have
/// taken longer to accumulate than any measurement runs — and a report showing
/// "allocations per entry" would rather have a slightly rounded number than no number.
/// One place carries the justification so no call site has to.
#[allow(clippy::cast_precision_loss)]
#[must_use]
pub fn ratio(numerator: u64, denominator: u64) -> f64 {
    let denominator = if denominator == 0 { 1.0 } else { denominator as f64 };
    numerator as f64 / denominator
}

/// Render `(group, label, value)` rows as aligned, grouped lines.
///
/// Shared so that every counter set in a process reports in one format, which matters
/// more than it sounds: a human comparing two runs should be diffing numbers, not
/// layouts.
#[must_use]
pub fn render_rows(rows: &[(&str, &str, u64)]) -> String {
    use std::fmt::Write as _;

    let width = rows.iter().map(|(_, label, _)| label.len()).max().unwrap_or(0);
    let mut out = String::new();
    let mut group = "";
    for (row_group, label, value) in rows {
        if *row_group != group {
            if !group.is_empty() {
                out.push('\n');
            }
            let _ = writeln!(out, "{row_group}:");
            group = row_group;
        }
        let _ = writeln!(out, "  {label:<width$}  {value:>14}");
    }
    out
}

/// Serialize tests that touch process-global state.
///
/// The enable flag, the counter totals, and any statics a test's allocator sinks write
/// to are all process-wide — that is what lets any layer increment them without a
/// context object threaded through it. The cost is that concurrent tests interfere, and
/// every test asserting on absolute totals takes this first. Callers inherit the same
/// constraint, which is why it is documented on `counters!` rather than hidden here.
#[cfg(test)]
pub(crate) fn test_serial() -> std::sync::MutexGuard<'static, ()> {
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
    SERIAL.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabling_is_observable() {
        let _serial = test_serial();
        let restore = is_enabled();
        enable(true);
        assert!(is_enabled());
        enable(false);
        assert!(!is_enabled());
        enable(restore);
    }

    #[test]
    fn rows_render_grouped_and_aligned() {
        let rows =
            [("syscalls", "opens", 3_u64), ("syscalls", "reads", 42), ("memory", "allocs", 7)];
        let text = render_rows(&rows);
        assert!(text.starts_with("syscalls:\n"), "groups head their sections: {text}");
        assert!(text.contains("\nmemory:\n"), "a new group starts a section: {text}");
        assert!(text.contains("  opens  "), "labels are padded to a common width: {text}");
    }

    #[test]
    fn env_opt_in_reads_the_usual_falsey_spellings() {
        let _serial = test_serial();
        // SAFETY-adjacent: these mutate process environment, so they use one variable
        // name no other test touches.
        let var = "PERFKIT_TEST_OPT_IN";
        for falsey in ["0", "false", "no", ""] {
            unsafe { std::env::set_var(var, falsey) };
            enable(false);
            assert!(!enable_from_env(var), "{falsey:?} does not turn counting on");
        }
        unsafe { std::env::set_var(var, "1") };
        enable(false);
        assert!(enable_from_env(var), "a set variable turns counting on");
        unsafe { std::env::remove_var(var) };
        enable(false);
        assert!(!enable_from_env(var), "an absent variable leaves it off");
    }
}
