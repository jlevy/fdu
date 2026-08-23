//! Embed the checkout revision and compile repository-owned file-type rules.
//!
//! Development builds include the Git revision and dirty marker in `--version`.
//! `emit_version` is duplicated in `fdu/build.rs`; that script explains why neither copy
//! can be removed, and this one exists because the perf probe is this crate's example and
//! the provenance gate asserts the revision in its `--version` (fdu-zuyq).
//! The `[[kind]]` manifest is validated and rendered as native Rust data so runtime
//! classification never parses configuration or builds match structures.

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const RULES_PATH: &str = "rules/file-types.toml";
const GENERATED_NAME: &str = "file_type_rules.rs";
/// The dialect's one parser, compiled into the crate and included here.
///
/// Sharing it is the point: the rules a caller supplies at run time are read by exactly
/// the code that read this repository's manifest at build time, so the two cannot come
/// to disagree about what `[[kind]]` means.
const MANIFEST_PARSER_PATH: &str = "src/classify/type_rule_manifest.rs";

include!("src/classify/type_rule_manifest.rs");

fn main() {
    emit_version();
    println!("cargo:rerun-if-changed={RULES_PATH}");
    println!("cargo:rerun-if-changed={MANIFEST_PARSER_PATH}");
    let source = fs::read_to_string(RULES_PATH).expect("read file-type rules");
    let rules = parse_manifest(&source).unwrap_or_else(|error| panic!("{RULES_PATH}: {error}"));
    validate_manifest(&rules).unwrap_or_else(|error| panic!("{RULES_PATH}: {error}"));
    let generated = render_rules(&rules, manifest_fingerprint(&source));
    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"));
    fs::write(output.join(GENERATED_NAME), generated).expect("write compiled file-type rules");
}

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    Some(text.trim().to_string())
}

fn emit_version() {
    let semver = std::env::var("CARGO_PKG_VERSION").expect("cargo sets CARGO_PKG_VERSION");
    println!("cargo:rerun-if-env-changed=FDU_RELEASE_TAG");
    if let Ok(tag) = std::env::var("FDU_RELEASE_TAG") {
        let expected = format!("v{semver}");
        assert_eq!(tag, expected, "FDU_RELEASE_TAG must exactly match the Cargo package version");
        println!("cargo:rustc-env=FDU_BUILD_VERSION={semver}");
        return;
    }
    let version = match git(&["rev-parse", "--short=9", "HEAD"]) {
        Some(revision) => {
            // This preserves Cargo's default whole-package dirty tracking after adding
            // the narrower manifest and Git rerun directives below.
            println!("cargo:rerun-if-changed=.");
            if let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) {
                println!("cargo:rerun-if-changed={git_dir}/HEAD");
                if let Some(head_ref) = git(&["symbolic-ref", "-q", "HEAD"]) {
                    if let Some(common) = git(&["rev-parse", "--git-common-dir"]) {
                        println!("cargo:rerun-if-changed={common}/{head_ref}");
                    }
                }
            }
            let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
                .is_some_and(|status| !status.is_empty());
            let expected_tag = format!("v{semver}");
            let exact_release = !dirty
                && git(&["tag", "--points-at", "HEAD"])
                    .is_some_and(|tags| tags.lines().any(|tag| tag == expected_tag));
            if exact_release {
                semver
            } else {
                format!("{semver}-dev+g{revision}{}", if dirty { ".dirty" } else { "" })
            }
        }
        None => semver,
    };
    println!("cargo:rustc-env=FDU_BUILD_VERSION={version}");
}

fn render_rules(rules: &[ManifestRule], fingerprint: u64) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "#[allow(clippy::unreadable_literal)]\nconst TYPE_RULE_FINGERPRINT: u64 = {fingerprint};"
    )
    .expect("write to string");
    output.push_str("static GENERATED_RULES: &[GeneratedRule] = &[\n");
    for rule in rules {
        writeln!(
            output,
            "    GeneratedRule {{ id: {:?}, family: ContentFamily::{}, extensions: &{:?}, filenames: &{:?}, shebangs: &{:?}, priority: {} }},",
            rule.id,
            family_variant(&rule.family),
            rule.extensions,
            rule.filenames,
            rule.shebangs,
            rule.priority,
        )
        .expect("write to string");
    }
    output.push_str("];\n");
    output
}

fn family_variant(family: &str) -> &'static str {
    match family {
        "code" => "Code",
        "prose" => "Prose",
        "markup" => "Markup",
        "data" => "Data",
        "binary" => "Binary",
        "unknown" => "Unknown",
        _ => unreachable!("validated family"),
    }
}
