//! A report's `scan_started_at` must be usable as the watermark for the next query.
//!
//! This is the property that makes incremental follow-up work: feed a report's
//! `scan_started_at` back as `--modified-since` and you get exactly what changed since,
//! with no gap. The dangerous case is a write that lands *while* the walk is in flight.
//! If the watermark were stamped when the walk finished, a file the walker had already
//! passed would be older than the watermark and newer than nothing -- missed forever, and
//! silently, because every later query would use a still-later watermark.
//!
//! Stamping before the walk makes the window inclusive rather than exclusive: a mid-scan
//! write may be reported twice, which costs a caller nothing, instead of zero times.
#![cfg(all(feature = "cli", not(target_os = "windows")))]

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Enough entries for the walk to take real time, and no more.
///
/// The tree used to be far larger, on the theory that a long walk was needed for a
/// concurrent write to land inside it. The continuous writer removed that need, and the
/// cost of building and deleting thousands of files dominated the test's runtime.
const DIRECTORIES: usize = 40;
const FILES_PER_DIRECTORY: usize = 4;

/// Gap between the writer's repeated writes.
///
/// The writer rewrites its file on this cadence for as long as the scan runs, rather than
/// sleeping once and hoping to land inside the walk. A single timed write cannot be relied
/// on: process spawn alone costs tens of milliseconds, so a short delay fires before the
/// scan starts and a long one fires after it ends, and which happens varies by machine.
/// Writing throughout guarantees that at least one write lands mid-walk on any machine,
/// without the test ever depending on how fast that machine is.
const WRITE_CADENCE: Duration = Duration::from_millis(5);

fn run(tree: &Path, args: &[&str]) -> String {
    let output =
        Command::new(env!("CARGO_BIN_EXE_fdu")).args(args).arg(tree).output().expect("run fdu");
    assert!(
        output.status.success(),
        "fdu {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Pull one string field out of a report without a JSON dependency.
fn field(report: &str, name: &str) -> String {
    let key = format!("\"{name}\": \"");
    let start = report.find(&key).unwrap_or_else(|| panic!("no {name} in: {report}")) + key.len();
    let rest = &report[start..];
    let end = rest.find('"').expect("unterminated field");
    rest[..end].to_string()
}

#[test]
fn a_reports_watermark_lists_everything_changed_since_it_began() {
    let root = tempfile::tempdir().expect("tempdir");
    let tree = root.path();

    for directory in 0..DIRECTORIES {
        let path = tree.join(format!("dir{directory:04}"));
        fs::create_dir(&path).expect("create directory");
        for file in 0..FILES_PER_DIRECTORY {
            fs::write(path.join(format!("file{file}.txt")), b"seed").expect("seed file");
        }
    }

    // A writer running for the whole duration of the scan, so some write provably lands
    // while the walk is in flight. `--cache off` keeps this about the walk itself rather
    // than about a snapshot.
    let midscan = tree.join("midscan.txt");
    let stop = Arc::new(AtomicBool::new(false));
    let writer = {
        let midscan = midscan.clone();
        let stop = Arc::clone(&stop);
        thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                fs::write(&midscan, b"written while the walk was in flight")
                    .expect("mid-scan write");
                thread::sleep(WRITE_CADENCE);
            }
        })
    };

    let report = run(tree, &["--view", "summary", "--format", "json", "--cache", "off"]);
    stop.store(true, Ordering::Relaxed);
    writer.join().expect("writer thread");

    // One write from this thread after the scan has definitely finished. The concurrent
    // writer is what exercises the mid-walk case, but a starved thread could in principle
    // get only its first write in before being stopped -- and a test that depends on the
    // scheduler being fair is a test that fails on a loaded CI runner for no real reason.
    // This makes the assertion below true by construction; the race above is what makes it
    // *interesting*.
    fs::write(&midscan, b"written after the walk finished").expect("post-scan write");

    let watermark = field(&report, "scan_started_at");
    let generated = field(&report, "generated_at");
    assert!(
        watermark <= generated,
        "the watermark must precede the report it came from: {watermark} > {generated}",
    );

    // The round trip: everything modified since the walk began, per the report itself.
    let listing = run(
        tree,
        &["--view", "files", "--format", "jsonl", "--cache", "off", "--modified-since", &watermark],
    );

    assert!(
        listing.contains("midscan.txt"),
        "a file written while the walk was in flight is missing from its own watermark \
         query, so an incremental consumer would never see it. Watermark was {watermark}, \
         listing was: {listing}",
    );

    // The window is bounded, not "everything": the seeded files predate the walk and must
    // not come back, or the watermark would be useless for incremental work.
    assert!(
        !listing.contains("file0.txt"),
        "files that predate the walk must fall outside the watermark window: {listing}",
    );
}
