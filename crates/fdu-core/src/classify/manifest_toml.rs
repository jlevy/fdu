// NOTE: `//!` is impossible here -- `build.rs` includes this file into a module of its
// own, where an inner doc comment is a parse error. The module's rustdoc lives on the
// owning `classify` module.
// The TOML both manifest dialects are written in, read by one cursor.
//
// The compact `[[kind]]` dialect and the File Rollup registry are both documents of
// `[[table]]` headers and `key = value` lines. Each once had its own reader, and the
// compact one split a line at the first `=` and stripped a quote from each end of the
// value, so `id = "notes" # "x"` read as the id `notes" # "x` there while the registry
// reader knew it as `notes`. One cursor means one answer to what a line says.
//
// What it reads, as TOML defines it: `\n` or `\r\n` line endings; comments on their own
// lines or after a header or value; basic strings with every TOML 1.0 escape, literal
// strings, and the multi-line form of each; string arrays spread over several lines, with
// comments and a trailing comma; and decimal integers with `_` digit separators. A
// leading byte-order mark is the caller's to strip, with `BYTE_ORDER_MARK`. Forms neither
// dialect needs are rejected with an error naming the form, so nothing is ever read as a
// different value: single-bracket tables, quoted and dotted keys, inline tables, nested
// arrays, and hexadecimal, octal, or binary integers. Which headers and keys exist, and
// what type each value must have, is the dialect's to say.
//
// Being shared with a build script constrains it the way it constrains the compact
// dialect: no `use` statements and nothing from `crate::`. And every item here must be
// used by the compact reader, because the build script compiles only that reader and
// would find anything else dead. The registry's float field is read by
// `file_rollup_manifest` for that reason.

/// A byte-order mark, which TOML permits before the first line.
pub(super) const BYTE_ORDER_MARK: char = '\u{feff}';

/// One value as written, before a field says which type it must have.
pub(super) enum Value<'a> {
    String(String),
    Strings(Vec<String>),
    /// An unquoted token: a number here, or a boolean or date no field accepts.
    Bare(&'a str),
}

impl Value<'_> {
    pub(super) fn string(self, line: usize) -> Result<String, String> {
        match self {
            Value::String(value) => Ok(value),
            Value::Strings(_) | Value::Bare(_) => {
                Err(format!("line {line}: expected a quoted string"))
            }
        }
    }

    pub(super) fn strings(self, line: usize) -> Result<Vec<String>, String> {
        match self {
            Value::Strings(values) => Ok(values),
            Value::String(_) | Value::Bare(_) => {
                Err(format!("line {line}: expected a string array"))
            }
        }
    }

    pub(super) fn integer<T: std::str::FromStr>(self, line: usize) -> Result<T, String> {
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
        // The digits are valid by now, so a parse that fails is one the field cannot hold.
        separated_digits(unsigned, false)
            .ok_or_else(invalid)?
            .parse()
            .map_err(|_| format!("line {line}: {token} is out of range for this field"))
    }
}

/// A TOML digit run: ASCII digits with single `_` separators between them, and no leading
/// zero unless `leading_zero` allows one. Returns the digits without separators.
pub(super) fn separated_digits(text: &str, leading_zero: bool) -> Option<String> {
    let bytes = text.as_bytes();
    let valid = bytes.first().is_some_and(u8::is_ascii_digit)
        && bytes.last().is_some_and(u8::is_ascii_digit)
        && bytes.iter().all(|byte| byte.is_ascii_digit() || *byte == b'_')
        && !text.contains("__")
        && (leading_zero || bytes.len() == 1 || bytes[0] != b'0');
    valid.then(|| text.replace('_', ""))
}

/// A cursor over a manifest document that knows the line it is on.
///
/// Every syntax byte is ASCII, so the cursor advances over syntax a byte at a time and
/// over string contents a character at a time, and always rests on a character boundary.
pub(super) struct Document<'a> {
    source: &'a str,
    at: usize,
    /// The line the cursor is on, counting from 1.
    pub(super) line: usize,
}

impl<'a> Document<'a> {
    pub(super) const fn new(source: &'a str) -> Self {
        Self { source, at: 0, line: 1 }
    }

    pub(super) fn peek(&self) -> Option<u8> {
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
    pub(super) fn next_line(&mut self) -> bool {
        self.skip_blank();
        self.peek().is_some()
    }

    /// After a header or value only whitespace and a comment may precede the line end.
    pub(super) fn end_of_line(&mut self, what: &str) -> Result<(), String> {
        self.skip_whitespace();
        self.skip_comment();
        if self.newline() || self.peek().is_none() {
            Ok(())
        } else {
            Err(format!("line {}: unexpected text after the {what}", self.line))
        }
    }

    /// Read a `[[name]]` header and return the name, with the cursor on its `[`.
    ///
    /// `tables` lists the dialect's headers for the message that refuses a single-bracket
    /// table; whether the name is one of them is the caller's to judge.
    pub(super) fn table_header(&mut self, tables: &str) -> Result<&'a str, String> {
        let line = self.line;
        if !self.starts_with("[[") {
            return Err(format!(
                "line {line}: single-bracket tables are not supported; use {tables}"
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
        Ok(name)
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

    pub(super) fn key(&mut self) -> Result<&'a str, String> {
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

    pub(super) fn value(&mut self) -> Result<Value<'a>, String> {
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
