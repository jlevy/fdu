//! The macro that declares a counter set.

/// Declare a set of counters as a module.
///
/// Counters are domain-specific — a walker counts directory opens, a parser counts
/// tokens — so this crate supplies the mechanism and the application supplies the names.
/// The generated module has `bump`, `snapshot`, `flush_thread`, `reset`, `rows` and
/// `render`, and a `Counts` struct with one `u64` per counter.
///
/// Each entry is `field: "label", "group";`. The label is what a human reads; the group
/// heads a section in the rendered report. Declaration order is report order.
///
/// ```
/// perfkit::counters! {
///     /// Counters for a directory walk.
///     pub mod walk_metrics {
///         dir_opens: "directory opens", "syscalls";
///         entries: "entries enumerated", "syscalls";
///     }
/// }
///
/// perfkit::enable(true);
/// walk_metrics::reset();
/// walk_metrics::bump(|c| c.dir_opens += 1);
/// assert_eq!(walk_metrics::snapshot().dir_opens, 1);
/// ```
///
/// # Threads and global state
///
/// The counters and the enable flag are **process-global**. That is what makes them
/// usable from anywhere without threading a context object through every call, and the
/// cost is that concurrent tests touching the same counter set interfere — serialize
/// them with a mutex, or assert on differences rather than absolute totals.
///
/// Counts accumulate in thread-local storage and reach the process total when a thread
/// calls `flush_thread`. A worker that exits without flushing loses its own counts and
/// corrupts nothing. `snapshot` includes the calling thread's unflushed counts, so a
/// single-threaded program never has to think about this.
#[macro_export]
macro_rules! counters {
    (
        $(#[$module_meta:meta])*
        $vis:vis mod $module:ident {
            $($field:ident: $label:literal, $group:literal;)+
        }
    ) => {
        $(#[$module_meta])*
        $vis mod $module {
            use ::std::cell::Cell;
            use ::std::sync::atomic::{AtomicU64, Ordering};

            /// One set of counts: a thread's, or the whole process's.
            #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
            pub struct Counts {
                $(
                    #[doc = $label]
                    pub $field: u64,
                )+
            }

            impl Counts {
                /// Every counter as `(group, label, value)`, in declaration order.
                #[must_use]
                pub fn rows(&self) -> ::std::vec::Vec<(&'static str, &'static str, u64)> {
                    ::std::vec![$(($group, $label, self.$field),)+]
                }

                /// Whether every counter is zero.
                ///
                /// Distinguishes "this run did nothing" from "recording was off", which
                /// are very different conclusions to draw from the same page of zeroes.
                #[must_use]
                pub fn is_empty(&self) -> bool {
                    $(self.$field == 0 &&)+ true
                }

                /// Fold another set into this one.
                pub fn add(&mut self, other: &Self) {
                    $(self.$field += other.$field;)+
                }

                /// Per-event ratios against a denominator, for reports that care about
                /// "allocations per entry" rather than absolute counts — which is
                /// usually the number that transfers between trees of different sizes.
                #[must_use]
                pub fn per(&self, denominator: u64) -> ::std::vec::Vec<(&'static str, &'static str, f64)> {
                    ::std::vec![$(($group, $label, $crate::ratio(self.$field, denominator)),)+]
                }
            }

            mod global {
                use super::AtomicU64;
                $(
                    #[allow(non_upper_case_globals)]
                    pub(super) static $field: AtomicU64 = AtomicU64::new(0);
                )+
            }

            ::std::thread_local! {
                static LOCAL: Cell<Counts> = const { Cell::new(Counts { $($field: 0,)+ }) };
            }

            /// Add to one or more counters on the calling thread.
            ///
            /// Does nothing unless `perfkit::is_enabled()`.
            #[inline]
            pub fn bump(f: impl FnOnce(&mut Counts)) {
                if $crate::is_enabled() {
                    LOCAL.with(|cell| {
                        let mut counts = cell.get();
                        f(&mut counts);
                        cell.set(counts);
                    });
                }
            }

            /// Fold this thread's counts into the process total and clear them.
            pub fn flush_thread() {
                LOCAL.with(|cell| {
                    let counts = cell.replace(Counts::default());
                    $(
                        if counts.$field != 0 {
                            global::$field.fetch_add(counts.$field, Ordering::Relaxed);
                        }
                    )+
                });
            }

            /// Process-wide totals, including the calling thread's unflushed counts.
            #[must_use]
            pub fn snapshot() -> Counts {
                let mut total = Counts {
                    $($field: global::$field.load(Ordering::Relaxed),)+
                };
                LOCAL.with(|cell| total.add(&cell.get()));
                total
            }

            /// Clear the process total and the calling thread's counts.
            ///
            /// Cannot clear another thread's unflushed counts, which is why workers
            /// flush as they finish.
            pub fn reset() {
                LOCAL.with(|cell| cell.set(Counts::default()));
                $(global::$field.store(0, Ordering::Relaxed);)+
            }

            /// The current totals, rendered as grouped lines.
            #[must_use]
            pub fn render() -> ::std::string::String {
                let counts = snapshot();
                if !$crate::is_enabled() && counts.is_empty() {
                    return ::std::string::String::from(
                        "counters: recording is off (enable with perfkit::enable(true))\n",
                    );
                }
                $crate::render_rows(&counts.rows())
            }
        }
    };
}

#[cfg(test)]
mod tests {
    crate::counters! {
        pub mod demo {
            opens: "opens", "syscalls";
            allocs: "allocations", "memory";
        }
    }

    /// Counters are process-wide, so one test owns the whole lifecycle rather than
    /// racing sibling tests for the same globals.
    #[test]
    fn a_counter_set_accumulates_folds_and_resets() {
        let _serial = crate::test_serial();
        crate::enable(true);
        demo::reset();

        demo::bump(|c| c.opens += 2);
        assert_eq!(demo::snapshot().opens, 2, "unflushed local counts are visible");

        demo::flush_thread();
        assert_eq!(demo::snapshot().opens, 2, "flushing moves counts rather than losing them");

        std::thread::spawn(|| {
            demo::bump(|c| c.opens += 5);
            demo::flush_thread();
        })
        .join()
        .expect("counting thread");
        assert_eq!(demo::snapshot().opens, 7, "another thread's flushed counts fold in");

        // A thread that never flushes loses its own counts and nothing else.
        std::thread::spawn(|| demo::bump(|c| c.opens += 100)).join().expect("thread");
        assert_eq!(demo::snapshot().opens, 7, "an unflushed thread contributes nothing");

        demo::reset();
        assert!(demo::snapshot().is_empty(), "reset clears both scopes");
        crate::enable(false);
    }

    #[test]
    fn recording_off_means_no_counts() {
        let _serial = crate::test_serial();
        crate::enable(false);
        demo::reset();
        demo::bump(|c| c.allocs += 1);
        assert!(demo::snapshot().is_empty(), "bumps do nothing while recording is off");
        assert!(demo::render().contains("recording is off"), "and the report says why");
    }

    #[test]
    fn ratios_divide_by_the_denominator() {
        let _serial = crate::test_serial();
        crate::enable(true);
        demo::reset();
        demo::bump(|c| c.allocs += 10);
        let per = demo::snapshot().per(4);
        let allocs = per.iter().find(|(_, label, _)| *label == "allocations").expect("row");
        assert!((allocs.2 - 2.5).abs() < f64::EPSILON, "10 over 4 is 2.5, got {}", allocs.2);
        // Zero denominators must not produce infinities in a report.
        let safe = demo::snapshot().per(0);
        assert!(safe.iter().all(|(_, _, v)| v.is_finite()), "a zero denominator stays finite");
        demo::reset();
        crate::enable(false);
    }
}
