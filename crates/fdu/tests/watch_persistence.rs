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

/// Wait for a snapshot to appear under `cache_dir`, if one ever does.
fn wait_for_snapshot(cache_dir: &Path) -> Option<fs::DirEntry> {
    let started = Instant::now();
    while started.elapsed() < DEADLINE {
        if let Ok(entries) = fs::read_dir(cache_dir.join("fdu")) {
            // Any non-empty snapshot means the save completed rather than merely began.
            if let Some(entry) =
                entries.flatten().find(|entry| entry.metadata().is_ok_and(|meta| meta.len() > 0))
            {
                return Some(entry);
            }
        }
        sleep(Duration::from_millis(100));
    }
    None
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

    // Give the initial scan time to land, then make a change worth persisting.
    sleep(Duration::from_secs(2));
    fs::write(tree.join("second.txt"), b"second").expect("write second file");

    let snapshot = wait_for_snapshot(cache.path());

    // SIGKILL: the exit no signal handler can intercept. Whatever is on disk now is
    // exactly what a real interrupted session would have left.
    let _ = child.kill();
    let _ = child.wait();

    let snapshot = snapshot.expect("watch session wrote no snapshot before being killed");
    assert!(
        snapshot.metadata().expect("snapshot metadata").len() > 0,
        "snapshot exists but is empty, so the next run would treat it as absent",
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
}
