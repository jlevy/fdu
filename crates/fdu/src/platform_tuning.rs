//! Defaults that measurement has shown, or may yet show, to diverge by platform.
//!
//! There are two kinds of platform difference in this engine and only one of them
//! belongs here.
//!
//! A platform **API** — `getattrlistbulk` — does not exist elsewhere and cannot compile
//! elsewhere.  Those stay `cfg`-gated at their call site, layered over a portable path
//! as optional accelerators that fall back rather than fail.  That pattern is settled
//! and this module does not touch it.
//!
//! A platform **tuning** is the same portable code wanting a different value, or a
//! different one of two portable strategies, because the machine underneath behaves
//! differently.  Until now those were single shared constants, which meant Linux
//! silently inherited numbers measured on one 10-core M1 Pro and nothing in the source
//! said so.  Three things follow from moving them here as data:
//!
//! 1. **A divergence becomes a one-line edit** rather than a `cfg` threaded through the
//!    engine, so a Linux result cannot regress macOS on its way in.
//! 2. **An inherited default cannot pass for a measured one.** Every value carries
//!    [`Evidence`], and `Inherited` is a standing admission that this platform has no
//!    number of its own.
//! 3. **Both arms stay compiled everywhere.** A strategy is just a [`Tuned`] whose type
//!    is an enum, so the arm that is not the local default is still type-checked, still
//!    parity-tested, and cannot rot. `cfg` selects the *default*; it never selects the
//!    *existence*. [`crate::ScanOrder`] is the existing precedent — breadth-first and
//!    depth-first both compile on every platform, and only the default is chosen.
//!
//! The rule that governs edits here is in
//! `docs/project/guides/platform-tuning.md`: a shared default needs evidence in every
//! regime it claims, and a branch with a guess on one side is worse than a shared value
//! because it looks like evidence.

/// Whether a value was chosen by measurement on this platform, or carried over.
///
/// The distinction is the point of the module.  A number that has never been measured
/// where it now runs is not evidence about this platform, and recording that in the data
/// keeps the claim honest without needing a reader to cross-reference the ledger.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Evidence {
    /// An experiment on this platform chose this value.
    Measured,
    /// Carried over from a platform that measured it.  Not a claim about this one.
    Inherited,
}

impl Evidence {
    /// Usable in a `const` assertion, which is how the tables are checked in builds that
    /// do not select them.
    pub(crate) const fn is_measured(self) -> bool {
        matches!(self, Self::Measured)
    }
}

/// One tuning value together with the standing of the number behind it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Tuned<T> {
    value: T,
    evidence: Evidence,
}

impl<T: Copy> Tuned<T> {
    /// An experiment on this platform settled this value.
    pub(crate) const fn measured(value: T) -> Self {
        Self { value, evidence: Evidence::Measured }
    }

    /// This platform has no measurement; the value comes from one that does.
    pub(crate) const fn inherited(value: T) -> Self {
        Self { value, evidence: Evidence::Inherited }
    }

    /// The value itself.  Deliberately a call rather than a bare field, so a read is
    /// visibly a read of something tuned.
    pub(crate) const fn get(self) -> T {
        self.value
    }

    pub(crate) const fn evidence(self) -> Evidence {
        self.evidence
    }
}

/// The platform-dependent defaults, resolved once at the top of the engine.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Tuning {
    /// Workers an automatic scan starts with.
    pub(crate) scan_threads_cap: Tuned<usize>,
    /// Ceiling an automatic scan may unlock once it establishes the tree is large.
    pub(crate) adaptive_scan_threads_cap: Tuned<usize>,
    /// Reserve depth relative to the host's reported parallelism.
    pub(crate) adaptive_scan_parallelism_multiplier: Tuned<usize>,
    /// Entries used to calibrate initial filesystem service time.
    pub(crate) adaptive_scan_calibration_entries: Tuned<u64>,
    /// Average worker time per entry that identifies a latency-bound scan.
    pub(crate) adaptive_scan_slow_work_ns_per_entry: Tuned<u64>,
    /// Workers for an immutable-baseline reconciliation wave.
    pub(crate) reconcile_threads_cap: Tuned<usize>,
    /// Directories compared per reconciliation wave.
    pub(crate) reconcile_wave_directories: Tuned<usize>,
    /// Observations batched before one apply.
    pub(crate) batch_size: Tuned<usize>,
}

/// macOS on APFS, where every one of these was measured.
///
/// The citations are in `docs/project/guides/platform-tuning.md` and the experiments
/// themselves are in the ledger; the short version is exp-015 through exp-021 for the
/// adaptive policy, exp-025 and exp-036 for worker depth, and exp-030 and exp-031 for
/// the reconciliation wave.
const MACOS: Tuning = Tuning {
    scan_threads_cap: Tuned::measured(6),
    adaptive_scan_threads_cap: Tuned::measured(16),
    adaptive_scan_parallelism_multiplier: Tuned::measured(2),
    adaptive_scan_calibration_entries: Tuned::measured(16 * 1024),
    adaptive_scan_slow_work_ns_per_entry: Tuned::measured(30_000),
    reconcile_threads_cap: Tuned::measured(4),
    reconcile_wave_directories: Tuned::measured(1_024),
    batch_size: Tuned::measured(1_024),
};

/// Everything that is not macOS, which today means Linux and Windows.
///
/// Every value is [`Evidence::Inherited`]: the numbers are macOS's, and no experiment
/// has chosen one here.  That is not an argument for changing them blind — an inherited
/// default that works is better than a guess — it is a statement of what is and is not
/// known, and the place a sweep writes its answer when it has one.
///
/// The adaptive threshold is the one with a specific reason to doubt it: 30 µs was
/// placed between APFS regimes of roughly 18, 22 and 42 µs per entry, and the Linux warm
/// floor is about 1.5 µs, so the trigger may never fire here at all (H84, `fdu-tk1b`).
const PORTABLE: Tuning = Tuning {
    scan_threads_cap: Tuned::inherited(6),
    adaptive_scan_threads_cap: Tuned::inherited(16),
    adaptive_scan_parallelism_multiplier: Tuned::inherited(2),
    adaptive_scan_calibration_entries: Tuned::inherited(16 * 1024),
    adaptive_scan_slow_work_ns_per_entry: Tuned::inherited(30_000),
    reconcile_threads_cap: Tuned::inherited(4),
    reconcile_wave_directories: Tuned::inherited(1_024),
    batch_size: Tuned::inherited(1_024),
};

/// Every table is checked at compile time in every build, whichever one is selected.
///
/// This is the anti-rot guarantee, and the reason the module is worth its weight. A
/// table that is not the local default is still evaluated here, so a Linux edit that
/// broke the macOS table would fail a Linux build — nobody has to own a Mac to keep the
/// Mac arm honest. It also pins the standing of each number, so promoting one to
/// `measured` is a deliberate edit rather than a typo.
const _: () = {
    assert!(MACOS.scan_threads_cap.get() >= 1);
    assert!(MACOS.adaptive_scan_threads_cap.get() >= MACOS.scan_threads_cap.get());
    assert!(MACOS.reconcile_threads_cap.get() >= 1);
    assert!(MACOS.reconcile_wave_directories.get() >= 1);
    assert!(MACOS.batch_size.get() >= 1);
    assert!(MACOS.adaptive_scan_slow_work_ns_per_entry.get() > 0);
    assert!(MACOS.scan_threads_cap.evidence().is_measured(), "macOS measured its own");

    assert!(PORTABLE.scan_threads_cap.get() >= 1);
    assert!(PORTABLE.adaptive_scan_threads_cap.get() >= PORTABLE.scan_threads_cap.get());
    assert!(PORTABLE.reconcile_threads_cap.get() >= 1);
    assert!(PORTABLE.reconcile_wave_directories.get() >= 1);
    assert!(PORTABLE.batch_size.get() >= 1);
    assert!(PORTABLE.adaptive_scan_slow_work_ns_per_entry.get() > 0);
    assert!(
        !PORTABLE.scan_threads_cap.evidence().is_measured(),
        "promote this to Tuned::measured in the same change that lands the sweep"
    );
};

/// The defaults for the platform this build targets.
///
/// `cfg` chooses between two tables that both exist in every build, so neither table can
/// stop compiling because the other one changed.
pub(crate) const fn tuning() -> Tuning {
    #[cfg(target_os = "macos")]
    {
        MACOS
    }
    #[cfg(not(target_os = "macos"))]
    {
        PORTABLE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both tables are read on every platform, so neither can rot behind a `cfg`.
    ///
    /// This is the property that makes the module worth having: the arm that is not the
    /// local default still has to hold together, which is what lets a Linux measurement
    /// land without anyone able to build macOS to check it.
    #[test]
    fn every_platform_table_is_usable_from_every_platform() {
        for table in [MACOS, PORTABLE] {
            assert!(table.scan_threads_cap.get() >= 1, "a scan needs at least one worker");
            assert!(
                table.adaptive_scan_threads_cap.get() >= table.scan_threads_cap.get(),
                "the adaptive ceiling cannot sit below the starting pool"
            );
            assert!(table.adaptive_scan_parallelism_multiplier.get() >= 1);
            assert!(table.adaptive_scan_calibration_entries.get() > 0);
            assert!(table.adaptive_scan_slow_work_ns_per_entry.get() > 0);
            assert!(table.reconcile_threads_cap.get() >= 1);
            assert!(table.reconcile_wave_directories.get() >= 1);
            assert!(table.batch_size.get() >= 1);
        }
    }

    /// macOS measured all of these; nothing else has measured any of them.
    ///
    /// The assertion is deliberately blunt. When a Linux sweep lands, this test fails,
    /// and the person changing it has to say in the same edit which value became
    /// `measured` and which experiment settled it.
    #[test]
    fn the_portable_table_admits_that_it_inherits() {
        assert_eq!(MACOS.scan_threads_cap.evidence(), Evidence::Measured);
        assert_eq!(MACOS.adaptive_scan_slow_work_ns_per_entry.evidence(), Evidence::Measured);
        assert_eq!(MACOS.reconcile_wave_directories.evidence(), Evidence::Measured);

        assert_eq!(PORTABLE.scan_threads_cap.evidence(), Evidence::Inherited);
        assert_eq!(PORTABLE.adaptive_scan_slow_work_ns_per_entry.evidence(), Evidence::Inherited);
        assert_eq!(PORTABLE.reconcile_wave_directories.evidence(), Evidence::Inherited);
    }

    /// Every table's values produce the same answer, on whatever platform runs this.
    ///
    /// The `const` block proves each table is *well-formed* everywhere. This proves the
    /// engine still agrees with itself when driven by values it would not normally see,
    /// which is the property that actually protects the other platform: a divergence
    /// landed for Linux is exercised by macOS CI and vice versa, so a tuning change that
    /// altered the answer rather than just the speed fails on both runners rather than
    /// waiting for someone to boot the other one.
    ///
    /// Speed may differ between tables — that is the entire point of having two. The
    /// answer may not.
    #[test]
    fn every_platform_table_produces_the_same_index() {
        let dir = tempfile::Builder::new().prefix("fdu-tuning-").tempdir().expect("tempdir");
        let root = dir.path();
        // Enough fan-out and depth that a worker count and a batch size can actually
        // change how the walk is scheduled, rather than everything landing in one batch.
        for directory in 0..12 {
            for file in 0..24 {
                let path = root.join(format!("d{directory}/nested/f{file}.rs"));
                std::fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
                std::fs::write(&path, vec![b'x'; directory * 16 + file]).expect("write");
            }
            std::fs::write(root.join(format!("d{directory}/top.md")), b"# top").expect("write");
        }

        let image_for = |threads: usize, batch_size: usize| {
            let config = crate::ScanConfig {
                threads: Some(threads),
                batch_size,
                ..crate::ScanConfig::default()
            };
            let (index, report) = crate::scan::scan_into_index(root, &config).expect("scan");
            assert!(report.is_complete(), "the fixture scan must be complete");
            let total = index.total();
            // Kind is compared through its `Debug` form: the enum is deliberately not
            // `Ord`, and sorting is what makes two walk orders comparable at all.
            let mut entries: Vec<(std::path::PathBuf, String, u64)> = Vec::new();
            let mut stack = vec![crate::index::EntryId::ROOT];
            while let Some(id) = stack.pop() {
                let path = index.path_of(id).expect("path");
                let kind = format!("{:?}", index.kind_of(id).expect("kind"));
                let size = index.attrs_of(id).expect("attrs").size;
                entries.push((path, kind, size));
                if let Some(children) = index.children_of(id) {
                    stack.extend(children.map(|(_, child)| child));
                }
            }
            entries.sort();
            (
                index.len(),
                total.files,
                total.dirs,
                total.bytes,
                total.newest_mtime_ns,
                index.by_ext_named(total),
                entries,
            )
        };

        // The tables hold the same values today, so comparing only those two would be
        // comparing a scan with itself. The sweep is over every distinct setting any
        // table asks for, plus a one-worker serial reference, which keeps the assertion
        // load-bearing now and turns it into a genuine cross-table check the moment a
        // platform diverges.
        let mut settings: Vec<(usize, usize)> = vec![(1, 1)];
        for table in [MACOS, PORTABLE] {
            settings.push((table.scan_threads_cap.get(), table.batch_size.get()));
            settings.push((table.reconcile_threads_cap.get(), table.batch_size.get()));
            settings.push((table.adaptive_scan_threads_cap.get(), table.batch_size.get()));
        }
        settings.sort_unstable();
        settings.dedup();

        let reference = image_for(settings[0].0, settings[0].1);
        for &(threads, batch_size) in &settings[1..] {
            assert_eq!(
                image_for(threads, batch_size),
                reference,
                "a scan with {threads} workers and a batch size of {batch_size} \
                 disagreed with the serial reference; a tuning value may change speed \
                 but never the answer"
            );
        }
    }

    /// The engine reads the table its target actually selected.
    #[test]
    fn the_selected_table_matches_the_target() {
        let selected = tuning();
        #[cfg(target_os = "macos")]
        assert_eq!(selected.scan_threads_cap.evidence(), Evidence::Measured);
        #[cfg(not(target_os = "macos"))]
        assert_eq!(selected.scan_threads_cap.evidence(), Evidence::Inherited);
    }
}
