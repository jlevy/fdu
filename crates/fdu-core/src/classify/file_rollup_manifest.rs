//! Dependency-free parser for the File Rollup v3 registry profile.
//!
//! The engine needs classifier and browsing semantics, not TOML as a general-purpose
//! configuration language. Keeping this parser beside the existing compact fdu manifest
//! parser admits the shared reviewed document without adding a TOML dependency to every
//! standalone fdu binary.
//!
//! **The TOML it reads.** The document is a sequence of `[[group]]`, `[[family]]`, and
//! `[[kind]]` headers and `key = value` lines, and every way TOML lets a writer spell
//! those is read as TOML defines it: a leading byte-order mark; `\n` or `\r\n` line
//! endings; comments on their own lines or after a header or value; basic strings with
//! every TOML 1.0 escape, literal strings, and the multi-line form of each; string arrays
//! spread over several lines, with comments and a trailing comma; and decimal integers
//! and floats with `_` digit separators. Forms the registry never needs are rejected with
//! an error naming the form, so nothing is ever read as a different value: single-bracket
//! tables, quoted and dotted keys, inline tables, nested arrays, and hexadecimal, octal,
//! or binary integers.
//!
//! The cursor that reads it lives in `manifest_toml`, shared with the compact `[[kind]]`
//! dialect, so both manifests accept the same TOML and refuse the same forms by name.

use super::manifest_toml::{BYTE_ORDER_MARK, Document, Value, separated_digits};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const SCHEMA_VERSION: u32 = 3;
pub(super) const MAX_EXTENSION_COMPONENTS: u8 = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Group {
    pub id: String,
    pub label: String,
    pub order: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Family {
    pub id: String,
    pub label: String,
    pub group: String,
    pub order: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Kind {
    pub id: String,
    pub family: String,
    pub group: String,
    pub content_family: String,
    pub extensions: Vec<String>,
    pub filenames: Vec<String>,
    pub shebangs: Vec<String>,
    pub priority: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Registry {
    pub schema_version: u32,
    pub revision: u32,
    pub max_extension_components: u8,
    pub groups: Vec<Group>,
    pub families: Vec<Family>,
    pub kinds: Vec<Kind>,
}

enum Block {
    Group(Group, BTreeSet<String>),
    Family(Family, BTreeSet<String>),
    Kind(Kind, BTreeSet<String>),
}

pub(super) fn looks_like_registry(source: &str) -> bool {
    let source = source.strip_prefix(BYTE_ORDER_MARK).unwrap_or(source);
    source.lines().any(|line| {
        let line = line.trim();
        let header = line.split('#').next().unwrap_or_default().bytes().filter(|byte| {
            // Only a header is compared, and TOML allows spaces inside its brackets.
            !matches!(byte, b' ' | b'\t')
        });
        line.starts_with("schema_version")
            || header.clone().eq(*b"[[group]]")
            || header.eq(*b"[[family]]")
    })
}

pub(super) fn parse(source: &str) -> Result<Registry, String> {
    let mut document = Document::new(source.strip_prefix(BYTE_ORDER_MARK).unwrap_or(source));
    let mut registry = Registry::default();
    let mut top_seen = BTreeSet::new();
    let mut block = None;
    while document.next_line() {
        let line_number = document.line;
        if document.peek() == Some(b'[') {
            let next = match document.table_header("[[group]], [[family]], or [[kind]]")? {
                "group" => Block::Group(Group::default(), BTreeSet::new()),
                "family" => Block::Family(Family::default(), BTreeSet::new()),
                "kind" => Block::Kind(Kind { priority: 100, ..Kind::default() }, BTreeSet::new()),
                name => return Err(format!("line {line_number}: unknown table [[{name}]]")),
            };
            document.end_of_line("header")?;
            close(&mut registry, block.take(), line_number)?;
            block = Some(next);
            continue;
        }
        let key = document.key()?;
        // Read the value before judging the key, so an unknown field is reported as one
        // rather than as whatever its value happens to be.
        let value = document.value();
        match block.as_mut() {
            None => {
                unique(&mut top_seen, key, line_number)?;
                match key {
                    "schema_version" => registry.schema_version = value?.integer(line_number)?,
                    "registry_revision" => registry.revision = value?.integer(line_number)?,
                    "max_extension_components" => {
                        registry.max_extension_components = value?.integer(line_number)?;
                    }
                    _ => return Err(format!("line {line_number}: unknown registry field {key:?}")),
                }
            }
            Some(Block::Group(group, seen)) => {
                unique(seen, key, line_number)?;
                match key {
                    "id" => group.id = value?.string(line_number)?,
                    "label" => group.label = value?.string(line_number)?,
                    "order" => group.order = value?.integer(line_number)?,
                    _ => return Err(format!("line {line_number}: unknown group field {key:?}")),
                }
            }
            Some(Block::Family(family, seen)) => {
                unique(seen, key, line_number)?;
                match key {
                    "id" => family.id = value?.string(line_number)?,
                    "label" => family.label = value?.string(line_number)?,
                    "group" => family.group = value?.string(line_number)?,
                    "order" => family.order = value?.integer(line_number)?,
                    // Presentation metadata is validated by shape, but is intentionally
                    // not retained by the filesystem engine.
                    "hue" => {
                        let hue = value?.finite_number(key, line_number)?;
                        if !(0.0..360.0).contains(&hue) {
                            return Err(format!(
                                "line {line_number}: hue must be in [0, 360) degrees"
                            ));
                        }
                    }
                    "lightness_rank" => {
                        let _ = value?.finite_number(key, line_number)?;
                    }
                    "linguist" | "linguist_color" | "deviation" => {
                        let _ = value?.string(line_number)?;
                    }
                    _ => return Err(format!("line {line_number}: unknown family field {key:?}")),
                }
            }
            Some(Block::Kind(kind, seen)) => {
                unique(seen, key, line_number)?;
                match key {
                    "id" => kind.id = value?.string(line_number)?,
                    "family" => kind.family = value?.string(line_number)?,
                    "group" => kind.group = value?.string(line_number)?,
                    "content_family" => kind.content_family = value?.string(line_number)?,
                    "extensions" => kind.extensions = value?.strings(line_number)?,
                    "filenames" => kind.filenames = value?.strings(line_number)?,
                    "shebangs" => kind.shebangs = value?.strings(line_number)?,
                    "priority" => kind.priority = value?.integer(line_number)?,
                    _ => return Err(format!("line {line_number}: unknown kind field {key:?}")),
                }
            }
        }
        document.end_of_line("value")?;
    }
    close(&mut registry, block.take(), source.lines().count().saturating_add(1))?;
    require_fields(
        &top_seen,
        &["schema_version", "registry_revision", "max_extension_components"],
        "registry",
    )?;
    validate(&registry)?;
    Ok(registry)
}

fn close(registry: &mut Registry, block: Option<Block>, line: usize) -> Result<(), String> {
    match block {
        Some(Block::Group(value, seen)) => {
            require_fields(&seen, &["id", "label", "order"], "group")?;
            registry.groups.push(value);
        }
        Some(Block::Family(value, seen)) => {
            require_fields(&seen, &["id", "label", "group", "order", "hue"], "family")?;
            if seen.contains("linguist") != seen.contains("linguist_color") {
                return Err(format!(
                    "line {line}: family linguist and linguist_color must be present together"
                ));
            }
            if seen.contains("lightness_rank") && !seen.contains("deviation") {
                return Err(format!("line {line}: family lightness_rank requires a deviation"));
            }
            registry.families.push(value);
        }
        Some(Block::Kind(value, seen)) => {
            require_fields(
                &seen,
                &["id", "content_family", "extensions", "filenames", "shebangs", "priority"],
                "kind",
            )?;
            registry.kinds.push(value);
        }
        None => {}
    }
    Ok(())
}

fn require_fields(seen: &BTreeSet<String>, required: &[&str], table: &str) -> Result<(), String> {
    if let Some(missing) = required.iter().find(|field| !seen.contains(**field)) {
        return Err(format!("{table} is missing required field {missing:?}"));
    }
    Ok(())
}

fn unique(seen: &mut BTreeSet<String>, key: &str, line: usize) -> Result<(), String> {
    if !seen.insert(key.to_string()) {
        return Err(format!("line {line}: duplicate field {key:?}"));
    }
    Ok(())
}

/// The registry's one float form. It is read here rather than beside the other value
/// types because the compact dialect has no float, and the shared reader is compiled
/// into that dialect's build script too, where this would be dead code.
impl Value<'_> {
    fn finite_number(self, key: &str, line: usize) -> Result<f64, String> {
        let invalid = || format!("line {line}: {key} must be a finite number");
        let Value::Bare(token) = self else {
            return Err(invalid());
        };
        let (sign, unsigned) = match token.as_bytes().first() {
            Some(b'-') => ("-", &token[1..]),
            Some(b'+') => ("", &token[1..]),
            _ => ("", token),
        };
        let (mantissa, exponent) = match unsigned.find(['e', 'E']) {
            Some(at) => (&unsigned[..at], Some(&unsigned[at + 1..])),
            None => (unsigned, None),
        };
        let (whole, fraction) = match mantissa.split_once('.') {
            Some((whole, fraction)) => (whole, Some(fraction)),
            None => (mantissa, None),
        };
        // TOML's decimal float: an integer part, then a fraction, an exponent, or both, each
        // a digit run that may use `_` separators. `inf` and `nan` are not finite.
        let mut normalized = sign.to_string();
        normalized.push_str(&separated_digits(whole, false).ok_or_else(invalid)?);
        if let Some(fraction) = fraction {
            normalized.push('.');
            normalized.push_str(&separated_digits(fraction, true).ok_or_else(invalid)?);
        }
        if let Some(exponent) = exponent {
            let (exponent_sign, digits) = match exponent.as_bytes().first() {
                Some(b'-') => ("-", &exponent[1..]),
                Some(b'+') => ("", &exponent[1..]),
                _ => ("", exponent),
            };
            normalized.push('e');
            normalized.push_str(exponent_sign);
            normalized.push_str(&separated_digits(digits, true).ok_or_else(invalid)?);
        }
        let number = normalized.parse::<f64>().map_err(|_| invalid())?;
        if !number.is_finite() {
            return Err(invalid());
        }
        Ok(number)
    }
}

fn validate(registry: &Registry) -> Result<(), String> {
    if registry.schema_version != SCHEMA_VERSION {
        return Err(format!("unsupported schema_version {}", registry.schema_version));
    }
    if registry.revision == 0 {
        return Err("registry_revision must be positive".to_string());
    }
    if registry.max_extension_components != MAX_EXTENSION_COMPONENTS {
        return Err(format!("max_extension_components must be {MAX_EXTENSION_COMPONENTS}"));
    }
    if registry.groups.is_empty() || registry.kinds.is_empty() {
        return Err("a registry requires at least one group and one kind".to_string());
    }

    let mut group_ids = BTreeSet::new();
    let mut group_orders = BTreeSet::new();
    for group in &registry.groups {
        valid_identity(&group.id, "group")?;
        if group.label.is_empty() {
            return Err(format!("group {} has no label", group.id));
        }
        if !group_ids.insert(group.id.as_str()) {
            return Err(format!("duplicate group id {:?}", group.id));
        }
        if !group_orders.insert(group.order) {
            return Err(format!("duplicate group order {}", group.order));
        }
    }
    if !group_ids.contains("other") {
        return Err("registry must declare the other group".to_string());
    }

    let mut family_groups = BTreeMap::new();
    let mut family_orders = BTreeSet::new();
    for family in &registry.families {
        valid_identity(&family.id, "family")?;
        if family.label.is_empty() {
            return Err(format!("family {} has no label", family.id));
        }
        if !group_ids.contains(family.group.as_str()) {
            return Err(format!("family {} names unknown group {:?}", family.id, family.group));
        }
        if family_groups.insert(family.id.as_str(), family.group.as_str()).is_some() {
            return Err(format!("duplicate family id {:?}", family.id));
        }
        if !family_orders.insert((family.group.as_str(), family.order)) {
            return Err(format!(
                "duplicate family order {} in group {:?}",
                family.order, family.group
            ));
        }
    }

    let mut kind_ids = BTreeSet::new();
    let mut extensions = BTreeMap::new();
    let mut filenames = BTreeMap::new();
    for kind in &registry.kinds {
        valid_identity(&kind.id, "kind")?;
        if !kind_ids.insert(kind.id.as_str()) {
            return Err(format!("duplicate kind id {:?}", kind.id));
        }
        let group = if kind.family.is_empty() {
            if kind.group.is_empty() {
                return Err(format!("kind {} names neither family nor group", kind.id));
            }
            kind.group.as_str()
        } else {
            let family_group = family_groups.get(kind.family.as_str()).ok_or_else(|| {
                format!("kind {} names unknown family {:?}", kind.id, kind.family)
            })?;
            if !kind.group.is_empty() && kind.group != *family_group {
                return Err(format!("kind {} group conflicts with family", kind.id));
            }
            family_group
        };
        if !group_ids.contains(group) {
            return Err(format!("kind {} names unknown group {group:?}", kind.id));
        }
        if !super::type_rule_manifest::MANIFEST_FAMILIES.contains(&kind.content_family.as_str()) {
            return Err(format!(
                "kind {} has invalid content_family {:?}",
                kind.id, kind.content_family
            ));
        }
        if kind.extensions.is_empty() && kind.filenames.is_empty() && kind.shebangs.is_empty() {
            return Err(format!("kind {} declares no evidence", kind.id));
        }
        for extension in &kind.extensions {
            let components: Vec<_> = extension.split('.').collect();
            if extension.starts_with('.')
                || extension != &extension.to_ascii_lowercase()
                || components.is_empty()
                || components.len() > usize::from(MAX_EXTENSION_COMPONENTS)
                || components.iter().any(|component| {
                    component.is_empty()
                        || component.len() > 12
                        || !component
                            .bytes()
                            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
                })
            {
                return Err(format!("kind {} has invalid extension {:?}", kind.id, extension));
            }
            if let Some(previous) = extensions.insert(extension.as_str(), kind.id.as_str()) {
                return Err(format!(
                    "extension {extension:?} belongs to {previous} and {}",
                    kind.id
                ));
            }
        }
        for filename in &kind.filenames {
            if filename.is_empty()
                || filename != &filename.to_ascii_lowercase()
                || filename.contains('/')
                || filename.contains('\\')
            {
                return Err(format!("kind {} has invalid filename {:?}", kind.id, filename));
            }
            if let Some(previous) = filenames.insert(filename.as_str(), kind.id.as_str()) {
                return Err(format!("filename {filename:?} belongs to {previous} and {}", kind.id));
            }
        }
    }
    for family in &registry.families {
        if !registry
            .kinds
            .iter()
            .any(|kind| kind.family == family.id && !kind.extensions.is_empty())
        {
            return Err(format!("family {} has no declared extension evidence", family.id));
        }
    }
    Ok(())
}

fn valid_identity(value: &str, kind: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || !value.as_bytes()[0].is_ascii_lowercase()
    {
        return Err(format!("invalid {kind} id {value:?}"));
    }
    Ok(())
}

/// Identity of a validated registry's semantic values.
///
/// Every value is length-prefixed, and so is every sequence of them: each array and each
/// section hashes its count before its entries. Without the counts, the byte stream
/// cannot tell where one array ends and the next begins, so moving a key from
/// `extensions` to `filenames` -- which changes what files classify as -- left the
/// identity unchanged, and snapshots recorded under the old registry still matched.
pub(super) fn fingerprint(registry: &Registry) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    fn add(hash: &mut u64, value: &[u8]) {
        let length = u64::try_from(value.len()).unwrap_or(u64::MAX);
        for byte in length.to_le_bytes().iter().chain(value) {
            *hash = (*hash ^ u64::from(*byte)).wrapping_mul(PRIME);
        }
    }
    fn count(hash: &mut u64, items: usize) {
        add(hash, &u64::try_from(items).unwrap_or(u64::MAX).to_le_bytes());
    }
    fn values(hash: &mut u64, items: &[String]) {
        count(hash, items.len());
        for item in items {
            add(hash, item.as_bytes());
        }
    }
    let mut hash = OFFSET;
    add(&mut hash, b"file-rollup-registry-v3");
    add(&mut hash, &registry.revision.to_le_bytes());
    count(&mut hash, registry.groups.len());
    for group in &registry.groups {
        add(&mut hash, group.id.as_bytes());
        add(&mut hash, group.label.as_bytes());
        add(&mut hash, &group.order.to_le_bytes());
    }
    count(&mut hash, registry.families.len());
    for family in &registry.families {
        add(&mut hash, family.id.as_bytes());
        add(&mut hash, family.label.as_bytes());
        add(&mut hash, family.group.as_bytes());
        add(&mut hash, &family.order.to_le_bytes());
    }
    count(&mut hash, registry.kinds.len());
    for kind in &registry.kinds {
        add(&mut hash, kind.id.as_bytes());
        add(&mut hash, kind.family.as_bytes());
        add(&mut hash, kind.group.as_bytes());
        add(&mut hash, kind.content_family.as_bytes());
        values(&mut hash, &kind.extensions);
        values(&mut hash, &kind.filenames);
        values(&mut hash, &kind.shebangs);
        add(&mut hash, &kind.priority.to_le_bytes());
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The smallest valid registry, in the plainest spelling.
    const PLAIN: &str = r#"schema_version = 3
registry_revision = 1
max_extension_components = 2

[[group]]
id = "other"
label = "Other"
order = 10

[[family]]
id = "notes"
label = "Notes"
group = "other"
order = 1
hue = 120.5

[[kind]]
id = "notes"
family = "notes"
content_family = "prose"
extensions = ["md", "txt"]
filenames = ["readme"]
shebangs = []
priority = 100
"#;

    fn plain() -> Registry {
        parse(PLAIN).expect("the plain registry parses")
    }

    /// `PLAIN` with each `(from, to)` replaced once, so a case says only what it changes.
    fn spelled(replacements: &[(&str, &str)]) -> String {
        replacements.iter().fold(PLAIN.to_string(), |source, (from, to)| {
            assert_eq!(source.matches(from).count(), 1, "{from:?} names one place in PLAIN");
            source.replacen(from, to, 1)
        })
    }

    #[test]
    fn each_toml_spelling_of_the_registry_reads_as_the_plain_one() {
        let cases: &[(&str, String)] = &[
            ("a byte-order mark", format!("{BYTE_ORDER_MARK}{PLAIN}")),
            ("CRLF line endings", PLAIN.replace('\n', "\r\n")),
            ("no final line ending", PLAIN.trim_end().to_string()),
            (
                "comments after headers and values",
                spelled(&[
                    ("[[group]]", "[[group]] # the fallback group"),
                    ("label = \"Other\"", "label = \"Other\" # \"not\" ] part of it"),
                    ("order = 10", "order = 10 # ten"),
                    ("hue = 120.5", "hue = 120.5# no space before the comment"),
                    ("extensions = [\"md\", \"txt\"]", "extensions = [\"md\", \"txt\"] # ]"),
                ]),
            ),
            ("spaces inside a header", spelled(&[("[[kind]]", "[[ kind ]]")])),
            (
                "an array over several lines",
                spelled(&[(
                    "extensions = [\"md\", \"txt\"]",
                    "extensions = [\n  \"md\", # Markdown\n\n  \"txt\", # trailing comma\n]",
                )]),
            ),
            (
                "literal strings",
                spelled(&[
                    ("label = \"Other\"", "label = 'Other'"),
                    ("extensions = [\"md\", \"txt\"]", "extensions = ['md', \"txt\"]"),
                ]),
            ),
            (
                "multi-line basic strings",
                spelled(&[
                    ("label = \"Other\"", "label = \"\"\"\nOther\"\"\""),
                    ("label = \"Notes\"", "label = \"\"\"No\\\n\n      tes\"\"\""),
                ]),
            ),
            (
                "a multi-line literal string",
                spelled(&[("label = \"Other\"", "label = '''Other'''")]),
            ),
            (
                "unicode escapes",
                spelled(&[
                    ("label = \"Other\"", "label = \"\\u004Fther\""),
                    ("id = \"notes\"\nlabel", "id = \"n\\U0000006Ftes\"\nlabel"),
                ]),
            ),
            (
                "digit separators, a sign, and an exponent",
                spelled(&[
                    ("order = 10", "order = 1_0"),
                    ("priority = 100", "priority = +100"),
                    ("hue = 120.5", "hue = 1.20_5e+2"),
                ]),
            ),
            (
                "presentation fields as the shared registry writes them",
                spelled(&[(
                    "hue = 120.5",
                    "hue = 120.5\nlinguist = \"Text\"\nlinguist_color = \"#aabbcc\"\n\
                     lightness_rank = 3\ndeviation = \"\"\"Moved from the upstream hue, \\\n\
                     which sat against \"another\" family's.\"\"\"",
                )]),
            ),
        ];
        for (form, source) in cases {
            assert_eq!(parse(source).as_ref(), Ok(&plain()), "{form}");
            assert!(looks_like_registry(source), "{form} is recognized as a registry");
        }
    }

    #[test]
    fn strings_decode_escapes_and_literal_strings_keep_backslashes() {
        let cases = [
            (r#""say \"hi\" \\ \t \u00e9 \U0001F600""#, "say \"hi\" \\ \t \u{e9} \u{1f600}"),
            (r#""\b\f\n\r""#, "\u{8}\u{c}\n\r"),
            (r#""C# notes""#, "C# notes"),
            (r#"'C:\dir\"x'"#, r#"C:\dir\"x"#),
            (r#""""quoted ""end""""""#, r#"quoted ""end"""#),
            (r"'''it's ''quoted'''''", r"it's ''quoted''"),
        ];
        for (written, label) in cases {
            let line = format!("label = {written}");
            let source = spelled(&[("label = \"Other\"", line.as_str())]);
            let registry = parse(&source).unwrap_or_else(|error| panic!("{written}: {error}"));
            assert_eq!(registry.groups[0].label, label, "{written}");
        }
    }

    #[test]
    fn forms_the_registry_does_not_need_are_rejected_by_name() {
        let cases = [
            (("[[group]]", "[group]"), "single-bracket tables are not supported"),
            (("[[group]]", "[[widget]]"), "unknown table [[widget]]"),
            (("[[group]]", "[[group]] id = \"other\""), "unexpected text after the header"),
            (("[[group]]", "[[group.sub]]"), "dotted keys are not supported"),
            (("id = \"other\"", "\"id\" = \"other\""), "quoted keys are not supported"),
            (("label = \"Other\"", "label.text = \"Other\""), "dotted keys are not supported"),
            (
                ("label = \"Other\"", "label = { text = \"Other\" }"),
                "inline tables are not supported",
            ),
            (
                ("extensions = [\"md\", \"txt\"]", "extensions = [[\"md\"], \"txt\"]"),
                "nested arrays",
            ),
            (
                ("extensions = [\"md\", \"txt\"]", "extensions = [\"md\" \"txt\"]"),
                "expected , or ]",
            ),
            (
                ("extensions = [\"md\", \"txt\"]", "extensions = [\"md\", 1]"),
                "expected a string array",
            ),
            (("order = 10", "order = 0x0A"), "hexadecimal, octal, and binary integers"),
            (("order = 10", "order = 010"), "expected a nonnegative integer"),
            (("order = 10", "order = 1__0"), "expected a nonnegative integer"),
            (("hue = 120.5", "hue = nan"), "hue must be a finite number"),
            (("hue = 120.5", "hue = .5"), "hue must be a finite number"),
            (("label = \"Other\"", "label = \"Oth\\qer\""), "invalid escape \\q"),
            (("label = \"Other\"", "label = \"\\uD800\""), "\\uD800 is not a Unicode scalar value"),
            (("label = \"Other\"", "label = \"\\u12\""), "\\u needs 4 hexadecimal digits"),
            (("label = \"Other\"", "label = \"Other"), "unterminated string"),
            (("label = \"Other\"", "label = \"Oth\u{1}er\""), "control characters in strings"),
            (
                ("label = \"Other\"", "label = \"Other\" \"again\""),
                "unexpected text after the value",
            ),
            (("label = \"Other\"", "label = \"\"\"never closed"), "unterminated string"),
        ];
        for ((from, to), message) in cases {
            let error = parse(&spelled(&[(from, to)])).expect_err(to);
            assert!(error.contains(message), "{to:?} gave {error:?}, not {message:?}");
        }
        let error = parse(&PLAIN.replace("shebangs = []\npriority = 100\n", "shebangs = [\n"))
            .expect_err("an array open at the end of the document");
        assert!(error.contains("unterminated array"), "{error}");
    }
}
