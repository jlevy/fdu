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

/// Another producer's commit is delivered by this stream, not stepped over by it.
///
/// An index has one writer at a time but not one producer: a caller can refresh a subtree,
/// ingest its own hints, or rebind tag rules against the same handle while a watch runs.
/// `apply_next` also reconciles through several separately locked flushes, so such a
/// commit can land *between* two watcher deltas.
///
/// A batch assembled from what the watcher handed back omitted that commit while advancing
/// its cursor past it, and resuming from the cursor skipped it permanently with nothing
/// reporting the loss. Building the batch from the journal since the consumer's own
/// position makes the omission unrepresentable: whoever committed it, it is in the slice.
///
/// The earlier version of this test asserted that such a batch reported `reset`. That was
/// the weaker contract -- telling a consumer to throw everything away is not the same as
/// handing it what it missed -- and it would have passed on a stream that simply never
/// delivered the commit.
#[test]
fn a_commit_from_another_producer_is_delivered_rather_than_skipped() {
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

    // No filesystem write, deliberately. The first version of this test created a file
    // after the direct apply, and that unrelated event is what woke the watcher -- so the
    // test passed over a stream that would have withheld the commit indefinitely on a tree
    // nobody was touching. A journal that moved is the thing being waited for.
    let batch = next_applied_batch(&mut session).expect("a journal-only commit must arrive");
    let seen: Vec<std::path::PathBuf> =
        batch.changes.iter().map(|change| change.path.clone()).collect();

    assert!(
        seen.iter().any(|path| path.ends_with("elsewhere.txt")),
        "the other producer's commit must be delivered, not skipped: {seen:?}"
    );
    assert!(!batch.reset, "and delivering it is not a reset -- nothing was lost");

    // And the watcher still works afterwards, so waking on the journal did not consume the
    // event path it shares.
    fs::write(dir.path().join("b.txt"), b"678").expect("create");
    let watched = next_applied_batch(&mut session).expect("the watched change should arrive");
    assert!(
        watched.changes.iter().any(|change| change.path.ends_with("b.txt")),
        "{:?}",
        watched.changes
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
        batch.transitions.iter().any(|committed| matches!(
            &committed.change,
            fdu_core::StateChange::Retagged { directories, .. } if !directories.is_empty()
        )),
        "the batch must carry the re-tag it committed: {:?}",
        batch.transitions
    );
    // Every transition sits at its own commit, inside the range the batch carried. The
    // re-tag is last because it is committed after the deltas that triggered it.
    let cursor = batch.cursor.expect("a batch that applied something names a position");
    assert!(
        batch.transitions.iter().all(|committed| committed.clock <= cursor.clock),
        "a transition cannot sit past the position the batch reports: {:?}",
        batch.transitions
    );
    let highest_op = batch.changes.iter().map(|change| change.clock).max().unwrap_or(0);
    assert!(
        batch
            .transitions
            .iter()
            .filter(|committed| matches!(committed.change, fdu_core::StateChange::Retagged { .. }))
            .all(|committed| committed.clock.0 >= highest_op),
        "the re-tag commits after what triggered it: {:?}",
        batch.transitions
    );
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

/// Losing watch precision is an issue the engine covered, not a consumer reset.
///
/// The two were one flag, and they are different facts about different parties. A dropped
/// event means the *provider* stopped seeing precisely; it re-scans, so the index is right
/// and the batch's rows are complete, and the consumer's own position is perfectly
/// resumable. A reset means the *consumer's* history has expired: nothing can be replayed
/// to it and everything it holds must be re-read.
///
/// Reporting the first as the second costs a consumer a full re-read on every kernel queue
/// overflow -- and, worse, teaches it that reset does not really mean what it says.
#[test]
fn losing_watch_precision_is_an_issue_rather_than_a_reset() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(dir.path().join("src")).expect("mkdir");
    fs::write(dir.path().join("src/a.txt"), b"12345").expect("seed");

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

    // What a kernel queue overflow leaves behind, applied directly so the test does not
    // depend on being able to provoke one.
    handle
        .apply(&fdu_core::Observation::new(vec![fdu_core::Op::InvalidateSubtree {
            path: std::path::PathBuf::from("src"),
            reason: fdu_core::InvalidateReason::WatchOverflow,
        }]))
        .expect("apply");

    let batch = next_applied_batch(&mut session).expect("the escalation should arrive");

    assert!(!batch.reset, "the consumer's position is fine: it can replay this batch");
    let gap = batch
        .issues
        .iter()
        .find(|issue| issue.kind == fdu_core::IssueKind::ObservationGap)
        .expect("the gap must be reported as a typed issue");
    assert_eq!(gap.path.as_deref(), Some(Path::new("src")), "and it names where");
    assert!(
        batch.changes.iter().any(|change| change.kind == ChangeKind::Invalidate),
        "the re-scan's own signal still reaches the consumer: {:?}",
        batch.changes
    );
    assert!(
        !batch.dirty_rollups.is_empty() || batch.all_dirty,
        "and the aggregates it may have moved are still named"
    );
}

/// A batch carries the terminal state at its own cursor, from the read that built it.
///
/// The changes say what moved and the transitions say what shifted underneath them;
/// neither says where it all ended up. A consumer that answered that with a follow-up
/// read would pair this batch's changes with a later commit's lifecycle, and nothing in
/// either value would say so -- and folding the transitions into a consumer-side copy is
/// the mirror the boundary exists to forbid.
///
/// The commit here lands *after* the batch and before the assertions, which is what gives
/// the test teeth: an implementation that read the state at assertion time would report
/// the later one, and the two are deliberately made to differ.
#[test]
fn a_batch_carries_the_terminal_state_at_its_own_cursor() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("seed.txt"), b"seed").expect("seed");

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

    fs::write(dir.path().join("later.txt"), b"later").expect("write");
    let deadline = Instant::now() + SETTLE;
    let mut carried = None;
    while Instant::now() < deadline && carried.is_none() {
        let Some(batch) = session.next_batch(Duration::from_millis(100)).expect("batch") else {
            continue;
        };
        if batch.cursor.is_some() {
            carried = Some(batch);
        }
    }
    let batch = carried.expect("a write should produce a batch that names a position");
    let cursor = batch.cursor.expect("filtered on above");

    assert_eq!(
        batch.state.clock, cursor.clock,
        "the state must name the position the batch reports, or the two can be paired wrongly"
    );
    assert_eq!(batch.state.freshness, fdu_core::Freshness::Fresh, "nothing has cost trust yet");

    // The commit a follow-up read would have seen and this batch must not.
    handle
        .apply(&fdu_core::Observation::new(vec![fdu_core::Op::InvalidateSubtree {
            path: std::path::PathBuf::new(),
            reason: fdu_core::InvalidateReason::WatchOverflow,
        }]))
        .expect("a direct producer commits against the same handle");

    assert_eq!(
        handle.freshness().expect("freshness"),
        fdu_core::Freshness::Stale,
        "the index has moved on"
    );
    assert_eq!(
        batch.state.freshness,
        fdu_core::Freshness::Fresh,
        "and the batch still describes the instant it was taken"
    );
    assert!(
        batch.state.clock < handle.clock().expect("clock"),
        "which is strictly behind where the index now is"
    );
}

/// An invalidations-only feed derives everything it acts on, and builds no rows.
///
/// The point is what is *absent*: a consumer that re-reads on dirty never looks at
/// `changes`, and materialising them costs a tag lookup and a path clone per operation,
/// then the whole crossing. So the assertion that matters is that `changes` is empty while
/// every signal a consumer acts on is still present and equal to what the row-carrying mode
/// reports -- an empty batch would satisfy the first half and none of the second.
#[test]
fn an_invalidations_only_feed_carries_no_rows_and_loses_no_signal() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(dir.path().join("src")).expect("mkdir");
    fs::write(dir.path().join("src/seed.rs"), b"fn main() {}").expect("seed");

    let config = OpenConfig { policy: CachePolicy::Off, ..OpenConfig::default() };
    let query = Query {
        selection: Selection::default(),
        views: vec![ViewSpec::Summary],
        ..Query::default()
    };

    let mut sessions: Vec<Session> = Vec::new();
    for interest in [fdu_core::Interest::Rows, fdu_core::Interest::Invalidations] {
        let (index, _) = open(dir.path(), &config).expect("open");
        sessions.push(
            Session::new(
                IndexHandle::new(index),
                ScanConfig::default(),
                query.clone(),
                WatchConfig::default(),
            )
            .expect("session")
            .with_interest(interest),
        );
    }

    fs::write(dir.path().join("src/added.rs"), b"fn other() {}").expect("write");

    let mut carried: Vec<fdu_core::Batch> = Vec::new();
    for session in &mut sessions {
        let deadline = Instant::now() + SETTLE;
        let mut found = None;
        while Instant::now() < deadline && found.is_none() {
            let Some(batch) = session.next_batch(Duration::from_millis(100)).expect("batch") else {
                continue;
            };
            if batch.dirty {
                found = Some(batch);
            }
        }
        carried.push(found.expect("a write should produce a dirty batch"));
    }
    let (rows, invalidations) = (&carried[0], &carried[1]);

    assert!(!rows.changes.is_empty(), "the row-carrying mode carries rows");
    assert!(
        invalidations.changes.is_empty(),
        "and the other builds none at all: {:?}",
        invalidations.changes
    );
    assert_eq!(invalidations.work.rows, 0, "which is measured, not merely unreported");
    assert_eq!(invalidations.work.name_bytes, 0);

    // Everything a consumer acts on survives. An empty batch would pass the two assertions
    // above and fail every one of these.
    assert!(invalidations.dirty, "it still says something moved");
    assert_eq!(invalidations.dirty_rollups, rows.dirty_rollups, "and which roll-ups to discard");
    assert_eq!(invalidations.dirty_queries, rows.dirty_queries, "and which projections");
    assert_eq!(invalidations.all_dirty, rows.all_dirty);
    assert_eq!(invalidations.reset, rows.reset);
    assert!(invalidations.cursor.is_some(), "and where to resume from");
    assert_eq!(
        invalidations.state.freshness, rows.state.freshness,
        "and how far to trust what it kept"
    );
    assert!(invalidations.work.wall_ns > 0, "the mode still charges its own assembly");
}

/// A socket created under a live watch does not enter an index whose scope excludes them.
///
/// The watcher is the third producer of rows, beside the walk and reconciliation, and the
/// only one that learns a kind from an event rather than from a listing it controls. A
/// scope enforced by the other two and not by this one would hold until somebody started
/// watching -- an index whose contents depend on whether anyone was looking.
#[test]
fn a_socket_created_under_a_pruning_watch_never_enters_the_index() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("existing.txt"), b"hello").expect("seed");
    let scan = ScanConfig { exclude_special: true, ..ScanConfig::default() };
    let (handle, mut session) = pruning_session(dir.path(), scan);

    let Ok(_listener) = std::os::unix::net::UnixListener::bind(dir.path().join("sock")) else {
        return;
    };
    // Created after the socket, so its arrival proves the watcher reached the socket's
    // event too: one backend, one queue, and this is behind it.
    fs::write(dir.path().join("after.rs"), b"fn main() {}").expect("create");

    wait_for(&mut session, |change| change.path.ends_with("after.rs"))
        .expect("the ordinary file should arrive");
    assert!(!holds(&handle, "sock"), "a socket is outside this scope however it arrives");
    assert!(holds(&handle, "after.rs"), "and nothing else was dropped with it");
}

/// A file replaced in place by a socket loses its row, under a live watch too.
///
/// The case that separates "skip the event" from "exclude the object". The path survives
/// the replacement, so nothing else will revisit it; unless the exclusion removes the row,
/// the index reports a five-byte file at a path that holds a socket for as long as the
/// session runs.
#[test]
fn a_watched_file_replaced_by_a_socket_loses_the_row_it_had() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("doomed.txt"), b"hello").expect("seed");
    let scan = ScanConfig { exclude_special: true, ..ScanConfig::default() };
    let (handle, mut session) = pruning_session(dir.path(), scan);
    assert!(holds(&handle, "doomed.txt"), "the file starts recorded");

    fs::remove_file(dir.path().join("doomed.txt")).expect("unlink");
    let Ok(_listener) = std::os::unix::net::UnixListener::bind(dir.path().join("doomed.txt"))
    else {
        return;
    };
    fs::write(dir.path().join("after.rs"), b"fn main() {}").expect("create");

    wait_for(&mut session, |change| change.path.ends_with("after.rs"))
        .expect("the ordinary file should arrive");
    assert!(!holds(&handle, "doomed.txt"), "the row outlived the file it described");
}

/// A socket renamed *over* a live file removes the row, with no absence in between.
///
/// The case that separates "drop the event" from "exclude the object", and the reason the
/// unlink-then-bind test above cannot make it: an unlink is itself an event, so the row is
/// already gone before the socket exists and a watcher that merely ignored the create
/// would look correct. A rename onto the path is one event, File to Other, with the path
/// never absent -- so the removal has to be the exclusion's own doing or it does not
/// happen at all.
#[test]
fn a_socket_renamed_over_a_watched_file_removes_the_row() {
    let dir = tempfile::tempdir().expect("tempdir");
    let staging = tempfile::tempdir().expect("staging");
    fs::write(dir.path().join("doomed.txt"), b"hello").expect("seed");
    let scan = ScanConfig { exclude_special: true, ..ScanConfig::default() };
    let (handle, mut session) = pruning_session(dir.path(), scan);
    assert!(holds(&handle, "doomed.txt"), "the file starts recorded");

    // Bound outside the watched root, so the only event this produces inside it is the
    // arrival on the destination. `rename` needs one filesystem, which two directories
    // under the same temporary root are.
    let Ok(_listener) = std::os::unix::net::UnixListener::bind(staging.path().join("sock")) else {
        return;
    };
    if fs::rename(staging.path().join("sock"), dir.path().join("doomed.txt")).is_err() {
        return;
    }
    fs::write(dir.path().join("after.rs"), b"fn main() {}").expect("create");

    wait_for(&mut session, |change| change.path.ends_with("after.rs"))
        .expect("the ordinary file should arrive");
    assert!(!holds(&handle, "doomed.txt"), "the row outlived the file it described");
}

/// Keeping is still keeping: the default scope records a socket the watcher reports.
///
/// Without this, the two tests above pass for a watcher that drops every special object
/// unconditionally and never reads the flag at all.
#[test]
fn a_socket_created_under_a_default_watch_is_recorded() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("existing.txt"), b"hello").expect("seed");
    let (handle, mut session) = pruning_session(dir.path(), ScanConfig::default());

    let Ok(_listener) = std::os::unix::net::UnixListener::bind(dir.path().join("sock")) else {
        return;
    };
    fs::write(dir.path().join("after.rs"), b"fn main() {}").expect("create");

    wait_for(&mut session, |change| change.path.ends_with("after.rs"))
        .expect("the ordinary file should arrive");
    assert!(holds(&handle, "sock"), "nothing excludes it, so the watcher's row stands");
}

/// A session sharing its index with the caller, so the index can be read while it watches.
fn pruning_session(root: &Path, scan: ScanConfig) -> (IndexHandle, Session) {
    let config =
        OpenConfig { scan: scan.clone(), policy: CachePolicy::Off, ..OpenConfig::default() };
    let (index, _report) = open(root, &config).expect("open");
    let handle = IndexHandle::new(index);
    let session = Session::new(handle.clone(), scan, Query::default(), WatchConfig::default())
        .expect("session");
    (handle, session)
}

/// Whether the index holds a row for `name` directly under the root.
fn holds(handle: &IndexHandle, name: &str) -> bool {
    let request = fdu_core::ReadRequest {
        entry_page: Some(fdu_core::EntryPageRequest { limit: u32::MAX, ..Default::default() }),
        ..Default::default()
    };
    handle
        .read(&request)
        .expect("read")
        .entry_page
        .expect("page")
        .rows
        .iter()
        .any(|row| row.path == Path::new(name))
}
