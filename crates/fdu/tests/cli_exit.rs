//! End-to-end process exit-code contract for incomplete filesystem scans.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

/// Whether this process is actually subject to Unix permission bits.
///
/// A privileged process reads the denied directory anyway, so the fixture below cannot
/// produce a partial scan and the exit-code contract it pins is untestable. Probing the
/// capability asks the question the fixture depends on, rather than inferring it from a
/// user id.
fn require_permission_bits() -> bool {
    let Ok(directory) = tempfile::tempdir() else {
        panic!("permission fixture precondition failed: could not create its probe directory");
    };
    let path = directory.path().join("unreadable");
    let enforced = fs::write(&path, b"probe").is_ok()
        && fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).is_ok()
        && fs::read(&path).is_err();
    if enforced {
        return true;
    }
    if std::env::var_os("FDU_TEST_ALLOW_NO_PERMISSION_BITS").as_deref()
        == Some(std::ffi::OsStr::new("1"))
    {
        eprintln!(
            "skipped by FDU_TEST_ALLOW_NO_PERMISSION_BITS=1: this host does not enforce Unix \
             permission bits for the test process"
        );
        return false;
    }
    panic!(
        "permission fixture precondition failed: this process can read a mode-000 file; \
         run on a host that enforces Unix permission bits, or explicitly opt out with \
         FDU_TEST_ALLOW_NO_PERMISSION_BITS=1"
    );
}

#[test]
fn partial_results_use_exit_two_unless_explicitly_allowed() {
    if !require_permission_bits() {
        return;
    }

    let root = tempfile::tempdir().expect("tempdir");
    let denied = root.path().join("denied");
    fs::create_dir(&denied).expect("create denied directory");
    fs::write(denied.join("hidden.txt"), b"hidden").expect("write hidden file");
    // Match the portable engine golden: an unreadable branch plus an exact 1%
    // sibling, tiny verified entries, and a verified empty file.
    for (name, size) in [("large", 9898), ("one-percent", 100), ("tiny", 1), ("zero", 0)] {
        fs::write(root.path().join(name), vec![b'x'; size]).expect("write sized file");
    }
    fs::create_dir(root.path().join("small")).expect("create healthy directory");
    fs::write(root.path().join("small/tiny"), b"x").expect("write small file");
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).expect("deny reads");

    let run = |allow_partial: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_fdu"));
        command.args(["--cache", "off", "--format", "json"]);
        if allow_partial {
            command.arg("--allow-partial");
        }
        command.arg(root.path()).output().expect("run fdu")
    };

    let partial = run(false);
    let allowed = run(true);

    let traced = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(["--cache", "off", "--view", "summary", "--format", "json"])
        .env("FDU_SCAN_DIAGNOSTICS", "1")
        .arg(root.path())
        .output()
        .expect("run traced partial fdu");

    let human = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(["--cache", "off", "--color", "never", "--size", "apparent"])
        .arg(root.path())
        .output()
        .expect("run human fdu");
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o700)).expect("restore reads");

    assert_eq!(
        partial.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&partial.stderr)
    );
    let stdout = String::from_utf8(partial.stdout).expect("JSON is UTF-8");
    assert!(stdout.contains("\"complete\": false"), "{stdout}");
    assert!(stdout.contains("/denied"), "error details missing: {stdout}");
    assert!(allowed.status.success(), "stderr: {}", String::from_utf8_lossy(&allowed.stderr));

    assert_eq!(traced.status.code(), Some(2));
    let traced_stdout = String::from_utf8(traced.stdout).expect("traced JSON is UTF-8");
    let traced_stderr = String::from_utf8(traced.stderr).expect("trace is UTF-8");
    assert!(traced_stdout.contains("\"complete\": false"), "{traced_stdout}");
    assert!(traced_stdout.contains("/denied"), "{traced_stdout}");
    assert!(
        traced_stderr.contains("__FDU_SCAN_DIAGNOSTICS__={\"backend\":{\"linux_dents_attempts\":"),
        "{traced_stderr}"
    );
    assert!(traced_stderr.contains("\"schema\":\"fdu-scan-diagnostics-v1\""), "{traced_stderr}");

    assert_eq!(human.status.code(), Some(2));
    let human_stdout = String::from_utf8(human.stdout).expect("human stdout is UTF-8");
    let human_stderr = String::from_utf8(human.stderr).expect("human stderr is UTF-8");
    assert!(
        human_stdout.starts_with(include_str!("../../fdu-core/tests/golden/partial-tree.txt")),
        "partial CLI report differs from the engine golden: {human_stdout}"
    );
    assert!(!human_stdout.contains("warn:"), "diagnostic leaked to stdout: {human_stdout}");
    assert!(
        human_stderr.lines().any(|line| line.starts_with("warn:")),
        "missing stderr warn: {human_stderr}"
    );
}

/// Text mode prints one warning per retained issue, and the retention bound drops the
/// rest; the count of what it dropped must reach the terminal too, not only the machine
/// formats, or 200 unreadable directories read as 64 (fdu-peil).
#[test]
fn text_warnings_state_how_many_errors_were_omitted() {
    if !require_permission_bits() {
        return;
    }

    let denied_count = fdu_core::MAX_RETAINED_ISSUES + 6;
    let root = tempfile::tempdir().expect("tempdir");
    let denied: Vec<_> =
        (0..denied_count).map(|n| root.path().join(format!("denied-{n:03}"))).collect();
    for directory in &denied {
        fs::create_dir(directory).expect("create denied directory");
        fs::set_permissions(directory, fs::Permissions::from_mode(0o000)).expect("deny reads");
    }
    let run = |format: &str| {
        Command::new(env!("CARGO_BIN_EXE_fdu"))
            .args(["--cache", "off", "--color", "never", "--format", format])
            .arg(root.path())
            .output()
            .expect("run fdu")
    };
    let text = run("text");
    let json = run("json");
    for directory in &denied {
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).expect("restore");
    }

    assert_eq!(text.status.code(), Some(2));
    let json = String::from_utf8(json.stdout).expect("JSON is UTF-8");
    assert!(json.contains("\"errors_omitted\": 6"), "{json}");
    let stderr = String::from_utf8(text.stderr).expect("stderr is UTF-8");
    let warnings: Vec<&str> = stderr.lines().filter(|line| line.starts_with("warn:")).collect();
    assert_eq!(warnings.len(), fdu_core::MAX_RETAINED_ISSUES + 1, "{stderr}");
    assert_eq!(
        warnings.last().copied(),
        Some("warn: 6 more errors omitted; details are kept for the first 64"),
        "{stderr}"
    );
}

/// Every report reads `.gitignore`, so one it cannot read is an unreadable path like any
/// other: the report is partial and exits 2, `--allow-partial` accepts it, and
/// `--no-gitignore`, which reads no rule, is the escape (fdu-elnn).
#[test]
fn an_unreadable_gitignore_is_a_partial_result_that_no_gitignore_avoids() {
    if !require_permission_bits() {
        return;
    }

    let root = tempfile::tempdir().expect("tempdir");
    let control = root.path().join(".gitignore");
    fs::write(&control, b"*.log\n").expect("write the control file");
    fs::write(root.path().join("kept.txt"), b"kept").expect("write a file");
    fs::set_permissions(&control, fs::Permissions::from_mode(0o000)).expect("deny reads");

    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_fdu"))
            .args(["--cache", "off", "--format", "json", "--view", "summary"])
            .args(extra)
            .arg(root.path())
            .output()
            .expect("run fdu")
    };
    let partial = run(&[]);
    let allowed = run(&["--allow-partial"]);
    let unread = run(&["--no-gitignore"]);
    fs::set_permissions(&control, fs::Permissions::from_mode(0o600)).expect("restore reads");

    assert_eq!(partial.status.code(), Some(2), "{}", String::from_utf8_lossy(&partial.stderr));
    let stdout = String::from_utf8(partial.stdout).expect("JSON is UTF-8");
    assert!(stdout.contains("\"complete\": false"), "{stdout}");
    assert!(stdout.contains(".gitignore"), "the error names the control file: {stdout}");
    assert!(allowed.status.success(), "{}", String::from_utf8_lossy(&allowed.stderr));

    assert!(unread.status.success(), "{}", String::from_utf8_lossy(&unread.stderr));
    let stdout = String::from_utf8(unread.stdout).expect("JSON is UTF-8");
    assert!(stdout.contains("\"complete\": true"), "{stdout}");
    assert!(stdout.contains("\"ignore_rules\": null"), "{stdout}");
}

#[test]
fn runtime_instrumentation_is_available_on_the_shipped_binary() {
    let enabled = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .arg("--version")
        .env("FDU_COUNTERS", "1")
        .output()
        .expect("run instrumented fdu");
    let disabled = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .arg("--version")
        .env("FDU_COUNTERS", "0")
        .output()
        .expect("run ordinary fdu");

    assert!(enabled.status.success());
    let enabled_stderr = String::from_utf8(enabled.stderr).expect("counter report is UTF-8");
    assert!(enabled_stderr.contains("[filesystem operations]"), "{enabled_stderr}");
    assert!(enabled_stderr.contains("[memory]"), "{enabled_stderr}");
    // The worker-scaling policy is only auditable if its history reaches the artifacts
    // the performance loop actually reads.
    assert!(enabled_stderr.contains("[adaptive scan policy]"), "{enabled_stderr}");
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    assert!(enabled_stderr.contains("[process ("), "{enabled_stderr}");
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    assert!(enabled_stderr.contains("process counters: not available"), "{enabled_stderr}");

    assert!(disabled.status.success());
    assert!(disabled.stderr.is_empty(), "falsey toggle emitted a report");
}

#[test]
fn installed_summary_measurements_can_emit_the_versioned_scan_trace() {
    let root = tempfile::tempdir().expect("tempdir");
    fs::create_dir(root.path().join("nested")).expect("create nested directory");
    fs::write(root.path().join("nested/file.txt"), b"trace me").expect("write file");

    let run = |value: Option<&str>| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_fdu"));
        command
            .args(["--cache", "off", "--view", "summary", "--format", "json", "--color", "never"]);
        command.arg(root.path());
        match value {
            Some(value) => {
                command.env("FDU_SCAN_DIAGNOSTICS", value);
            }
            None => {
                command.env_remove("FDU_SCAN_DIAGNOSTICS");
            }
        }
        command.output().expect("run summary")
    };

    let enabled = run(Some("1"));
    assert!(enabled.status.success());
    let stderr = String::from_utf8(enabled.stderr).expect("diagnostic trace is UTF-8");
    let trace = stderr
        .strip_prefix("__FDU_SCAN_DIAGNOSTICS__=")
        .expect("measurement trace prefix")
        .trim_end();
    assert!(trace.starts_with('{') && trace.ends_with('}'), "{trace}");
    assert!(trace.contains("\"schema\":\"fdu-scan-diagnostics-v1\""), "{trace}");
    assert!(trace.contains("\"worker_policy\":"), "{trace}");
    assert!(trace.contains("\"backend\":"), "{trace}");

    assert!(run(None).stderr.is_empty(), "ordinary summary emitted a trace");
    assert!(run(Some("0")).stderr.is_empty(), "falsey toggle emitted a trace");
}

#[test]
fn installed_full_index_measurements_can_emit_the_versioned_scan_trace() {
    let root = tempfile::tempdir().expect("tempdir");
    fs::create_dir(root.path().join("nested")).expect("create nested directory");
    fs::write(root.path().join("nested/file.txt"), b"trace me").expect("write file");

    let completed = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(["--cache", "off", "--view", "tree", "--format", "json", "--color", "never"])
        .env("FDU_SCAN_DIAGNOSTICS", "1")
        .arg(root.path())
        .output()
        .expect("run full-index report");

    assert!(completed.status.success());
    let stderr = String::from_utf8(completed.stderr).expect("diagnostic trace is UTF-8");
    let trace = stderr
        .strip_prefix("__FDU_SCAN_DIAGNOSTICS__=")
        .expect("measurement trace prefix")
        .trim_end();
    assert!(trace.contains("\"schema\":\"fdu-scan-diagnostics-v1\""), "{trace}");
    assert!(trace.contains("\"worker_policy\":"), "{trace}");
    assert!(trace.contains("\"backend\":"), "{trace}");
}

#[test]
fn only_ignored_code_analysis_is_complete_across_cache_routes() {
    let root = tempfile::tempdir().expect("root");
    let cache = tempfile::tempdir().expect("cache");
    fs::create_dir(root.path().join("vendor")).expect("vendor");
    fs::write(root.path().join(".gitignore"), b"vendor/\n").expect("control");
    fs::write(root.path().join("main.rs"), b"fn main() {}\n").expect("unignored code");
    fs::write(root.path().join("vendor/lib.rs"), b"fn lib() {}\n").expect("ignored code");

    for policy in [&["--cache", "off"][..], &["--cache", "auto"], &["--stale-ok"]] {
        let policy = policy.join(" ");
        let output = Command::new(env!("CARGO_BIN_EXE_fdu"))
            .args(policy.split(' '))
            .arg("--cache-dir")
            .arg(cache.path())
            .args(["--ignored", "only", "--analyze", "code", "--format", "json"])
            .arg(root.path())
            .output()
            .expect("run only-ignored report");
        let stdout = String::from_utf8(output.stdout).expect("JSON output is UTF-8");
        assert!(output.status.success(), "{policy}: {stdout}");
        assert!(stdout.contains("\"complete\": true"), "{policy}: {stdout}");
    }
}

/// Three sparse files that claim more apparent bytes together than a `u64` can hold, in
/// a directory on a filesystem that lets this process create them, or `None` with the
/// reason printed.
///
/// tmpfs, XFS, and btrfs accept an 8 EiB sparse file from anyone and allocate nothing
/// for it; ext4 refuses one with `EFBIG`, and other filesystems cap a file far lower. So
/// the fixture is tried in the system temporary directory and then in `/dev/shm`, a
/// tmpfs on every Linux host this is tested on, and a host with neither is said to be
/// skipped rather than passed.
fn unrepresentable_sparse_tree() -> Option<tempfile::TempDir> {
    let size = u64::try_from(i64::MAX).expect("positive");
    let mut refusals = Vec::new();
    for base in [std::env::temp_dir(), std::path::PathBuf::from("/dev/shm")] {
        let directory = match tempfile::tempdir_in(&base) {
            Ok(directory) => directory,
            Err(error) => {
                refusals.push(format!("{}: {error}", base.display()));
                continue;
            }
        };
        let created = ["a", "b", "c"].iter().try_for_each(|name| {
            fs::File::create(directory.path().join(name)).and_then(|file| file.set_len(size))
        });
        match created {
            Ok(()) => return Some(directory),
            Err(error) => refusals.push(format!("{}: {error}", base.display())),
        }
    }
    eprintln!(
        "skipped: no filesystem this test can write to holds three sparse files of 8 EiB \
         apparent each ({})",
        refusals.join("; ")
    );
    None
}

/// A tree whose apparent bytes no `u64` can hold fails every route alike, with exit 1
/// and the same error, rather than wrapping the total on the routes a one-shot report
/// takes and refusing it only where a batch is committed (fdu-sqyk).
#[test]
fn a_total_no_u64_can_hold_fails_every_route_alike() {
    let Some(tree) = unrepresentable_sparse_tree() else { return };
    let cache = tempfile::tempdir().expect("cache directory");
    let cache_dir = cache.path().to_str().expect("UTF-8 cache path");
    let routes: [&[&str]; 7] = [
        &["--cache", "off"],
        &["--cache", "off", "--size", "apparent", "--format", "json"],
        &["--cache", "off", "--view", "summary"],
        &["--cache", "off", "--view", "summary", "--no-gitignore"],
        &["--cache", "off", "--view", "files"],
        &["--cache", "off", "--ignored", "exclude"],
        &["--cache", "on", "--cache-dir", cache_dir, "--view", "files"],
    ];
    for route in routes {
        let output = Command::new(env!("CARGO_BIN_EXE_fdu"))
            .args(["--color", "never"])
            .args(route)
            .arg(tree.path())
            .output()
            .expect("run fdu");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{route:?}: {stderr}");
        assert!(output.stdout.is_empty(), "{route:?} printed an answer");
        assert!(
            stderr.contains("would carry the tree's bytes total past what a u64 can hold"),
            "{route:?}: {stderr}"
        );
    }
}
