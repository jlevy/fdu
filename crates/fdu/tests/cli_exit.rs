//! End-to-end process exit-code contract for incomplete filesystem scans.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[test]
fn partial_results_use_exit_two_unless_explicitly_allowed() {
    let root = tempfile::tempdir().expect("tempdir");
    let denied = root.path().join("denied");
    fs::create_dir(&denied).expect("create denied directory");
    fs::write(denied.join("hidden.txt"), b"hidden").expect("write hidden file");
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).expect("deny reads");

    let run = |allow_partial: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_fdu"));
        command.args(["--no-cache", "--json"]);
        if allow_partial {
            command.arg("--allow-partial");
        }
        command.arg(root.path()).output().expect("run fdu")
    };

    let partial = run(false);
    let allowed = run(true);

    let human = Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(["--no-cache", "--color", "never"])
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
