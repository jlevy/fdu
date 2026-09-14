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

const BYTE_ORDER_MARK: char = '\u{feff}';

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
            let header = document.table_header()?;
            document.end_of_line("header")?;
            close(&mut registry, block.take(), line_number)?;
            block = Some(match header {
                Header::Group => Block::Group(Group::default(), BTreeSet::new()),
                Header::Family => Block::Family(Family::default(), BTreeSet::new()),
                Header::Kind => {
                    Block::Kind(Kind { priority: 100, ..Kind::default() }, BTreeSet::new())
                }
            });
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

enum Header {
    Group,
    Family,
    Kind,
}

/// One value as written, before a field says which type it must have.
enum Value<'a> {
    String(String),
    Strings(Vec<String>),
    /// An unquoted token: a number here, or a boolean or date no field accepts.
    Bare(&'a str),
}

impl Value<'_> {
    fn string(self, line: usize) -> Result<String, String> {
        match self {
            Value::String(value) => Ok(value),
            Value::Strings(_) | Value::Bare(_) => {
                Err(format!("line {line}: expected a quoted string"))
            }
        }
    }

    fn strings(self, line: usize) -> Result<Vec<String>, String> {
        match self {
            Value::Strings(values) => Ok(values),
            Value::String(_) | Value::Bare(_) => {
                Err(format!("line {line}: expected a string array"))
            }
        }
    }

    fn integer<T: std::str::FromStr>(self, line: usize) -> Result<T, String> {
        let invalid = || format!("line {line}: expected a nonnegative integer");
        let Value::Bare(token) = self else {
            return Err(invalid());
        };
        if ["0x", "0o", "0b"].iter().any(|prefix| token.starts_with(prefix)) {
            return Err(format!(
                "line {line}: hexadecimal, octal, and binary integers are not supported"
            ));
        }
        let unsigned = token.strip_prefix('+').unwrap_or(token);
        separated_digits(unsigned, false).ok_or_else(invalid)?.parse().map_err(|_| invalid())
    }

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

/// A TOML digit run: ASCII digits with single `_` separators between them, and no leading
/// zero unless `leading_zero` allows one. Returns the digits without separators.
fn separated_digits(text: &str, leading_zero: bool) -> Option<String> {
    let bytes = text.as_bytes();
    let valid = bytes.first().is_some_and(u8::is_ascii_digit)
        && bytes.last().is_some_and(u8::is_ascii_digit)
        && bytes.iter().all(|byte| byte.is_ascii_digit() || *byte == b'_')
        && !text.contains("__")
        && (leading_zero || bytes.len() == 1 || bytes[0] != b'0');
    valid.then(|| text.replace('_', ""))
}

/// A cursor over the registry document that knows the line it is on.
///
/// Every syntax byte is ASCII, so the cursor advances over syntax a byte at a time and
/// over string contents a character at a time, and always rests on a character boundary.
struct Document<'a> {
    source: &'a str,
    at: usize,
    line: usize,
}

impl<'a> Document<'a> {
    const fn new(source: &'a str) -> Self {
        Self { source, at: 0, line: 1 }
    }

    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.at).copied()
    }

    fn starts_with(&self, text: &str) -> bool {
        self.source.as_bytes()[self.at..].starts_with(text.as_bytes())
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t')) {
            self.at += 1;
        }
    }

    fn skip_comment(&mut self) {
        if self.peek() == Some(b'#') {
            while !matches!(self.peek(), None | Some(b'\n')) && !self.starts_with("\r\n") {
                self.at += 1;
            }
        }
    }

    /// Consume one line ending, if the cursor is at one.
    fn newline(&mut self) -> bool {
        let width = if self.peek() == Some(b'\n') {
            1
        } else if self.starts_with("\r\n") {
            2
        } else {
            return false;
        };
        self.at += width;
        self.line += 1;
        true
    }

    /// Skip whitespace, comments, and line endings, as between array elements.
    fn skip_blank(&mut self) {
        loop {
            self.skip_whitespace();
            self.skip_comment();
            if !self.newline() {
                return;
            }
        }
    }

    /// Move to the start of the next header or key, or report the end of the document.
    fn next_line(&mut self) -> bool {
        self.skip_blank();
        self.peek().is_some()
    }

    /// After a header or value only whitespace and a comment may precede the line end.
    fn end_of_line(&mut self, what: &str) -> Result<(), String> {
        self.skip_whitespace();
        self.skip_comment();
        if self.newline() || self.peek().is_none() {
            Ok(())
        } else {
            Err(format!("line {}: unexpected text after the {what}", self.line))
        }
    }

    fn table_header(&mut self) -> Result<Header, String> {
        let line = self.line;
        if !self.starts_with("[[") {
            return Err(format!(
                "line {line}: single-bracket tables are not supported; \
                 use [[group]], [[family]], or [[kind]]"
            ));
        }
        self.at += 2;
        self.skip_whitespace();
        let name = self.bare_key(line)?;
        self.skip_whitespace();
        if self.peek() == Some(b'.') {
            return Err(format!("line {line}: dotted keys are not supported"));
        }
        if !self.starts_with("]]") {
            return Err(format!("line {line}: expected ]] to close the table header"));
        }
        self.at += 2;
        match name {
            "group" => Ok(Header::Group),
            "family" => Ok(Header::Family),
            "kind" => Ok(Header::Kind),
            _ => Err(format!("line {line}: unknown table [[{name}]]")),
        }
    }

    fn bare_key(&mut self, line: usize) -> Result<&'a str, String> {
        let start = self.at;
        while matches!(self.peek(), Some(b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-')) {
            self.at += 1;
        }
        if self.at == start {
            return Err(match self.peek() {
                Some(b'"' | b'\'') => format!("line {line}: quoted keys are not supported"),
                _ => format!("line {line}: expected key = value"),
            });
        }
        Ok(&self.source[start..self.at])
    }

    fn key(&mut self) -> Result<&'a str, String> {
        let line = self.line;
        let key = self.bare_key(line)?;
        self.skip_whitespace();
        match self.peek() {
            Some(b'=') => {
                self.at += 1;
                self.skip_whitespace();
                Ok(key)
            }
            Some(b'.') => Err(format!("line {line}: dotted keys are not supported")),
            _ => Err(format!("line {line}: expected key = value")),
        }
    }

    fn value(&mut self) -> Result<Value<'a>, String> {
        let line = self.line;
        match self.peek() {
            Some(b'"' | b'\'') => self.string().map(Value::String),
            Some(b'[') => self.string_array().map(Value::Strings),
            Some(b'{') => Err(format!("line {line}: inline tables are not supported")),
            _ => {
                let start = self.at;
                while self
                    .peek()
                    .is_some_and(|byte| !matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | b'#'))
                {
                    self.at += 1;
                }
                if self.at == start {
                    return Err(format!("line {line}: expected a value"));
                }
                Ok(Value::Bare(&self.source[start..self.at]))
            }
        }
    }

    fn string_array(&mut self) -> Result<Vec<String>, String> {
        let line = self.line;
        self.at += 1;
        let mut values = Vec::new();
        loop {
            self.skip_blank();
            match self.peek() {
                Some(b']') => {
                    self.at += 1;
                    return Ok(values);
                }
                Some(b'"' | b'\'') => values.push(self.string()?),
                // Refusing nesting also keeps the reader free of recursion.
                Some(b'[') => {
                    return Err(format!("line {}: nested arrays are not supported", self.line));
                }
                Some(b'{') => {
                    return Err(format!("line {}: inline tables are not supported", self.line));
                }
                None => return Err(format!("line {line}: unterminated array")),
                Some(_) => return Err(format!("line {}: expected a string array", self.line)),
            }
            self.skip_blank();
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {
                    self.at += 1;
                    return Ok(values);
                }
                None => return Err(format!("line {line}: unterminated array")),
                Some(_) => {
                    return Err(format!("line {}: expected , or ] in an array", self.line));
                }
            }
        }
    }

    /// Read any of TOML's four string forms, with the cursor on its opening quote.
    fn string(&mut self) -> Result<String, String> {
        let line = self.line;
        let quote = self.peek().expect("a string starts at a quote");
        let literal = quote == b'\'';
        let multiline = self.starts_with(if literal { "'''" } else { "\"\"\"" });
        if multiline {
            self.at += 3;
            // A line ending right after the opening delimiter is not part of the value.
            self.newline();
        } else {
            self.at += 1;
        }
        let mut value = String::new();
        loop {
            if self.peek() == Some(quote) {
                if !multiline {
                    self.at += 1;
                    return Ok(value);
                }
                let run = self.source.as_bytes()[self.at..]
                    .iter()
                    .take_while(|byte| **byte == quote)
                    .count();
                self.at += run;
                if run < 3 {
                    value.extend(std::iter::repeat_n(char::from(quote), run));
                    continue;
                }
                // Up to two quotes may sit against the closing delimiter.
                if run > 5 {
                    return Err(format!(
                        "line {}: too many quotes close a multi-line string",
                        self.line
                    ));
                }
                value.extend(std::iter::repeat_n(char::from(quote), run - 3));
                return Ok(value);
            }
            let Some(character) = self.source[self.at..].chars().next() else {
                return Err(format!("line {line}: unterminated string"));
            };
            match character {
                '\\' if !literal => self.escape(&mut value, multiline)?,
                '\n' | '\r' if multiline => {
                    if !self.newline() {
                        return Err(format!(
                            "line {}: a carriage return must be followed by a line feed",
                            self.line
                        ));
                    }
                    value.push('\n');
                }
                '\n' | '\r' => {
                    return Err(format!("line {line}: unterminated string"));
                }
                '\u{0}'..='\u{8}' | '\u{a}'..='\u{1f}' | '\u{7f}' => {
                    return Err(format!(
                        "line {}: control characters in strings must be escaped",
                        self.line
                    ));
                }
                _ => {
                    value.push(character);
                    self.at += character.len_utf8();
                }
            }
        }
    }

    /// Read one escape in a basic string, with the cursor on its backslash.
    fn escape(&mut self, value: &mut String, multiline: bool) -> Result<(), String> {
        let line = self.line;
        self.at += 1;
        let Some(escaped) = self.peek() else {
            return Err(format!("line {line}: unterminated string"));
        };
        let simple = match escaped {
            b'b' => Some('\u{8}'),
            b't' => Some('\t'),
            b'n' => Some('\n'),
            b'f' => Some('\u{c}'),
            b'r' => Some('\r'),
            b'"' => Some('"'),
            b'\\' => Some('\\'),
            _ => None,
        };
        if let Some(character) = simple {
            value.push(character);
            self.at += 1;
            return Ok(());
        }
        match escaped {
            b'u' | b'U' => {
                let digits = if escaped == b'u' { 4 } else { 8 };
                let name = char::from(escaped);
                let hex = self
                    .source
                    .get(self.at + 1..self.at + 1 + digits)
                    .filter(|hex| hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
                    .ok_or_else(|| {
                        format!("line {line}: \\{name} needs {digits} hexadecimal digits")
                    })?;
                let character =
                    u32::from_str_radix(hex, 16).ok().and_then(char::from_u32).ok_or_else(
                        || format!("line {line}: \\{name}{hex} is not a Unicode scalar value"),
                    )?;
                value.push(character);
                self.at += 1 + digits;
            }
            // A backslash ending a line of a multi-line string removes the line ending and
            // the whitespace and blank lines after it.
            b' ' | b'\t' | b'\r' | b'\n' if multiline => {
                self.skip_whitespace();
                if !self.newline() {
                    return Err(format!(
                        "line {line}: a backslash followed by whitespace must end the line"
                    ));
                }
                loop {
                    self.skip_whitespace();
                    if !self.newline() {
                        break;
                    }
                }
            }
            _ => {
                let shown = self.source[self.at..].chars().next().unwrap_or_default();
                return Err(format!("line {line}: invalid escape \\{shown}"));
            }
        }
        Ok(())
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
