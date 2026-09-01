//! Deterministic structural guards for one-shot and progressive index construction.
//!
//! This is a separate test binary so its counting allocator observes no concurrent test
//! traffic. The guard constrains work rather than elapsed time and is therefore suitable
//! for shared CI.

use std::fs;
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

const DIRECTORY_COUNT: u64 = 64;
const FILES_PER_DIRECTORY: u64 = 64;
const FIXED_ALLOCATION_ALLOWANCE: u64 = 512;
const DETACHED_ALLOCATIONS_PER_ENTRY: u64 = 8;
const OPENED_ALLOCATIONS_PER_ENTRY: u64 = 26;
const OPENED_JOURNAL_CAPACITY: usize = 4 * 1024 * 1024;

#[test]
fn construction_routes_keep_their_allocation_and_work_boundaries() {
    struct DisableCounters;

    impl Drop for DisableCounters {
        fn drop(&mut self) {
            fdu_core::counters::enable(false);
            fdu_core::counters::reset();
        }
    }

    let root = tempfile::tempdir().expect("detached invariant root");
    for directory in 0..DIRECTORY_COUNT {
        let directory = root.path().join(format!("d{directory}"));
        fs::create_dir(&directory).expect("fixture directory");
        for file in 0..FILES_PER_DIRECTORY {
            fs::write(directory.join(format!("f{file}.dat")), []).expect("fixture file");
        }
    }
    let config = ScanConfig { read_controls: false, threads: Some(1), ..ScanConfig::default() };

    fdu_core::counters::enable(true);
    fdu_core::counters::reset();
    let _disable = DisableCounters;
    let (index, report) = scan_into_index(root.path(), &config).expect("detached scan");
    fdu_core::counters::flush_thread();
    let counts = fdu_core::counters::snapshot();
    fdu_core::counters::enable(false);

    let entries = DIRECTORY_COUNT + DIRECTORY_COUNT * FILES_PER_DIRECTORY;
    assert_eq!(report.entries, entries);
    assert_eq!(index.len(), entries + 1);
    assert_eq!(counts.detached_builds, 1);
    assert_eq!(counts.detached_entries, entries);
    assert_eq!(counts.baseline_batches, 1);
    assert_eq!(counts.baseline_accepted_ops, entries);

    let violations = detached_guard_violations(&counts, entries);
    assert!(violations.is_empty(), "detached guard violations: {violations:?}");

    // Prove the allocation ceiling has less than one allocation of slack per entry.
    // A reintroduced per-entry path or name clone must make this fixture fail.
    let mut one_extra_allocation_per_entry = counts;
    one_extra_allocation_per_entry.allocs =
        one_extra_allocation_per_entry.allocs.saturating_add(entries);
    assert!(
        detached_guard_violations(&one_extra_allocation_per_entry, entries)
            .contains(&"allocation ceiling"),
        "the allocation guard must reject one extra allocation per entry"
    );

    // Prove the zero-work half is active rather than a list of counters that happens to
    // read zero in the current implementation.
    let mut one_exact_consequence = counts;
    one_exact_consequence.effect_paths = 1;
    assert!(
        detached_guard_violations(&one_exact_consequence, entries).contains(&"exact mutation work"),
        "the zero-work guard must reject an exact consequence"
    );

    drop(index);
    fdu_core::counters::reset();
    fdu_core::counters::enable(true);
    let options =
        OpenOptions { journal_capacity: OPENED_JOURNAL_CAPACITY, ..OpenOptions::default() };
    let opened = OpenedIndex::open(root.path(), options).expect("opened discovery");
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
    let opened_counts = fdu_core::counters::snapshot();
    fdu_core::counters::enable(false);

    assert_eq!(opened_counts.opened_accepted_ops, entries);
    let violations = opened_guard_violations(&opened_counts, entries);
    assert!(violations.is_empty(), "opened guard violations: {violations:?}");

    let mut one_extra_allocation_per_entry = opened_counts;
    one_extra_allocation_per_entry.allocs =
        one_extra_allocation_per_entry.allocs.saturating_add(entries);
    assert!(
        opened_guard_violations(&one_extra_allocation_per_entry, entries)
            .contains(&"allocation ceiling"),
        "the opened allocation guard must reject one extra allocation per entry"
    );

    let mut one_detached_build = opened_counts;
    one_detached_build.detached_builds = 1;
    assert!(
        opened_guard_violations(&one_detached_build, entries).contains(&"detached builder work"),
        "the opened route guard must reject detached-builder work"
    );
}

fn detached_guard_violations(counts: &Counts, entries: u64) -> Vec<&'static str> {
    let allocation_limit = entries
        .saturating_mul(DETACHED_ALLOCATIONS_PER_ENTRY)
        .saturating_add(FIXED_ALLOCATION_ALLOWANCE);
    let mut violations = Vec::new();
    if counts.allocs > allocation_limit {
        violations.push("allocation ceiling");
    }
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

fn opened_guard_violations(counts: &Counts, entries: u64) -> Vec<&'static str> {
    let allocation_limit = entries
        .saturating_mul(OPENED_ALLOCATIONS_PER_ENTRY)
        .saturating_add(FIXED_ALLOCATION_ALLOWANCE);
    let mut violations = Vec::new();
    if counts.allocs > allocation_limit {
        violations.push("allocation ceiling");
    }
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
