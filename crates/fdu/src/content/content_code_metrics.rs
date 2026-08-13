//! Allocation-bounded common-language source-line classification.

use super::MetricValues;

/// Streaming `code-sloc-v1` counter for a supported language.
///
/// The counter retains only the current logical line and parser state. Mixed
/// code/comment lines are code, blank lines inside block comments are comments, and
/// multiline string lines are code. Line endings follow the same LF, CRLF, lone-CR,
/// and unterminated-final-line contract as the basic analyzer.
#[derive(Debug)]
pub struct CodeAccumulator {
    syntax: Syntax,
    state: State,
    line: Vec<u8>,
    previous_cr: bool,
    metrics: MetricValues,
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
        let class = classify_line(self.syntax, &mut self.state, &self.line);
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
    line_comments: &'static [&'static [u8]],
    block: Option<BlockSyntax>,
    nested_blocks: bool,
    triple_quotes: bool,
    backtick_strings: bool,
    rust_raw_strings: bool,
    shell_hash_boundary: bool,
    ruby_blocks: bool,
}

impl Syntax {
    fn for_type(file_type: &str) -> Option<Self> {
        let c_like = Self {
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
            "rust" => Some(Self { nested_blocks: true, rust_raw_strings: true, ..c_like }),
            "javascript" | "typescript" | "go" => Some(Self { backtick_strings: true, ..c_like }),
            "c" | "cpp" | "csharp" | "java" => Some(c_like),
            "kotlin" | "swift" => Some(Self { nested_blocks: true, triple_quotes: true, ..c_like }),
            "php" => Some(Self { line_comments: &[b"//", b"#"], ..c_like }),
            "python" | "ruby" => Some(Self {
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

#[derive(Clone, Copy, Debug)]
enum State {
    Normal,
    BlockComment { depth: u16 },
    Quoted { quote: u8, escaped: bool, multiline: bool },
    TripleQuoted { quote: u8 },
    RustRaw { hashes: u8 },
    RubyBlock,
}

#[derive(Clone, Copy, Debug)]
enum LineClass {
    Code,
    Comment,
    Blank,
}

fn classify_line(syntax: Syntax, state: &mut State, line: &[u8]) -> LineClass {
    let mut index = usize::from(line.starts_with(&[0xef, 0xbb, 0xbf]));
    index = index.saturating_mul(3);
    let mut whitespace_boundary = matches!(state, State::Normal);
    let mut code =
        matches!(state, State::Quoted { .. } | State::TripleQuoted { .. } | State::RustRaw { .. });
    let mut comment = matches!(state, State::BlockComment { .. });

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
        match *state {
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
            State::Quoted { quote, mut escaped, multiline } => {
                code = true;
                let byte = line[index];
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == quote {
                    *state = State::Normal;
                    index += 1;
                    continue;
                }
                *state = State::Quoted { quote, escaped, multiline };
                index += 1;
            }
            State::TripleQuoted { quote } => {
                code = true;
                if line[index..].starts_with(&[quote, quote, quote]) {
                    *state = State::Normal;
                    index += 3;
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
                if syntax.triple_quotes
                    && matches!(byte, b'\'' | b'"')
                    && line[index..].starts_with(&[byte, byte, byte])
                {
                    code = true;
                    whitespace_boundary = false;
                    *state = State::TripleQuoted { quote: byte };
                    index += 3;
                    continue;
                }
                if matches!(byte, b'\'' | b'"') || (syntax.backtick_strings && byte == b'`') {
                    code = true;
                    whitespace_boundary = false;
                    *state = State::Quoted { quote: byte, escaped: false, multiline: byte == b'`' };
                    index += 1;
                    continue;
                }
                code = true;
                whitespace_boundary = false;
                index += 1;
            }
        }
    }

    if let State::Quoted { multiline: false, .. } = state {
        *state = State::Normal;
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
