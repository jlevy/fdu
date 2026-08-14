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
fn permission_bits_are_enforced() -> bool {
    let Ok(directory) = tempfile::tempdir() else {
        return false;
    };
    let path = directory.path().join("unreadable");
    fs::write(&path, b"probe").is_ok()
        && fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).is_ok()
        && fs::read(&path).is_err()
}

#[test]
fn partial_results_use_exit_two_unless_explicitly_allowed() {
    if !permission_bits_are_enforced() {
        eprintln!("skipped: this process is not subject to Unix permission bits");
        return;
    }

    let root = tempfile::tempdir().expect("tempdir");
    let denied = root.path().join("denied");
    fs::create_dir(&denied).expect("create denied directory");
    fs::write(denied.join("hidden.txt"), b"hidden").expect("write hidden file");
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

    let human = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(["--cache", "off", "--color", "never"])
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

    assert_eq!(human.status.code(), Some(2));
    let human_stdout = String::from_utf8(human.stdout).expect("human stdout is UTF-8");
    let human_stderr = String::from_utf8(human.stderr).expect("human stderr is UTF-8");
    assert!(!human_stdout.contains("warning:"), "diagnostic leaked to stdout: {human_stdout}");
    assert!(human_stderr.starts_with("warning:"), "missing stderr warning: {human_stderr}");
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
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    assert!(enabled_stderr.contains("[process ("), "{enabled_stderr}");
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    assert!(enabled_stderr.contains("process counters: not available"), "{enabled_stderr}");

    assert!(disabled.status.success());
    assert!(disabled.stderr.is_empty(), "falsey toggle emitted a report");
}
