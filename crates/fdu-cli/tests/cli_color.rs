//! End-to-end color and machine-output contract at the real process boundary.

use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fdu"))
        .args(args)
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .output()
        .expect("run fdu")
}

fn run_with_color_env(args: &[&str], no_color: Option<&str>, force_color: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fdu"));
    command.args(args).env_remove("NO_COLOR").env_remove("FORCE_COLOR");
    if let Some(value) = no_color {
        command.env("NO_COLOR", value);
    }
    if let Some(value) = force_color {
        command.env("FORCE_COLOR", value);
    }
    command.output().expect("run fdu with color environment")
}

fn has_ansi(bytes: &[u8]) -> bool {
    bytes.windows(2).any(|bytes| bytes == b"\x1b[")
}

#[test]
fn explicit_color_controls_human_output_but_never_json() {
    let root = tempfile::tempdir().expect("tempdir");
    let root = root.path().to_str().expect("temporary path is Unicode");

    let always = run(&["--cache", "off", "--color", "always", "--depth", "0", root]);
    assert!(always.status.success());
    assert!(has_ansi(&always.stdout));

    let never = run(&["--cache", "off", "--color", "never", "--depth", "0", root]);
    assert!(never.status.success());
    assert!(!has_ansi(&never.stdout));

    // Every machine format stays plain even when colour is forced: a consumer parsing
    // the output should never have to strip escape sequences first.
    for format in ["json", "jsonl", "yaml"] {
        let machine =
            run(&["--cache", "off", "--color", "always", "--format", format, "--depth", "0", root]);
        assert!(machine.status.success(), "{format} run failed");
        assert!(!has_ansi(&machine.stdout), "{format} output was colourized");
        assert!(machine.stderr.is_empty(), "{format} wrote to stderr");
    }

    let skill = run(&["--color", "always", "--skill"]);
    assert!(skill.status.success());
    assert!(!has_ansi(&skill.stdout));
    assert!(skill.stderr.is_empty());
}

/// A multi-view text report labels each block, and the label follows the colour axis.
///
/// Asserted at the process boundary rather than only over `render`, because the header
/// is the one part of the report whose presence depends on how many views were asked
/// for — a flag-parsing decision — and this is where flag parsing actually happens.
#[test]
fn view_headers_appear_only_for_several_views_and_follow_the_color_axis() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join("main.rs"), b"fn main() {}\n").expect("write");
    let root = root.path().to_str().expect("temporary path is Unicode");

    let several = run(&[
        "--cache",
        "off",
        "--color",
        "never",
        "--view",
        "types,summary",
        "--depth",
        "0",
        root,
    ]);
    assert!(several.status.success());
    let text = String::from_utf8(several.stdout).expect("utf-8");
    assert!(text.contains("TYPES\n"), "{text}");
    assert!(text.contains("SUMMARY\n"), "{text}");

    // One view is unambiguous on its own, so the layout is left exactly as it was.
    for view in ["types", "summary", "files", "tree"] {
        let lone =
            run(&["--cache", "off", "--color", "never", "--view", view, "--depth", "0", root]);
        assert!(lone.status.success());
        let text = String::from_utf8(lone.stdout).expect("utf-8");
        assert!(
            !text.contains(&view.to_uppercase()),
            "a lone {view} view must not be labelled: {text}"
        );
    }

    // Colour applies to the label like any other human decoration: the word is still
    // there uncoloured, so the layout never depends on a terminal supporting escapes.
    let colored = run(&[
        "--cache",
        "off",
        "--color",
        "always",
        "--view",
        "types,summary",
        "--depth",
        "0",
        root,
    ]);
    assert!(colored.status.success());
    let colored = String::from_utf8(colored.stdout).expect("utf-8");
    assert!(colored.contains("TYPES"), "{colored:?}");
    let header_line = colored.lines().find(|line| line.contains("TYPES")).expect("a header line");
    assert!(has_ansi(header_line.as_bytes()), "header carries no escapes: {header_line:?}");

    // Machine formats name their view in a field and must not gain the text header.
    for format in ["json", "jsonl", "yaml"] {
        let machine = run(&[
            "--cache",
            "off",
            "--color",
            "always",
            "--format",
            format,
            "--view",
            "types,summary",
            "--depth",
            "0",
            root,
        ]);
        assert!(machine.status.success(), "{format} run failed");
        let text = String::from_utf8(machine.stdout).expect("utf-8");
        assert!(!text.contains("TYPES"), "{format} leaked a text header:\n{text}");
        assert!(!text.contains("SUMMARY"), "{format} leaked a text header:\n{text}");
    }
}

#[test]
fn explicit_color_also_controls_self_documenting_help() {
    let always = run(&["--color", "always", "--help"]);
    assert!(always.status.success());
    assert!(has_ansi(&always.stdout));
    assert!(always.stderr.is_empty());

    let never = run(&["--color", "never", "--help"]);
    assert!(never.status.success());
    assert!(!has_ansi(&never.stdout));
    assert!(never.stderr.is_empty());
}

#[test]
fn automatic_color_honors_environment_with_documented_precedence() {
    let root = tempfile::tempdir().expect("tempdir");
    let root = root.path().to_str().expect("temporary path is Unicode");
    let base = ["--cache", "off", "--depth", "0", root];

    let forced = run_with_color_env(&base, None, Some("1"));
    assert!(forced.status.success());
    assert!(has_ansi(&forced.stdout));

    let zero_is_not_forced = run_with_color_env(&base, None, Some("0"));
    assert!(zero_is_not_forced.status.success());
    assert!(!has_ansi(&zero_is_not_forced.stdout));

    let suppressed = run_with_color_env(&base, Some("1"), Some("1"));
    assert!(suppressed.status.success());
    assert!(!has_ansi(&suppressed.stdout));

    let empty_no_color_is_ignored = run_with_color_env(&base, Some(""), Some("1"));
    assert!(empty_no_color_is_ignored.status.success());
    assert!(has_ansi(&empty_no_color_is_ignored.stdout));

    let explicit_always = run_with_color_env(
        &["--color", "always", "--cache", "off", "--depth", "0", root],
        Some("1"),
        None,
    );
    assert!(explicit_always.status.success());
    assert!(has_ansi(&explicit_always.stdout));

    let explicit_never = run_with_color_env(
        &["--color", "never", "--cache", "off", "--depth", "0", root],
        None,
        Some("1"),
    );
    assert!(explicit_never.status.success());
    assert!(!has_ansi(&explicit_never.stdout));
}
