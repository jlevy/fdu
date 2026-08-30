//! Canonical transparent-box sessions for the opened-root contract.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use super::golden_support::{ContractCoverage, SessionTrace};
use super::{
    DiscoveryBudget, OpenOptions, OpenedIndex, TEST_GATE_TIMEOUT, TestControls, TestPoint,
};
use crate::{
    ChangeRequest, Clock, ContinuationId, EngineVersion, Error, Knowledge, LifecyclePhase,
    PageRequest, ProjectionResult, ReadProjection, ReadRequest, ReadResponse, ReportRequest,
    RowShape,
};

#[test]
fn opened_root_session_goldens() {
    let traces = [
        cold_progressive_knowledge(),
        exact_mutation_and_refresh(),
        coherent_projections_and_continuations(),
        journal_and_observation_recovery(),
        ownership_races_and_shutdown(),
    ];
    let mut coverage = ContractCoverage::default();
    for trace in &traces {
        coverage.merge(trace.coverage());
    }
    coverage.assert_required(REQUIRED_CONTRACT_OUTCOMES);
    for trace in traces {
        trace.assert_golden();
    }
}

const REQUIRED_CONTRACT_OUTCOMES: &[&str] = &[
    "operation.read",
    "operation.changes",
    "operation.refresh",
    "prioritize.ok",
    "close.ok",
    "read.ok",
    "changes.commits",
    "changes.idle",
    "changes.reset",
    "refresh.ok",
    "phase.discovering",
    "phase.reconciling",
    "phase.ready",
    "phase.watching",
    "phase.stopped",
    "coverage.complete",
    "coverage.partial",
    "coverage_reason.building",
    "coverage_reason.budget",
    "freshness.fresh",
    "freshness.reconciling",
    "freshness.stale",
    "knowledge.present",
    "knowledge.absent",
    "knowledge.unknown",
    "projection.lookup",
    "projection.rollup",
    "projection.tree",
    "projection.flat",
    "projection.aggregate",
    "projection.report",
    "projection.diagnostics",
    "projection.limit",
    "change.inserted",
    "change.updated",
    "change.removed",
    "change.control_updated",
    "change.reclassified",
    "change.invalidated",
    "transition.freshness",
    "transition.verified",
    "transition.directory_complete",
    "transition.index_state",
    "error.closed",
    "error.continuation_unavailable",
    "error.continuation_stale",
    "error.change_cursor_unavailable",
    "error.worker_panicked",
];

fn cold_progressive_knowledge() -> SessionTrace {
    let root = tempfile::tempdir().expect("progressive root");
    std::fs::create_dir(root.path().join("alpha")).expect("alpha fixture");
    std::fs::create_dir(root.path().join("target")).expect("target fixture");
    std::fs::write(root.path().join("root.txt"), b"root").expect("root fixture");
    std::fs::write(root.path().join("alpha/leaf.txt"), b"alpha").expect("alpha leaf");
    std::fs::write(root.path().join("target/leaf.txt"), b"target").expect("target leaf");

    let controls = deterministic_controls();
    controls.gate(TestPoint::AfterRootDirectory).arm();
    let options = OpenOptions {
        batch_size: 1,
        budget: DiscoveryBudget { max_files: Some(2) },
        ..OpenOptions::default()
    };
    let mut trace = SessionTrace::new("cold-progressive-knowledge", root.path());
    trace.record("action.open", &options);
    let opened = OpenedIndex::open_for_test(root.path(), options, Arc::clone(&controls))
        .expect("open progressive session");
    trace.bind_session(opened.state.session);
    trace.record("result.open", &opened);
    controls.gate(TestPoint::AfterRootDirectory).wait_reached();
    trace.record_text("barrier", "reached after-root-directory");

    let mut cursor = zero_cursor(&opened, &mut trace);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);
    let request = ReadRequest {
        projections: vec![
            ReadProjection::Lookup { path: PathBuf::from("root.txt") },
            ReadProjection::Lookup { path: PathBuf::from("target/leaf.txt") },
            ReadProjection::Lookup { path: PathBuf::from("missing.txt") },
            ReadProjection::RollUp { path: PathBuf::new() },
            ReadProjection::Diagnostics,
        ],
        expected: None,
    };
    let _ = read(&opened, &mut trace, request);
    prioritize(&opened, &mut trace, &[PathBuf::from("target")]);
    controls.gate(TestPoint::AfterRootDirectory).release();
    trace.record_text("barrier", "released after-root-directory");
    wait_for_phase(&opened, &mut trace, LifecyclePhase::Stopped);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);

    std::fs::write(root.path().join("late.txt"), b"late").expect("late fixture");
    refresh(&opened, &mut trace, &[PathBuf::from("late.txt")]);
    let _ = poll(&opened, &mut trace, cursor, Duration::ZERO);
    final_read(&opened, &mut trace);
    close(&opened, &mut trace);
    trace.record_text("final", "joined=true workers=0 waiters=0 continuations=0");
    trace
}

fn exact_mutation_and_refresh() -> SessionTrace {
    let root = tempfile::tempdir().expect("mutation root");
    let controls = deterministic_controls();
    let options = OpenOptions {
        hidden: Some(Arc::new(crate::HiddenPolicy::prune_hidden::<[&str; 0], &str>([]))),
        ..OpenOptions::default()
    };
    let mut trace = SessionTrace::new("exact-mutation-and-refresh", root.path());
    trace.record("action.open", &options);
    let opened =
        OpenedIndex::open_for_test(root.path(), options, controls).expect("open mutation session");
    trace.bind_session(opened.state.session);
    trace.record("result.open", &opened);
    wait_for_phase(&opened, &mut trace, LifecyclePhase::Ready);
    let mut cursor = zero_cursor(&opened, &mut trace);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);

    std::fs::write(root.path().join("kind"), b"one").expect("kind fixture");
    std::fs::write(root.path().join("same"), b"same").expect("same fixture");
    refresh(
        &opened,
        &mut trace,
        &[PathBuf::from("same"), PathBuf::from("kind"), PathBuf::from("kind")],
    );
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);

    std::fs::write(root.path().join("kind"), b"one-updated").expect("update fixture");
    refresh(&opened, &mut trace, &[PathBuf::from("kind")]);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);
    refresh(&opened, &mut trace, &[PathBuf::from("kind")]);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);

    std::fs::remove_file(root.path().join("kind")).expect("replace file");
    std::fs::create_dir(root.path().join("kind")).expect("replace directory");
    std::fs::write(root.path().join("kind/child"), b"child").expect("replacement child");
    refresh(&opened, &mut trace, &[PathBuf::from("kind")]);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);

    std::fs::write(root.path().join("debug.log"), b"log").expect("log fixture");
    std::fs::write(root.path().join(".gitignore"), b"*.log\n").expect("control fixture");
    refresh(&opened, &mut trace, &[PathBuf::from("debug.log"), PathBuf::from(".gitignore")]);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);
    std::fs::write(root.path().join(".gitignore"), b"*.tmp\n").expect("edit control");
    refresh(&opened, &mut trace, &[PathBuf::from(".gitignore")]);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);
    std::fs::remove_file(root.path().join(".gitignore")).expect("remove control");
    refresh(&opened, &mut trace, &[PathBuf::from(".gitignore")]);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);

    std::fs::remove_file(root.path().join("same")).expect("remove retained file");
    refresh(&opened, &mut trace, &[PathBuf::from("same")]);
    let _ = poll(&opened, &mut trace, cursor, Duration::ZERO);
    final_read(&opened, &mut trace);
    close(&opened, &mut trace);
    trace.record_text("final", "joined=true workers=0 waiters=0 continuations=0");
    trace
}

fn coherent_projections_and_continuations() -> SessionTrace {
    let root = tempfile::tempdir().expect("projection root");
    std::fs::create_dir(root.path().join("dir")).expect("directory fixture");
    for (path, contents) in [
        ("a.txt", b"a".as_slice()),
        ("b.txt", b"bb".as_slice()),
        ("c.txt", b"ccc".as_slice()),
        ("dir/leaf.rs", b"rust".as_slice()),
    ] {
        std::fs::write(root.path().join(path), contents).expect("projection fixture");
    }
    let controls = deterministic_controls();
    let options = OpenOptions::default();
    let mut trace = SessionTrace::new("coherent-projections-and-continuations", root.path());
    trace.record("action.open", &options);
    let opened = OpenedIndex::open_for_test(root.path(), options, controls)
        .expect("open projection session");
    trace.bind_session(opened.state.session);
    trace.record("result.open", &opened);
    wait_for_phase(&opened, &mut trace, LifecyclePhase::Ready);
    let mut cursor = zero_cursor(&opened, &mut trace);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);

    let page = PageRequest { limit: 1, max_work: 16 };
    let response = read(
        &opened,
        &mut trace,
        ReadRequest {
            projections: vec![
                ReadProjection::Lookup { path: PathBuf::from("a.txt") },
                ReadProjection::RollUp { path: PathBuf::new() },
                ReadProjection::Tree {
                    path: PathBuf::new(),
                    depth: crate::query::Bound::Limit(1),
                    include_ignored: true,
                    page,
                },
                ReadProjection::Flat {
                    selection: crate::query::EntrySelection::default(),
                    shape: RowShape::Full,
                    page: PageRequest { limit: 2, max_work: 16 },
                },
                ReadProjection::Aggregate {
                    selection: crate::query::EntrySelection::default(),
                    count_cap: 2,
                    max_work: 16,
                },
                ReadProjection::Report(ReportRequest {
                    query: crate::query::Query::default(),
                    generated_at: SystemTime::UNIX_EPOCH,
                    max_work: 16,
                }),
                ReadProjection::Diagnostics,
            ],
            expected: None,
        },
    )
    .expect("complete projection read");
    let tree_token = continuation(&response, 2);
    let _ = read(
        &opened,
        &mut trace,
        ReadRequest {
            projections: vec![ReadProjection::Continue { continuation: tree_token, page }],
            expected: Some(response.version),
        },
    );
    let _ = read(
        &opened,
        &mut trace,
        ReadRequest {
            projections: vec![ReadProjection::Continue { continuation: tree_token, page }],
            expected: None,
        },
    );

    let stale_source = read(
        &opened,
        &mut trace,
        ReadRequest {
            projections: vec![ReadProjection::Flat {
                selection: crate::query::EntrySelection::default(),
                shape: RowShape::Compact,
                page,
            }],
            expected: None,
        },
    )
    .expect("stale-token source");
    let stale = continuation(&stale_source, 0);
    std::fs::write(root.path().join("z.txt"), b"z").expect("advance projection version");
    refresh(&opened, &mut trace, &[PathBuf::from("z.txt")]);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);
    let _ = read(
        &opened,
        &mut trace,
        ReadRequest {
            projections: vec![ReadProjection::Continue { continuation: stale, page }],
            expected: None,
        },
    );

    let foreign_source = read(
        &opened,
        &mut trace,
        ReadRequest {
            projections: vec![ReadProjection::Flat {
                selection: crate::query::EntrySelection::default(),
                shape: RowShape::Compact,
                page,
            }],
            expected: None,
        },
    )
    .expect("foreign-token source");
    let foreign = continuation(&foreign_source, 0);
    let other_root = tempfile::tempdir().expect("foreign root");
    trace.alias_path(other_root.path(), "$OTHER_ROOT");
    let other = OpenedIndex::open(other_root.path(), OpenOptions::default()).expect("other open");
    trace.bind_session(other.state.session);
    trace.record("action.read.foreign", &foreign);
    let foreign_result = other.read(ReadRequest {
        projections: vec![ReadProjection::Continue { continuation: foreign, page }],
        expected: None,
    });
    trace.record("result.read.foreign", &foreign_result);
    trace.observe_read(&foreign_result);
    close(&other, &mut trace);

    let future = EngineVersion { sequence: Clock(cursor.sequence.0 + 1), ..cursor };
    trace.record("action.changes.future", &future);
    let future_result = opened.changes(ChangeRequest { after: future, timeout: Duration::ZERO });
    trace.record("result.changes.future", &future_result);
    trace.observe_poll(&future_result);

    let limited = read(
        &opened,
        &mut trace,
        ReadRequest {
            projections: vec![ReadProjection::Flat {
                selection: crate::query::EntrySelection::default(),
                shape: RowShape::Compact,
                page: PageRequest { limit: 1, max_work: 1 },
            }],
            expected: None,
        },
    );
    assert!(matches!(
        limited,
        Ok(ReadResponse { results, .. })
            if matches!(results.as_slice(), [ProjectionResult::Limit(_)])
    ));
    final_read(&opened, &mut trace);
    close(&opened, &mut trace);
    let closed = opened.read(ReadRequest::default());
    trace.record("result.read.after-close", &closed);
    trace.observe_read(&closed);
    trace.record_text("final", "joined=true workers=0 waiters=0 continuations=0");
    trace
}

fn journal_and_observation_recovery() -> SessionTrace {
    let root = tempfile::tempdir().expect("observation root");
    let scripts = tempfile::tempdir().expect("observation scripts");
    let script = scripts.path().join("events.script");
    std::fs::write(root.path().join("baseline.txt"), b"before").expect("baseline fixture");
    std::fs::write(&script, b"modify\tbaseline.txt\n").expect("initial observation script");
    let controls = deterministic_controls();
    controls.gate(TestPoint::BeforeDiscovery).arm();
    let options = scripted_options(&script, 32);
    let mut trace = SessionTrace::new("journal-and-observation-recovery", root.path());
    trace.alias_path(scripts.path(), "$SCRIPT_ROOT");
    trace.record("action.open", &options);
    let opened = OpenedIndex::open_for_test(root.path(), options, Arc::clone(&controls))
        .expect("open observed session");
    trace.bind_session(opened.state.session);
    trace.record("result.open", &opened);
    controls.gate(TestPoint::BeforeDiscovery).wait_reached();
    trace.record_text("barrier", "reached before-discovery");
    std::fs::write(root.path().join("baseline.txt"), b"changed-during-baseline")
        .expect("mutate during baseline");
    trace.record_text("action.fixture", "write baseline.txt size=23");
    controls.gate(TestPoint::BeforeDiscovery).release();
    trace.record_text("barrier", "released before-discovery");
    wait_for_phase(&opened, &mut trace, LifecyclePhase::Watching);
    let mut cursor = zero_cursor(&opened, &mut trace);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);
    cursor = poll(&opened, &mut trace, cursor, Duration::ZERO);

    let reset_cursor = cursor;
    let mut bulk_paths = Vec::new();
    for index in 0..12 {
        let relative = PathBuf::from(format!("bulk-{index:02}.txt"));
        std::fs::write(root.path().join(&relative), b"bulk").expect("bulk fixture");
        bulk_paths.push(relative);
    }
    trace.record_text("action.fixture", "write bulk-00.txt..bulk-11.txt");
    refresh(&opened, &mut trace, &bulk_paths);
    cursor = poll(&opened, &mut trace, reset_cursor, Duration::ZERO);

    std::fs::write(root.path().join("recovered.txt"), b"recovered").expect("unobserved fixture");
    trace.record_text("action.fixture", "write recovered.txt without a precise hint");
    trace.record_text("action.observer-hint", "rescan .");
    controls.send_observation_hints("rescan\t.\n");
    wait_until_path(&opened, Path::new("recovered.txt"));
    wait_for_watching_fresh(&opened, &mut trace);
    let _ = poll(&opened, &mut trace, cursor, Duration::ZERO);
    final_read(&opened, &mut trace);
    close(&opened, &mut trace);
    trace.record_text("final", "joined=true workers=0 waiters=0 continuations=0");
    trace
}

fn ownership_races_and_shutdown() -> SessionTrace {
    let root = tempfile::tempdir().expect("ownership root");
    let controls = deterministic_controls();
    controls.discovery_disabled.store(true, std::sync::atomic::Ordering::Release);
    let options = OpenOptions::default();
    let mut trace = SessionTrace::new("ownership-races-and-shutdown", root.path());
    trace.record("action.open", &options);
    let opened = OpenedIndex::open_for_test(root.path(), options, Arc::clone(&controls))
        .expect("open ownership session");
    trace.bind_session(opened.state.session);
    trace.record("result.open", &opened);
    let cursor = zero_cursor(&opened, &mut trace);

    controls.gate(TestPoint::BeforeJournalWait).arm();
    trace.record_text("action.changes.blocking", "changes(after=current, timeout=1s)");
    let poller = opened.clone();
    let blocked_poll = std::thread::spawn(move || {
        poller.changes(ChangeRequest { after: cursor, timeout: Duration::from_secs(1) })
    });
    controls.gate(TestPoint::BeforeJournalWait).wait_reached();
    trace.record_text("barrier", "reached before-journal-wait");
    let closer = opened.clone();
    trace.record_text("action.close", "close() from clone while change poll is blocked");
    let joined_close = std::thread::spawn(move || closer.close());
    wait_until_cancelled(&opened);
    trace.record_text("barrier", "owner cancellation published");
    controls.gate(TestPoint::BeforeJournalWait).release();
    trace.record_text("barrier", "released before-journal-wait");
    let poll_result = blocked_poll.join().expect("blocked poll thread");
    trace.record("result.changes.blocking", &poll_result);
    trace.observe_poll(&poll_result);
    let close_result = joined_close.join().expect("concurrent close thread");
    trace.record("result.close.concurrent", &close_result);
    trace.observe_close(&close_result);
    let repeat = opened.close();
    trace.record("result.close.repeat", &repeat);
    trace.observe_close(&repeat);

    let race_root = tempfile::tempdir().expect("refresh race root");
    trace.alias_path(race_root.path(), "$RACE_ROOT");
    let race_controls = deterministic_controls();
    race_controls.discovery_disabled.store(true, std::sync::atomic::Ordering::Release);
    let raced = OpenedIndex::open_for_test(
        race_root.path(),
        OpenOptions::default(),
        Arc::clone(&race_controls),
    )
    .expect("open refresh race session");
    trace.bind_session(raced.state.session);
    trace.record("result.open.race", &raced);
    std::fs::write(race_root.path().join("late.txt"), b"late").expect("late race fixture");
    race_controls.gate(TestPoint::AfterRefreshVerification).arm();
    trace.record_text("action.refresh.concurrent", "refresh([late.txt])");
    let refresher = raced.clone();
    let refresh_thread =
        std::thread::spawn(move || refresher.refresh(&[PathBuf::from("late.txt")]));
    race_controls.gate(TestPoint::AfterRefreshVerification).wait_reached();
    trace.record_text("barrier", "reached after-refresh-verification");
    let race_closer = raced.clone();
    let race_close = std::thread::spawn(move || race_closer.close());
    wait_until_cancelled(&raced);
    race_controls.gate(TestPoint::AfterRefreshVerification).release();
    trace.record_text("barrier", "released after-refresh-verification");
    let refresh_result = refresh_thread.join().expect("refresh race thread");
    trace.record("result.refresh.concurrent", &refresh_result);
    trace.observe_refresh(&refresh_result);
    let race_close_result = race_close.join().expect("refresh close thread");
    trace.record("result.close.race", &race_close_result);
    trace.observe_close(&race_close_result);

    let panic_root = tempfile::tempdir().expect("panic root");
    trace.alias_path(panic_root.path(), "$PANIC_ROOT");
    let panic_controls = deterministic_controls();
    panic_controls.discovery_disabled.store(true, std::sync::atomic::Ordering::Release);
    let panicked =
        OpenedIndex::open_for_test(panic_root.path(), OpenOptions::default(), panic_controls)
            .expect("open panic session");
    trace.bind_session(panicked.state.session);
    trace.record("result.open.panic", &panicked);
    trace.record_text("action.fault", "panic worker=golden-panic");
    panicked
        .spawn_worker("golden-panic", |_cancellation| panic!("injected golden worker panic"))
        .expect("spawn injected panic worker");
    let panic_close = panicked.close();
    trace.record("result.close.panic", &panic_close);
    trace.observe_close(&panic_close);
    assert!(matches!(panic_close, Err(Error::OpenedWorkerPanicked { .. })));
    trace.record_text("final", "joined=true workers=0 waiters=0 continuations=0");
    trace
}

fn zero_cursor(opened: &OpenedIndex, trace: &mut SessionTrace) -> EngineVersion {
    let response = read(opened, trace, ReadRequest::default()).expect("initial version read");
    EngineVersion { sequence: Clock::ZERO, ..response.version }
}

fn deterministic_controls() -> Arc<TestControls> {
    let controls = Arc::new(TestControls::default());
    controls.use_deterministic_discovery_order();
    controls
}

fn read(
    opened: &OpenedIndex,
    trace: &mut SessionTrace,
    request: ReadRequest,
) -> crate::Result<ReadResponse> {
    trace.record("action.read", &request);
    let result = opened.read(request);
    trace.record("result.read", &result);
    trace.observe_read(&result);
    result
}

fn poll(
    opened: &OpenedIndex,
    trace: &mut SessionTrace,
    after: EngineVersion,
    timeout: Duration,
) -> EngineVersion {
    let request = ChangeRequest { after, timeout };
    trace.record("action.changes", &request);
    let result = opened.changes(request);
    trace.record("result.changes", &result);
    trace.observe_poll(&result);
    trace.verify_poll(opened, &result);
    result.expect("change poll").cursor
}

fn refresh(opened: &OpenedIndex, trace: &mut SessionTrace, paths: &[PathBuf]) {
    trace.record("action.refresh", &paths);
    let result = opened.refresh(paths);
    trace.record("result.refresh", &result);
    trace.observe_refresh(&result);
    result.expect("refresh result");
}

fn prioritize(opened: &OpenedIndex, trace: &mut SessionTrace, paths: &[PathBuf]) {
    trace.record("action.prioritize", &paths);
    let result = opened.prioritize(paths);
    trace.record("result.prioritize", &result);
    trace.observe_priority(&result);
    result.expect("priority result");
}

fn close(opened: &OpenedIndex, trace: &mut SessionTrace) {
    trace.record_text("action.close", "close()");
    let result = opened.close();
    trace.record("result.close", &result);
    trace.observe_close(&result);
}

fn wait_for_phase(opened: &OpenedIndex, trace: &mut SessionTrace, expected: LifecyclePhase) {
    let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
    loop {
        let state = opened.state.index.state().expect("read phase");
        if state.phase == expected {
            trace.record("barrier.phase", &state);
            trace.observe_state(state);
            return;
        }
        assert!(std::time::Instant::now() < deadline, "phase did not become {expected:?}");
        std::thread::yield_now();
    }
}

fn continuation(response: &ReadResponse, result_index: usize) -> ContinuationId {
    match &response.results[result_index] {
        ProjectionResult::Tree(Knowledge::Present(page)) => page.next.expect("tree continuation"),
        ProjectionResult::Flat(page) => page.next.expect("flat continuation"),
        other => panic!("projection did not return a continuation: {other:?}"),
    }
}

fn final_read(opened: &OpenedIndex, trace: &mut SessionTrace) {
    let _ = read(
        opened,
        trace,
        ReadRequest {
            projections: vec![
                ReadProjection::Diagnostics,
                ReadProjection::RollUp { path: PathBuf::new() },
                ReadProjection::Tree {
                    path: PathBuf::new(),
                    depth: crate::query::Bound::Limit(1),
                    include_ignored: true,
                    page: PageRequest { limit: 4_096, max_work: 1_000_000 },
                },
                ReadProjection::Flat {
                    selection: crate::query::EntrySelection::default(),
                    shape: RowShape::Full,
                    page: PageRequest { limit: 4_096, max_work: 1_000_000 },
                },
            ],
            expected: None,
        },
    );
}

fn scripted_options(script: &Path, journal_capacity: usize) -> OpenOptions {
    OpenOptions {
        observation: Some(crate::watch::WatchConfig {
            settle: Duration::from_millis(1),
            max_hold: Duration::from_millis(10),
            ..crate::watch::WatchConfig::default()
        }),
        observation_script: Some(script.to_path_buf()),
        journal_capacity,
        ..OpenOptions::default()
    }
}

fn wait_until_path(opened: &OpenedIndex, path: &Path) {
    let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
    loop {
        if opened.state.index.kind(path).expect("path lookup").is_some() {
            return;
        }
        assert!(std::time::Instant::now() < deadline, "path was not observed: {}", path.display());
        std::thread::yield_now();
    }
}

fn wait_until_cancelled(opened: &OpenedIndex) {
    let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
    while !opened.state.cancellation.is_cancelled() {
        assert!(std::time::Instant::now() < deadline, "close did not publish cancellation");
        std::thread::yield_now();
    }
}

fn wait_for_watching_fresh(opened: &OpenedIndex, trace: &mut SessionTrace) {
    let deadline = std::time::Instant::now() + TEST_GATE_TIMEOUT;
    loop {
        let state = opened.state.index.state().expect("read watching freshness");
        if state.phase == LifecyclePhase::Watching && state.freshness == crate::Freshness::Fresh {
            trace.record("barrier.watching-fresh", &state);
            trace.observe_state(state);
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "observation did not return to watching and fresh"
        );
        std::thread::yield_now();
    }
}
