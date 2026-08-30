// NOTE: `//!` is impossible here -- `build.rs` includes this file into the middle of
// its own module, where an inner doc comment is a parse error. The module's rustdoc
// lives on the owning `classify` module.
// The `[[kind]]` manifest dialect, parsed and validated by one implementation.
//
// This file is compiled into the crate *and* `include!`d by `build.rs`, so the rules a
// caller supplies at run time are read by exactly the code that read the repository's
// own manifest at build time. Two parsers for one dialect is how a manifest comes to
// mean one thing to the compiler and another to a consumer, and neither is wrong on its
// own terms.
//
// Being shared with a build script constrains it: no `use` statements, because
// `build.rs` has its own and a duplicate import is an error; and nothing from `crate::`,
// because a build script has no crate to reach into. The dialect's own vocabulary
// (`family` as a string, for one) survives here for the same reason — mapping it onto
// engine types is the caller's job on either side.
//
// The dialect is deliberately a subset of TOML rather than TOML: `[[kind]]` tables of
// quoted strings, string arrays, and one integer. A real TOML parser would accept more
// than the dialect means and cost a dependency on the engine's always-on list.

/// Default tie-break weight when a rule omits `priority`.
const DEFAULT_MANIFEST_PRIORITY: u16 = 100;

/// Field-presence bits used to reject duplicate assignments within one rule.
const FIELD_ID: u8 = 1 << 0;
const FIELD_FAMILY: u8 = 1 << 1;
const FIELD_EXTENSIONS: u8 = 1 << 2;
const FIELD_FILENAMES: u8 = 1 << 3;
const FIELD_SHEBANGS: u8 = 1 << 4;
const FIELD_PRIORITY: u8 = 1 << 5;

/// Domain separator for the semantic registry fingerprint format.
const FINGERPRINT_DOMAIN: &[u8] = b"fdu-type-rules-v1";

/// One `[[kind]]` block, still in the manifest's own vocabulary.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ManifestRule {
    /// Stable machine identifier, lowercase ASCII with hyphens.
    pub id: String,
    /// Analyzer family name, validated against the engine's closed set.
    pub family: String,
    /// Extensions this rule claims, without a leading dot.
    pub extensions: Vec<String>,
    /// Exact basenames this rule claims.
    pub filenames: Vec<String>,
    /// Shebang interpreters this rule claims.
    pub shebangs: Vec<String>,
    /// Tie-break weight for overlapping content evidence such as shebangs.
    pub priority: u16,
}

/// The family names the engine's closed analyzer set admits.
pub(crate) const MANIFEST_FAMILIES: &[&str] =
    &["code", "prose", "markup", "data", "binary", "unknown"];

/// Read every `[[kind]]` block in `source`.
///
/// Errors name a line number: a manifest is something a person edits, and "expected a
/// quoted string" without a location is a worse message than no message.
pub(crate) fn parse_manifest(source: &str) -> Result<Vec<ManifestRule>, String> {
    let mut rules = Vec::new();
    let mut current: Option<ManifestRule> = None;
    let mut seen_fields = 0_u8;
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
            current = Some(ManifestRule {
                priority: DEFAULT_MANIFEST_PRIORITY,
                ..ManifestRule::default()
            });
            seen_fields = 0;
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
        let field = match key {
            "id" => FIELD_ID,
            "family" => FIELD_FAMILY,
            "extensions" => FIELD_EXTENSIONS,
            "filenames" => FIELD_FILENAMES,
            "shebangs" => FIELD_SHEBANGS,
            "priority" => FIELD_PRIORITY,
            _ => return Err(format!("line {line_number}: unknown field {key:?}")),
        };
        if seen_fields & field != 0 {
            return Err(format!("line {line_number}: duplicate field {key:?}"));
        }
        seen_fields |= field;
        match key {
            "id" => rule.id = parse_manifest_string(value, line_number)?,
            "family" => rule.family = parse_manifest_string(value, line_number)?,
            "extensions" => rule.extensions = parse_manifest_array(value, line_number)?,
            "filenames" => rule.filenames = parse_manifest_array(value, line_number)?,
            "shebangs" => rule.shebangs = parse_manifest_array(value, line_number)?,
            "priority" => {
                rule.priority = value
                    .parse()
                    .map_err(|_| format!("line {line_number}: priority must fit u16"))?;
            }
            _ => unreachable!("field name was resolved above"),
        }
    }
    if let Some(rule) = current {
        rules.push(rule);
    }
    Ok(rules)
}

fn parse_manifest_string(value: &str, line: usize) -> Result<String, String> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| format!("line {line}: expected a quoted string"))
}

fn parse_manifest_array(value: &str, line: usize) -> Result<Vec<String>, String> {
    let inner = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| format!("line {line}: expected a string array"))?;
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner.split(',').map(|item| parse_manifest_string(item.trim(), line)).collect()
}

/// Reject a manifest that would classify ambiguously or name something the engine cannot.
///
/// Tested for what it rejects rather than what it accepts. An exact name or extension
/// claimed by two rules is rejected even when priorities differ: authors should see and
/// remove the collision instead of relying on distant rule order or weights.
pub(crate) fn validate_manifest(rules: &[ManifestRule]) -> Result<(), String> {
    if rules.is_empty() {
        return Err("at least one [[kind]] rule is required".to_string());
    }
    let mut ids = std::collections::BTreeMap::new();
    let mut extensions = std::collections::BTreeMap::new();
    let mut filenames = std::collections::BTreeMap::new();
    for rule in rules {
        if rule.id.is_empty()
            || !rule.id.bytes().all(|byte| byte.is_ascii_lowercase() || byte == b'-')
        {
            return Err(format!("invalid rule id {:?}", rule.id));
        }
        if !MANIFEST_FAMILIES.contains(&rule.family.as_str()) {
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
            insert_unique_key(&mut extensions, extension, rule)?;
        }
        for filename in &rule.filenames {
            if filename.is_empty()
                || !filename.is_ascii()
                || filename.contains('/')
                || filename.contains('\\')
            {
                return Err(format!("rule {} has invalid filename {:?}", rule.id, filename));
            }
            insert_unique_key(&mut filenames, filename, rule)?;
        }
    }
    Ok(())
}

fn insert_unique_key<'a>(
    values: &mut std::collections::BTreeMap<&'a String, &'a str>,
    value: &'a String,
    rule: &'a ManifestRule,
) -> Result<(), String> {
    if let Some(previous) = values.insert(value, &rule.id) {
        return Err(format!("{value:?} is assigned to both {previous} and {}", rule.id));
    }
    Ok(())
}

/// Identity of a validated manifest's ordered semantic values.
///
/// Presentation details such as comments, spacing, and line endings are excluded. Rule
/// and array order remain significant because equal-priority shebang matches use manifest
/// order as their deterministic tie-break. Length-prefixing every value prevents adjacent
/// strings from producing the same byte stream.
pub(crate) fn manifest_fingerprint(rules: &[ManifestRule]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn bytes(hash: &mut u64, value: &[u8]) {
        let len = u64::try_from(value.len()).expect("manifest values fit in u64");
        for byte in len.to_le_bytes().iter().chain(value) {
            *hash = (*hash ^ u64::from(*byte)).wrapping_mul(PRIME);
        }
    }

    fn strings(hash: &mut u64, values: &[String]) {
        let len = u64::try_from(values.len()).expect("manifest arrays fit in u64");
        bytes(hash, &len.to_le_bytes());
        for value in values {
            bytes(hash, value.as_bytes());
        }
    }

    let mut hash = OFFSET;
    bytes(&mut hash, FINGERPRINT_DOMAIN);
    let rule_count = u64::try_from(rules.len()).expect("manifest rule count fits in u64");
    bytes(&mut hash, &rule_count.to_le_bytes());
    for rule in rules {
        bytes(&mut hash, rule.id.as_bytes());
        bytes(&mut hash, rule.family.as_bytes());
        strings(&mut hash, &rule.extensions);
        strings(&mut hash, &rule.filenames);
        strings(&mut hash, &rule.shebangs);
        bytes(&mut hash, &rule.priority.to_le_bytes());
    }
    hash
}
