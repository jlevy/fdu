// NOTE: `//!` is impossible here -- `build.rs` includes this file into a module of its
// own, where an inner doc comment is a parse error. The module's rustdoc lives on the
// owning `classify` module.
// The `[[kind]]` manifest dialect, parsed and validated by one implementation.
//
// This file is compiled into the crate *and* `include!`d by `build.rs`, so the rules a
// caller supplies at run time are read by exactly the code that read the repository's
// own manifest at build time. Two parsers for one dialect is how a manifest comes to
// mean one thing to the compiler and another to a consumer, and neither is wrong on its
// own terms.
//
// Being shared with a build script constrains it: nothing from `crate::`, because a
// build script has no crate to reach into, and nothing from `super::` but the sibling
// `manifest_toml`, which `build.rs` includes beside it under the same name. The dialect's
// own vocabulary (`family` as a string, for one) survives here for the same reason —
// mapping it onto engine types is the caller's job on either side.
//
// The dialect is deliberately a subset of TOML rather than TOML: `[[kind]]` tables of
// strings, string arrays, and one integer. A real TOML parser would accept more than the
// dialect means and cost a dependency on the engine's always-on list. The TOML it is
// written in is read by `manifest_toml`, the same cursor that reads the File Rollup
// registry, so a comment after a value, a comma inside a string, or an escaped quote
// means here what it means there.

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
    let source = source.strip_prefix(super::manifest_toml::BYTE_ORDER_MARK).unwrap_or(source);
    let mut document = super::manifest_toml::Document::new(source);
    let mut rules = Vec::new();
    let mut current: Option<ManifestRule> = None;
    let mut seen_fields = 0_u8;
    while document.next_line() {
        let line_number = document.line;
        if document.peek() == Some(b'[') {
            let name = document.table_header("[[kind]]")?;
            if name != "kind" {
                return Err(format!("line {line_number}: unknown table [[{name}]]"));
            }
            document.end_of_line("header")?;
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
        let key = document.key()?;
        // Read the value before judging the key, so an unknown field is reported as one
        // rather than as whatever its value happens to be.
        let value = document.value();
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
            "id" => rule.id = value?.string(line_number)?,
            "family" => rule.family = value?.string(line_number)?,
            "extensions" => rule.extensions = value?.strings(line_number)?,
            "filenames" => rule.filenames = value?.strings(line_number)?,
            "shebangs" => rule.shebangs = value?.strings(line_number)?,
            "priority" => rule.priority = value?.integer(line_number)?,
            _ => unreachable!("field name was resolved above"),
        }
        document.end_of_line("value")?;
    }
    if let Some(rule) = current {
        rules.push(rule);
    }
    Ok(rules)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The smallest manifest that sets every field, in the plainest spelling.
    const PLAIN: &str = r#"[[kind]]
id = "notes"
family = "prose"
extensions = ["md", "txt"]
filenames = ["readme"]
shebangs = []
priority = 90
"#;

    fn plain() -> Vec<ManifestRule> {
        parse_manifest(PLAIN).expect("the plain manifest parses")
    }

    /// `PLAIN` with each `(from, to)` replaced once, so a case says only what it changes.
    fn spelled(replacements: &[(&str, &str)]) -> String {
        replacements.iter().fold(PLAIN.to_string(), |source, (from, to)| {
            assert_eq!(source.matches(from).count(), 1, "{from:?} names one place in PLAIN");
            source.replacen(from, to, 1)
        })
    }

    /// The same form table as the File Rollup registry's, because one cursor reads both.
    #[test]
    fn each_toml_spelling_of_a_manifest_reads_as_the_plain_one() {
        let cases: &[(&str, String)] = &[
            ("a byte-order mark", format!("\u{feff}{PLAIN}")),
            ("CRLF line endings", PLAIN.replace('\n', "\r\n")),
            ("no final line ending", PLAIN.trim_end().to_string()),
            (
                "comments after headers and values",
                spelled(&[
                    ("[[kind]]", "[[kind]] # a rule"),
                    ("id = \"notes\"", "id = \"notes\" # \"x\""),
                    ("extensions = [\"md\", \"txt\"]", "extensions = [\"md\", \"txt\"] # ]"),
                    ("priority = 90", "priority = 90# no space before the comment"),
                ]),
            ),
            ("spaces inside a header", spelled(&[("[[kind]]", "[[ kind ]]")])),
            (
                "arrays over several lines",
                spelled(&[
                    (
                        "extensions = [\"md\", \"txt\"]",
                        "extensions = [\n  \"md\", # Markdown\n\n  \"txt\", # trailing comma\n]",
                    ),
                    ("shebangs = []", "shebangs = [\n  # none yet\n]"),
                ]),
            ),
            (
                "literal strings",
                spelled(&[
                    ("family = \"prose\"", "family = 'prose'"),
                    ("extensions = [\"md\", \"txt\"]", "extensions = ['md', \"txt\"]"),
                ]),
            ),
            (
                "multi-line basic strings",
                spelled(&[
                    ("id = \"notes\"", "id = \"\"\"\nnotes\"\"\""),
                    ("family = \"prose\"", "family = \"\"\"pro\\\n\n      se\"\"\""),
                ]),
            ),
            (
                "a multi-line literal string",
                spelled(&[("family = \"prose\"", "family = '''prose'''")]),
            ),
            (
                "unicode escapes",
                spelled(&[
                    ("id = \"notes\"", "id = \"n\\u006Ftes\""),
                    ("family = \"prose\"", "family = \"\\U00000070rose\""),
                ]),
            ),
            ("digit separators and a sign", spelled(&[("priority = 90", "priority = +9_0")])),
        ];
        for (form, source) in cases {
            assert_eq!(parse_manifest(source).as_ref(), Ok(&plain()), "{form}");
        }
    }

    /// Each value is one TOML string: a comma or bracket inside it is not a separator, and
    /// an escape is decoded rather than kept.
    #[test]
    fn strings_decode_escapes_and_keep_their_separators() {
        let cases = [
            (r#""say \"hi\" \\ \t \u00e9 \U0001F600""#, "say \"hi\" \\ \t \u{e9} \u{1f600}"),
            (r#""\b\f\n\r""#, "\u{8}\u{c}\n\r"),
            (r#""C# notes""#, "C# notes"),
            (r#""a,b""#, "a,b"),
            (r#""]""#, "]"),
            (r#"'C:\dir\"x'"#, r#"C:\dir\"x"#),
            (r#""""quoted ""end""""""#, r#"quoted ""end"""#),
            (r"'''it's ''quoted'''''", r"it's ''quoted''"),
        ];
        for (written, shebang) in cases {
            let line = format!("shebangs = [{written}]");
            let source = spelled(&[("shebangs = []", line.as_str())]);
            let rules =
                parse_manifest(&source).unwrap_or_else(|error| panic!("{written}: {error}"));
            assert_eq!(rules[0].shebangs, [shebang], "{written}");
        }
        let rules = parse_manifest(&spelled(&[("shebangs = []", "shebangs = [\"a,b\", 'c']")]))
            .expect("two items, one with a comma");
        assert_eq!(rules[0].shebangs, ["a,b", "c"]);
    }

    #[test]
    fn forms_the_manifest_does_not_need_are_rejected_by_name() {
        let cases = [
            (("[[kind]]", "[kind]"), "single-bracket tables are not supported; use [[kind]]"),
            (("[[kind]]", "[[widget]]"), "unknown table [[widget]]"),
            (("[[kind]]", "[[kind]] id = \"notes\""), "unexpected text after the header"),
            (("[[kind]]", "[[kind.sub]]"), "dotted keys are not supported"),
            (("id = \"notes\"", "\"id\" = \"notes\""), "quoted keys are not supported"),
            (("family = \"prose\"", "family.name = \"prose\""), "dotted keys are not supported"),
            (
                ("family = \"prose\"", "family = { name = \"prose\" }"),
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
            (("extensions = [\"md\", \"txt\"]", "extensions = \"md\""), "expected a string array"),
            (("id = \"notes\"", "id = notes"), "expected a quoted string"),
            (("id = \"notes\"", "id ="), "expected a value"),
            (("id = \"notes\"", "name = \"notes\""), "unknown field \"name\""),
            (("priority = 90", "priority = 90\npriority = 80"), "duplicate field \"priority\""),
            (("priority = 90", "priority = 0x5A"), "hexadecimal, octal, and binary integers"),
            (("priority = 90", "priority = 090"), "expected a nonnegative integer"),
            (("priority = 90", "priority = 9__0"), "expected a nonnegative integer"),
            (("priority = 90", "priority = -90"), "expected a nonnegative integer"),
            (("priority = 90", "priority = \"90\""), "expected a nonnegative integer"),
            (("priority = 90", "priority = 70000"), "70000 is out of range for this field"),
            (("family = \"prose\"", "family = \"pro\\qse\""), "invalid escape \\q"),
            (
                ("family = \"prose\"", "family = \"\\uD800\""),
                "\\uD800 is not a Unicode scalar value",
            ),
            (("family = \"prose\"", "family = \"\\u12\""), "\\u needs 4 hexadecimal digits"),
            (("family = \"prose\"", "family = \"prose"), "unterminated string"),
            (("family = \"prose\"", "family = \"pro\u{1}se\""), "control characters in strings"),
            (
                ("family = \"prose\"", "family = \"prose\" \"again\""),
                "unexpected text after the value",
            ),
            (("family = \"prose\"", "family = \"\"\"never closed"), "unterminated string"),
        ];
        for ((from, to), message) in cases {
            let error = parse_manifest(&spelled(&[(from, to)])).expect_err(to);
            assert!(error.contains(message), "{to:?} gave {error:?}, not {message:?}");
        }
        let error =
            parse_manifest(&PLAIN.replace("shebangs = []\npriority = 90\n", "shebangs = [\n"))
                .expect_err("an array open at the end of the document");
        assert!(error.contains("unterminated array"), "{error}");
        let error = parse_manifest(&format!("id = \"early\"\n{PLAIN}")).expect_err("no rule yet");
        assert!(error.contains("line 1: field appears before [[kind]]"), "{error}");
    }

    /// An error names the line it is on, not the line its value started on.
    #[test]
    fn an_error_inside_a_multi_line_array_names_its_own_line() {
        let source = spelled(&[("shebangs = []", "shebangs = [\n  \"sh\"\n  \"bash\"\n]")]);
        let error = parse_manifest(&source).expect_err("a missing comma");
        assert!(error.starts_with("line 8: expected , or ]"), "{error}");
    }
}
