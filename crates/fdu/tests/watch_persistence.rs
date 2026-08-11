//! A watch session must leave a usable cache behind even when it is killed outright.
//!
//! Watch sessions end by signal far more often than they end politely -- Ctrl-C, a
//! closed terminal, a supervisor restart -- so persisting only at exit would persist
//! approximately never, and every session would hand the next run a cold start it had
//! already paid for. This drives the real binary and kills it without warning, because
//! the property is specifically about the ending no handler gets to observe.
#![cfg(all(feature = "watch", unix))]

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Generous: a cold scan, an event round trip, and a save all have to fit.
const DEADLINE: Duration = Duration::from_secs(30);

/// Identify the snapshot on disk by size and modification time.
///
/// Distinguishes one *write* from another, which is the whole difficulty here: the
/// initial `open()` leaves a snapshot before the watch loop starts, so presence alone
/// says nothing about whether the loop ever saved.
fn snapshot_fingerprint(cache_dir: &Path) -> Option<(u64, std::time::SystemTime)> {
    let entries = fs::read_dir(cache_dir.join("fdu")).ok()?;
    entries
        .flatten()
        .filter_map(|entry| entry.metadata().ok())
        .filter(|meta| meta.len() > 0)
        .find_map(|meta| Some((meta.len(), meta.modified().ok()?)))
}

/// Wait until the snapshot differs from `before`, or give up.
fn wait_for_rewrite(cache_dir: &Path, before: Option<(u64, std::time::SystemTime)>) -> bool {
    let started = Instant::now();
    while started.elapsed() < DEADLINE {
        let now = snapshot_fingerprint(cache_dir);
        if now.is_some() && now != before {
            return true;
        }
        sleep(Duration::from_millis(100));
    }
    false
}

#[test]
fn a_killed_watch_still_leaves_a_warm_cache() {
    let root = tempfile::tempdir().expect("tempdir");
    let cache = tempfile::tempdir().expect("cache tempdir");
    let tree = root.path().join("tree");
    fs::create_dir(&tree).expect("create tree");
    fs::write(tree.join("first.txt"), b"first").expect("write first file");

    let mut child = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(["--watch", "--view", "files", "--interval", "1s"])
        .arg(&tree)
        .env("XDG_CACHE_HOME", cache.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn watching fdu");

    // Let the initial open's snapshot land and record it, so a later write is provably a
    // second one rather than that same file seen again.
    sleep(Duration::from_secs(2));
    let initial = snapshot_fingerprint(cache.path());

    fs::write(tree.join("second.txt"), b"second").expect("write second file");
    let rewritten = wait_for_rewrite(cache.path(), initial);

    // SIGKILL: the exit no signal handler can intercept. Whatever is on disk now is
    // exactly what a real interrupted session would have left.
    let _ = child.kill();
    let _ = child.wait();

    assert!(
        rewritten,
        "the watch loop never rewrote the snapshot after a change, so everything observed \
         while watching would be lost",
    );

    // The saved snapshot has to be usable, not merely present: a later run must accept
    // it as warm rather than rescanning.
    let output = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(["--view", "summary", "--format", "json", "--cache", "only"])
        .arg(&tree)
        .env("XDG_CACHE_HOME", cache.path())
        .output()
        .expect("run fdu against the saved cache");

    assert!(
        output.status.success(),
        "cache-only read of the watch session's snapshot failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    let report = String::from_utf8_lossy(&output.stdout);
    assert!(
        report.contains("\"source\": \"cache_only\"")
            || report.contains("\"source\":\"cache_only\""),
        "expected a cache-only read, got: {report}",
    );

    // The load-bearing assertion. `open()` writes a snapshot before the watch loop even
    // starts, so "a snapshot exists" proves nothing about incremental saving -- this test
    // passed without the feature it exists to pin until this check was added. Only a file
    // created *after* the watch began can distinguish the two.
    let listing = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(["--view", "files", "--format", "jsonl", "--cache", "only"])
        .arg(&tree)
        .env("XDG_CACHE_HOME", cache.path())
        .output()
        .expect("list the saved cache");
    let listed = String::from_utf8_lossy(&listing.stdout);
    assert!(
        listed.contains("second.txt"),
        "the snapshot predates the watch session: it holds only the initial open's index, \
         so incremental saves never ran. Listing was: {listed}",
    );
}
