//! Deterministic structural guards for one-shot and progressive index construction.
//!
//! This is a separate test binary so its counting allocator observes no concurrent test
//! traffic. Allocation growth is measured between two fixture sizes, which cancels
//! platform-specific fixed costs while constraining per-entry ownership without a
//! shared-runner timing threshold.

use std::fs;
use std::path::Path;
use std::time::Duration;

use fdu_core::counters::Counts;
use fdu_core::scan::{ScanConfig, scan_into_index};
use fdu_core::{
    ChangeOutcome, ChangeRequest, Clock, EngineVersion, LifecyclePhase, OpenOptions, OpenedIndex,
    ReadRequest,
};

#[global_allocator]
static ALLOCATOR: fdu_core::counters::alloc::CountingAlloc<std::alloc::System> =
    fdu_core::counters::system_allocator();

const SMALL_DIRECTORY_COUNT: u64 = 32;
const LARGE_DIRECTORY_COUNT: u64 = 64;
const FILES_PER_DIRECTORY: u64 = 64;

// Directory enumeration and fresh metadata use different native ownership on each
// supported platform. These are measured *slopes*, not total-allocation allowances.
// Compact detached storage lowers macOS from 7.x to 5.13 allocations per added entry;
// the Linux and Windows ceilings subtract the same two removed representation
// allocations from their last measured 8.26 and 14.33 slopes. Each ceiling leaves less
// than one allocation per entry of slack, which the injected check below re-proves on
// every runner rather than trusting this comment.
#[cfg(target_os = "macos")]
const DETACHED_ALLOCATIONS_PER_ADDED_ENTRY: u64 = 6;
#[cfg(target_os = "linux")]
const DETACHED_ALLOCATIONS_PER_ADDED_ENTRY: u64 = 7;
#[cfg(target_os = "windows")]
const DETACHED_ALLOCATIONS_PER_ADDED_ENTRY: u64 = 13;
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
const DETACHED_ALLOCATIONS_PER_ADDED_ENTRY: u64 = 7;

// Progressive discovery keeps keyed child topology but still benefits from inline arena
// entries: macOS falls from 25.x to 24.24 allocations per added entry. The Linux and
// Windows ceilings remove the same one arena allocation from their last measured 26.29
// and 34.43 slopes. The same runtime slack proof guards every ceiling.
#[cfg(target_os = "macos")]
const OPENED_ALLOCATIONS_PER_ADDED_ENTRY: u64 = 25;
#[cfg(target_os = "linux")]
const OPENED_ALLOCATIONS_PER_ADDED_ENTRY: u64 = 26;
#[cfg(target_os = "windows")]
const OPENED_ALLOCATIONS_PER_ADDED_ENTRY: u64 = 34;
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
const OPENED_ALLOCATIONS_PER_ADDED_ENTRY: u64 = 26;
const OPENED_JOURNAL_CAPACITY: usize = 4 * 1024 * 1024;

struct DisableCounters;

impl Drop for DisableCounters {
    fn drop(&mut self) {
        fdu_core::counters::enable(false);
        fdu_core::counters::reset();
    }
}

#[test]
fn construction_routes_keep_their_allocation_and_work_boundaries() {
    let small = fixture(SMALL_DIRECTORY_COUNT);
    let large = fixture(LARGE_DIRECTORY_COUNT);
    let config = ScanConfig { read_controls: false, threads: Some(1), ..ScanConfig::default() };
    let _disable = DisableCounters;

    let small_entries = fixture_entries(SMALL_DIRECTORY_COUNT);
    let large_entries = fixture_entries(LARGE_DIRECTORY_COUNT);
    let added_entries = large_entries - small_entries;

    let small_detached = measure_detached(small.path(), &config, small_entries);
    let large_detached = measure_detached(large.path(), &config, large_entries);
    assert_route_is_detached(&small_detached, small_entries);
    assert_route_is_detached(&large_detached, large_entries);
    assert_allocation_slope(
        "detached",
        small_detached.allocs,
        large_detached.allocs,
        added_entries,
        DETACHED_ALLOCATIONS_PER_ADDED_ENTRY,
    );

    let small_opened = measure_opened(small.path(), small_entries);
    let large_opened = measure_opened(large.path(), large_entries);
    assert_route_is_opened(&small_opened, small_entries);
    assert_route_is_opened(&large_opened, large_entries);
    assert_allocation_slope(
        "opened",
        small_opened.allocs,
        large_opened.allocs,
        added_entries,
        OPENED_ALLOCATIONS_PER_ADDED_ENTRY,
    );
}

fn fixture(directory_count: u64) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("construction invariant root");
    for directory in 0..directory_count {
        let directory = root.path().join(format!("d{directory}"));
        fs::create_dir(&directory).expect("fixture directory");
        for file in 0..FILES_PER_DIRECTORY {
            fs::write(directory.join(format!("f{file}.dat")), []).expect("fixture file");
        }
    }
    root
}

const fn fixture_entries(directory_count: u64) -> u64 {
    directory_count + directory_count * FILES_PER_DIRECTORY
}

fn measure_detached(root: &Path, config: &ScanConfig, expected_entries: u64) -> Counts {
    fdu_core::counters::reset();
    fdu_core::counters::enable(true);
    let (index, report) = scan_into_index(root, config).expect("detached scan");
    fdu_core::counters::flush_thread();
    let counts = fdu_core::counters::snapshot();
    fdu_core::counters::enable(false);

    assert_eq!(report.entries, expected_entries);
    assert_eq!(index.len(), expected_entries + 1);
    drop(index);
    counts
}

fn measure_opened(root: &Path, expected_entries: u64) -> Counts {
    fdu_core::counters::reset();
    fdu_core::counters::enable(true);
    let options =
        OpenOptions { journal_capacity: OPENED_JOURNAL_CAPACITY, ..OpenOptions::default() };
    let opened = OpenedIndex::open(root, options).expect("opened discovery");
    let initial = opened.read(ReadRequest::default()).expect("initial opened read");
    let mut cursor = EngineVersion { sequence: Clock::ZERO, ..initial.version };
    loop {
        let poll = opened
            .changes(ChangeRequest { after: cursor, timeout: Duration::from_secs(30) })
            .expect("opened discovery changes");
        cursor = poll.cursor;
        match poll.outcome {
            ChangeOutcome::Changes { .. } => {}
            ChangeOutcome::Idle => panic!("opened discovery did not settle"),
            ChangeOutcome::Reset { .. } => panic!("opened discovery outran the test journal"),
        }
        if matches!(
            poll.state.phase,
            LifecyclePhase::Ready
                | LifecyclePhase::Watching
                | LifecyclePhase::Stopped
                | LifecyclePhase::Failed
        ) {
            break;
        }
    }
    opened.close().expect("close opened discovery");
    fdu_core::counters::flush_thread();
    let counts = fdu_core::counters::snapshot();
    fdu_core::counters::enable(false);
    assert_eq!(counts.opened_accepted_ops, expected_entries);
    counts
}

fn assert_allocation_slope(
    route: &str,
    smaller: u64,
    larger: u64,
    added_entries: u64,
    allocations_per_added_entry: u64,
) {
    let growth = larger.checked_sub(smaller).unwrap_or_else(|| {
        panic!("{route} allocations shrank from {smaller} to {larger} on the larger fixture")
    });
    let limit = added_entries.saturating_mul(allocations_per_added_entry);
    assert!(
        growth <= limit,
        "{route} allocations grew by {growth} for {added_entries} entries; limit is {limit}"
    );

    // Prove this ceiling has less than one allocation of slack per added entry. A
    // reintroduced path or name clone must make the measured fixture fail.
    let restored = growth.saturating_add(added_entries);
    assert!(
        restored > limit,
        "{route} allocation ceiling has at least one allocation per entry of slack: \
         growth {growth}, restored {restored}, limit {limit}"
    );
}

fn assert_route_is_detached(counts: &Counts, entries: u64) {
    assert_eq!(counts.detached_builds, 1);
    assert_eq!(counts.detached_entries, entries);
    assert_eq!(counts.baseline_batches, 1);
    assert_eq!(counts.baseline_accepted_ops, entries);

    let violations = detached_guard_violations(counts);
    assert!(violations.is_empty(), "detached guard violations: {violations:?}");

    // Prove the zero-work half is active rather than a list of counters that happens to
    // read zero in the current implementation.
    let mut one_exact_consequence = *counts;
    one_exact_consequence.effect_paths = 1;
    assert!(
        detached_guard_violations(&one_exact_consequence).contains(&"exact mutation work"),
        "the zero-work guard must reject an exact consequence"
    );
}

fn assert_route_is_opened(counts: &Counts, entries: u64) {
    assert_eq!(counts.opened_accepted_ops, entries);
    let violations = opened_guard_violations(counts);
    assert!(violations.is_empty(), "opened guard violations: {violations:?}");

    let mut one_detached_build = *counts;
    one_detached_build.detached_builds = 1;
    assert!(
        opened_guard_violations(&one_detached_build).contains(&"detached builder work"),
        "the opened route guard must reject detached-builder work"
    );
}

fn detached_guard_violations(counts: &Counts) -> Vec<&'static str> {
    let mut violations = Vec::new();
    if counts.scanner_prepare_us != 0
        || counts.scanner_control_projection_us != 0
        || counts.scanner_reduce_us != 0
        || counts.ancestry_overlay_inserts != 0
        || counts.ancestry_path_comparisons != 0
        || counts.ancestry_parent_proofs != 0
    {
        violations.push("streaming preparation work");
    }
    if counts.opened_batches != 0
        || counts.public_batches != 0
        || counts.effect_paths != 0
        || counts.effect_path_bytes != 0
        || counts.impact_candidates != 0
        || counts.impact_ancestor_visits != 0
        || counts.impact_retained_dirty_paths != 0
        || counts.impact_all_dirty != 0
        || counts.journal_retained_commits != 0
        || counts.journal_cloned_commits != 0
        || counts.journal_oversized_commits != 0
        || counts.journal_dropped_commits != 0
    {
        violations.push("exact mutation work");
    }
    violations
}

fn opened_guard_violations(counts: &Counts) -> Vec<&'static str> {
    let mut violations = Vec::new();
    if counts.detached_builds != 0
        || counts.detached_entries != 0
        || counts.detached_walk_us != 0
        || counts.detached_finish_us != 0
    {
        violations.push("detached builder work");
    }
    if counts.baseline_batches != 0
        || counts.public_batches != 0
        || counts.ancestry_overlay_inserts != 0
    {
        violations.push("wrong reducer work");
    }
    violations
}
