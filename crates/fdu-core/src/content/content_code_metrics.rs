//! Streaming common-language source-line classification.

use super::MetricValues;

/// Streaming `code-sloc-v1` counter for a supported language (analyzer version 3).
///
/// The counter retains the current logical line, its allocated capacity, and parser
/// state. A one-line minified or generated source can therefore require file-sized
/// memory per active worker. Mixed
/// code/comment lines are code, blank lines inside block comments are comments, and
/// multiline string lines are code. Line endings follow the same LF, CRLF, lone-CR,
/// and unterminated-final-line contract as the basic analyzer.
#[derive(Debug)]
pub struct CodeAccumulator {
    syntax: Syntax,
    state: State,
    line: Vec<u8>,
    previous_cr: bool,
    javascript: JavaScriptContext,
    shell: ShellContext,
    metrics: MetricValues,
}

#[derive(Debug)]
struct JavaScriptContext {
    regex_allowed: bool,
    pending_control_paren: bool,
    paren_control: Vec<bool>,
    after_dot: bool,
}

#[derive(Debug, Default)]
struct ShellContext {
    // The number of unmatched parentheses in a shell arithmetic expression.
    // `<<` there is a shift operator, not a heredoc opener.
    arithmetic_parens: usize,
}

impl Default for JavaScriptContext {
    fn default() -> Self {
        Self {
            regex_allowed: true,
            pending_control_paren: false,
            paren_control: Vec::new(),
            after_dot: false,
        }
    }
}

impl CodeAccumulator {
    /// Create a counter for one stable file-type ID, or return `None` when
    /// `code-sloc-v1` does not claim that language.
    pub fn for_type(file_type: &str) -> Option<Self> {
        let syntax = Syntax::for_type(file_type)?;
        Some(Self {
            syntax,
            state: State::Normal,
            line: Vec::new(),
            previous_cr: false,
            javascript: JavaScriptContext::default(),
            shell: ShellContext::default(),
            metrics: MetricValues::default(),
        })
    }

    /// Consume an arbitrary byte chunk.
    pub fn push(&mut self, chunk: &[u8]) {
        for &byte in chunk {
            if self.previous_cr {
                self.previous_cr = false;
                if byte == b'\n' {
                    continue;
                }
            }
            match byte {
                b'\r' => {
                    self.finish_line();
                    self.previous_cr = true;
                }
                b'\n' => self.finish_line(),
                _ => self.line.push(byte),
            }
        }
    }

    /// Finish an unterminated final line and return its additive metrics.
    pub fn finish(mut self) -> MetricValues {
        if !self.line.is_empty() && self.line.as_slice() != [0xef, 0xbb, 0xbf] {
            self.finish_line();
        }
        self.metrics
    }

    fn finish_line(&mut self) {
        let class = classify_line(
            self.syntax,
            &mut self.state,
            &mut self.javascript,
            &mut self.shell,
            &self.line,
        );
        self.metrics.physical_lines = self.metrics.physical_lines.saturating_add(1);
        match class {
            LineClass::Code => {
                self.metrics.code_lines = self.metrics.code_lines.saturating_add(1);
            }
            LineClass::Comment => {
                self.metrics.comment_lines = self.metrics.comment_lines.saturating_add(1);
            }
            LineClass::Blank => {
                self.metrics.code_blank_lines = self.metrics.code_blank_lines.saturating_add(1);
            }
        }
        self.line.clear();
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(clippy::struct_excessive_bools)]
struct Syntax {
    language: Language,
    line_comments: &'static [&'static [u8]],
    block: Option<BlockSyntax>,
    nested_blocks: bool,
    triple_quotes: bool,
    backtick_strings: bool,
    rust_raw_strings: bool,
    shell_hash_boundary: bool,
    ruby_blocks: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Language {
    Rust,
    JavaScript,
    C,
    Cpp,
    CSharp,
    Java,
    Kotlin,
    Swift,
    Go,
    Php,
    Python,
    Ruby,
    Shell,
    Sql,
}

impl Syntax {
    fn for_type(file_type: &str) -> Option<Self> {
        let c_like = Self {
            language: Language::C,
            line_comments: &[b"//"],
            block: Some(BlockSyntax { open: b"/*", close: b"*/" }),
            nested_blocks: false,
            triple_quotes: false,
            backtick_strings: false,
            rust_raw_strings: false,
            shell_hash_boundary: false,
            ruby_blocks: false,
        };
        match file_type {
            "rust" => Some(Self {
                language: Language::Rust,
                nested_blocks: true,
                rust_raw_strings: true,
                ..c_like
            }),
            "javascript" | "typescript" => {
                Some(Self { language: Language::JavaScript, backtick_strings: true, ..c_like })
            }
            "go" => Some(Self { language: Language::Go, backtick_strings: true, ..c_like }),
            "c" => Some(c_like),
            "cpp" => Some(Self { language: Language::Cpp, ..c_like }),
            "csharp" => Some(Self { language: Language::CSharp, ..c_like }),
            "java" => Some(Self { language: Language::Java, ..c_like }),
            "kotlin" => Some(Self {
                language: Language::Kotlin,
                nested_blocks: true,
                triple_quotes: true,
                ..c_like
            }),
            "swift" => Some(Self {
                language: Language::Swift,
                nested_blocks: true,
                triple_quotes: true,
                ..c_like
            }),
            "php" => {
                Some(Self { language: Language::Php, line_comments: &[b"//", b"#"], ..c_like })
            }
            "python" | "ruby" => Some(Self {
                language: if file_type == "ruby" { Language::Ruby } else { Language::Python },
                line_comments: &[b"#"],
                block: None,
                nested_blocks: false,
                triple_quotes: true,
                backtick_strings: false,
                rust_raw_strings: false,
                shell_hash_boundary: false,
                ruby_blocks: file_type == "ruby",
            }),
            "shell" => Some(Self {
                language: Language::Shell,
                line_comments: &[b"#"],
                block: None,
                nested_blocks: false,
                triple_quotes: false,
                backtick_strings: true,
                rust_raw_strings: false,
                shell_hash_boundary: true,
                ruby_blocks: false,
            }),
            "sql" => Some(Self {
                language: Language::Sql,
                line_comments: &[b"--"],
                block: Some(BlockSyntax { open: b"/*", close: b"*/" }),
                nested_blocks: false,
                triple_quotes: false,
                backtick_strings: true,
                rust_raw_strings: false,
                shell_hash_boundary: false,
                ruby_blocks: false,
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct BlockSyntax {
    open: &'static [u8],
    close: &'static [u8],
}

#[derive(Clone, Debug)]
enum State {
    Normal,
    BlockComment { depth: u16 },
    Quoted { quote: u8, escaped: bool, multiline: bool, doubled: bool },
    TripleQuoted { quote: u8, width: usize },
    RustRaw { hashes: u8 },
    Delimited { close: Vec<u8> },
    Heredoc { terminator: Vec<u8>, indent: bool, php: bool },
    RubyPercent { open: u8, close: u8, depth: usize },
    Regex { escaped: bool, in_class: bool },
    RubyBlock,
}

#[derive(Clone, Copy, Debug)]
enum LineClass {
    Code,
    Comment,
    Blank,
}

fn classify_line(
    syntax: Syntax,
    state: &mut State,
    javascript: &mut JavaScriptContext,
    shell: &mut ShellContext,
    line: &[u8],
) -> LineClass {
    let mut index = usize::from(line.starts_with(&[0xef, 0xbb, 0xbf]));
    index = index.saturating_mul(3);
    let mut whitespace_boundary = matches!(state, State::Normal);
    let mut code = matches!(
        state,
        State::Quoted { .. }
            | State::TripleQuoted { .. }
            | State::RustRaw { .. }
            | State::Delimited { .. }
            | State::Heredoc { .. }
            | State::RubyPercent { .. }
            | State::Regex { .. }
    );
    let mut comment = matches!(state, State::BlockComment { .. });

    if let State::Heredoc { terminator, indent, php } = state {
        let candidate = if *indent {
            let indentation = line
                .iter()
                .take_while(|byte| {
                    if syntax.language == Language::Shell {
                        **byte == b'\t'
                    } else {
                        byte.is_ascii_whitespace()
                    }
                })
                .count();
            &line[indentation..]
        } else {
            line
        };
        if candidate == terminator
            || (*php && candidate.strip_suffix(b";") == Some(terminator.as_slice()))
        {
            *state = State::Normal;
        }
        return LineClass::Code;
    }

    if matches!(state, State::RubyBlock) {
        if line.starts_with(b"=end") {
            *state = State::Normal;
        }
        return LineClass::Comment;
    }
    if syntax.ruby_blocks && matches!(state, State::Normal) && line.starts_with(b"=begin") {
        *state = State::RubyBlock;
        return LineClass::Comment;
    }

    while index < line.len() {
        if let State::Delimited { close } = state {
            code = true;
            if line[index..].starts_with(close) {
                index += close.len();
                *state = State::Normal;
            } else {
                index += 1;
            }
            continue;
        }
        match state.clone() {
            State::BlockComment { mut depth } => {
                comment = true;
                let block = syntax.block.expect("block state requires block syntax");
                if syntax.nested_blocks && line[index..].starts_with(block.open) {
                    depth = depth.saturating_add(1);
                    *state = State::BlockComment { depth };
                    index += block.open.len();
                } else if line[index..].starts_with(block.close) {
                    depth = depth.saturating_sub(1);
                    *state = if depth == 0 { State::Normal } else { State::BlockComment { depth } };
                    index += block.close.len();
                } else {
                    index += 1;
                }
            }
            State::Quoted { quote, mut escaped, multiline, doubled } => {
                code = true;
                let byte = line[index];
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == quote {
                    if doubled && line.get(index + 1) == Some(&quote) {
                        index += 2;
                        continue;
                    }
                    *state = State::Normal;
                    if syntax.language == Language::JavaScript {
                        javascript.regex_allowed = false;
                    }
                    index += 1;
                    continue;
                }
                *state = State::Quoted { quote, escaped, multiline, doubled };
                index += 1;
            }
            State::TripleQuoted { quote, width } => {
                code = true;
                if line[index..].iter().take(width).all(|b| *b == quote)
                    && line.len() - index >= width
                {
                    *state = State::Normal;
                    if syntax.language == Language::JavaScript {
                        javascript.regex_allowed = false;
                    }
                    index += width;
                } else {
                    index += 1;
                }
            }
            State::RustRaw { hashes } => {
                code = true;
                if rust_raw_close(&line[index..], hashes) {
                    *state = State::Normal;
                    index += usize::from(hashes) + 1;
                } else {
                    index += 1;
                }
            }
            State::Delimited { .. } => {
                unreachable!("delimited strings are handled before matching")
            }
            State::RubyPercent { open, close, mut depth } => {
                code = true;
                match line[index] {
                    b'\\' => index += usize::min(2, line.len() - index),
                    byte if byte == open && open != close => {
                        depth += 1;
                        *state = State::RubyPercent { open, close, depth };
                        index += 1;
                    }
                    byte if byte == close => {
                        depth -= 1;
                        *state = if depth == 0 {
                            State::Normal
                        } else {
                            State::RubyPercent { open, close, depth }
                        };
                        index += 1;
                    }
                    _ => index += 1,
                }
            }
            State::Regex { mut escaped, mut in_class } => {
                code = true;
                let byte = line[index];
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'[' {
                    in_class = true;
                } else if byte == b']' {
                    in_class = false;
                } else if byte == b'/' && !in_class {
                    *state = State::Normal;
                    index += 1;
                    javascript.regex_allowed = false;
                    continue;
                }
                *state = State::Regex { escaped, in_class };
                index += 1;
            }
            State::Heredoc { .. } => unreachable!("heredocs return before byte scanning"),
            State::RubyBlock => unreachable!("Ruby blocks return before byte scanning"),
            State::Normal => {
                let byte = line[index];
                if byte.is_ascii_whitespace() {
                    whitespace_boundary = true;
                    index += 1;
                    continue;
                }
                if let Some((character, width)) = leading_utf8_character(&line[index..]) {
                    if super::content_basic_metrics::is_content_whitespace(character) {
                        whitespace_boundary = true;
                        index += width;
                        continue;
                    }
                }
                if syntax.language == Language::Shell {
                    if shell.arithmetic_parens == 0 && line[index..].starts_with(b"$((") {
                        shell.arithmetic_parens = 2;
                        code = true;
                        whitespace_boundary = false;
                        index += 3;
                        continue;
                    }
                    if shell.arithmetic_parens == 0
                        && whitespace_boundary
                        && line[index..].starts_with(b"((")
                    {
                        shell.arithmetic_parens = 2;
                        code = true;
                        whitespace_boundary = false;
                        index += 2;
                        continue;
                    }
                    if shell.arithmetic_parens > 0 {
                        match byte {
                            b'(' => {
                                shell.arithmetic_parens = shell.arithmetic_parens.saturating_add(1);
                            }
                            b')' => shell.arithmetic_parens -= 1,
                            _ => {}
                        }
                    }
                }
                if syntax.language == Language::JavaScript
                    && byte == b'/'
                    && javascript.regex_allowed
                    && !line[index..].starts_with(b"//")
                    && !line[index..].starts_with(b"/*")
                {
                    code = true;
                    whitespace_boundary = false;
                    *state = State::Regex { escaped: false, in_class: false };
                    index += 1;
                    continue;
                }
                if syntax.line_comments.iter().any(|marker| {
                    line[index..].starts_with(marker)
                        && (!syntax.shell_hash_boundary || whitespace_boundary)
                }) {
                    comment = true;
                    break;
                }
                if let Some(block) = syntax.block {
                    if line[index..].starts_with(block.open) {
                        comment = true;
                        whitespace_boundary = false;
                        *state = State::BlockComment { depth: 1 };
                        index += block.open.len();
                        continue;
                    }
                }
                if syntax.rust_raw_strings {
                    if let Some((hashes, consumed)) = rust_raw_open(&line[index..]) {
                        code = true;
                        whitespace_boundary = false;
                        *state = State::RustRaw { hashes };
                        index += consumed;
                        continue;
                    }
                }
                if syntax.language == Language::Cpp {
                    if let Some((close, consumed)) = cpp_raw_open(&line[index..]) {
                        code = true;
                        *state = State::Delimited { close };
                        index += consumed;
                        continue;
                    }
                }
                if syntax.language == Language::Sql {
                    if let Some((close, consumed)) = sql_dollar_open(&line[index..]) {
                        code = true;
                        *state = State::Delimited { close };
                        index += consumed;
                        continue;
                    }
                }
                if syntax.language == Language::Ruby {
                    if let Some((open, close, consumed)) = ruby_percent_open(&line[index..]) {
                        code = true;
                        *state = State::RubyPercent { open, close, depth: 1 };
                        index += consumed;
                        continue;
                    }
                }
                if let Some((terminator, indent, php)) = (syntax.language != Language::Shell
                    || shell.arithmetic_parens == 0)
                    .then(|| heredoc_open(syntax.language, &line[index..]))
                    .flatten()
                {
                    code = true;
                    *state = State::Heredoc { terminator, indent, php };
                    break;
                }
                if syntax.language == Language::CSharp && line[index..].starts_with(b"@\"") {
                    code = true;
                    *state = State::Quoted {
                        quote: b'"',
                        escaped: false,
                        multiline: true,
                        doubled: true,
                    };
                    index += 2;
                    continue;
                }
                if matches!(syntax.language, Language::CSharp | Language::Java)
                    && line[index..].starts_with(b"\"\"\"")
                {
                    code = true;
                    let width = if syntax.language == Language::CSharp {
                        line[index..].iter().take_while(|b| **b == b'"').count()
                    } else {
                        3
                    };
                    *state = State::TripleQuoted { quote: b'"', width };
                    index += width;
                    continue;
                }
                if syntax.triple_quotes
                    && matches!(byte, b'\'' | b'"')
                    && line[index..].starts_with(&[byte, byte, byte])
                {
                    code = true;
                    whitespace_boundary = false;
                    *state = State::TripleQuoted { quote: byte, width: 3 };
                    index += 3;
                    continue;
                }
                if matches!(byte, b'\'' | b'"') || (syntax.backtick_strings && byte == b'`') {
                    if syntax.language == Language::Rust
                        && byte == b'\''
                        && rust_lifetime(&line[index..])
                    {
                        code = true;
                        index += 1;
                        continue;
                    }
                    code = true;
                    whitespace_boundary = false;
                    let multiline = byte == b'`'
                        || (byte == b'"' && syntax.language == Language::Rust)
                        || matches!(
                            syntax.language,
                            Language::Ruby | Language::Shell | Language::Sql
                        )
                        || (syntax.language == Language::C
                            && byte == b'"'
                            && line.ends_with(b"\\"));
                    *state = State::Quoted {
                        quote: byte,
                        escaped: false,
                        multiline,
                        doubled: syntax.language == Language::Sql,
                    };
                    index += 1;
                    continue;
                }
                if syntax.language == Language::JavaScript {
                    if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$') {
                        let end = line[index..]
                            .iter()
                            .take_while(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$'))
                            .count()
                            + index;
                        let word = &line[index..end];
                        javascript.pending_control_paren = !javascript.after_dot
                            && matches!(word, b"if" | b"while" | b"for" | b"with");
                        javascript.after_dot = false;
                        javascript.regex_allowed = matches!(
                            word,
                            b"return"
                                | b"else"
                                | b"do"
                                | b"throw"
                                | b"case"
                                | b"delete"
                                | b"typeof"
                                | b"void"
                                | b"yield"
                                | b"await"
                                | b"instanceof"
                                | b"in"
                                | b"of"
                        );
                        code = true;
                        index = end;
                        continue;
                    }
                    if byte == b'(' {
                        javascript.paren_control.push(javascript.pending_control_paren);
                        javascript.pending_control_paren = false;
                        javascript.after_dot = false;
                        javascript.regex_allowed = true;
                        code = true;
                        index += 1;
                        continue;
                    }
                    if byte == b')' {
                        javascript.regex_allowed = javascript.paren_control.pop().unwrap_or(false);
                        javascript.pending_control_paren = false;
                        javascript.after_dot = false;
                        code = true;
                        index += 1;
                        continue;
                    }
                    javascript.pending_control_paren = false;
                    javascript.after_dot = byte == b'.';
                    javascript.regex_allowed = matches!(
                        byte,
                        b'=' | b'('
                            | b'['
                            | b'{'
                            | b':'
                            | b','
                            | b';'
                            | b'!'
                            | b'?'
                            | b'&'
                            | b'|'
                            | b'+'
                            | b'-'
                            | b'*'
                            | b'%'
                            | b'^'
                            | b'~'
                            | b'<'
                            | b'>'
                            | b'/'
                    );
                }
                code = true;
                whitespace_boundary = syntax.language == Language::Shell
                    && matches!(byte, b';' | b'&' | b'|' | b'(' | b')' | b'<' | b'>');
                index += 1;
            }
        }
    }

    match state {
        State::Quoted { multiline: false, escaped: true, .. }
            if matches!(syntax.language, Language::C | Language::Cpp) =>
        {
            *state =
                State::Quoted { quote: b'"', escaped: false, multiline: false, doubled: false };
        }
        State::Quoted { multiline: false, .. } | State::Regex { .. } => *state = State::Normal,
        State::Quoted { multiline: true, escaped, .. } => *escaped = false,
        _ => {}
    }
    if code {
        LineClass::Code
    } else if comment {
        LineClass::Comment
    } else {
        LineClass::Blank
    }
}

fn leading_utf8_character(input: &[u8]) -> Option<(char, usize)> {
    let width = match *input.first()? {
        0x00..=0x7f => 1,
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return None,
    };
    let character = std::str::from_utf8(input.get(..width)?).ok()?.chars().next()?;
    Some((character, width))
}

fn rust_raw_open(input: &[u8]) -> Option<(u8, usize)> {
    if input.first() != Some(&b'r') {
        return None;
    }
    let hashes = input[1..].iter().take_while(|byte| **byte == b'#').count();
    if hashes > usize::from(u8::MAX) || input.get(hashes + 1) != Some(&b'"') {
        return None;
    }
    Some((u8::try_from(hashes).expect("bounded above"), hashes + 2))
}

fn rust_raw_close(input: &[u8], hashes: u8) -> bool {
    input.first() == Some(&b'"')
        && (hashes == 0
            || input
                .get(1..=usize::from(hashes))
                .is_some_and(|tail| tail.iter().all(|byte| *byte == b'#')))
}

fn rust_lifetime(input: &[u8]) -> bool {
    let Some(first) = input.get(1) else { return false };
    if !first.is_ascii_alphabetic() && *first != b'_' {
        return false;
    }
    let name_len =
        input[1..].iter().take_while(|byte| byte.is_ascii_alphanumeric() || **byte == b'_').count();
    !(name_len == 1 && input.get(2) == Some(&b'\''))
}

fn cpp_raw_open(input: &[u8]) -> Option<(Vec<u8>, usize)> {
    if !input.starts_with(b"R\"") {
        return None;
    }
    let end = input[2..].iter().position(|byte| *byte == b'(')? + 2;
    let delimiter = &input[2..end];
    if delimiter.len() > 16
        || delimiter.iter().any(|b| b.is_ascii_whitespace() || matches!(b, b'\\' | b'(' | b')'))
    {
        return None;
    }
    let mut close = Vec::with_capacity(delimiter.len() + 2);
    close.push(b')');
    close.extend_from_slice(delimiter);
    close.push(b'"');
    Some((close, end + 1))
}

fn sql_dollar_open(input: &[u8]) -> Option<(Vec<u8>, usize)> {
    if input.first() != Some(&b'$') {
        return None;
    }
    let end = input[1..].iter().position(|byte| *byte == b'$')? + 1;
    let tag = &input[1..end];
    if !tag.is_empty()
        && (!tag[0].is_ascii_alphabetic() && tag[0] != b'_'
            || tag.iter().any(|b| !b.is_ascii_alphanumeric() && *b != b'_'))
    {
        return None;
    }
    Some((input[..=end].to_vec(), end + 1))
}

fn ruby_percent_open(input: &[u8]) -> Option<(u8, u8, usize)> {
    if input.first() != Some(&b'%') {
        return None;
    }
    let delimiter_index = match input.get(1) {
        Some(b'q' | b'Q' | b'w' | b'W' | b'i' | b'I' | b'r' | b'x' | b's') => 2,
        Some(b'{' | b'[' | b'(' | b'<') => 1,
        _ => return None,
    };
    let open = *input.get(delimiter_index)?;
    let close = match open {
        b'{' => b'}',
        b'[' => b']',
        b'(' => b')',
        b'<' => b'>',
        b'/' | b'!' | b'|' => open,
        _ => return None,
    };
    Some((open, close, delimiter_index + 1))
}

fn heredoc_open(language: Language, input: &[u8]) -> Option<(Vec<u8>, bool, bool)> {
    let php = language == Language::Php;
    if !matches!(language, Language::Ruby | Language::Shell | Language::Php) {
        return None;
    }
    let mut tail = if php { input.strip_prefix(b"<<<")? } else { input.strip_prefix(b"<<")? };
    let indent = if !php && matches!(tail.first(), Some(b'-' | b'~')) {
        tail = &tail[1..];
        true
    } else {
        false
    };
    let quote = if matches!(tail.first(), Some(b'\'' | b'"')) {
        let quote = tail[0];
        tail = &tail[1..];
        Some(quote)
    } else {
        None
    };
    let width = tail.iter().take_while(|b| b.is_ascii_alphanumeric() || **b == b'_').count();
    if width == 0 || !tail[0].is_ascii_alphabetic() && tail[0] != b'_' {
        return None;
    }
    if let Some(quote) = quote {
        if tail.get(width) != Some(&quote) {
            return None;
        }
    }
    Some((tail[..width].to_vec(), indent, php))
}

#[cfg(test)]
mod tests {
    use super::*;

    type PartitionCase<'a> = (&'a str, &'a [u8], (u64, u64, u64));

    fn count(language: &str, chunks: &[&[u8]]) -> MetricValues {
        let mut counter = CodeAccumulator::for_type(language).expect("supported language");
        for chunk in chunks {
            counter.push(chunk);
        }
        counter.finish()
    }

    #[test]
    fn partitions_c_like_source_and_counts_mixed_lines_as_code() {
        let metrics = count(
            "javascript",
            &[b"// first\r\nlet url = \"https://example.test\"; // tail\r/* block\n\nend */\n`// text\nmore`;"],
        );
        assert_eq!(metrics.physical_lines, 7);
        assert_eq!(metrics.code_lines, 3);
        assert_eq!(metrics.comment_lines, 4);
        assert_eq!(metrics.code_blank_lines, 0);
    }

    #[test]
    fn rust_nested_comments_and_raw_strings_ignore_comment_markers() {
        let source =
            b"/* outer\n/* inner */\n*/\nlet raw = r##\"/* text */\n// still text\"##;\n\n";
        let expected = count("rust", &[source]);
        assert_eq!(expected.physical_lines, 6);
        assert_eq!(expected.code_lines, 2);
        assert_eq!(expected.comment_lines, 3);
        assert_eq!(expected.code_blank_lines, 1);
        for split in 0..=source.len() {
            assert_eq!(count("rust", &[&source[..split], &source[split..]]), expected);
        }
    }

    #[test]
    fn triple_quoted_docstrings_are_code_in_v1() {
        let metrics = count("python", &[b"\"\"\"docs\n# text\n\"\"\"\n# comment\npass\n"]);
        assert_eq!(metrics.physical_lines, 5);
        assert_eq!(metrics.code_lines, 4);
        assert_eq!(metrics.comment_lines, 1);
        assert_eq!(metrics.code_blank_lines, 0);
    }

    #[test]
    fn every_line_ending_convention_has_the_same_partition() {
        for source in [
            "// comment\nlet value = 1;\n\n",
            "// comment\r\nlet value = 1;\r\n\r\n",
            "// comment\rlet value = 1;\r\r",
            "// comment\r\nlet value = 1;\r\n",
        ] {
            let metrics = count("rust", &[source.as_bytes()]);
            let expected_lines = if source.ends_with("value = 1;\r\n") { 2 } else { 3 };
            assert_eq!(metrics.physical_lines, expected_lines, "{source:?}");
            assert_eq!(metrics.code_lines, 1, "{source:?}");
            assert_eq!(metrics.comment_lines, 1, "{source:?}");
            assert_eq!(metrics.code_blank_lines, expected_lines - 2, "{source:?}");
        }
    }

    #[test]
    fn a_leading_utf8_bom_is_not_an_invented_line() {
        let empty = count("rust", &[b"\xef\xbb\xbf"]);
        assert_eq!(empty.physical_lines, 0);

        let blank = count("rust", &[b"\xef\xbb\xbf\n"]);
        assert_eq!(blank.physical_lines, 1);
        assert_eq!(blank.code_blank_lines, 1);
    }

    #[test]
    fn unicode_whitespace_uses_the_basic_analyzers_pinned_table() {
        let metrics = count("rust", &["\u{3000}\n\u{2003}// comment\n".as_bytes()]);
        assert_eq!(metrics.physical_lines, 2);
        assert_eq!(metrics.code_blank_lines, 1);
        assert_eq!(metrics.comment_lines, 1);
        assert_eq!(metrics.code_lines, 0);

        let shell = count("shell", &["\u{3000}# comment\n".as_bytes()]);
        assert_eq!(shell.comment_lines, 1);
        assert_eq!(shell.code_lines, 0);
    }

    #[test]
    fn unsupported_languages_are_explicit() {
        assert!(CodeAccumulator::for_type("haskell").is_none());
    }

    #[test]
    fn rust_multiline_string_and_lifetime_preserve_following_comment_state() {
        let string = b"const S: &str = \"first\n// text\nlast\";\n";
        let lifetime = b"fn x<'a>() { /*\ncomment\n*/ }\n";
        for (source, code, comment) in [(string.as_slice(), 3, 0), (lifetime.as_slice(), 2, 1)] {
            for split in 0..=source.len() {
                let metrics = count("rust", &[&source[..split], &source[split..]]);
                assert_eq!(
                    (metrics.code_lines, metrics.comment_lines),
                    (code, comment),
                    "split {split}"
                );
            }
        }
    }

    #[test]
    fn multiline_literal_families_preserve_comment_markers() {
        let cases: &[PartitionCase<'_>] = &[
            ("cpp", b"const char *s = R\"tag(first\n// text\nlast)tag\";\n", (3, 0, 0)),
            ("c", b"const char *s = \"first\\\n// text\";\n", (2, 0, 0)),
            ("java", b"class C { String s = \"\"\"\n// text\nlast\n\"\"\"; }\n", (4, 0, 0)),
            ("csharp", b"class C { string s = @\"first\n// text\nlast\"; }\n", (3, 0, 0)),
            ("csharp", b"class C { string s = \"\"\"\n// text\nlast\n\"\"\"; }\n", (4, 0, 0)),
            ("ruby", b"s = %q{first\n# text\nlast}\n", (3, 0, 0)),
            ("ruby", b"s = <<~TEXT\n# text\nlast\nTEXT\n", (4, 0, 0)),
            ("ruby", b"s = \"first\n# text\nlast\"\n", (3, 0, 0)),
            ("shell", b"cat <<'TEXT'\n# text\nlast\nTEXT\n", (4, 0, 0)),
            ("shell", b"value='first\n# text\nlast'\n", (3, 0, 0)),
            ("sql", b"SELECT $tag$first\n-- text\nlast$tag$;\n", (3, 0, 0)),
            ("sql", b"SELECT 'first\n-- text\nlast';\n", (3, 0, 0)),
            ("php", b"<?php\n$s = <<<TEXT\n// text\nlast\nTEXT;\n", (5, 0, 0)),
        ];
        for (language, source, expected) in cases {
            for split in 0..=source.len() {
                let metrics = count(language, &[&source[..split], &source[split..]]);
                assert_eq!(
                    (metrics.code_lines, metrics.comment_lines, metrics.code_blank_lines),
                    *expected,
                    "{language} split {split}"
                );
            }
        }
    }

    #[test]
    fn multiline_delimiters_restore_comment_recognition_after_closing() {
        let cases: &[(&str, &[u8], (u64, u64))] = &[
            ("cpp", b"auto s = R\"x(/*\n// body\n)x\";\n// comment\n", (3, 1)),
            ("csharp", b"var s = @\"first \"\" quote\n// body\nlast\";\n// comment\n", (3, 1)),
            ("ruby", b"s = %q{outer {inner\n# body\n}}\n# comment\n", (3, 1)),
            ("ruby", b"s = <<~TEXT\n# body\n  TEXT\n# comment\n", (3, 1)),
            ("shell", b"cat <<'TEXT'\n# body\nTEXT\n# comment\n", (3, 1)),
            ("shell", b"cat <<-TEXT\n TEXT\n# body\nTEXT\n# comment\n", (4, 1)),
            ("shell", b"cat <<-TEXT\n\tTEXT\n# comment\n", (2, 1)),
            ("sql", b"SELECT $tag$first\n-- body\nlast$tag$;\n-- comment\n", (3, 1)),
            ("php", b"$s = <<<TEXT\n// body\nTEXT;\n// comment\n", (3, 1)),
        ];
        for (language, source, expected) in cases {
            let metrics = count(language, &[source]);
            assert_eq!((metrics.code_lines, metrics.comment_lines), *expected, "{language}");
        }
    }

    #[test]
    fn shell_comments_and_arithmetic_shifts_do_not_hold_lexer_state() {
        let cases: &[PartitionCase<'_>] = &[
            // A command separator starts a new shell word, so the unmatched quote is
            // comment text and cannot turn the next line into a multiline string.
            ("shell", b"true;# \"unterminated\n# following\nprintf ok\n", (2, 1, 0)),
            // In arithmetic expansion and arithmetic commands, `<<` shifts a value.
            // Neither spelling opens a heredoc that consumes the following comment.
            ("shell", b"N=2\nx=$((1<<N))\n# following\n", (2, 1, 0)),
            ("shell", b"N=2\n((1<<N))\n# following\n", (2, 1, 0)),
            // A hash within a shell word is literal, while a real heredoc body is code.
            ("shell", b"printf '%s' foo#bar\ncat <<TEXT\n# literal\nTEXT\n# comment\n", (4, 1, 0)),
        ];
        for (language, source, expected) in cases {
            for split in 0..=source.len() {
                let metrics = count(language, &[&source[..split], &source[split..]]);
                assert_eq!(
                    (metrics.code_lines, metrics.comment_lines, metrics.code_blank_lines),
                    *expected,
                    "source {source:?}, split {split}"
                );
            }
        }
    }

    #[test]
    fn javascript_regex_and_division_leave_comment_state_correct() {
        let cases: &[PartitionCase<'_>] = &[
            ("javascript", b"const re = /[/*]/;\nconst answer = 42;\n", (2, 0, 0)),
            ("typescript", b"const re = /\\/\\* inside [/] /;\nconst n = 9;\n", (2, 0, 0)),
            ("javascript", b"const ratio = total / count;\n/* comment */\nconst re = /[//]/;\n", (2, 1, 0)),
            ("typescript", b"return /[/*]/.test(value);\n// comment\nnext();\n", (2, 1, 0)),
            ("javascript", b"const quotient = total\n / count; /* real comment */\nconst re = /[/*]/;\nnext();\n", (4, 0, 0)),
            ("javascript", b"const text = \"plain\" / count;\nconst re = /[/*]/;\nnext();\n", (3, 0, 0)),
            ("javascript", b"if (ok) /[/*]/.test(value);\nconst next = 1;\n", (2, 0, 0)),
            ("javascript", b"if (first) {}\nif (ok) /[/*]/.test(value);\nnext();\n", (3, 0, 0)),
            ("typescript", b"while (ready && check(x)) /[/*]/.test(value);\nnext();\n", (2, 0, 0)),
            ("javascript", b"if (ok &&\n check(x)) /[/*]/.test(value);\nnext();\n", (3, 0, 0)),
            ("javascript", b"if (ok) fn(value) / count;\n/* comment */\n", (1, 1, 0)),
            ("javascript", b"const ratio = object.if(value) / count;\n/* comment */\n", (1, 1, 0)),
        ];
        for (language, source, expected) in cases {
            for split in 0..=source.len() {
                let metrics = count(language, &[&source[..split], &source[split..]]);
                assert_eq!(
                    (metrics.code_lines, metrics.comment_lines, metrics.code_blank_lines),
                    *expected,
                    "{language} split {split}"
                );
            }
        }
    }
}
