//! End-to-end behaviour of a live watch session.
//!
//! These run against real filesystem events rather than injected observations, because
//! the property under test is that fdu sees what the operating system reports. Each test
//! makes one change and waits for it, so ordering is determined by the change rather than
//! by a sleep.
#![cfg(all(feature = "watch", unix))]

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use fdu_core::query::{Bound, Query, Selection, ViewSpec};
use fdu_core::session::{ChangeKind, Session};
use fdu_core::watch::WatchConfig;
use fdu_core::{CachePolicy, IndexHandle, OpenConfig, ScanConfig, open};

/// Long enough for a backend to deliver and coalesce, short enough to fail fast.
const SETTLE: Duration = Duration::from_secs(60);

fn session(root: &Path, selection: Selection, views: Vec<ViewSpec>) -> Session {
    let config = OpenConfig { policy: CachePolicy::Off, ..OpenConfig::default() };
    let (index, _report) = open(root, &config).expect("open");
    Session::new(
        IndexHandle::new(index),
        ScanConfig::default(),
        Query { selection, views, ..Query::default() },
        WatchConfig::default(),
    )
    .expect("session")
}

/// Collect changes until `wanted` matches one, or the settle window expires.
fn wait_for(
    session: &mut Session,
    wanted: impl Fn(&fdu_core::Change) -> bool,
) -> Option<fdu_core::Change> {
    let deadline = Instant::now() + SETTLE;
    while Instant::now() < deadline {
        let Some(batch) = session.next_batch(Duration::from_millis(250)).expect("batch") else {
            continue;
        };
        if let Some(found) = batch.changes.into_iter().find(&wanted) {
            return Some(found);
        }
    }
    None
}

#[test]
fn a_created_file_arrives_as_an_upsert() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("existing.txt"), b"hello").expect("seed");
    let mut session = session(dir.path(), Selection::default(), vec![ViewSpec::Files]);

    fs::write(dir.path().join("created.rs"), b"fn main() {}").expect("create");

    let change = wait_for(&mut session, |change| change.path.ends_with("created.rs"))
        .expect("the created file should arrive");
    assert_eq!(change.kind, ChangeKind::Upsert);
    assert_eq!(change.bytes, Some(12));
    assert!(change.mtime_ns.is_some(), "an upsert carries verified metadata, not just a path");
}

#[test]
fn a_deleted_file_arrives_as_a_remove() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("doomed.txt"), b"hello").expect("seed");
    let mut session = session(dir.path(), Selection::default(), vec![ViewSpec::Files]);

    fs::remove_file(dir.path().join("doomed.txt")).expect("remove");

    let change = wait_for(&mut session, |change| change.path.ends_with("doomed.txt"))
        .expect("the deletion should arrive");
    assert_eq!(change.kind, ChangeKind::Remove);
    assert_eq!(change.bytes, None, "a removed entry has no attributes to report");
}

#[test]
fn the_run_selection_filters_the_stream() {
    // Watch is the same query repeated: the filter that shapes a one-shot listing shapes
    // the live stream too, with no separate watch grammar.
    let dir = tempfile::tempdir().expect("tempdir");
    let selection = Selection {
        include: vec![fdu_core::query::Pattern::parse("*.rs").expect("pattern")],
        ..Selection::default()
    };
    let mut session = session(dir.path(), selection, vec![ViewSpec::Files]);

    fs::write(dir.path().join("ignored.txt"), b"no").expect("create");
    fs::write(dir.path().join("watched.rs"), b"yes").expect("create");

    let change = wait_for(&mut session, |change| change.path.ends_with("watched.rs"))
        .expect("the matching file should arrive");
    assert_eq!(change.kind, ChangeKind::Upsert);

    // Drain briefly and confirm the excluded path never appears.
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if let Some(batch) = session.next_batch(Duration::from_millis(200)).expect("batch") {
            assert!(
                !batch.changes.iter().any(|change| change.path.ends_with("ignored.txt")),
                "a filtered path must not reach the stream"
            );
        }
    }
}

#[test]
fn an_idle_tree_yields_nothing_and_costs_nothing() {
    // The efficiency contract: detection is event-driven, so an unchanging tree produces
    // no batches at all. A polling implementation would return work here.
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("still.txt"), b"unchanged").expect("seed");
    let mut session = session(dir.path(), Selection::default(), vec![ViewSpec::Summary]);

    // Establish quiet by *positive confirmation*, not by waiting for silence.
    //
    // Waiting for silence cannot work here, and the previous two attempts at it failed
    // for the same reason in different disguises. The seed write lands just before the
    // watcher binds, so its events may still be in flight; a run of empty polls proves
    // only that none has arrived *yet*, and on a loaded machine one can arrive after
    // three consecutive quiet polls and land in the assertion below — reported as "an
    // idle tree must produce no batches", which is a statement about the product
    // contract and not about what went wrong.
    //
    // Writing a sentinel and waiting for its own change is deterministic: the event is
    // one this test caused, so its arrival is a fact rather than a timeout, and it
    // cannot be delivered before the seed events that preceded it. Once it lands, the
    // backend has demonstrably drained everything older, and any further batch is
    // genuinely spurious — which is exactly the claim the assertion wants to make.
    fs::write(dir.path().join("sentinel.txt"), b"sentinel").expect("sentinel");
    assert!(
        wait_for(&mut session, |change| change.path.ends_with("sentinel.txt")).is_some(),
        "the backend never delivered the sentinel within {SETTLE:?}, so the tree was never \
         known to be drained; this is a delivery timeout, not a claim about idleness"
    );

    // Nothing has changed since the sentinel, so nothing should arrive. A polling
    // implementation would still return work here.
    for _ in 0..4 {
        assert!(
            session.next_batch(Duration::from_millis(250)).expect("batch").is_none(),
            "an idle tree must produce no batches"
        );
    }
}

#[test]
fn a_live_report_is_the_same_query_re_evaluated() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("a.txt"), b"12345").expect("seed");
    let mut session = session(
        dir.path(),
        Selection { depth: Some(Bound::All), ..Selection::default() },
        vec![ViewSpec::Summary],
    );

    let before = session
        .report(&session.live_provenance(std::time::SystemTime::UNIX_EPOCH))
        .expect("report");
    let first = match &before.sections[0] {
        fdu_core::query::Section::Summary(row) => *row,
        other => panic!("expected a summary, got {other:?}"),
    };
    assert_eq!(first.files, 1);

    fs::write(dir.path().join("b.txt"), b"678").expect("create");
    wait_for(&mut session, |change| change.path.ends_with("b.txt"))
        .expect("the change should arrive");

    let after = session
        .report(&session.live_provenance(std::time::SystemTime::UNIX_EPOCH))
        .expect("report");
    let second = match &after.sections[0] {
        fdu_core::query::Section::Summary(row) => *row,
        other => panic!("expected a summary, got {other:?}"),
    };
    assert_eq!(second.files, 2, "the live report reflects the applied change");
    assert_eq!(second.bytes, first.bytes + 3);
}
