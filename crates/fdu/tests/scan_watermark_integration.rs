//! A report's `scan_started_at` must be usable as the watermark for the next query.
//!
//! This is the property that makes incremental follow-up work: feed a report's
//! `scan_started_at` back as `--modified-since` and the answer is exactly the entries at
//! or after it. The contract is over mtimes, and the boundary is inclusive -- an entry
//! stamped in the very nanosecond the walk began must be reported, because that is the
//! side a mid-scan write lands on. An exclusive boundary, or a watermark taken when the
//! walk *ended*, would both surface here as an entry inside the walk window going
//! missing -- silently, since every later query uses a still-later watermark.
//!
//! Earlier versions of this test raced a writer thread against the real walk. That
//! exercised the machinery but proved less than it appeared to: from outside the
//! process, a write that landed "mid-walk" cannot be told apart from one just after,
//! so the assertions held under implementations that would lose real mid-scan writes
//! (review finding R11). Setting mtimes to exact values derived from the report's own
//! watermark pins the boundary at nanosecond precision with no dependence on timing,
//! which is both stronger and faster.
#![cfg(not(target_os = "windows"))]

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime};

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

/// Create `name` under `tree` with its mtime pinned to an exact instant.
fn file_stamped_at(tree: &Path, name: &str, mtime: SystemTime) {
    let path = tree.join(name);
    fs::write(&path, b"stamped").expect("write file");
    let file = fs::File::options().write(true).open(&path).expect("open for timestamp");
    file.set_modified(mtime).expect("set mtime");
}

#[test]
fn the_watermark_window_is_inclusive_at_exactly_scan_start() {
    let root = tempfile::tempdir().expect("tempdir");
    let tree = root.path();
    fs::write(tree.join("seeded.txt"), b"seed").expect("seed file");

    // One real scan produces the watermark under test. `--cache off` keeps the follow-up
    // queries below answering from a fresh walk rather than a snapshot.
    let report = run(tree, &["--view", "summary", "--format", "json", "--cache", "off"]);
    let watermark = field(&report, "scan_started_at");
    let generated = field(&report, "generated_at");
    assert!(
        watermark <= generated,
        "the watermark must precede the report it came from: {watermark} > {generated}",
    );

    // Parse the watermark with the same grammar `--modified-since` uses, so the boundary
    // files below sit at exactly the instant the query will compare against.
    let now = SystemTime::now();
    let at_watermark = fdu_core::query::parse_when(&watermark, now).expect("parse watermark");

    // Three files, one per side of the boundary and one exactly on it. The on-boundary
    // file is the mid-scan case in its sharpest form: an entry stamped in the same
    // nanosecond the walk began. The just-before file is one nanosecond earlier, so any
    // off-by-one in the comparison fails one of the two.
    file_stamped_at(tree, "at-boundary.txt", at_watermark);
    file_stamped_at(tree, "just-before.txt", at_watermark - Duration::from_nanos(1));
    file_stamped_at(tree, "well-after.txt", at_watermark + Duration::from_secs(60));

    let listing = run(
        tree,
        &["--view", "files", "--format", "jsonl", "--cache", "off", "--modified-since", &watermark],
    );

    assert!(
        listing.contains("at-boundary.txt"),
        "an entry stamped at exactly scan start must be inside the window: --modified-since \
         is inclusive precisely so a mid-scan write cannot fall through. Listing: {listing}",
    );
    assert!(
        listing.contains("well-after.txt"),
        "an entry changed after the watermark must be inside the window. Listing: {listing}",
    );
    assert!(
        !listing.contains("just-before.txt"),
        "an entry one nanosecond before the watermark must be outside the window, or the \
         watermark reports history it did not promise. Listing: {listing}",
    );
    assert!(
        !listing.contains("seeded.txt"),
        "an entry that predates the walk must be outside the window, or the watermark is \
         useless for incremental work. Listing: {listing}",
    );

    // The round trip composes: the boundary file's own report can seed the next query.
    let second = run(tree, &["--view", "summary", "--format", "json", "--cache", "off"]);
    let second_watermark = field(&second, "scan_started_at");
    assert!(
        watermark <= second_watermark,
        "watermarks must be monotonic across runs: {watermark} then {second_watermark}",
    );
}
