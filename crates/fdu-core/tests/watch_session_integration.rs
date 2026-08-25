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
const SETTLE: Duration = Duration::from_secs(10);

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

    // Drain first. A backend may still be delivering events for the seed write, which
    // happened just before the watcher was bound — that is arrival latency, not activity,
    // and asserting through it makes the test fail on a loaded machine rather than on a
    // real defect.
    // Requires a *sustained* quiet period, not one empty poll. A single `None` can simply
    // mean the backend has not delivered the seed event yet, and breaking on it lets that
    // late batch land in the assertion below — a flake that looks like a polling bug.
    let settle = Instant::now() + Duration::from_secs(3);
    let mut quiet_polls = 0;
    while Instant::now() < settle && quiet_polls < 3 {
        if session.next_batch(Duration::from_millis(200)).expect("batch").is_none() {
            quiet_polls += 1;
        } else {
            quiet_polls = 0;
        }
    }

    // Now nothing is changing, so nothing should arrive. A polling implementation would
    // still return work here.
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

/// A session and the handle it was opened from are one authority, not two.
///
/// `Session` took an `IndexHandle` and the Python binding handed it
/// `IndexHandle::new(inner.snapshot())` -- a deep clone into a second index. Every
/// mutation the watcher applied then landed somewhere the opener could not see, so a
/// server holding that index served numbers that stopped being true at the first event,
/// with nothing in the answer saying so. A handle clone is an `Arc` to the same lock, and
/// that is what a session should be given.
///
/// Reported as FDU47-R2 and tracked as `fdu-37dv`.
#[test]
fn a_session_mutates_the_handle_it_was_opened_from() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("a.txt"), b"12345").expect("seed");

    let config = OpenConfig { policy: CachePolicy::Off, ..OpenConfig::default() };
    let (index, _report) = open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);
    // What the caller keeps. Sharing is the whole claim, so the test has to hold the
    // handle the session was built from rather than ask the session for one.
    let mut session = Session::new(
        handle.clone(),
        ScanConfig::default(),
        Query {
            selection: Selection::default(),
            views: vec![ViewSpec::Summary],
            ..Query::default()
        },
        WatchConfig::default(),
    )
    .expect("session");

    let before = handle.clock().expect("clock");
    fs::write(dir.path().join("b.txt"), b"678").expect("create");
    wait_for(&mut session, |change| change.path.ends_with("b.txt"))
        .expect("the change should arrive");

    assert_ne!(
        handle.clock().expect("clock"),
        before,
        "a mutation the session applied must be visible from the opened handle"
    );
    let total = handle.total().expect("total");
    assert_eq!(total.files, 2, "and the numbers it carries must be the new ones");

    // Dropping the session drops a reference, not the index. This is the fear the deep
    // clone was defending against; sharing an `Arc` makes it a non-event.
    drop(session);
    assert_eq!(handle.total().expect("total").files, 2);
}

/// A batch's cursor names a commit the batch carried, never one it did not.
///
/// `next_batch` used to let the watcher's write guards drop and then sample
/// `index.cursor()`. A refresh committing in that window produced a batch with no record
/// of it and a cursor past it, so resuming skipped that commit permanently -- the same
/// defect `fdu-325q` fixed for `since`, one path over. Fixing an instance is not fixing
/// the class, which is the reason this test exists rather than a comment.
///
/// The interleaving is forced rather than hoped for: a second writer commits while the
/// batch is being assembled, and the assertion is the property that survives either
/// ordering -- the write is in this batch, or it is strictly after this batch's cursor.
/// Never both absent and behind.
#[test]
fn a_batch_cursor_never_runs_ahead_of_the_deltas_it_carried() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("a.txt"), b"12345").expect("seed");

    let config = OpenConfig { policy: CachePolicy::Off, ..OpenConfig::default() };
    let (index, _report) = open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);
    let mut session = Session::new(
        handle.clone(),
        ScanConfig::default(),
        Query {
            selection: Selection::default(),
            views: vec![ViewSpec::Summary],
            ..Query::default()
        },
        WatchConfig::default(),
    )
    .expect("session");

    // A writer that keeps committing straight through the batch boundary.
    let writer_handle = handle.clone();
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let writer_stop = std::sync::Arc::clone(&stop);
    let writer = std::thread::spawn(move || {
        let mut n = 0_u64;
        while !writer_stop.load(std::sync::atomic::Ordering::Relaxed) {
            let path = std::path::PathBuf::from(format!("w{n}.txt"));
            writer_handle
                .apply(&fdu_core::Observation::new(vec![fdu_core::Op::Upsert {
                    path,
                    kind: fdu_core::EntryKind::File,
                    attrs: fdu_core::Attrs { size: 1, ..fdu_core::Attrs::default() },
                }]))
                .expect("apply");
            n += 1;
            std::thread::yield_now();
        }
        n
    });

    fs::write(dir.path().join("b.txt"), b"678").expect("create");
    let mut checked = 0_u32;
    let deadline = Instant::now() + SETTLE;
    while Instant::now() < deadline && checked < 20 {
        let Some(batch) = session.next_batch(Duration::from_millis(50)).expect("batch") else {
            continue;
        };
        let Some(cursor) = batch.cursor else {
            assert!(batch.changes.is_empty(), "a batch with changes must name a position");
            continue;
        };
        let highest = batch.changes.iter().map(|change| change.clock).max().unwrap_or(0);
        assert!(
            highest <= cursor.clock.0,
            "the cursor must not sit behind a change this batch carried: {highest} > {}",
            cursor.clock.0
        );
        assert_eq!(cursor.session, handle.session().expect("session"));
        checked += 1;
    }

    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let written = writer.join().expect("writer");
    assert!(written > 0, "the writer must actually have interleaved");
    assert!(checked > 0, "at least one batch must have been examined");
}

/// A batch that stepped over another producer's commit says so, rather than skipping it.
///
/// An index has one writer at a time but not one producer: a caller can refresh a subtree,
/// ingest its own hints, or rebind tag rules against the same handle while a watch runs.
/// Those commits are real and this stream does not deliver them, so a batch naming a
/// position past one would drop it permanently -- and nothing in the batch would report
/// the loss. A gap is exactly what `reset` means, so that is what it is called.
///
/// The control matters as much as the case: a stream with no second producer must *not*
/// report a reset, or the signal would mean nothing and a consumer would re-read forever.
#[test]
fn a_commit_this_stream_did_not_deliver_makes_the_batch_a_reset() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("a.txt"), b"12345").expect("seed");

    let config = OpenConfig { policy: CachePolicy::Off, ..OpenConfig::default() };
    let (index, _report) = open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);
    let mut session = Session::new(
        handle.clone(),
        ScanConfig::default(),
        Query {
            selection: Selection::default(),
            views: vec![ViewSpec::Summary],
            ..Query::default()
        },
        WatchConfig::default(),
    )
    .expect("session");

    // A second producer, committing where this stream cannot see it.
    handle
        .apply(&fdu_core::Observation::new(vec![fdu_core::Op::Upsert {
            path: std::path::PathBuf::from("elsewhere.txt"),
            kind: fdu_core::EntryKind::File,
            attrs: fdu_core::Attrs { size: 1, ..fdu_core::Attrs::default() },
        }]))
        .expect("apply");

    fs::write(dir.path().join("b.txt"), b"678").expect("create");
    assert!(
        next_applied_batch(&mut session).expect("the created file should arrive").reset,
        "a batch whose deltas do not continue from this session's position is a reset"
    );

    fs::write(dir.path().join("c.txt"), b"9").expect("create");
    assert!(
        !next_applied_batch(&mut session).expect("the second file should arrive").reset,
        "and an uninterrupted stream is not"
    );
}

/// The next batch that actually applied something, or nothing within the settle window.
fn next_applied_batch(session: &mut Session) -> Option<fdu_core::session::Batch> {
    let deadline = Instant::now() + SETTLE;
    while Instant::now() < deadline {
        if let Some(batch) = session.next_batch(Duration::from_millis(100)).expect("batch")
            && batch.dirty
        {
            return Some(batch);
        }
    }
    None
}

/// A re-tag is a commit, and the batch that caused it names a position past it.
///
/// A saved `.gitignore` changes what the rules decide about entries nothing touched, so it
/// is an answer-affecting change with no path event of its own. The batch used to capture
/// its cursor *before* rebinding: the position it handed back therefore sat behind a
/// transition the batch had already applied, and a consumer resuming from it would be told
/// nothing had happened. Now the rebind commits, the batch carries it, and the cursor
/// follows it -- which is what makes `since(batch.cursor)` empty rather than a lie.
#[test]
#[cfg(feature = "gitignore")]
fn a_re_tag_commits_and_the_batch_cursor_follows_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(dir.path().join("src")).expect("mkdir");
    fs::write(dir.path().join("src/keep.rs"), b"fn main() {}").expect("seed");

    let tags = std::sync::Arc::new(
        fdu_core::tags::TagRules::from_names(["gitignore"]).expect("the rule is compiled in"),
    );
    let scan = ScanConfig { tags: Some(tags.clone()), ..ScanConfig::default() };
    let config =
        OpenConfig { policy: CachePolicy::Off, scan: scan.clone(), ..OpenConfig::default() };
    let (index, _report) = open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);
    let mut session = Session::new(
        handle.clone(),
        scan,
        Query {
            selection: Selection::default(),
            views: vec![ViewSpec::Summary],
            ..Query::default()
        },
        WatchConfig::default(),
    )
    .expect("session");

    fs::write(dir.path().join(".gitignore"), b"*.rs\n").expect("save a control file");

    let deadline = Instant::now() + SETTLE;
    let mut retagged = None;
    while Instant::now() < deadline && retagged.is_none() {
        let Some(batch) = session.next_batch(Duration::from_millis(100)).expect("batch") else {
            continue;
        };
        if batch.changes.iter().any(|change| change.path.ends_with(".gitignore")) {
            retagged = Some(batch);
        }
    }
    let batch = retagged.expect("the saved control file should arrive");

    assert!(
        batch.state.iter().any(|change| matches!(
            change,
            fdu_core::StateChange::Retagged { directories } if !directories.is_empty()
        )),
        "the batch must carry the re-tag it committed: {:?}",
        batch.state
    );
    let cursor = batch.cursor.expect("a batch that applied something names a position");
    assert!(
        handle.since(cursor).expect("resume").deltas.is_empty(),
        "the cursor must sit past the re-tag, not behind it"
    );
    // And the tag itself moved, which is what made the transition worth reporting.
    assert_eq!(
        handle
            .with_index(|index| {
                index
                    .tags_of(Path::new("src/keep.rs"))
                    .into_iter()
                    .map(str::to_owned)
                    .collect::<Vec<String>>()
            })
            .expect("tags"),
        vec!["gitignore"],
        "the rebind must have taken effect, or the batch reported a transition that did not happen"
    );
}
