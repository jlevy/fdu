//! A watch session must leave a usable cache behind even when it is killed outright.
//!
//! Watch sessions end by signal far more often than they end politely -- Ctrl-C, a
//! closed terminal, a supervisor restart -- so persisting only at exit would persist
//! approximately never, and every session would hand the next run a cold start it had
//! already paid for. This drives the real binary and kills it without warning, because
//! the property is specifically about the ending no handler gets to observe.
#![cfg(all(feature = "watch", unix))]

use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

/// Generous: a cold scan, an event round trip, and a save all have to fit.
const DEADLINE: Duration = Duration::from_secs(30);

/// The watcher under test, with everything it has written to stderr so far.
///
/// Its stderr is kept rather than discarded because a watcher that stops persisting and
/// a watcher that has exited look the same from the snapshot's side: nothing rewrites
/// it. A failure has to say which one it saw (fdu-jqhd), so the exit status and the
/// warnings the watcher printed are part of every assertion about it.
struct WatchChild {
    child: Child,
    stderr: Arc<Mutex<Vec<u8>>>,
}

impl WatchChild {
    /// Spawn `fdu --watch` over `tree` with the cache at `cache`, keeping its stderr.
    fn spawn(tree: &Path, cache: &Path, args: &[&str], stdout: Stdio) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_fdu"))
            .arg("--watch")
            .args(args)
            .arg(tree)
            .env("XDG_CACHE_HOME", cache)
            // FDU_CACHE_DIR outranks XDG_CACHE_HOME; an exported one would reach the real
            // cache.
            .env_remove("FDU_CACHE_DIR")
            .stdout(stdout)
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn watching fdu");
        let stderr = Arc::new(Mutex::new(Vec::new()));
        let mut pipe = child.stderr.take().expect("watch stderr");
        let sink = Arc::clone(&stderr);
        // Appended as it arrives, so the evidence is readable while the watcher runs.
        std::thread::spawn(move || {
            let mut chunk = [0_u8; 4096];
            while let Ok(read) = pipe.read(&mut chunk) {
                if read == 0 {
                    break;
                }
                if let Ok(mut buffer) = sink.lock() {
                    buffer.extend_from_slice(&chunk[..read]);
                }
            }
        });
        Self { child, stderr }
    }

    /// What can be said about the watcher when a wait on it fails: whether it is still
    /// running or how it exited, and what it wrote to stderr.
    fn evidence(&mut self) -> String {
        let status = match self.child.try_wait() {
            Ok(None) => "still running".to_owned(),
            Ok(Some(status)) => format!("exited with {status}"),
            Err(error) => format!("in an unknown state ({error})"),
        };
        let stderr = self
            .stderr
            .lock()
            .map(|buffer| String::from_utf8_lossy(&buffer).into_owned())
            .unwrap_or_default();
        format!("the watcher was {status}; its stderr was {stderr:?}")
    }
}

/// Reap a watcher even when an assertion fails before the test's explicit kill.
impl Drop for WatchChild {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Identify the snapshot on disk by size and modification time.
///
/// Distinguishes one *write* from another, which is the whole difficulty here: the
/// initial `open()` leaves a snapshot before the watch loop starts, so presence alone
/// says nothing about whether the loop ever saved.
fn snapshot_fingerprint(cache_dir: &Path) -> Option<(u64, std::time::SystemTime)> {
    let entries = fs::read_dir(cache_dir.join("fdu")).ok()?;
    entries
        .flatten()
        // An atomic save writes a staging sibling before renaming it over the
        // conventional metadata file. Only the final 16-hex-key name proves a
        // replacement committed.
        .filter(|entry| {
            entry.file_name().to_str().is_some_and(|name| {
                name.strip_suffix(".metadata.bin").is_some_and(|key| {
                    key.len() == 16 && key.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
            })
        })
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

/// Wait until the spawned watcher has provably registered, then return its baseline.
///
/// A watch process is not watching the moment it starts: registration is requested first
/// and takes effect a little later, and anything written before then produces no event at
/// all. The engine reports that honestly as a setup race meaning "relist the root", so a
/// test whose subject write lands in that window is asserting against a reply it never
/// asked for. Both tests here used a fixed sleep, which is a guess about how long
/// registration takes, and the guess is wrong exactly when the machine is busy.
///
/// A warm-up write proves it instead. When the snapshot changes because of that write,
/// the watcher is demonstrably live and persisting, so the subject write that follows
/// cannot fall into the setup window. The value returned is the fingerprint *after* the
/// warm-up, which is the baseline a later rewrite must differ from.
///
/// The baseline the warm-up must move is taken only once a snapshot exists. A cold open
/// writes its own snapshot before the watcher is bound, and a baseline read before that
/// write landed let it count as the warm-up's rewrite: the warm-up then proved the open
/// had saved, which it always does, and nothing about the loop.
fn establish_watch(
    cache_dir: &Path,
    tree: &Path,
    child: &mut WatchChild,
) -> Option<(u64, std::time::SystemTime)> {
    let started = Instant::now();
    let mut before = snapshot_fingerprint(cache_dir);
    while before.is_none() && started.elapsed() < DEADLINE {
        sleep(Duration::from_millis(100));
        before = snapshot_fingerprint(cache_dir);
    }
    assert!(
        before.is_some(),
        "the open never left a snapshot within {DEADLINE:?}, so there is no baseline a \
         watch loop's save could be told apart from; {}",
        child.evidence(),
    );
    fs::write(tree.join("warmup.txt"), b"warmup").expect("write warm-up file");
    assert!(
        wait_for_rewrite(cache_dir, before),
        "the watcher never persisted a warm-up change within {DEADLINE:?}, so it was never \
         observed to be watching and nothing after this point would be evidence about fdu; \
         {}",
        child.evidence(),
    );
    snapshot_fingerprint(cache_dir)
}

/// Run `fdu` to completion and return stdout.
fn report(tree: &Path, cache: &Path, args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(args)
        .arg(tree)
        .env("XDG_CACHE_HOME", cache)
        // FDU_CACHE_DIR outranks XDG_CACHE_HOME; an exported one would reach the real cache.
        .env_remove("FDU_CACHE_DIR")
        .output()
        .expect("run fdu");
    assert!(
        output.status.success(),
        "fdu {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn a_watch_started_from_a_warm_cache_still_persists_what_it_sees() {
    // The cold path and the warm path reach the save through different index states, and
    // only the cold one was covered. This matters now that a loaded index carries
    // `Cached` provenance: if that were ever conflated with the `Freshness` the save gates
    // on, a warm-started watch would silently stop persisting, and every existing test
    // would still pass because they all start cold.
    let root = tempfile::tempdir().expect("tempdir");
    let cache = tempfile::tempdir().expect("cache tempdir");
    let tree = root.path().join("tree");
    fs::create_dir(&tree).expect("create tree");
    fs::write(tree.join("first.txt"), b"first").expect("write first file");

    // Prime the cache, then prove the snapshot is usable with a cache-only read.
    //
    // The priming asks for `--cache on` because under `auto` a one-shot metadata report
    // writes nothing: no later one-shot report would read it. The usability proof uses
    // `--stale-ok` rather than a second ordinary run, because a one-shot metadata report
    // deliberately does not read the snapshot — the read cannot save the walk the report
    // is already doing. The watch below opens through the library path, which does read
    // it: a session amortises the load across its whole lifetime, and that warm start is
    // what this test pins.
    report(&tree, cache.path(), &["--view", "files", "--format", "json", "--cache", "on"]);
    let usable =
        report(&tree, cache.path(), &["--view", "files", "--format", "json", "--stale-ok"]);
    assert!(
        usable.contains("cache_only"),
        "expected the priming run to leave a usable snapshot, got: {usable}",
    );

    let mut child = WatchChild::spawn(
        &tree,
        cache.path(),
        &["--view", "files", "--interval", "1s"],
        Stdio::null(),
    );

    // Baseline *after* the watcher is provably registered, so a later rewrite is
    // provably the incremental one rather than that same startup file seen again.
    let initial = establish_watch(cache.path(), &tree, &mut child);

    fs::write(tree.join("second.txt"), b"second").expect("write second file");
    let rewritten = wait_for_rewrite(cache.path(), initial);
    let evidence = child.evidence();

    let _ = child.child.kill();
    let _ = child.child.wait();

    assert!(
        rewritten,
        "a warm-started watch never rewrote the snapshot after a change; {evidence}"
    );
    let listed =
        report(&tree, cache.path(), &["--view", "files", "--format", "jsonl", "--stale-ok"]);
    assert!(
        listed.contains("second.txt"),
        "a warm-started watch did not persist what it observed. Listing was: {listed}",
    );
}

#[test]
fn a_projected_controls_off_watch_never_replaces_the_stronger_snapshot() {
    let root = tempfile::tempdir().expect("tempdir");
    let cache = tempfile::tempdir().expect("cache tempdir");
    let tree = root.path().join("tree");
    fs::create_dir(&tree).expect("create tree");
    fs::write(tree.join(".gitignore"), b"*.log\n").expect("write controls");
    fs::write(tree.join("ignored.log"), b"ignored").expect("write ignored file");
    fs::write(tree.join("first.txt"), b"first").expect("write first file");

    report(&tree, cache.path(), &["--view", "files", "--format", "json", "--cache", "on"]);
    let stronger = snapshot_fingerprint(cache.path()).expect("controls-on snapshot");

    let mut child = WatchChild::spawn(
        &tree,
        cache.path(),
        &["--no-gitignore", "--view", "files", "--format", "jsonl", "--interval", "1s"],
        Stdio::piped(),
    );
    let stdout = child.child.stdout.take().expect("watch stdout");
    let (sent, received) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if sent.send(line).is_err() {
                break;
            }
        }
    });

    let initial = received
        .recv_timeout(DEADLINE)
        .unwrap_or_else(|_| panic!("watch initial report; {}", child.evidence()))
        .expect("read initial report");
    assert!(initial.contains("fdu.report/"), "unexpected initial report: {initial}");
    let started = Instant::now();
    loop {
        let line = received
            .recv_timeout(DEADLINE.saturating_sub(started.elapsed()))
            .unwrap_or_else(|_| panic!("watch initial files section; {}", child.evidence()))
            .expect("read initial files section");
        if line.contains("\"view\": \"files\"") {
            break;
        }
    }

    // `Session::new` binds the observer before the initial report is rendered, and the
    // files section proves that rendering completed. Seeing this change record then
    // proves the projected CLI route reached its incremental save path.
    fs::write(tree.join("warmup.txt"), b"warmup").expect("write warm-up file");
    let started = Instant::now();
    let mut observed = false;
    while started.elapsed() < DEADLINE {
        let Ok(line) = received.recv_timeout(DEADLINE.saturating_sub(started.elapsed())) else {
            break;
        };
        let Ok(line) = line else { break };
        if line.contains("\"record\": \"change\"") && line.contains("\"path\": \"warmup.txt\"") {
            observed = true;
            break;
        }
    }

    // The dirty warm-up batch may arrive before the one-second save throttle. Let the
    // idle path attempt its deferred save, then prove the projection guard kept the
    // stronger image.
    sleep(Duration::from_millis(1_500));
    let after = snapshot_fingerprint(cache.path()).expect("snapshot remains");
    let evidence = child.evidence();

    let _ = child.child.kill();
    let _ = child.child.wait();

    assert!(
        observed,
        "the projected watcher did not report the change used to test its save guard; \
         {evidence}"
    );
    assert_eq!(after, stronger, "the projected watch replaced the controls-on snapshot");
}

#[test]
fn a_killed_watch_still_leaves_a_warm_cache() {
    let root = tempfile::tempdir().expect("tempdir");
    let cache = tempfile::tempdir().expect("cache tempdir");
    let tree = root.path().join("tree");
    fs::create_dir(&tree).expect("create tree");
    fs::write(tree.join("first.txt"), b"first").expect("write first file");

    let mut child = WatchChild::spawn(
        &tree,
        cache.path(),
        &["--view", "files", "--interval", "1s"],
        Stdio::null(),
    );

    // Let the initial open's snapshot land and record it, so a later write is provably a
    // second one rather than that same file seen again.
    let initial = establish_watch(cache.path(), &tree, &mut child);

    fs::write(tree.join("second.txt"), b"second").expect("write second file");
    let rewritten = wait_for_rewrite(cache.path(), initial);
    let evidence = child.evidence();

    // SIGKILL: the exit no signal handler can intercept. Whatever is on disk now is
    // exactly what a real interrupted session would have left.
    let _ = child.child.kill();
    let _ = child.child.wait();

    assert!(
        rewritten,
        "the watch loop never rewrote the snapshot after a change, so everything observed \
         while watching would be lost; {evidence}",
    );

    // The saved snapshot has to be usable, not merely present: a later run must accept
    // it as warm rather than rescanning.
    let output = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(["--view", "summary", "--format", "json", "--stale-ok"])
        .arg(&tree)
        .env("XDG_CACHE_HOME", cache.path())
        .env_remove("FDU_CACHE_DIR")
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
        .args(["--view", "files", "--format", "jsonl", "--stale-ok"])
        .arg(&tree)
        .env("XDG_CACHE_HOME", cache.path())
        .env_remove("FDU_CACHE_DIR")
        .output()
        .expect("list the saved cache");
    let listed = String::from_utf8_lossy(&listing.stdout);
    assert!(
        listed.contains("second.txt"),
        "the snapshot predates the watch session: it holds only the initial open's index, \
         so incremental saves never ran. Listing was: {listed}",
    );
}
