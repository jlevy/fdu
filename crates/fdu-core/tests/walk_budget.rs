//! A file cap stops the walk reading, rather than trimming what the walk produced.
//!
//! The distinction is the whole reason the cap is scope. A bound on an *answer* still
//! reads the tree, so it saves serialization and nothing else -- and every assertion a
//! naive test makes about a capped result (fewer rows, partial coverage, a typed issue)
//! passes just as well against that useless implementation. So the load-bearing assertion
//! here is about **directories read**, taken from the walk's own report, and the row count
//! is checked only for the property a between-directories bound can actually promise.
//!
//! Scope also decides what a snapshot means: two caps over one tree produce two different
//! retained sets, and neither is a subset the other could be corrected into, because which
//! entries the cap admitted depends on the order the walk reached them in. That is why the
//! cap is fingerprinted, and why a `--cache only` open under a different cap must refuse
//! rather than reinterpret.

use std::path::Path;

use fdu_core::{CachePolicy, Index, OpenConfig, ScanConfig, ScanReport, ScanScope, Status};

/// Files per directory in the fixture below.
const PER_DIR: usize = 8;

/// Directories in the fixture, each holding [`PER_DIR`] files.
const DIRS: usize = 12;

/// A wide, shallow tree: many sibling directories, each with the same small file count.
///
/// Wide rather than deep so the cap is reached with directories still unvisited under both
/// traversal orders -- a deep chain would let depth-first spend the whole budget before
/// breadth-first had left the root, and the test would then be measuring the order.
fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp");
    let root = dir.path();
    for directory in 0..DIRS {
        let child = root.join(format!("d{directory:02}"));
        std::fs::create_dir_all(&child).expect("mkdir");
        for file in 0..PER_DIR {
            std::fs::write(child.join(format!("f{file}.txt")), b"x").expect("write");
        }
    }
    dir
}

fn scanned(root: &Path, max_files: Option<u64>, threads: Option<usize>) -> (Index, ScanReport) {
    let config = ScanConfig { max_files, threads, ..ScanConfig::default() };
    fdu_core::scan::scan_into_index(root, &config).expect("scan")
}

/// The cap stops directories being read, which is the only thing that saves any work.
///
/// A projection limit would leave `dirs_read` at the full count and still satisfy every
/// other assertion here, so this is the one that separates a budget from a truncation.
#[test]
fn a_capped_walk_reads_fewer_directories_than_an_uncapped_one() {
    let dir = fixture();

    let (full, full_report) = scanned(dir.path(), None, Some(1));
    assert_eq!(full_report.dirs_read, (DIRS + 1) as u64, "root plus every child directory");
    assert!(full_report.is_complete(), "nothing bounded it");
    assert_eq!(full_report.coverage(), Status::Complete);
    assert_eq!(full.total().files, (DIRS * PER_DIR) as u64);

    // Two directories' worth of files: enough that the cap falls with most of the tree
    // still unvisited, and not so tight that the root alone exhausts it.
    let cap = (2 * PER_DIR) as u64;
    let (capped, capped_report) = scanned(dir.path(), Some(cap), Some(1));

    assert!(
        capped_report.dirs_read < full_report.dirs_read,
        "the cap must stop the walk reading, not trim what it read: {} vs {}",
        capped_report.dirs_read,
        full_report.dirs_read
    );
    assert!(capped_report.budget_stopped, "and the walk has to say the cap is why");
    assert!(!capped_report.is_complete(), "an answer that omits scope is not complete");
    assert_eq!(
        capped_report.coverage(),
        Status::Partial(fdu_core::CoverageReason::Budget),
        "partial, and naming the cap rather than a failure"
    );
    assert!(capped.total().files < full.total().files, "and holds fewer entries");
}

/// The overshoot a between-directories bound allows is bounded, and directories are whole.
///
/// A cap checked inside a directory would hit the number exactly and leave that directory
/// reporting its own tallies as though it had been listed completely -- wrong, and silent,
/// which is the pair "truncate freely, never silently" exists to forbid. Overshooting by
/// at most the directories already in flight is the price, and it is the right one.
#[test]
fn a_directory_is_read_whole_or_not_at_all() {
    let dir = fixture();
    let cap = (2 * PER_DIR) as u64;
    let (capped, report) = scanned(dir.path(), Some(cap), Some(1));

    assert!(report.files_walked >= cap, "the cap is a floor for a serial walk, not a ceiling");
    assert_eq!(
        report.files_walked % (PER_DIR as u64),
        0,
        "every directory that was read contributed all {PER_DIR} of its files: {}",
        report.files_walked
    );

    // A directory *row* exists for every child of a directory that was read, whether or
    // not that child was itself entered -- so a row is not evidence the subtree was walked,
    // and the two cases have to be told apart by what they report rather than by presence.
    // Every retained roll-up is therefore either the whole directory or nothing at all.
    let mut read = 0;
    let mut unread = 0;
    for directory in 0..DIRS {
        let name = format!("d{directory:02}");
        let Some(rollup) = capped.rollup(Path::new(&name)) else {
            continue;
        };
        assert!(
            rollup.files == PER_DIR as u64 || rollup.files == 0,
            "{name} was listed whole or not at all, never partly: {}",
            rollup.files
        );
        if rollup.files == 0 { unread += 1 } else { read += 1 }
    }
    assert!(
        read > 0 && unread > 0,
        "the fixture must produce both cases: {read} read, {unread} not"
    );

    // And the zero is never presented as a complete answer. This is the assertion that
    // makes the design honest rather than merely bounded: a consumer asking about a
    // directory the cap kept the walk out of is told its coverage is partial and why,
    // instead of reading an empty directory that is not empty.
    for directory in 0..DIRS {
        let name = format!("d{directory:02}");
        let Some(provenance) = capped.provenance(Path::new(&name)) else {
            continue;
        };
        assert_eq!(
            provenance.status,
            Status::Partial(fdu_core::CoverageReason::Budget),
            "{name} reports what the cap cost it"
        );
    }
}

/// The cap is scope, so it rides in the identity a consumer keys a cache on.
#[test]
fn the_cap_is_part_of_the_scope_identity() {
    let dir = fixture();
    let uncapped = scanned(dir.path(), None, Some(1)).0.scope();
    let tight = scanned(dir.path(), Some(8), Some(1)).0.scope();
    let loose = scanned(dir.path(), Some(16), Some(1)).0.scope();

    assert_eq!(uncapped.max_files, None);
    assert_eq!(tight.max_files, Some(8));
    assert_ne!(tight, loose, "two caps are two scopes, not one scope with two sizes");
    assert_ne!(tight, uncapped, "and a cap is not the same scope as no cap");

    // Nothing else moved: the cap must not be smuggled into another fingerprint, which
    // would make it invalidate the wrong caches.
    assert_eq!(
        ScanScope { max_files: None, ..tight },
        uncapped,
        "the cap is the only field that differs"
    );
}

/// A snapshot recorded under one cap is refused under another rather than reinterpreted.
///
/// `CachePolicy::Only` is what makes this checkable: it is contractually forbidden to touch
/// the tree, so a run that answers at all answered from the snapshot. Under `Auto` the same
/// assertions pass with the fingerprint wired up wrongly, because revalidation rebuilds
/// what the check was supposed to have refused.
#[test]
fn a_snapshot_is_not_readable_under_a_different_cap() {
    let dir = fixture();
    let cache = dir.path().join(".fdu-cache");

    // Wide enough to finish: a budget-stopped walk is deliberately never saved, which the
    // test below this one pins, so a snapshot only exists when the cap did not bite.
    let write = OpenConfig {
        scan: ScanConfig {
            max_files: Some((DIRS * PER_DIR * 2) as u64),
            threads: Some(1),
            ..ScanConfig::default()
        },
        cache_path: Some(cache.clone()),
        policy: CachePolicy::Refresh,
        ..OpenConfig::default()
    };
    let (recorded, _) = fdu_core::open(dir.path(), &write).expect("record a capped snapshot");
    let recorded_files = recorded.total().files;
    assert!(recorded_files > 0, "the fixture must produce something to reread");

    let same = OpenConfig { policy: CachePolicy::Only, ..write.clone() };
    let (reread, _) = fdu_core::open(dir.path(), &same).expect("the same scope reads back");
    assert_eq!(reread.total().files, recorded_files, "same cap, same retained set");

    let different = OpenConfig {
        scan: ScanConfig { max_files: Some((DIRS * PER_DIR * 3) as u64), ..same.scan.clone() },
        ..same.clone()
    };
    assert!(
        fdu_core::open(dir.path(), &different).is_err(),
        "a wider cap describes a different retained set, so the snapshot cannot answer it"
    );

    let uncapped =
        OpenConfig { scan: ScanConfig { max_files: None, ..same.scan.clone() }, ..same.clone() };
    assert!(
        fdu_core::open(dir.path(), &uncapped).is_err(),
        "and neither can it answer a walk that was never bounded"
    );
}

/// A walk the cap stopped is never saved as a warm baseline.
///
/// It would be a stable answer to reread and a wrong one to build on: the retained set
/// depends on the order the walk happened to reach directories in, so a later reconcile
/// against it would treat "never discovered" and "since deleted" as the same thing. The
/// completeness gate that already refuses to cache a walk with read errors covers this for
/// the same reason, and this pins that it does.
#[test]
fn a_budget_stopped_walk_is_not_cached() {
    let dir = fixture();
    let cache = dir.path().join(".fdu-cache");
    let config = OpenConfig {
        scan: ScanConfig {
            max_files: Some((2 * PER_DIR) as u64),
            threads: Some(1),
            ..ScanConfig::default()
        },
        cache_path: Some(cache.clone()),
        policy: CachePolicy::Refresh,
        ..OpenConfig::default()
    };
    let (_, report) = fdu_core::open(dir.path(), &config).expect("open");
    assert!(!report.is_complete(), "the cap bit, so this walk covers less than its scope");

    let only = OpenConfig { policy: CachePolicy::Only, ..config };
    assert!(
        fdu_core::open(dir.path(), &only).is_err(),
        "nothing was written, so there is nothing for a cache-only open to read"
    );
}

/// The cap is one budget for the whole walk, not one per worker.
///
/// A per-worker counter admits `cap * workers` files and makes the bound depend on a thread
/// count that is deliberately not part of the scope -- so the same cap over the same tree
/// would retain different sets on two machines, which is exactly what fingerprinting the
/// cap is supposed to prevent.
#[test]
fn the_cap_is_shared_across_workers() {
    let dir = fixture();
    // Five directories' worth, chosen so the two hypotheses give different numbers. A
    // shared counter admits at most `cap + workers * PER_DIR` = 56; a per-worker counter
    // admits about `workers * cap` = 80. At the obvious cap of two directories they are
    // both 32 and the test proves nothing -- which is what the first draft did, and the
    // per-worker mutation passed against it.
    let cap = (5 * PER_DIR) as u64;
    let workers = 2;
    let (_, serial) = scanned(dir.path(), Some(cap), Some(1));
    let (_, parallel) = scanned(dir.path(), Some(cap), Some(workers));

    assert!(serial.budget_stopped && parallel.budget_stopped, "both walks hit the cap");

    // The invariant, stated rather than inferred. One shared counter means at most one
    // directory per worker is still being read when the cap falls, so the overshoot is
    // bounded by `workers * PER_DIR`. A per-worker counter admits `workers * cap` before
    // the same overshoot -- which is a different number, and the reason to write the bound
    // out: "fewer than the whole tree" is satisfied by both, and a first draft of this test
    // asserted exactly that and passed against a per-worker budget.
    //
    // `threads` is explicit, so `WorkerPool::fixed` makes `workers` the real count rather
    // than a floor the adaptive pool may grow past.
    let overshoot = (workers * PER_DIR) as u64;
    assert!(
        parallel.files_walked <= cap + overshoot,
        "one budget for the walk, not one per worker: {} > {} + {}",
        parallel.files_walked,
        cap,
        overshoot
    );
    assert!(
        serial.files_walked <= cap + PER_DIR as u64,
        "and a serial walk overshoots by at most the one directory it was in: {}",
        serial.files_walked
    );
}

/// Stopping at the cap is reported as a typed condition, not inferred from a count.
#[test]
fn the_cap_reports_itself_as_a_typed_issue() {
    let dir = fixture();
    let config = OpenConfig {
        scan: ScanConfig {
            max_files: Some((2 * PER_DIR) as u64),
            threads: Some(1),
            ..ScanConfig::default()
        },
        policy: CachePolicy::Off,
        ..OpenConfig::default()
    };
    let (index, _) = fdu_core::open(dir.path(), &config).expect("open");
    let state = index.engine_state();

    assert!(!state.run.complete, "the run did not cover its scope");
    assert!(
        state.run.errors.iter().any(|issue| issue.kind == fdu_core::IssueKind::ResourceStop),
        "a consumer must be able to act on this from a value, not from a message: {:?}",
        state.run.errors
    );
    assert_eq!(state.coverage, Status::Partial(fdu_core::CoverageReason::Budget));
}

/// A cap of zero is refused rather than silently walking nothing.
#[test]
fn a_zero_cap_is_a_configuration_error() {
    let dir = fixture();
    let config = ScanConfig { max_files: Some(0), ..ScanConfig::default() };
    let error = fdu_core::scan::scan_into_index(dir.path(), &config)
        .expect_err("zero is not how an unlimited walk is spelled");
    assert!(
        error.to_string().contains("max_files"),
        "and the message has to name the knob: {error}"
    );
}
