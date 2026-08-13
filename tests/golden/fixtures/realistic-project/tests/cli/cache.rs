use assert_cmd::Command;

#[test]
fn a_warm_open_matches_a_cold_report() {
    let workspace = tempfile::tempdir().unwrap();
    Command::cargo_bin("acorn")
        .unwrap()
        .args(["--cache", "refresh", workspace.path().to_str().unwrap()])
        .assert()
        .success();
    Command::cargo_bin("acorn")
        .unwrap()
        .args(["--cache", "read-only", workspace.path().to_str().unwrap()])
        .assert()
        .success();
}
