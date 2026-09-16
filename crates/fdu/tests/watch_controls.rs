//! `fdu --watch` observes `.gitignore` control state as a one-shot report does, keeps every
//! row's ignored share current as control files change, and no control bound can end it.
//!
//! A watch run is the same query repeated, so it reads the rules a one-shot report reads
//! and repaints when they change (fdu-elnn). A control file past a bound is refused and
//! named rather than ending the session, before its first answer or after an edit
//! (fdu-1onj). Both runs observe by default, so they share one snapshot scope and each
//! warm-starts from the other's snapshot (fdu-w3l5).
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

use fdu_core::control::DEFAULT_CONTROL_LINE_LIMIT;

/// Generous: a cold scan, an event round trip, and a loaded CI runner all have to fit.
const DEADLINE: Duration = Duration::from_secs(30);

/// A control source whose single rule is one byte over the per-line bound.
fn oversized_rule() -> Vec<u8> {
    let mut source = vec![b'a'; DEFAULT_CONTROL_LINE_LIMIT + 1];
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
        Self::spawn_view(tree, cache, "files")
    }

    fn spawn_view(tree: &Path, cache: &Path, view: &str) -> Self {
        Self::spawn_selecting(tree, cache, view, &[])
    }

    fn spawn_selecting(tree: &Path, cache: &Path, view: &str, selection: &[&str]) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_fdu"))
            .args(["--watch", "--view", view, "--format", "jsonl", "--interval", "1s"])
            .args(selection)
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

    /// Wait for the change record that applies `op` to `path`.
    fn wait_for_op(&mut self, op: &str, path: &str) -> String {
        let quoted = format!("\"path\": \"{path}\"");
        let operation = format!("\"op\": \"{op}\"");
        self.wait_for(&format!("a {op} record for {path}"), |line| {
            line.contains("\"record\": \"change\"")
                && line.contains(&operation)
                && line.contains(&quoted)
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
    assert!(
        envelope.contains("\"applied\": 0, \"refused\": 1"),
        "the watch read the control file and named its refusal: {envelope}",
    );
}

#[test]
fn a_control_file_edit_repaints_the_ignored_share() {
    let (_root, tree) =
        tree_with(&[(".gitignore", b"*.log\n"), ("kept.txt", b"kept"), ("debug.log", b"debug")]);
    let cache = tempfile::tempdir().expect("cache tempdir");

    let mut watch = Watching::spawn_view(&tree, cache.path(), "summary");
    watch.wait_for("the initial summary's ignored share", |line| {
        line.contains("\"view\": \"summary\"")
            && line.contains("\"ignored\": {\"files\": 1, \"dirs\": 0, \"bytes\": 5,")
    });

    // No file changes size or kind; only the rule that classifies one of them does.
    fs::write(tree.join(".gitignore"), b"# nothing ignored\n").expect("rewrite the control file");
    watch.wait_for("a repaint with nothing ignored", |line| {
        line.contains("\"view\": \"summary\"")
            && line.contains("\"ignored\": {\"files\": 0, \"dirs\": 0, \"bytes\": 0,")
    });
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

/// The flags name an entry set, so a rule edit that moves an entry across the partition
/// has to move it in the stream (fdu-kj14, review of #65). Nothing on disk changes for the
/// file itself: only the rule that classifies it does, and a files-only watch repaints no
/// aggregate, so the stream is the whole answer.
#[test]
fn a_rule_edit_moves_a_streamed_entry_out_of_and_back_into_excluded_ignored() {
    let (_root, tree) = tree_with(&[
        (".gitignore", b"# nothing ignored\n"),
        ("kept.txt", b"kept"),
        ("debug.log", b"debug"),
    ]);
    let cache = tempfile::tempdir().expect("cache tempdir");

    let mut watch =
        Watching::spawn_selecting(&tree, cache.path(), "files", &["--exclude-ignored"]);
    // The initial listing contains the file, and its row says no rule ignores it.
    let row = watch.wait_for("the initial row for debug.log", |line| {
        line.contains("\"path\": \"debug.log\"")
    });
    assert!(
        row.contains("\"ignored\": false"),
        "the initial row must state the classification the stream maintains: {row}",
    );

    fs::write(tree.join(".gitignore"), b"*.log\n").expect("rewrite the control file");
    let left = watch.wait_for_op("remove", "debug.log");
    assert!(
        left.contains("\"ignored\": true"),
        "the record that drops the row must say why it left: {left}",
    );

    // And back: lifting the rule returns the entry with the facts needed to draw it.
    fs::write(tree.join(".gitignore"), b"# nothing ignored\n").expect("rewrite the control file");
    let returned = watch.wait_for_op("upsert", "debug.log");
    assert!(
        returned.contains("\"ignored\": false")
            && returned.contains("\"kind\": \"file\"")
            && returned.contains("\"bytes\": 5"),
        "the record that restores the row must carry the facts to draw it: {returned}",
    );
}

/// The mirror: under `--only-ignored` an entry appears when a rule starts ignoring it and
/// leaves when the rule is lifted.
#[test]
fn a_rule_edit_moves_a_streamed_entry_into_and_out_of_only_ignored() {
    let (_root, tree) = tree_with(&[
        (".gitignore", b"# nothing ignored\n"),
        ("kept.txt", b"kept"),
        ("debug.log", b"debug"),
    ]);
    let cache = tempfile::tempdir().expect("cache tempdir");

    let mut watch = Watching::spawn_selecting(&tree, cache.path(), "files", &["--only-ignored"]);
    let section = watch.wait_for("the initial files section", |line| {
        line.contains("\"view\": \"files\"")
    });
    assert!(
        !section.contains("debug.log"),
        "no rule ignores anything yet, so the listing is empty: {section}",
    );

    fs::write(tree.join(".gitignore"), b"*.log\n").expect("rewrite the control file");
    let arrived = watch.wait_for_op("upsert", "debug.log");
    assert!(
        arrived.contains("\"ignored\": true") && arrived.contains("\"bytes\": 5"),
        "an entry entering the selection arrives with its facts: {arrived}",
    );

    fs::write(tree.join(".gitignore"), b"# nothing ignored\n").expect("rewrite the control file");
    let left = watch.wait_for_op("remove", "debug.log");
    assert!(
        left.contains("\"ignored\": false"),
        "the record that drops the row must say why it left: {left}",
    );
}
