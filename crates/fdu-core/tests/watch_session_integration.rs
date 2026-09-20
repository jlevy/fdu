//! End-to-end behaviour of a live watch session.
//!
//! These run against real filesystem events rather than injected observations, because
//! the property under test is that fdu sees what the operating system reports. Each test
//! makes one change and waits for it, so ordering is determined by the change rather than
//! by a sleep.
#![cfg(all(feature = "watch", unix))]

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

use fdu_core::content::{AnalysisRequest, AnalysisSet};
use fdu_core::query::{
    AxisNames, Basis, Bound, Query, ReadSpec, Request, RequestSpec, Selection, SizeMetric,
    ViewSpec, WatchDelivery,
};
// The module's own `Delivery` is how a change reached this process; the request model's is
// how an answer is carried out. Two different questions, so the import names the crate.
use fdu_core::query::Delivery as RequestDelivery;
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
        request(root, AnalysisSet::NONE, Query { selection, views, ..Query::default() }),
        &watching(&config),
        WatchConfig::default(),
    )
    .expect("session")
}

/// The delivery a test watch runs under: the one its index was opened with, repeating.
///
/// Named rather than defaulted, because the cache policy is what a session validates the
/// cache-only rule against and a fabricated one always read `auto` (fdu-i18y).
fn watching(config: &OpenConfig) -> RequestDelivery {
    let (_basis, delivery) = config.split(Path::new("/unused"));
    RequestDelivery {
        watch: Some(WatchDelivery { interval: Duration::from_millis(200) }),
        ..delivery
    }
}

/// The request a watch answers: the basis its index was opened under, and this query.
fn request(root: &Path, content: AnalysisSet, query: Query) -> Request {
    Request::new(
        Basis { root: root.to_path_buf(), scope: ScanConfig::default(), content },
        query,
        std::time::SystemTime::now(),
    )
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
///
/// Returns `false` only for a host explicitly declared unable to deliver native watch
/// events, with `FDU_TEST_ALLOW_NO_NATIVE_WATCH=1`. Without that declaration, silence
/// during the warm-up is an actionable precondition failure rather than a passing test.
fn establish_watch(session: &mut Session, warm: &Path, contents: &[u8]) -> bool {
    fs::write(warm, contents).expect("warm-up rewrite");
    let name = warm.file_name().expect("warm-up name").to_owned();
    match wait_for_delivery(session, |change| change.path.ends_with(&name)) {
        Delivery::Delivered(_) => true,
        Delivery::Mismatched(seen) => mismatched("native watch warm-up", seen),
        Delivery::Silent => {
            if std::env::var_os("FDU_TEST_ALLOW_NO_NATIVE_WATCH").as_deref()
                == Some(std::ffi::OsStr::new("1"))
            {
                eprintln!(
                    "skipped by FDU_TEST_ALLOW_NO_NATIVE_WATCH=1: the host event service \
                     delivered no changes to this session"
                );
                return false;
            }
            panic!(
                "native watch precondition failed because the host delivered no changes in \
                 {SETTLE:?}; run on a host with event delivery, or explicitly opt out with \
                 FDU_TEST_ALLOW_NO_NATIVE_WATCH=1"
            )
        }
    }
}

/// Collect changes until `wanted` matches one, separating three outcomes.
///
/// `Delivered` — the change arrived. Assertions run at full strength.
///
/// `Mismatched` — batches arrived but never carried the awaited change. A real
/// disagreement about content, and the caller must fail.
///
/// `Silent` — no batch arrived at all. What that means depends on whether the watch was
/// already known to deliver, so the caller decides: [`establish_watch`] may read it as
/// the host's silence, and [`wait_for`] reads it as fdu's. A working backend delivers
/// this test's own write in milliseconds, so silence for a full minute means the stream
/// is dead rather than slow.
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

/// Wait on a session whose watch [`establish_watch`] has already proven live.
///
/// Silence here is evidence about fdu rather than about the host: the backend delivered
/// the warm-up, so a later change it never reports is a lost event. No opt-out applies,
/// which is why the message offers none; a mismatch is a disagreement about content and
/// fails the same way.
fn wait_for(
    session: &mut Session,
    test: &str,
    wanted: impl Fn(&fdu_core::Change) -> bool,
) -> fdu_core::Change {
    match wait_for_delivery(session, wanted) {
        Delivery::Delivered(change) => *change,
        Delivery::Mismatched(seen) => mismatched(test, seen),
        Delivery::Silent => panic!(
            "{test}: the watch was established and then delivered nothing in {SETTLE:?}, so \
             this is a lost event rather than a host precondition; \
             FDU_TEST_ALLOW_NO_NATIVE_WATCH does not apply here"
        ),
    }
}

fn mismatched(test: &str, seen: usize) -> ! {
    panic!(
        "{test}: {seen} change(s) arrived in {SETTLE:?} but never the awaited one, so this is \
         a disagreement about content rather than a delivery failure"
    )
}

#[test]
fn a_created_file_arrives_as_an_upsert() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("existing.txt"), b"hello").expect("seed");
    let mut session = session(dir.path(), Selection::default(), vec![ViewSpec::Files]);
    if !establish_watch(&mut session, &dir.path().join("existing.txt"), b"hello") {
        return;
    }

    fs::write(dir.path().join("created.rs"), b"fn main() {}").expect("create");

    let change = wait_for(&mut session, "a_created_file_arrives_as_an_upsert", |change| {
        change.path.ends_with("created.rs")
    });
    assert_eq!(change.kind, ChangeKind::Upsert);
    assert_eq!(change.bytes, Some(12));
    assert!(change.mtime_ns.is_some(), "an upsert carries verified metadata, not just a path");
}

#[test]
fn session_reconciles_a_mutation_that_precedes_watcher_binding() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("before-bind.txt");
    fs::write(&path, b"old").expect("seed");
    let config = OpenConfig { policy: CachePolicy::Off, ..OpenConfig::default() };
    let (index, _) = open(dir.path(), &config).expect("open baseline");

    fs::write(&path, b"changed before binding").expect("mutate before session");
    let session = Session::new(
        IndexHandle::new(index),
        request(
            dir.path(),
            AnalysisSet::NONE,
            Query { views: vec![ViewSpec::Files], ..Query::default() },
        ),
        &watching(&config),
        WatchConfig::default(),
    )
    .expect("session closes scan-to-bind gap");

    let report = session.report(SystemTime::now()).expect("initial report");
    let row = report
        .sections
        .iter()
        .find_map(|section| match section {
            fdu_core::query::Section::Files { rows, .. } => {
                rows.iter().find(|row| row.path == Path::new("before-bind.txt"))
            }
            _ => None,
        })
        .expect("file row");
    assert_eq!(row.bytes, 22);
}

#[test]
fn a_deleted_file_arrives_as_a_remove() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("doomed.txt"), b"hello").expect("seed");
    let mut session = session(dir.path(), Selection::default(), vec![ViewSpec::Files]);
    if !establish_watch(&mut session, &dir.path().join("doomed.txt"), b"hello") {
        return;
    }

    fs::remove_file(dir.path().join("doomed.txt")).expect("remove");

    let change = wait_for(&mut session, "a_deleted_file_arrives_as_a_remove", |change| {
        change.path.ends_with("doomed.txt")
    });
    assert_eq!(change.kind, ChangeKind::Remove);
    assert_eq!(change.bytes, None, "a removed entry has no attributes to report");
}

#[test]
fn a_file_that_leaves_attribute_selection_arrives_as_a_remove() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("shrinking.txt");
    let warm = dir.path().join("warm.txt");
    fs::write(&path, b"12345678").expect("seed");
    fs::write(&warm, b"ready").expect("warm-up seed");
    let selection =
        Selection { min_size: Some(4), size: SizeMetric::Apparent, ..Selection::default() };
    let mut session = session(dir.path(), selection, vec![ViewSpec::Files]);
    establish_watch(&mut session, &warm, b"ready");

    fs::write(&path, b"x").expect("shrink below selection");

    let Some(change) = wait_for(
        &mut session,
        "a_file_that_leaves_attribute_selection_arrives_as_a_remove",
        false,
        |change| change.path.ends_with("shrinking.txt") && change.kind == ChangeKind::Remove,
    ) else {
        return;
    };
    assert_eq!(change.kind, ChangeKind::Remove);
    let report = session.report(SystemTime::now()).expect("report after shrink");
    let files = report
        .sections
        .iter()
        .find_map(|section| match section {
            fdu_core::query::Section::Files { rows, .. } => Some(rows),
            _ => None,
        })
        .expect("files section");
    assert!(files.iter().all(|row| row.path != Path::new("shrinking.txt")));
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
    if !establish_watch(&mut session, &dir.path().join("warmup.rs"), b"fn warm() {}") {
        return;
    }

    fs::write(dir.path().join("ignored.txt"), b"no").expect("create");
    fs::write(dir.path().join("watched.rs"), b"yes").expect("create");

    let change = wait_for(&mut session, "the_run_selection_filters_the_stream", |change| {
        change.path.ends_with("watched.rs")
    });
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
    if !establish_watch(&mut session, &dir.path().join("still.txt"), b"unchanged") {
        return;
    }

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
    wait_for(&mut session, "an_idle_tree_yields_nothing_and_costs_nothing", |change| {
        change.path.ends_with("sentinel.txt")
    });

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
    if !establish_watch(&mut session, &dir.path().join("a.txt"), b"12345") {
        return;
    }

    let before = session.report(std::time::SystemTime::UNIX_EPOCH).expect("report");
    let first = match &before.sections[0] {
        fdu_core::query::Section::Summary(row) => *row,
        other => panic!("expected a summary, got {other:?}"),
    };
    assert_eq!(first.files, 1);

    fs::write(dir.path().join("b.txt"), b"678").expect("create");
    wait_for(&mut session, "a_live_report_is_the_same_query_re_evaluated", |change| {
        change.path.ends_with("b.txt")
    });

    let after = session.report(std::time::SystemTime::UNIX_EPOCH).expect("report");
    let second = match &after.sections[0] {
        fdu_core::query::Section::Summary(row) => *row,
        other => panic!("expected a summary, got {other:?}"),
    };
    assert_eq!(second.files, 2, "the live report reflects the applied change");
    assert_eq!(second.bytes, first.bytes + 3);
}

/// Analysis is one-shot on every surface. Nothing re-reads a file a watch sees change, so
/// a session over an analyzed index would keep serving the metrics it opened with, marked
/// fresh (fdu-snv3). The engine refuses the pairing rather than leaving only the command
/// line to, and it refuses a request that claims no analyzers over such an index too --
/// that one is a read the index cannot answer at all.
#[test]
fn a_session_refuses_an_analyzed_index() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("a.txt"), b"one two\n").expect("seed");
    let lines = AnalysisSet::NONE.with_lines();
    let config = OpenConfig {
        policy: CachePolicy::Off,
        analysis: AnalysisRequest { profile: lines, ..AnalysisRequest::default() },
        ..OpenConfig::default()
    };
    let (index, _report) = open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);

    let refused = Session::new(
        handle.clone(),
        request(dir.path(), lines, Query::default()),
        &watching(&config),
        WatchConfig::default(),
    );
    assert!(
        matches!(
            refused,
            Err(fdu_core::Error::InvalidRequest(fdu_core::query::RequestError::WatchContent))
        ),
        "expected a refusal, got {:?}",
        refused.err().map(|error| error.to_string())
    );

    let mismatched = Session::new(
        handle,
        request(dir.path(), AnalysisSet::NONE, Query::default()),
        &watching(&config),
        WatchConfig::default(),
    );
    assert!(
        matches!(
            mismatched,
            Err(fdu_core::Error::InvalidRequest(
                fdu_core::query::RequestError::ContentMismatch { .. }
            ))
        ),
        "expected a refusal, got {:?}",
        mismatched.err().map(|error| error.to_string())
    );
}

/// The three rules a watch cannot meet are the engine's, not the two front doors'.
///
/// `Session::new` used to fabricate the delivery it validated against, which read
/// `cache: Auto` whatever its caller had opened with, so the cache-only rule could not
/// fire here at all and a library caller reached a watch the command line refuses
/// (fdu-i18y). It also asked the index's scope before the request's own rule, so the same
/// narrowed request was a scope mismatch to a library caller and a watch-scope refusal on
/// the command line.
#[test]
fn a_session_refuses_what_its_callers_delivery_cannot_carry() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("a.txt"), b"one\n").expect("seed");
    let config = OpenConfig { policy: CachePolicy::Off, ..OpenConfig::default() };
    let (index, _report) = open(dir.path(), &config).expect("open");
    let handle = IndexHandle::new(index);

    let cache_only = Session::new(
        handle.clone(),
        request(dir.path(), AnalysisSet::NONE, Query::default()),
        &RequestDelivery { cache: CachePolicy::Only, ..watching(&config) },
        WatchConfig::default(),
    );
    assert!(
        matches!(
            cache_only,
            Err(fdu_core::Error::InvalidRequest(fdu_core::query::RequestError::WatchCacheOnly))
        ),
        "expected a refusal, got {:?}",
        cache_only.err().map(|error| error.to_string())
    );

    // A narrowed scan scope is refused as the request-level rule it is, before the index
    // is asked what scope it was taken under: both are true of this call, and the one the
    // caller can act on is the one that speaks.
    let narrowed = Request::new(
        Basis {
            root: dir.path().to_path_buf(),
            scope: ScanConfig { max_depth: Some(2), ..ScanConfig::default() },
            content: AnalysisSet::NONE,
        },
        Query::default(),
        std::time::SystemTime::now(),
    );
    let refused = Session::new(handle, narrowed, &watching(&config), WatchConfig::default());
    assert!(
        matches!(
            refused,
            Err(fdu_core::Error::InvalidRequest(fdu_core::query::RequestError::WatchScope))
        ),
        "expected a watch-scope refusal, got {:?}",
        refused.err().map(|error| error.to_string())
    );
}

/// A watch answers one request, so a relative time window is resolved once and never
/// slides underneath it.
///
/// The claim `Session::new`'s doc makes: `now` is fixed when the request is built, so two
/// repaints of one session answer one question. A window re-resolved per repaint would
/// quietly drop entries out of `--modified-since 2h` as the session aged, which reads as
/// the tree changing rather than as the question changing.
#[test]
fn a_watchs_time_window_is_fixed_when_its_request_is_built() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::write(dir.path().join("a.txt"), b"one\n").expect("seed");
    let config = OpenConfig { policy: CachePolicy::Off, ..OpenConfig::default() };
    let (index, _report) = open(dir.path(), &config).expect("open");

    let started = SystemTime::now();
    let spec = RequestSpec {
        read: ReadSpec { modified_since: Some("2h"), ..ReadSpec::new() },
        ..RequestSpec::new(dir.path())
    };
    let request = Request::build(&spec, started, &AxisNames::FIELDS).expect("the spec parses");
    let session =
        Session::new(IndexHandle::new(index), request, &watching(&config), WatchConfig::default())
            .expect("session");

    let window = session.query().selection.modified.since.expect("a resolved lower bound");
    assert_eq!(session.request().now, started, "the session keeps the instant it was built at");

    // Two repaints, separated by a change the session applies.
    let first = session.report(SystemTime::now()).expect("report");
    fs::write(dir.path().join("b.txt"), b"two\n").expect("write");
    let second = session.report(SystemTime::now()).expect("report");
    assert!(!first.sections.is_empty() && !second.sections.is_empty());

    assert_eq!(session.request().now, started, "a repaint does not re-read the clock");
    assert_eq!(
        session.query().selection.modified.since,
        Some(window),
        "the window a watch selects by is absolute"
    );

    // And it is the instant the request was built at, not one re-resolved since: the same
    // spec built later names a later window, which is what the session must not do.
    let later = Request::build(&spec, started + Duration::from_secs(60), &AxisNames::FIELDS)
        .expect("the spec parses");
    assert!(
        later.query.selection.modified.since > Some(window),
        "a window rebuilt a minute later must have moved, or this test proves nothing"
    );
}
