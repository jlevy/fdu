use assert_cmd::Command;

#[test]
fn the_default_report_is_a_bounded_directory_overview() {
    let workspace = tempfile::tempdir().unwrap();
    Command::cargo_bin("acorn")
        .unwrap()
        .arg(workspace.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("100%  ."));
}

#[test]
fn a_bare_command_prints_help_without_scanning() {
    Command::cargo_bin("acorn")
        .unwrap()
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage:"));
}
