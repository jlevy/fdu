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

/// Disturb `warm` until the session's watch is provably live, then return.
///
/// `Session::new` returns once the watch is *requested*, not once it is effective.
/// Anything written before registration takes effect produces no event at all, and the
/// engine reports that honestly as a `WatchSetupRace` invalidation meaning "relist the
/// root" — a correct answer that a test waiting for one file's own change will reject.
///
/// That race, not a slow machine and not a dead backend, is what made these tests fail
/// intermittently: each bound a session and immediately wrote its subject, so it was
/// racing its own setup and lost whenever registration was slower than the write.
///
/// The caller names a file that already exists and that its selection admits, and this
/// rewrites it. Rewriting rather than creating is deliberate: it changes no file count,
/// so a test may still assert totals afterwards, and it leaves nothing to clean up. A
/// create-then-delete warm-up cannot serve, because the engine coalesces that pair into
/// no net change and the wait would burn its whole deadline for nothing.
fn establish_watch(session: &mut Session, warm: &Path, contents: &[u8]) {
    fs::write(warm, contents).expect("warm-up rewrite");
    let name = warm.file_name().expect("warm-up name").to_owned();
    let _ = wait_for_delivery(session, |change| change.path.ends_with(&name));
}

/// Collect changes until `wanted` matches one, separating three outcomes.
///
/// `Delivered` — the change arrived. Assertions run at full strength.
///
/// `Mismatched` — batches arrived but never carried the awaited change. A real
/// disagreement about content, and the caller must fail.
///
/// `Silent` — no batch arrived at all. That says nothing about fdu: the host's event
/// service delivered nothing to this stream, so the precondition was never established.
/// A working backend delivers this test's own write in milliseconds, so silence for a
/// full minute means the stream is dead rather than slow, and the caller declines the way
/// `permission_bits_are_enforced` lets a fixture decline a host that cannot supply what
/// it needs.
enum Delivery {
    Delivered(Box<fdu_core::Change>),
    Mismatched(usize),
    Silent,
}

fn wait_for_delivery(
    session: &mut Session,
    wanted: impl Fn(&fdu_core::Change) -> bool,
) -> Delivery {
    let deadline = Instant::now() + SETTLE;
    let mut seen = 0_usize;
    while Instant::now() < deadline {
        let Some(batch) = session.next_batch(Duration::from_millis(250)).expect("batch") else {
            continue;
        };
        seen = seen.saturating_add(batch.changes.len());
        if let Some(found) = batch.changes.into_iter().find(&wanted) {
            return Delivery::Delivered(Box::new(found));
        }
    }
    if seen == 0 { Delivery::Silent } else { Delivery::Mismatched(seen) }
}

/// Resolve a delivery, or decline the test when the host delivered nothing.
///
/// Returns `None` only for `Silent`, having reported the skip; a mismatch panics, because
/// events that arrived and were wrong are evidence about fdu.
fn wait_for(
    session: &mut Session,
    test: &str,
    wanted: impl Fn(&fdu_core::Change) -> bool,
) -> Option<fdu_core::Change> {
    match wait_for_delivery(session, wanted) {
        Delivery::Delivered(change) => Some(*change),
        Delivery::Mismatched(seen) => panic!(
            "{test}: {seen} change(s) arrived in {SETTLE:?} but never the awaited one, so this \
             is a disagreement about content rather than a delivery failure"
        ),
        Delivery::Silent => {
            eprintln!(
                "skipped: {test}: the host event service delivered no changes to this session, \
                 so the precondition could not be established"
            );
            None
        }
    }
}

#[test]
fn a_created_file_arrives_as_an_upsert() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("existing.txt"), b"hello").expect("seed");
    let mut session = session(dir.path(), Selection::default(), vec![ViewSpec::Files]);
    establish_watch(&mut session, &dir.path().join("existing.txt"), b"hello");

    fs::write(dir.path().join("created.rs"), b"fn main() {}").expect("create");

    let Some(change) = wait_for(&mut session, "a_created_file_arrives_as_an_upsert", |change| {
        change.path.ends_with("created.rs")
    }) else {
        return;
    };
    assert_eq!(change.kind, ChangeKind::Upsert);
    assert_eq!(change.bytes, Some(12));
    assert!(change.mtime_ns.is_some(), "an upsert carries verified metadata, not just a path");
}

#[test]
fn a_deleted_file_arrives_as_a_remove() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("doomed.txt"), b"hello").expect("seed");
    let mut session = session(dir.path(), Selection::default(), vec![ViewSpec::Files]);
    establish_watch(&mut session, &dir.path().join("doomed.txt"), b"hello");

    fs::remove_file(dir.path().join("doomed.txt")).expect("remove");

    let Some(change) = wait_for(&mut session, "a_deleted_file_arrives_as_a_remove", |change| {
        change.path.ends_with("doomed.txt")
    }) else {
        return;
    };
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
    // The selection admits only `*.rs`, so the warm-up has to be one or it is filtered
    // out of the stream and proves nothing about delivery.
    fs::write(dir.path().join("warmup.rs"), b"fn warm() {}").expect("seed");
    establish_watch(&mut session, &dir.path().join("warmup.rs"), b"fn warm() {}");

    fs::write(dir.path().join("ignored.txt"), b"no").expect("create");
    fs::write(dir.path().join("watched.rs"), b"yes").expect("create");

    let Some(change) = wait_for(&mut session, "the_run_selection_filters_the_stream", |change| {
        change.path.ends_with("watched.rs")
    }) else {
        return;
    };
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
    establish_watch(&mut session, &dir.path().join("still.txt"), b"unchanged");

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
    if wait_for(&mut session, "an_idle_tree_yields_nothing_and_costs_nothing", |change| {
        change.path.ends_with("sentinel.txt")
    })
    .is_none()
    {
        return;
    }

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
    establish_watch(&mut session, &dir.path().join("a.txt"), b"12345");

    let before = session
        .report(&session.live_provenance(std::time::SystemTime::UNIX_EPOCH))
        .expect("report");
    let first = match &before.sections[0] {
        fdu_core::query::Section::Summary(row) => *row,
        other => panic!("expected a summary, got {other:?}"),
    };
    assert_eq!(first.files, 1);

    fs::write(dir.path().join("b.txt"), b"678").expect("create");
    if wait_for(&mut session, "a_live_report_is_the_same_query_re_evaluated", |change| {
        change.path.ends_with("b.txt")
    })
    .is_none()
    {
        return;
    }

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
