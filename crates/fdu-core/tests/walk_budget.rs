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

use fdu_core::{
    CachePolicy, Index, IndexHandle, OpenConfig, ScanConfig, ScanReport, ScanScope, Status,
};

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
        // One level of nesting, so some directories are only *reached* after the cap has
        // filled. Every top-level directory is seen while listing the root, before a single
        // file has been counted, so a fixture one level deep cannot tell "refused because
        // the cap was spent" from "admitted because it was seen first".
        std::fs::create_dir_all(child.join("sub")).expect("mkdir");
        for file in 0..PER_DIR {
            std::fs::write(child.join(format!("f{file}.txt")), b"x").expect("write");
        }
    }
    dir
}

/// Directories in the fixture: one per top level, plus each one's `sub`, plus the root.
const TREE_DIRS: usize = DIRS * 2;

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
    assert_eq!(full_report.dirs_read, (TREE_DIRS + 1) as u64, "root plus every directory");
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

/// The cap is exact, and a short directory says why rather than saying nothing.
///
/// Strictness is not a nicety here: the cap is fingerprinted scope, so two indexes claiming
/// one scope identity have to hold the same inventory. A bound that admits "the cap, plus
/// whatever was already in flight" is a different number on every run and on every machine,
/// which makes the identity a claim the engine does not keep.
///
/// The first version stopped between directories so a directory was read whole or not at
/// all, reasoning that a half-listed directory would present a short tally as complete. The
/// reasoning was sound and the premise was wrong: a walk the cap stopped marks coverage
/// partial from the root down, so the short tally is already not silent. The objection it
/// answered did not exist, and the overshoot it bought was observable.
#[test]
fn the_cap_is_exact_and_a_short_directory_says_so() {
    let dir = fixture();
    let cap = (2 * PER_DIR + 3) as u64;
    assert!(
        !cap.is_multiple_of(PER_DIR as u64),
        "the cap must fall mid-directory, or this proves nothing about partial listings"
    );
    let (capped, report) = scanned(dir.path(), Some(cap), Some(1));

    assert_eq!(report.files_walked, cap, "exactly the cap, never one more");
    assert_eq!(capped.total().files, cap, "and the index holds exactly that many");

    // Nothing at all is retained past the file the cap stopped on -- not a directory, not
    // a symlink. Charging only files while still admitting everything else would keep the
    // file count exact and the *inventory* different, which is the thing the fingerprint
    // claims and the whole reason the cap is strict.
    assert!(
        capped.total().dirs < TREE_DIRS as u64,
        "no directory reached after the cap was spent is retained: {} of {TREE_DIRS}",
        capped.total().dirs
    );

    // Some directory was listed only partly, which is what the strict cap costs.
    let short = (0..DIRS)
        .filter_map(|directory| capped.rollup(Path::new(&format!("d{directory:02}"))))
        .any(|rollup| rollup.files > 0 && rollup.files < PER_DIR as u64);
    assert!(short, "the cap fell inside a directory, so one of them is short");

    // And no directory presents its short or absent tally as a complete answer. That is
    // what makes the strictness honest rather than merely exact: a consumer asking about a
    // directory the cap cut off is told its coverage is partial and why.
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

/// The cap is one number, whatever the thread count.
///
/// A per-worker counter admits `cap * workers`; an unsynchronised shared one admits
/// `cap + workers` when several read the old value and all add. "Exactly the cap" rules out
/// both, and it is what the fingerprint needs -- the thread count is deliberately not part
/// of the scope, so it must not decide how much a capped index holds.
#[test]
fn the_cap_is_shared_across_workers() {
    let dir = fixture();
    let cap = (5 * PER_DIR - 2) as u64;
    let (_, serial) = scanned(dir.path(), Some(cap), Some(1));
    let (_, parallel) = scanned(dir.path(), Some(cap), Some(4));

    assert!(serial.budget_stopped && parallel.budget_stopped, "both walks hit the cap");
    assert_eq!(serial.files_walked, cap, "serial");
    assert_eq!(parallel.files_walked, cap, "and four workers agree to the file");
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

/// A refresh of a capped index does not grow it past the cap.
///
/// The half the walk's own budget cannot cover. `Budget` stops *discovery*, which is what
/// makes a capped scan cheap, but reconciliation walks from the index and never consults
/// it -- so before the index kept the cap itself, one refresh turned a bounded inventory
/// into an unbounded one while the scan identity went on claiming a cap.
#[test]
fn a_refresh_does_not_grow_a_capped_index_past_its_cap() {
    let dir = tempfile::tempdir().expect("temp");
    for index in 0..6 {
        std::fs::write(dir.path().join(format!("f{index}.txt")), b"seed").expect("write");
    }
    let scan = ScanConfig { max_files: Some(4), ..ScanConfig::default() };
    let config = OpenConfig { scan: scan.clone(), policy: CachePolicy::Off, ..Default::default() };
    let (index, _) = fdu_core::open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);
    let held = handle.total().expect("total").files;
    assert_eq!(held, 4, "the walk stopped at the cap");

    for index in 0..8 {
        std::fs::write(dir.path().join(format!("late{index}.txt")), b"more").expect("write");
    }
    fdu_core::scan::reconcile_handle(&handle, &scan, &mut |_| {}).expect("refresh");

    assert_eq!(
        handle.total().expect("total").files,
        4,
        "a refresh finds more than the cap allows and keeps the cap"
    );
    assert_eq!(
        handle.with_index(|index| index.coverage_at(std::path::Path::new(""))).expect("coverage"),
        fdu_core::Status::Partial(fdu_core::CoverageReason::Budget),
        "and says the inventory is short rather than dropping rows silently"
    );
}

/// An uncapped index is not bounded by anything, which is what makes the test above a test.
#[test]
fn an_uncapped_refresh_admits_everything_it_finds() {
    let dir = tempfile::tempdir().expect("temp");
    for index in 0..6 {
        std::fs::write(dir.path().join(format!("f{index}.txt")), b"seed").expect("write");
    }
    let config = OpenConfig { policy: CachePolicy::Off, ..Default::default() };
    let (index, _) = fdu_core::open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);

    for index in 0..8 {
        std::fs::write(dir.path().join(format!("late{index}.txt")), b"more").expect("write");
    }
    fdu_core::scan::reconcile_handle(&handle, &ScanConfig::default(), &mut |_| {})
        .expect("refresh");

    assert_eq!(handle.total().expect("total").files, 14);
}

/// A refusal is one event with three faces, and they agree at one clock.
///
/// A count, a coverage verdict, and a typed issue, all committed together. Splitting them
/// is the failure this pins: the first version marked coverage partial and left the run
/// envelope saying the refresh had been complete, so a caller reading `is_complete()` and
/// a caller reading `coverage()` got opposite answers about the same call — and a consumer
/// mapping the typed issue to its own resource-budget state had nothing to map.
#[test]
fn a_refusal_reports_a_count_a_coverage_and_a_typed_issue_together() {
    let dir = tempfile::tempdir().expect("temp");
    for index in 0..4 {
        std::fs::write(dir.path().join(format!("f{index}.txt")), b"seed").expect("write");
    }
    let scan = ScanConfig { max_files: Some(4), ..ScanConfig::default() };
    let config = OpenConfig { scan: scan.clone(), policy: CachePolicy::Off, ..Default::default() };
    let (index, _) = fdu_core::open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);

    for index in 0..3 {
        std::fs::write(dir.path().join(format!("late{index}.txt")), b"more").expect("write");
    }
    let report = fdu_core::scan::reconcile_handle(&handle, &scan, &mut |_| {}).expect("refresh");

    assert!(report.apply.refused > 0, "the refusals are counted rather than dropped");
    assert!(!report.is_complete(), "so the operation is not complete");
    assert_eq!(
        report.coverage(),
        Status::Partial(fdu_core::CoverageReason::Budget),
        "and the reason is the cap rather than whatever the walk itself reported"
    );

    let state = handle.with_index(fdu_core::Index::engine_state).expect("state");
    assert_eq!(state.coverage, Status::Partial(fdu_core::CoverageReason::Budget));
    assert!(!state.run.complete, "the run envelope agrees with the coverage");
    let issue = state
        .run
        .errors
        .iter()
        .find(|issue| issue.kind == fdu_core::IssueKind::ResourceStop)
        .expect("a typed issue a consumer can match on, not prose it has to parse");
    assert!(issue.message.contains('4'), "naming the cap that refused: {}", issue.message);
}

/// A refusal that had to mutate to reach its verdict reports the mutation as well.
///
/// The cap is consulted at the point a *new file row* would be allocated, which is after
/// two things have already happened: the ancestors of a deep path have been created, and a
/// kind-changing entry at the path itself has been removed. Both are real mutations to real
/// rows and to every roll-up above them, and both are correct -- a directory the tree has
/// really gained is admitted whatever the cap says, and an object the tree has really lost
/// is gone whether or not its replacement is admitted.
///
/// What was wrong was the report. `upsert_beneath` returned "unchanged" on refusal
/// regardless, so those mutations reached the index with no delta naming them and no data
/// clock moving past them: a consumer resuming from its cursor was current on a tree that
/// had rows it had never been told about.
///
/// Both cases here, and the second refusal as well as the first, because a rule that
/// preflighted only the first arrival would pass a test that stopped at one.
#[test]
fn a_refused_upsert_reports_what_it_changed_on_the_way_to_refusing() {
    let dir = tempfile::tempdir().expect("temp");
    for index in 0..2 {
        std::fs::write(dir.path().join(format!("f{index}.txt")), b"seed").expect("write");
    }
    let scan = ScanConfig { max_files: Some(2), ..ScanConfig::default() };
    let config = OpenConfig { scan: scan.clone(), policy: CachePolicy::Off, ..Default::default() };
    let (index, _) = fdu_core::open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);
    let before = handle.cursor().expect("cursor");

    // A file below directories the index has never seen. The ancestors are admitted --
    // directories are not counted against the cap -- and the file is refused.
    std::fs::create_dir_all(dir.path().join("deep/nested")).expect("mkdir");
    std::fs::write(dir.path().join("deep/nested/late.txt"), b"more").expect("write");
    let report = fdu_core::scan::reconcile_handle(&handle, &scan, &mut |_| {}).expect("refresh");
    assert!(report.apply.refused > 0, "the file is over the cap");

    let after = handle.cursor().expect("cursor");
    assert_ne!(
        after.clock, before.clock,
        "the ancestors are real rows, so the clock has to move past them"
    );
    let root_rollup = |handle: &IndexHandle| {
        handle.rollup(Path::new("")).expect("read").expect("the root has a roll-up")
    };
    let dirs = |handle: &IndexHandle| root_rollup(handle).dirs;
    assert!(dirs(&handle) >= 2, "and the directories are in the index: {}", dirs(&handle));
    assert!(
        !handle.since(before).expect("resume").deltas.is_empty(),
        "a consumer resuming from before this must be told the rows exist"
    );

    // A second arrival, refused again, and again after building an ancestor. A preflight
    // that only held on the first refusal passes everything above and fails here.
    let second = handle.cursor().expect("cursor");
    std::fs::create_dir_all(dir.path().join("other")).expect("mkdir");
    std::fs::write(dir.path().join("other/also.txt"), b"more").expect("write");
    let again = fdu_core::scan::reconcile_handle(&handle, &scan, &mut |_| {}).expect("refresh");
    assert!(again.apply.refused > 0, "still over the cap");
    assert_ne!(
        handle.cursor().expect("cursor").clock,
        second.clock,
        "the second refusal built an ancestor too, and it is a row like any other"
    );

    // And the cap still holds: the refusals refused.
    let files = root_rollup(&handle).files;
    assert_eq!(files, 2, "the bound is a bound however many arrivals it turned away");
}

/// The other mutation before the verdict: a kind change removes before the cap is asked.
///
/// A directory replaced in place by a file is one event on a path that never goes absent.
/// The old row cannot survive it -- the tree does not hold a directory there any more --
/// so the removal is right, and it happens before the cap is consulted about the file that
/// replaces it. Refusing the file is also right. Reporting "unchanged" for the pair is not:
/// the directory is gone from the index and from every roll-up above it, and a consumer
/// resuming from its cursor would still be showing it.
///
/// The replaced directory is empty on purpose. A directory with files in it frees enough
/// room under the cap for the replacement to be admitted, which is a different case and not
/// this one.
#[test]
fn a_kind_change_refused_by_the_cap_still_reports_the_row_it_removed() {
    let dir = tempfile::tempdir().expect("temp");
    // One file under a cap of two, so the walk has budget left to discover the directory:
    // a capped walk stops *discovering* at the cap, and a fixture that starts full would
    // never have read `swap` at all.
    std::fs::write(dir.path().join("f0.txt"), b"seed").expect("write");
    std::fs::create_dir(dir.path().join("swap")).expect("mkdir");

    let scan = ScanConfig { max_files: Some(2), ..ScanConfig::default() };
    let config = OpenConfig { scan: scan.clone(), policy: CachePolicy::Off, ..Default::default() };
    let (index, _) = fdu_core::open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);
    let root_rollup = |handle: &IndexHandle| {
        handle.rollup(Path::new("")).expect("read").expect("the root has a roll-up")
    };
    assert_eq!(root_rollup(&handle).dirs, 1, "the directory is in the index to begin with");

    // Now fill the cap, so the replacement below has nowhere to go.
    std::fs::write(dir.path().join("f1.txt"), b"seed").expect("write");
    fdu_core::scan::reconcile_handle(&handle, &scan, &mut |_| {}).expect("refresh");
    assert_eq!(root_rollup(&handle).files, 2, "the index is at its cap");
    let before = handle.cursor().expect("cursor");

    std::fs::remove_dir(dir.path().join("swap")).expect("rmdir");
    std::fs::write(dir.path().join("swap"), b"now a file").expect("write");
    let report = fdu_core::scan::reconcile_handle(&handle, &scan, &mut |_| {}).expect("refresh");
    assert!(report.apply.refused > 0, "the replacement file is over the cap");

    assert_eq!(root_rollup(&handle).dirs, 0, "the directory the tree lost is gone from the index");
    assert_eq!(root_rollup(&handle).files, 2, "and the replacement was refused");
    assert_ne!(
        handle.cursor().expect("cursor").clock,
        before.clock,
        "a removal is a change, so the clock moves even though the upsert was refused"
    );
    assert!(
        !handle.since(before).expect("resume").deltas.is_empty(),
        "and a consumer resuming from before it is told the row is gone"
    );
}

/// The issue is reported once however many entries the cap refuses.
///
/// A long watch over a full tree refuses on every arrival. One issue per refusal would
/// make the run envelope grow without bound while saying the same thing each time.
#[test]
fn a_repeated_refusal_reports_one_issue_rather_than_one_each() {
    let dir = tempfile::tempdir().expect("temp");
    for index in 0..2 {
        std::fs::write(dir.path().join(format!("f{index}.txt")), b"seed").expect("write");
    }
    let scan = ScanConfig { max_files: Some(2), ..ScanConfig::default() };
    let config = OpenConfig { scan: scan.clone(), policy: CachePolicy::Off, ..Default::default() };
    let (index, _) = fdu_core::open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);

    for round in 0..3 {
        std::fs::write(dir.path().join(format!("late{round}.txt")), b"more").expect("write");
        fdu_core::scan::reconcile_handle(&handle, &scan, &mut |_| {}).expect("refresh");
    }

    let state = handle.with_index(fdu_core::Index::engine_state).expect("state");
    let stops = state
        .run
        .errors
        .iter()
        .filter(|issue| issue.kind == fdu_core::IssueKind::ResourceStop)
        .count();
    assert_eq!(stops, 1, "one scope, one issue: {:?}", state.run.errors);
}
