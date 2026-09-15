//! `fdu --watch` reads no `.gitignore` control state, so no control bound can end it.
//!
//! No command-line view reads ignore classification, which is why a one-shot report
//! already runs with control observation off. A watch run is the same query repeated, so
//! observing control state bought it nothing but the control-table bounds: one ignore rule
//! over 16 KiB, or 4 MiB of retained sources across a tree, ended the session before its
//! first answer, or ended it later when a control file was edited. `main` had no control
//! table and watched those trees (fdu-1onj). Sharing the one-shot scope also lets the two
//! runs warm-start from each other's snapshot (fdu-w3l5).
//!
//! These drive the real binary, because the property is about the configuration the
//! command line builds, not about what the engine can be configured to do.
#![cfg(all(feature = "watch", unix))]

use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use fdu_core::control::CONTROL_LINE_GUARD_BYTES;

/// Generous: a cold scan, an event round trip, and a loaded CI runner all have to fit.
const DEADLINE: Duration = Duration::from_secs(30);

/// A control source whose single rule is one byte over the per-line bound.
fn oversized_rule() -> Vec<u8> {
    let mut source = vec![b'a'; CONTROL_LINE_GUARD_BYTES + 1];
    source.push(b'\n');
    source
}

/// A running `fdu --watch` whose output is read line by line against a deadline.
struct Watching {
    child: Child,
    lines: Receiver<String>,
    stderr: Option<JoinHandle<String>>,
}

impl Watching {
    fn spawn(tree: &Path, cache: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_fdu"))
            .args(["--watch", "--view", "files", "--format", "jsonl", "--interval", "1s"])
            .arg(tree)
            .env("XDG_CACHE_HOME", cache)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn watching fdu");
        let stdout = child.stdout.take().expect("piped stdout");
        let (sender, lines) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        // Drained on its own thread so a chatty child can never block on a full pipe.
        let mut stderr_pipe = child.stderr.take().expect("piped stderr");
        let stderr = std::thread::spawn(move || {
            let mut text = String::new();
            let _ = stderr_pipe.read_to_string(&mut text);
            text
        });
        Self { child, lines, stderr: Some(stderr) }
    }

    /// The next line satisfying `wanted`, failing with the child's own diagnostics when
    /// it exits or stalls first.
    fn wait_for(&mut self, description: &str, wanted: impl Fn(&str) -> bool) -> String {
        let started = Instant::now();
        loop {
            match self.lines.recv_timeout(DEADLINE.saturating_sub(started.elapsed())) {
                Ok(line) if wanted(&line) => return line,
                Ok(_) => {}
                Err(RecvTimeoutError::Timeout) => {
                    self.fail(&format!("timed out after {DEADLINE:?} waiting for {description}"));
                }
                Err(RecvTimeoutError::Disconnected) => {
                    self.fail(&format!("the watch exited before {description}"));
                }
            }
        }
    }

    /// The opening answer's envelope, once the files section proves the watcher is bound.
    ///
    /// The session binds its watcher before rendering, so a change made after the files
    /// section arrives cannot fall before the watch began.
    fn initial_report(&mut self) -> String {
        let envelope =
            self.wait_for("the initial report envelope", |line| line.contains("\"fdu.report/"));
        self.wait_for("the initial files section", |line| line.contains("\"view\": \"files\""));
        envelope
    }

    /// Wait for the change record naming `path`.
    fn wait_for_change(&mut self, path: &str) -> String {
        let quoted = format!("\"path\": \"{path}\"");
        self.wait_for(&format!("a change record for {path}"), |line| {
            line.contains("\"record\": \"change\"") && line.contains(&quoted)
        })
    }

    fn fail(&mut self, reason: &str) -> ! {
        let _ = self.child.kill();
        let status = self.child.wait().ok();
        let stderr = self.stderr.take().and_then(|reader| reader.join().ok()).unwrap_or_default();
        panic!("{reason}; exit status {status:?}; stderr: {stderr}");
    }
}

impl Drop for Watching {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Run `fdu` to completion and return stdout.
fn report(tree: &Path, cache: &Path, args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(args)
        .arg(tree)
        .env("XDG_CACHE_HOME", cache)
        .output()
        .expect("run fdu");
    assert!(
        output.status.success(),
        "fdu {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn tree_with(files: &[(&str, &[u8])]) -> (tempfile::TempDir, std::path::PathBuf) {
    let root = tempfile::tempdir().expect("tempdir");
    let tree = root.path().join("tree");
    fs::create_dir(&tree).expect("create tree");
    for (name, bytes) in files {
        fs::write(tree.join(name), bytes).expect("write fixture file");
    }
    (root, tree)
}

#[test]
fn a_watch_serves_a_tree_whose_ignore_rule_exceeds_the_control_bound() {
    let rule = oversized_rule();
    let (_root, tree) = tree_with(&[(".gitignore", &rule), ("kept.txt", b"kept")]);
    let cache = tempfile::tempdir().expect("cache tempdir");

    let mut watch = Watching::spawn(&tree, cache.path());
    let envelope = watch.initial_report();

    assert!(
        envelope.contains("\"complete\": true"),
        "a watch over an oversized ignore rule answered only partially: {envelope}",
    );
}

#[test]
fn a_control_file_edited_past_the_bound_does_not_end_a_watch() {
    let (_root, tree) = tree_with(&[(".gitignore", b"*.log\n"), ("kept.txt", b"kept")]);
    let cache = tempfile::tempdir().expect("cache tempdir");

    let mut watch = Watching::spawn(&tree, cache.path());
    watch.initial_report();

    fs::write(tree.join(".gitignore"), oversized_rule()).expect("rewrite the control file");
    watch.wait_for_change(".gitignore");

    // The session outlived the edit and still applies ordinary changes after it.
    fs::write(tree.join("after.txt"), b"after").expect("write a later file");
    watch.wait_for_change("after.txt");

    // And it persisted them under its own scope: once its snapshot holds the later file,
    // the next watch starts warm from it and answers completely.
    let started = Instant::now();
    while !report(&tree, cache.path(), &["--view", "files", "--format", "jsonl", "--cache", "only"])
        .contains("\"path\": \"after.txt\"")
    {
        assert!(started.elapsed() < DEADLINE, "the watch never persisted the later change");
        std::thread::sleep(Duration::from_millis(100));
    }
    drop(watch);
    let mut next = Watching::spawn(&tree, cache.path());
    let envelope = next.initial_report();
    assert!(
        envelope.contains("\"source\": \"warm_revalidate\"")
            && envelope.contains("\"complete\": true"),
        "the snapshot a watch saved after a control edit did not start the next watch: {envelope}",
    );
}

#[test]
fn a_watch_and_a_one_shot_report_start_warm_from_each_others_snapshot() {
    let (_root, tree) = tree_with(&[(".gitignore", b"*.log\n"), ("kept.txt", b"kept")]);
    let cache = tempfile::tempdir().expect("cache tempdir");

    // With no cache, the watch scans cold and has saved its snapshot by the time it
    // renders: the initial open's write is joined before the first answer.
    {
        let mut watch = Watching::spawn(&tree, cache.path());
        let envelope = watch.initial_report();
        assert!(
            envelope.contains("\"source\": \"cold_scan\""),
            "expected a cold start: {envelope}"
        );
    }

    // A plain metadata report never reads the snapshot, so content analysis is the
    // one-shot run whose cost depends on the watch's snapshot sharing its scope.
    let analyzed =
        report(&tree, cache.path(), &["--analyze", "lines", "--view", "files", "--format", "json"]);
    assert!(
        analyzed.contains("\"source\": \"warm_revalidate\""),
        "a report after a watch rescanned instead of reusing the watch's snapshot: {analyzed}",
    );

    // And the other order: a one-shot report's snapshot starts the next watch warm.
    report(&tree, cache.path(), &["--view", "files", "--format", "json", "--cache", "refresh"]);
    let mut watch = Watching::spawn(&tree, cache.path());
    let envelope = watch.initial_report();
    assert!(
        envelope.contains("\"source\": \"warm_revalidate\""),
        "a watch after a one-shot report rescanned instead of reusing its snapshot: {envelope}",
    );
}
