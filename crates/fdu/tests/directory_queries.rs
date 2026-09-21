//! CLI projection, filtering, diagnostics, and compatibility on a real filesystem.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run(root: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(["--cache", "off", "--color", "never", "--size", "apparent"])
        .args(arguments)
        .arg(root)
        .output()
        .expect("run fdu")
}

fn body(output: Output) -> String {
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout)
        .expect("UTF-8 fixture output")
        .lines()
        .filter(|line| !line.starts_with("Performance:"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn default_tree_is_unchanged_while_flat_formats_are_complete_and_bounds_are_visible() {
    let root = tempfile::tempdir().expect("fixture");
    for number in 0..16 {
        let directory = root.path().join(format!("build-{number:02}"));
        fs::create_dir(&directory).expect("directory");
        fs::write(directory.join("payload"), vec![b'x'; number + 1]).expect("payload");
    }
    let default = body(run(root.path(), &[]));
    for arguments in [
        &["--view", "list"][..],
        &["--format", "tree"][..],
        &["--view", "list", "--format", "tree"][..],
        &["--view", "tree"][..],
        &["--tree"][..],
    ] {
        assert_eq!(body(run(root.path(), arguments)), default);
    }
    assert!(!default.contains("payload"), "files contribute totals without new tree leaves");
    let paths = run(root.path(), &["--kind", "dir", "--format", "paths"]);
    assert!(paths.status.success());
    let paths = String::from_utf8(paths.stdout).expect("paths");
    assert_eq!(paths.lines().count(), 16, "tree's implicit cap must not leak into flat output");
    assert_eq!(paths.lines().next(), Some("build-15"), "subtree sizes determine ordering");
    let long = body(run(root.path(), &["--kind", "dir", "--long"]));
    assert_eq!(long.lines().count(), 16);
    assert!(long.lines().next().expect("first row").contains("16 B"));
    let bounded = run(root.path(), &["--kind", "dir", "--format", "paths", "--limit", "1"]);
    assert!(bounded.status.success());
    assert_eq!(String::from_utf8(bounded.stdout).expect("path"), "build-15\n");
    assert!(String::from_utf8(bounded.stderr).expect("diagnostic").contains("16"));
    let json = body(run(root.path(), &["--kind", "dir", "--format", "json"]));
    assert!(json.contains("\"view\": \"list\""));
    assert!(json.contains("\"age_reference_ns\""));
    assert!(json.contains("\"age_ns\""));
    assert_eq!(json.matches("\"kind\": \"dir\"").count(), 16);
    let old_json = body(run(root.path(), &["--view", "tree", "--format", "json"]));
    assert!(old_json.contains("\"tree\": {"), "legacy structured tree remains available");
}

#[test]
fn incompatible_formats_and_alias_conflicts_fail_before_scanning() {
    let absent = Path::new("/missing-fdu-format-test-root");
    for arguments in [
        &["--view", "summary", "--format", "long"][..],
        &["--view", "full", "--format", "paths"][..],
        &["--view", "list,summary", "--tree"][..],
        &["--format", "paths", "--long"][..],
        &["--tree", "--long"][..],
    ] {
        let result = run(absent, arguments);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        let error = String::from_utf8(result.stderr).expect("diagnostic");
        assert!(error.contains("single list") || error.contains("cannot be used with"), "{error}");
        assert!(!error.contains("No such file"), "request validation must precede filesystem work");
    }
}
