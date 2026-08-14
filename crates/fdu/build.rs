//! Embed the checkout revision and compile repository-owned file-type rules.
//!
//! Development builds include the Git revision and dirty marker in `--version`.
//! The `[[kind]]` manifest is validated and rendered as native Rust data so runtime
//! classification never parses configuration or builds match structures.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const RULES_PATH: &str = "rules/file-types.toml";
const GENERATED_NAME: &str = "file_type_rules.rs";

#[derive(Default)]
struct Rule {
    id: String,
    family: String,
    extensions: Vec<String>,
    filenames: Vec<String>,
    shebangs: Vec<String>,
    priority: u16,
}

fn main() {
    emit_version();
    println!("cargo:rerun-if-changed={RULES_PATH}");
    let source = fs::read_to_string(RULES_PATH).expect("read file-type rules");
    let rules = parse_rules(&source).unwrap_or_else(|error| panic!("{RULES_PATH}: {error}"));
    validate_rules(&rules).unwrap_or_else(|error| panic!("{RULES_PATH}: {error}"));

    // Git may materialize text files with CRLF on Windows. The fingerprint describes
    // the manifest, not the checkout's line-ending convention, because reports and
    // content sidecars must remain compatible across hosts.
    let normalized_source = source.replace("\r\n", "\n").replace('\r', "\n");
    let generated = render_rules(&rules, fingerprint(normalized_source.as_bytes()));
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
            format!("{semver}-dev+g{revision}{}", if dirty { ".dirty" } else { "" })
        }
        None => semver,
    };
    println!("cargo:rustc-env=FDU_BUILD_VERSION={version}");
}

fn parse_rules(source: &str) -> Result<Vec<Rule>, String> {
    let mut rules = Vec::new();
    let mut current: Option<Rule> = None;
    for (line_index, raw) in source.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[kind]]" {
            if let Some(rule) = current.take() {
                rules.push(rule);
            }
            current = Some(Rule { priority: 100, ..Rule::default() });
            continue;
        }
        let rule = current
            .as_mut()
            .ok_or_else(|| format!("line {line_number}: field appears before [[kind]]"))?;
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {line_number}: expected key = value"))?;
        let key = key.trim();
        let value = value.trim();
        match key {
            "id" => rule.id = parse_string(value, line_number)?,
            "family" => rule.family = parse_string(value, line_number)?,
            "extensions" => rule.extensions = parse_array(value, line_number)?,
            "filenames" => rule.filenames = parse_array(value, line_number)?,
            "shebangs" => rule.shebangs = parse_array(value, line_number)?,
            "priority" => {
                rule.priority = value
                    .parse()
                    .map_err(|_| format!("line {line_number}: priority must fit u16"))?;
            }
            _ => return Err(format!("line {line_number}: unknown field {key:?}")),
        }
    }
    if let Some(rule) = current {
        rules.push(rule);
    }
    Ok(rules)
}

fn parse_string(value: &str, line: usize) -> Result<String, String> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| format!("line {line}: expected a quoted string"))
}

fn parse_array(value: &str, line: usize) -> Result<Vec<String>, String> {
    let inner = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| format!("line {line}: expected a string array"))?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner.split(',').map(|item| parse_string(item.trim(), line)).collect()
}

fn validate_rules(rules: &[Rule]) -> Result<(), String> {
    if rules.is_empty() {
        return Err("at least one [[kind]] rule is required".to_string());
    }
    let mut ids = BTreeMap::new();
    let mut extensions = BTreeMap::new();
    let mut filenames = BTreeMap::new();
    for rule in rules {
        if rule.id.is_empty()
            || !rule.id.bytes().all(|byte| byte.is_ascii_lowercase() || byte == b'-')
        {
            return Err(format!("invalid rule id {:?}", rule.id));
        }
        if !matches!(
            rule.family.as_str(),
            "code" | "prose" | "markup" | "data" | "binary" | "unknown"
        ) {
            return Err(format!("rule {} has invalid family {:?}", rule.id, rule.family));
        }
        if ids.insert(rule.id.as_str(), rule.id.as_str()).is_some() {
            return Err(format!("duplicate rule id {:?}", rule.id));
        }
        for extension in &rule.extensions {
            if extension.is_empty()
                || extension.starts_with('.')
                || !extension.is_ascii()
                || extension.bytes().any(|byte| byte.is_ascii_uppercase())
            {
                return Err(format!("rule {} has invalid extension {:?}", rule.id, extension));
            }
            insert_unique(&mut extensions, extension, rule)?;
        }
        for filename in &rule.filenames {
            if filename.is_empty()
                || !filename.is_ascii()
                || filename.contains('/')
                || filename.contains('\\')
            {
                return Err(format!("rule {} has invalid filename {:?}", rule.id, filename));
            }
            insert_unique(&mut filenames, filename, rule)?;
        }
    }
    Ok(())
}

fn insert_unique<'a>(
    values: &mut BTreeMap<&'a String, &'a str>,
    value: &'a String,
    rule: &'a Rule,
) -> Result<(), String> {
    if let Some(previous) = values.insert(value, &rule.id) {
        return Err(format!("{value:?} is assigned to both {previous} and {}", rule.id));
    }
    Ok(())
}

fn render_rules(rules: &[Rule], fingerprint: u64) -> String {
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

fn fingerprint(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    bytes.iter().fold(OFFSET, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(PRIME))
}
