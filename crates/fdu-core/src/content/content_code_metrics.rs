//! Streaming common-language source-line classification.

use super::MetricValues;

/// Bytes of the current line the accumulator holds before it classifies them piecewise.
///
/// A line shorter than this is classified whole, as every line always was; a longer one
/// is classified in pieces as it arrives, so what a line costs is this window plus the
/// longest token a piece cannot yet decide, never the line (fdu-1zb6).
const LINE_WINDOW_BYTES: usize = 64 * 1024;

/// Streaming `code-sloc-v1` counter for a supported language (analyzer version 3).
///
/// The counter retains parser state, the per-line facts the classifier reads, and a
/// bounded window of the current line: a line is classified in pieces as it arrives,
/// and a one-line minified or generated source costs a worker the window, not the file.
/// Mixed code/comment lines are code, blank lines inside block comments are comments,
/// and multiline string lines are code. Line endings follow the same LF, CRLF, lone-CR,
/// and unterminated-final-line contract as the basic analyzer.
#[derive(Debug)]
pub struct CodeAccumulator {
    syntax: Syntax,
    state: State,
    /// Bytes of the current line not yet classified.
    window: Vec<u8>,
    /// Bytes of the current line already classified and dropped from the window.
    consumed: usize,
    /// The last byte dropped from the window, which the line's trailing backslash reads.
    last_dropped: Option<u8>,
    /// The window length at which the next piece is classified.
    next_scan_at: usize,
    /// How far the window grows between piece scans.
    window_bytes: usize,
    /// The largest window held, for the bound's own test.
    #[cfg(test)]
    peak_window: usize,
    previous_cr: bool,
    line: LineScan,
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

/// What the classifier knows about the line in progress, carried across its pieces.
///
/// These were the locals of a classifier that saw each line whole. A piece scan reads
/// and leaves them exactly as the whole-line scan would have at that byte.
#[derive(Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
struct LineScan {
    /// Whether the decisions made at the start of a line have been made.
    started: bool,
    /// The line began with a byte-order mark, which is skipped and not scanned.
    bom: bool,
    code: bool,
    comment: bool,
    whitespace_boundary: bool,
    /// The rest of the line is dropped unread: after a line comment or a heredoc
    /// opener, and for a line whose class its start decided.
    rest_ignored: bool,
    /// A C string opened in this line: its `multiline` is whether the line ends with a
    /// backslash, which the whole-line classifier read when the string opened and a
    /// piece scan can only read at the line's end.
    c_string_opened: bool,
    /// The terminator probe while the line is a heredoc body.
    heredoc: Option<HeredocProbe>,
}

/// Whether a heredoc body line is its terminator, decided as the line arrives.
///
/// The whole-line classifier compared the line, less its indentation, with the
/// terminator; a piece scan keeps the indentation count and at most the terminator's
/// length of what follows, and a line already longer than that can never match.
#[derive(Debug, Default)]
struct HeredocProbe {
    indent_done: bool,
    mismatch: bool,
    candidate: Vec<u8>,
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

const UTF8_BOM: &[u8] = &[0xef, 0xbb, 0xbf];

impl CodeAccumulator {
    /// Create a counter for one stable file-type ID, or return `None` when
    /// `code-sloc-v1` does not claim that language.
    pub fn for_type(file_type: &str) -> Option<Self> {
        Self::with_window_bytes(file_type, LINE_WINDOW_BYTES)
    }

    /// [`Self::for_type`] with the window a line grows to before a piece is classified.
    fn with_window_bytes(file_type: &str, window_bytes: usize) -> Option<Self> {
        let syntax = Syntax::for_type(file_type)?;
        let window_bytes = window_bytes.max(1);
        Some(Self {
            syntax,
            state: State::Normal,
            window: Vec::new(),
            consumed: 0,
            last_dropped: None,
            next_scan_at: window_bytes,
            window_bytes,
            #[cfg(test)]
            peak_window: 0,
            previous_cr: false,
            line: LineScan::default(),
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
                _ => {
                    self.window.push(byte);
                    if self.window.len() >= self.next_scan_at {
                        self.scan(false);
                    }
                }
            }
        }
    }

    /// Finish an unterminated final line and return its additive metrics.
    pub fn finish(mut self) -> MetricValues {
        let bom_only = self.consumed == 0 && self.window == UTF8_BOM;
        if self.consumed + self.window.len() > 0 && !bom_only {
            self.finish_line();
        }
        self.metrics
    }

    /// The largest window the accumulator has held, in bytes.
    #[cfg(test)]
    fn peak_window(&self) -> usize {
        self.peak_window
    }

    fn finish_line(&mut self) {
        self.scan(true);
        let ends_with_backslash = self.window.last().copied().or(self.last_dropped) == Some(b'\\');
        if self.syntax.language == Language::C && self.line.c_string_opened {
            if let State::Quoted { quote: b'"', multiline, .. } = &mut self.state {
                *multiline = ends_with_backslash;
            }
        }
        match &mut self.state {
            State::Quoted { multiline: false, escaped: true, .. }
                if matches!(self.syntax.language, Language::C | Language::Cpp) =>
            {
                self.state =
                    State::Quoted { quote: b'"', escaped: false, multiline: false, doubled: false };
            }
            State::Quoted { multiline: false, .. } | State::Regex { .. } => {
                self.state = State::Normal;
            }
            State::Quoted { multiline: true, escaped, .. } => *escaped = false,
            _ => {}
        }
        self.metrics.physical_lines = self.metrics.physical_lines.saturating_add(1);
        if self.line.code {
            self.metrics.code_lines = self.metrics.code_lines.saturating_add(1);
        } else if self.line.comment {
            self.metrics.comment_lines = self.metrics.comment_lines.saturating_add(1);
        } else {
            self.metrics.code_blank_lines = self.metrics.code_blank_lines.saturating_add(1);
        }
        self.window.clear();
        self.consumed = 0;
        self.last_dropped = None;
        self.next_scan_at = self.window_bytes;
        self.line = LineScan::default();
    }

    /// Classify what the window holds, as far as it can be decided.
    ///
    /// `final_piece` says the line ends here, so every site decides on what there is,
    /// as the whole-line classifier did; otherwise a site that would read past the
    /// window waits for more bytes.
    fn scan(&mut self, final_piece: bool) {
        #[cfg(test)]
        {
            self.peak_window = self.peak_window.max(self.window.len());
        }
        if !self.line.started && !self.start_line(final_piece) {
            self.next_scan_at = self.window.len().saturating_mul(2).max(self.window_bytes);
            return;
        }
        if self.line.rest_ignored {
            self.drop_window();
        } else if self.line.heredoc.is_some() {
            self.probe_heredoc(final_piece);
        } else {
            let piece = scan_piece(
                self.syntax,
                &mut self.state,
                &mut self.javascript,
                &mut self.shell,
                &mut self.line,
                &self.window,
                final_piece,
            );
            let stalled = piece.consumed == 0 && !final_piece;
            self.drop_consumed(piece.consumed);
            if piece.rest_ignored {
                self.line.rest_ignored = true;
                self.drop_window();
            }
            if stalled {
                // A token the window's edge cuts, such as a run of raw-string hashes:
                // rescan after the window has grown, not after every byte.
                self.next_scan_at = self.window.len().saturating_mul(2).max(self.window_bytes);
                return;
            }
        }
        self.next_scan_at = self.window.len().saturating_add(self.window_bytes);
    }

    /// The decisions the whole-line classifier made before scanning bytes: the line's
    /// flags, a heredoc body, a Ruby block line, and a leading byte-order mark.
    ///
    /// Returns `false` when they need more of the line than the window holds.
    fn start_line(&mut self, final_piece: bool) -> bool {
        // `=begin` is the longest of the line-start patterns.
        if !final_piece && self.window.len() < 6 {
            return false;
        }
        self.line.started = true;
        self.line.whitespace_boundary = matches!(self.state, State::Normal);
        self.line.code = matches!(
            self.state,
            State::Quoted { .. }
                | State::TripleQuoted { .. }
                | State::RustRaw { .. }
                | State::Delimited { .. }
                | State::Heredoc { .. }
                | State::RubyPercent { .. }
                | State::Regex { .. }
        );
        self.line.comment = matches!(self.state, State::BlockComment { .. });
        if matches!(self.state, State::Heredoc { .. }) {
            self.line.heredoc = Some(HeredocProbe::default());
            return true;
        }
        if matches!(self.state, State::RubyBlock) {
            if self.window.starts_with(b"=end") {
                self.state = State::Normal;
            }
            self.line.comment = true;
            self.line.rest_ignored = true;
            return true;
        }
        if self.syntax.ruby_blocks
            && matches!(self.state, State::Normal)
            && self.window.starts_with(b"=begin")
        {
            self.state = State::RubyBlock;
            self.line.comment = true;
            self.line.rest_ignored = true;
            return true;
        }
        if self.window.starts_with(UTF8_BOM) {
            self.line.bom = true;
            self.drop_consumed(UTF8_BOM.len());
        }
        true
    }

    /// Feed the window to the heredoc terminator probe and drop it.
    fn probe_heredoc(&mut self, final_piece: bool) {
        let State::Heredoc { terminator, indent, php } = &self.state else {
            unreachable!("the probe runs in heredoc state only");
        };
        let (terminator, indent, php) = (terminator.clone(), *indent, *php);
        let shell = self.syntax.language == Language::Shell;
        let probe = self.line.heredoc.as_mut().expect("a heredoc line has its probe");
        let mut rest: &[u8] = &self.window;
        if indent && !probe.indent_done {
            let indentation = rest
                .iter()
                .take_while(|byte| if shell { **byte == b'\t' } else { byte.is_ascii_whitespace() })
                .count();
            rest = &rest[indentation..];
            if !rest.is_empty() || final_piece {
                probe.indent_done = true;
            }
        }
        let longest = terminator.len() + usize::from(php);
        if !probe.mismatch {
            if probe.candidate.len() + rest.len() > longest {
                probe.mismatch = true;
                probe.candidate = Vec::new();
            } else {
                probe.candidate.extend_from_slice(rest);
            }
        }
        if final_piece
            && !probe.mismatch
            && (probe.candidate == terminator
                || (php && probe.candidate.strip_suffix(b";") == Some(terminator.as_slice())))
        {
            self.state = State::Normal;
        }
        self.drop_window();
    }

    fn drop_window(&mut self) {
        self.drop_consumed(self.window.len());
    }

    fn drop_consumed(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        self.last_dropped = Some(self.window[count - 1]);
        self.consumed += count;
        self.window.drain(..count);
    }
}

/// What a piece scan came to.
struct PieceScan {
    /// Bytes of the window the scan decided; the rest wait for more bytes or the end.
    consumed: usize,
    /// The rest of the line is dropped unread: a line comment or a heredoc opener.
    rest_ignored: bool,
}

/// Whether `rest` is a proper prefix of `token`: more bytes could complete it.
fn prefix_of(rest: &[u8], token: &[u8]) -> bool {
    rest.len() < token.len() && token.starts_with(rest)
}

fn is_tag_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Whether `rest`, cut by the window's edge, could still become a C++ raw-string opener.
fn cpp_raw_could_continue(rest: &[u8]) -> bool {
    if prefix_of(rest, b"R\"") {
        return true;
    }
    let Some(delimiter) = rest.strip_prefix(b"R\"") else { return false };
    delimiter.len() <= 16
        && !delimiter.iter().any(|b| b.is_ascii_whitespace() || matches!(b, b'\\' | b'(' | b')'))
}

/// Whether `rest`, cut by the window's edge, could still become a SQL dollar-quote opener.
fn sql_dollar_could_continue(rest: &[u8]) -> bool {
    let Some(tag) = rest.strip_prefix(b"$") else { return false };
    tag.first().is_none_or(|first| first.is_ascii_alphabetic() || *first == b'_')
        && tag.iter().all(|byte| is_tag_byte(*byte))
}

/// Whether `rest`, cut by the window's edge, could still become a heredoc opener.
fn heredoc_could_continue(language: Language, rest: &[u8]) -> bool {
    let php = language == Language::Php;
    if !matches!(language, Language::Ruby | Language::Shell | Language::Php) {
        return false;
    }
    let opener: &[u8] = if php { b"<<<" } else { b"<<" };
    if prefix_of(rest, opener) {
        return true;
    }
    let Some(mut tail) = rest.strip_prefix(opener) else { return false };
    if !php && matches!(tail.first(), Some(b'-' | b'~')) {
        tail = &tail[1..];
    }
    if matches!(tail.first(), Some(b'\'' | b'"')) {
        tail = &tail[1..];
    }
    tail.iter().all(|byte| is_tag_byte(*byte))
}

/// Classify the bytes `window` holds of the current line, from its start, as far as
/// they can be decided.
///
/// This is the whole-line classifier's loop, with one addition at every site that
/// looks ahead: when the bytes it would read end at the window's edge and the line goes
/// on (`final_piece` false), it stops and the scan resumes there with more bytes. At the
/// line's end every site decides on what there is, so the decisions are the ones the
/// whole line gives.
fn scan_piece(
    syntax: Syntax,
    state: &mut State,
    javascript: &mut JavaScriptContext,
    shell: &mut ShellContext,
    line: &mut LineScan,
    window: &[u8],
    final_piece: bool,
) -> PieceScan {
    let mut index = 0;
    let mut rest_ignored = false;
    // Whether a site needing `need` bytes must wait for them.
    let short = |index: usize, need: usize| !final_piece && window.len() - index < need;

    'scan: while index < window.len() {
        let rest = &window[index..];
        if let State::Delimited { close } = state {
            line.code = true;
            if rest.starts_with(close) {
                index += close.len();
                *state = State::Normal;
            } else if !final_piece && prefix_of(rest, close) {
                break;
            } else {
                index += 1;
            }
            continue;
        }
        match state.clone() {
            State::BlockComment { mut depth } => {
                line.comment = true;
                let block = syntax.block.expect("block state requires block syntax");
                if syntax.nested_blocks && rest.starts_with(block.open) {
                    depth = depth.saturating_add(1);
                    *state = State::BlockComment { depth };
                    index += block.open.len();
                } else if rest.starts_with(block.close) {
                    depth = depth.saturating_sub(1);
                    *state = if depth == 0 { State::Normal } else { State::BlockComment { depth } };
                    index += block.close.len();
                } else if !final_piece
                    && (prefix_of(rest, block.close)
                        || (syntax.nested_blocks && prefix_of(rest, block.open)))
                {
                    break;
                } else {
                    index += 1;
                }
            }
            State::Quoted { quote, mut escaped, multiline, doubled } => {
                line.code = true;
                let byte = rest[0];
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == quote {
                    if doubled {
                        if short(index, 2) {
                            break;
                        }
                        if rest.get(1) == Some(&quote) {
                            index += 2;
                            continue;
                        }
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
                line.code = true;
                if rest.len() >= width && rest[..width].iter().all(|b| *b == quote) {
                    *state = State::Normal;
                    if syntax.language == Language::JavaScript {
                        javascript.regex_allowed = false;
                    }
                    index += width;
                } else if !final_piece && rest.len() < width && rest.iter().all(|b| *b == quote) {
                    break;
                } else {
                    index += 1;
                }
            }
            State::RustRaw { hashes } => {
                line.code = true;
                if rust_raw_close(rest, hashes) {
                    *state = State::Normal;
                    index += usize::from(hashes) + 1;
                } else if !final_piece
                    && rest.len() < usize::from(hashes) + 1
                    && rest.first() == Some(&b'"')
                    && rest[1..].iter().all(|b| *b == b'#')
                {
                    break;
                } else {
                    index += 1;
                }
            }
            State::Delimited { .. } => {
                unreachable!("delimited strings are handled before matching")
            }
            State::RubyPercent { open, close, mut depth } => {
                line.code = true;
                match rest[0] {
                    b'\\' => {
                        if short(index, 2) {
                            break;
                        }
                        index += usize::min(2, rest.len());
                    }
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
                line.code = true;
                let byte = rest[0];
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
            State::Heredoc { .. } => unreachable!("heredoc lines are probed, not scanned"),
            State::RubyBlock => unreachable!("Ruby block lines are decided at their start"),
            State::Normal => {
                let byte = rest[0];
                if byte.is_ascii_whitespace() {
                    line.whitespace_boundary = true;
                    index += 1;
                    continue;
                }
                let utf8_width = match byte {
                    0xc2..=0xdf => Some(2),
                    0xe0..=0xef => Some(3),
                    0xf0..=0xf4 => Some(4),
                    _ => None,
                };
                if let Some(width) = utf8_width {
                    if short(index, width) {
                        break;
                    }
                    if let Some((character, width)) = leading_utf8_character(rest) {
                        if super::content_basic_metrics::is_content_whitespace(character) {
                            line.whitespace_boundary = true;
                            index += width;
                            continue;
                        }
                    }
                }
                if syntax.language == Language::Shell {
                    if shell.arithmetic_parens == 0 {
                        if rest.starts_with(b"$((") {
                            shell.arithmetic_parens = 2;
                            line.code = true;
                            line.whitespace_boundary = false;
                            index += 3;
                            continue;
                        }
                        if !final_piece && prefix_of(rest, b"$((") {
                            break;
                        }
                    }
                    if shell.arithmetic_parens == 0 && line.whitespace_boundary {
                        if rest.starts_with(b"((") {
                            shell.arithmetic_parens = 2;
                            line.code = true;
                            line.whitespace_boundary = false;
                            index += 2;
                            continue;
                        }
                        if !final_piece && prefix_of(rest, b"((") {
                            break;
                        }
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
                {
                    if short(index, 2) {
                        break;
                    }
                    if !rest.starts_with(b"//") && !rest.starts_with(b"/*") {
                        line.code = true;
                        line.whitespace_boundary = false;
                        *state = State::Regex { escaped: false, in_class: false };
                        index += 1;
                        continue;
                    }
                }
                let marker_admitted = !syntax.shell_hash_boundary || line.whitespace_boundary;
                if syntax
                    .line_comments
                    .iter()
                    .any(|marker| rest.starts_with(marker) && marker_admitted)
                {
                    line.comment = true;
                    rest_ignored = true;
                    index = window.len();
                    break 'scan;
                }
                if !final_piece
                    && syntax
                        .line_comments
                        .iter()
                        .any(|marker| prefix_of(rest, marker) && marker_admitted)
                {
                    break;
                }
                if let Some(block) = syntax.block {
                    if rest.starts_with(block.open) {
                        line.comment = true;
                        line.whitespace_boundary = false;
                        *state = State::BlockComment { depth: 1 };
                        index += block.open.len();
                        continue;
                    }
                    if !final_piece && prefix_of(rest, block.open) {
                        break;
                    }
                }
                if syntax.rust_raw_strings {
                    if let Some((hashes, consumed)) = rust_raw_open(rest) {
                        line.code = true;
                        line.whitespace_boundary = false;
                        *state = State::RustRaw { hashes };
                        index += consumed;
                        continue;
                    }
                    if !final_piece && byte == b'r' && rest[1..].iter().all(|b| *b == b'#') {
                        break;
                    }
                }
                if syntax.language == Language::Cpp {
                    if let Some((close, consumed)) = cpp_raw_open(rest) {
                        line.code = true;
                        *state = State::Delimited { close };
                        index += consumed;
                        continue;
                    }
                    if !final_piece && cpp_raw_could_continue(rest) {
                        break;
                    }
                }
                if syntax.language == Language::Sql {
                    if let Some((close, consumed)) = sql_dollar_open(rest) {
                        line.code = true;
                        *state = State::Delimited { close };
                        index += consumed;
                        continue;
                    }
                    if !final_piece && sql_dollar_could_continue(rest) {
                        break;
                    }
                }
                if syntax.language == Language::Ruby {
                    if let Some((open, close, consumed)) = ruby_percent_open(rest) {
                        line.code = true;
                        *state = State::RubyPercent { open, close, depth: 1 };
                        index += consumed;
                        continue;
                    }
                    if !final_piece && byte == b'%' && rest.len() < 3 {
                        break;
                    }
                }
                if syntax.language != Language::Shell || shell.arithmetic_parens == 0 {
                    // Before the opener is read: a tag the window's edge cuts would read
                    // as a shorter tag.
                    if !final_piece && heredoc_could_continue(syntax.language, rest) {
                        break;
                    }
                    if let Some((terminator, indent, php)) = heredoc_open(syntax.language, rest) {
                        line.code = true;
                        *state = State::Heredoc { terminator, indent, php };
                        rest_ignored = true;
                        index = window.len();
                        break 'scan;
                    }
                }
                if syntax.language == Language::CSharp {
                    if rest.starts_with(b"@\"") {
                        line.code = true;
                        *state = State::Quoted {
                            quote: b'"',
                            escaped: false,
                            multiline: true,
                            doubled: true,
                        };
                        index += 2;
                        continue;
                    }
                    if !final_piece && rest == b"@" {
                        break;
                    }
                }
                if matches!(syntax.language, Language::CSharp | Language::Java) && byte == b'"' {
                    if short(index, 3) {
                        break;
                    }
                    if rest.starts_with(b"\"\"\"") {
                        let run = rest.iter().take_while(|b| **b == b'"').count();
                        if syntax.language == Language::CSharp && !final_piece && run == rest.len()
                        {
                            break;
                        }
                        line.code = true;
                        let width = if syntax.language == Language::CSharp { run } else { 3 };
                        *state = State::TripleQuoted { quote: b'"', width };
                        index += width;
                        continue;
                    }
                }
                if syntax.triple_quotes && matches!(byte, b'\'' | b'"') {
                    if short(index, 3) {
                        break;
                    }
                    if rest.starts_with(&[byte, byte, byte]) {
                        line.code = true;
                        line.whitespace_boundary = false;
                        *state = State::TripleQuoted { quote: byte, width: 3 };
                        index += 3;
                        continue;
                    }
                }
                if matches!(byte, b'\'' | b'"') || (syntax.backtick_strings && byte == b'`') {
                    if syntax.language == Language::Rust && byte == b'\'' {
                        if short(index, 3) {
                            break;
                        }
                        if rust_lifetime(rest) {
                            line.code = true;
                            index += 1;
                            continue;
                        }
                    }
                    line.code = true;
                    line.whitespace_boundary = false;
                    // A C string's `multiline` is whether the line ends with a backslash,
                    // which the line's end supplies (`c_string_opened`).
                    let c_string = syntax.language == Language::C && byte == b'"';
                    line.c_string_opened |= c_string;
                    let multiline = byte == b'`'
                        || (byte == b'"' && syntax.language == Language::Rust)
                        || matches!(
                            syntax.language,
                            Language::Ruby | Language::Shell | Language::Sql
                        );
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
                        let run = rest
                            .iter()
                            .take_while(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$'))
                            .count();
                        if !final_piece && run == rest.len() {
                            break;
                        }
                        let word = &rest[..run];
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
                        line.code = true;
                        index += run;
                        continue;
                    }
                    if byte == b'(' {
                        javascript.paren_control.push(javascript.pending_control_paren);
                        javascript.pending_control_paren = false;
                        javascript.after_dot = false;
                        javascript.regex_allowed = true;
                        line.code = true;
                        index += 1;
                        continue;
                    }
                    if byte == b')' {
                        javascript.regex_allowed = javascript.paren_control.pop().unwrap_or(false);
                        javascript.pending_control_paren = false;
                        javascript.after_dot = false;
                        line.code = true;
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
                line.code = true;
                line.whitespace_boundary = syntax.language == Language::Shell
                    && matches!(byte, b';' | b'&' | b'|' | b'(' | b')' | b'<' | b'>');
                index += 1;
            }
        }
    }

    PieceScan { consumed: index, rest_ignored }
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

    /// The whole-line classifier the streaming one replaced, verbatim, as the oracle
    /// every streaming answer is held to.
    #[allow(dead_code, clippy::pedantic)]
    mod reference {
        use super::super::{
            JavaScriptContext, Language, MetricValues, ShellContext, State, Syntax, cpp_raw_open,
            heredoc_open, leading_utf8_character, ruby_percent_open, rust_lifetime, rust_raw_close,
            rust_raw_open, sql_dollar_open,
        };

        /// Streaming `code-sloc-v1` counter for a supported language (analyzer version 3).
        ///
        /// The counter retains the current logical line, its allocated capacity, and parser
        /// state. A one-line minified or generated source can therefore require file-sized
        /// memory per active worker. Mixed
        /// code/comment lines are code, blank lines inside block comments are comments, and
        /// multiline string lines are code. Line endings follow the same LF, CRLF, lone-CR,
        /// and unterminated-final-line contract as the basic analyzer.
        #[derive(Debug)]
        pub(super) struct WholeLineAccumulator {
            syntax: Syntax,
            state: State,
            line: Vec<u8>,
            previous_cr: bool,
            javascript: JavaScriptContext,
            shell: ShellContext,
            metrics: MetricValues,
        }

        impl WholeLineAccumulator {
            /// Create a counter for one stable file-type ID, or return `None` when
            /// `code-sloc-v1` does not claim that language.
            pub(super) fn for_type(file_type: &str) -> Option<Self> {
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
            pub(super) fn push(&mut self, chunk: &[u8]) {
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
            pub(super) fn finish(mut self) -> MetricValues {
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
                        self.metrics.code_blank_lines =
                            self.metrics.code_blank_lines.saturating_add(1);
                    }
                }
                self.line.clear();
            }
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
                            *state = if depth == 0 {
                                State::Normal
                            } else {
                                State::BlockComment { depth }
                            };
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
                            if crate::content::content_basic_metrics::is_content_whitespace(
                                character,
                            ) {
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
                                        shell.arithmetic_parens =
                                            shell.arithmetic_parens.saturating_add(1);
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
                            if let Some((open, close, consumed)) = ruby_percent_open(&line[index..])
                            {
                                code = true;
                                *state = State::RubyPercent { open, close, depth: 1 };
                                index += consumed;
                                continue;
                            }
                        }
                        if let Some((terminator, indent, php)) =
                            (syntax.language != Language::Shell || shell.arithmetic_parens == 0)
                                .then(|| heredoc_open(syntax.language, &line[index..]))
                                .flatten()
                        {
                            code = true;
                            *state = State::Heredoc { terminator, indent, php };
                            break;
                        }
                        if syntax.language == Language::CSharp && line[index..].starts_with(b"@\"")
                        {
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
                        if matches!(byte, b'\'' | b'"') || (syntax.backtick_strings && byte == b'`')
                        {
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
                                    .take_while(|b| {
                                        b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$')
                                    })
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
                                javascript.regex_allowed =
                                    javascript.paren_control.pop().unwrap_or(false);
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
                    *state = State::Quoted {
                        quote: b'"',
                        escaped: false,
                        multiline: false,
                        doubled: false,
                    };
                }
                State::Quoted { multiline: false, .. } | State::Regex { .. } => {
                    *state = State::Normal
                }
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
    }

    fn streaming(language: &str, window: usize, chunks: &[&[u8]]) -> (MetricValues, usize) {
        let mut counter =
            CodeAccumulator::with_window_bytes(language, window).expect("supported language");
        for chunk in chunks {
            counter.push(chunk);
        }
        let peak = counter.peak_window();
        (counter.finish(), peak)
    }

    fn whole(language: &str, chunks: &[&[u8]]) -> MetricValues {
        let mut counter =
            reference::WholeLineAccumulator::for_type(language).expect("supported language");
        for chunk in chunks {
            counter.push(chunk);
        }
        counter.finish()
    }

    const LANGUAGES: [&str; 15] = [
        "rust",
        "javascript",
        "typescript",
        "go",
        "c",
        "cpp",
        "csharp",
        "java",
        "kotlin",
        "swift",
        "php",
        "python",
        "ruby",
        "shell",
        "sql",
    ];

    /// Every construct the classifier knows, in one source fed to every language: what a
    /// construct means to a language the reference decides, and the streaming scan has
    /// to agree byte for byte. Line endings, splices, a byte-order mark on a later line,
    /// invalid UTF-8, Unicode whitespace, and no trailing newline are all in it.
    fn everything() -> Vec<u8> {
        let mut source = Vec::new();
        let lines: &[&[u8]] = &[
            b"// line comment\r\n",
            b"# hash comment\n",
            b"-- dash comment\r",
            b"code(); // trailing\n",
            b"/* block\n",
            b"still block */ code /* again */\n",
            b"/* outer /* inner */ still */ done\n",
            b"let s = \"quoted \\\" escape // not comment\"; // yes\n",
            b"let c = 'x'; let d = '\\''; /* tail\n",
            b"*/\n",
            b"SELECT 'it''s' -- doubled\n",
            b"x = \"\"\"triple\n",
            b"# inside\n",
            b"\"\"\" # after\n",
            b"y = '''one line''' # after\n",
            b"let r = r#\"raw \"# // not\"#; // comment\n",
            b"let r2 = r####\"deep\"### still\"####; // comment\n",
            b"auto s = R\"x(raw // not\n",
            b")x\"; // comment\n",
            b"auto t = R\"toolongdelimiterforcpp(nope)\"; // comment\n",
            b"SELECT $tag$dollar -- not\n",
            b"$tag$; -- comment\n",
            b"SELECT $$anon$$; -- comment\n",
            b"SELECT $bad-tag$; -- comment\n",
            b"s = %q{outer {inner} # not\n",
            b"} # comment\n",
            b"w = %w[a b] # comment\n",
            b"cat <<TEXT\n",
            b"# heredoc body\n",
            b"TEXT\n",
            b"cat <<-TEXT\n",
            b"\t# indented body\n",
            b"\tTEXT\n",
            b"cat <<~TEXT # ruby\n",
            b"  TEXT\n",
            b"cat <<'QUOTED' # x\n",
            b"QUOTED\n",
            b"$s = <<<PHP\n",
            b"// body\n",
            b"PHP;\n",
            b"x=$((1<<2)) # comment\n",
            b"((1<<3)) # comment\n",
            b"a=1;# comment after semicolon\n",
            b"printf foo#bar # comment\n",
            b"=begin\n",
            b"ruby block\n",
            b"=end\n",
            b"const re = /[/*]/; // comment\n",
            b"const q = total / count; /* comment */\n",
            b"if (ok) /[/*]/.test(v); next();\n",
            b"return /\\/\\* x [/] /; // c\n",
            b"var t = `tick // not\n",
            b"` // comment\n",
            b"var v = @\"verbatim \"\" // not\n",
            b"\"; // comment\n",
            b"var j = \"\"\"\n",
            b"// java text block\n",
            b"\"\"\"; // comment\n",
            b"var cs = \"\"\"\"\"\n",
            b"// five quotes\n",
            b"\"\"\"\"\"; // comment\n",
            b"fn f<'a>(x: &'a str) -> &'a str { x } // lifetime\n",
            b"let ch = 'a'; // char\n",
            b"const char *s = \"spliced \\\n",
            b"// continued\";\n",
            b"const char *t = \"open\\\n",
            b"\n",
            b"\xef\xbb\xbfbom line // comment\n",
            b"\xef\xbb\xbf\n",
            b"\xff\xfe invalid utf8 // comment\n",
            b"\xe3\x80\x80// ideographic space then comment\n",
            b"\xe2\x80\x83code after em space\n",
            b"\xe3\x80\n",
            b"\r\n",
            b"\n",
            b"   \n",
            b"trailing backslash \\\n",
            b"last line without newline",
        ];
        for line in lines {
            source.extend_from_slice(line);
        }
        source
    }

    /// Tokens the window's edge can cut mid-way, each longer than any small window:
    /// raw-string hashes, a dollar-quote tag, a heredoc tag, a JavaScript identifier, a
    /// C# quote run, and a delimited closer the scan must wait for whole.
    fn long_tokens() -> Vec<u8> {
        let mut source = Vec::new();
        let hashes = "#".repeat(200);
        source.extend_from_slice(format!("let r = r{hashes}\"x // y\"{hashes}; // c\n").as_bytes());
        let tag = "t".repeat(300);
        source.extend_from_slice(format!("SELECT ${tag}$body -- not\n").as_bytes());
        source.extend_from_slice(format!("more ${tag}$; -- comment\n").as_bytes());
        source.extend_from_slice(format!("cat <<{tag}\n# body\n{tag}\n").as_bytes());
        source.extend_from_slice(format!("cat <<'{tag}'\n# body\n{tag}\n").as_bytes());
        let word = "w".repeat(400);
        source.extend_from_slice(format!("if ({word}) /[/*]/.test(v); // c\n").as_bytes());
        source.extend_from_slice(format!("{word}do /re/ // c\n").as_bytes());
        let quotes = "\"".repeat(40);
        source.extend_from_slice(format!("var s = {quotes}\n// text\n{quotes}; // c\n").as_bytes());
        source.extend_from_slice(b"auto s = R\"abcdefghijklmnop(raw)abcdefghijklmnop\"; // c\n");
        source.extend_from_slice(b"%q{ // tail\n");
        source
    }

    /// The streaming scan agrees with the whole-line classifier for every language,
    /// with a piece edge at every byte, with every chunking, and at every window size.
    #[test]
    fn piecewise_classification_agrees_with_the_whole_line_classifier_everywhere() {
        let mut both = everything();
        both.extend_from_slice(&long_tokens());
        let sources = [everything(), long_tokens(), both];
        for language in LANGUAGES {
            for source in &sources {
                let expected = whole(language, &[source]);
                for window in [1, 2, 3, 5, 7, 16, 61, 4096, LINE_WINDOW_BYTES] {
                    let (actual, _) = streaming(language, window, &[source]);
                    assert_eq!(actual, expected, "{language}: window {window}");
                }
                for split in (0..=source.len()).step_by(7) {
                    let chunks: [&[u8]; 2] = [&source[..split], &source[split..]];
                    let (actual, _) = streaming(language, 1, &chunks);
                    assert_eq!(actual, expected, "{language}: split {split}, window 1");
                    let (actual, _) = streaming(language, 3, &chunks);
                    assert_eq!(actual, expected, "{language}: split {split}, window 3");
                }
            }
        }
    }

    /// Every chunk boundary of a small mixed source, at window 1, for every language.
    #[test]
    fn every_chunk_boundary_of_a_mixed_source_agrees() {
        let source = b"a = \"q\\\"\" /* c\n */ r#\"x\"# 'y' $t$z$t$ <<T\nT\n// d\n\xef\xbb\xbf%q{w}\r\n\xe3\x80\x80e";
        for language in LANGUAGES {
            let expected = whole(language, &[source]);
            for first in 0..=source.len() {
                for second in first..=source.len() {
                    let chunks: [&[u8]; 3] =
                        [&source[..first], &source[first..second], &source[second..]];
                    let (actual, _) = streaming(language, 1, &chunks);
                    assert_eq!(actual, expected, "{language}: splits {first}, {second}");
                }
            }
        }
    }

    /// A long line is classified in pieces as it arrives, and the window never holds
    /// more than its bound plus what a piece cannot yet decide.
    #[test]
    fn a_long_line_costs_the_window_not_the_line() {
        let segment = everything();
        let mut line: Vec<u8> = Vec::new();
        while line.len() < 3 * LINE_WINDOW_BYTES {
            line.extend(
                segment.iter().map(
                    |byte| {
                        if matches!(*byte, b'\n' | b'\r') { b' ' } else { *byte }
                    },
                ),
            );
        }
        line.push(b'\n');
        for language in LANGUAGES {
            let expected = whole(language, &[&line]);
            let (actual, peak) = streaming(language, LINE_WINDOW_BYTES, &[&line]);
            assert_eq!(actual, expected, "{language}");
            assert!(
                peak < 2 * LINE_WINDOW_BYTES,
                "{language}: the window held {peak} bytes of a {}-byte line",
                line.len()
            );
        }
    }

    /// The bound this exists for: a single 64 MiB line of code holds at most the window
    /// and one read chunk, not the line (fdu-1zb6).
    #[test]
    fn a_sixty_four_mebibyte_line_holds_at_most_the_window_and_a_chunk() {
        const CHUNK: usize = 64 * 1024;
        let piece: &[u8] = b"var a = 1; /* c */ b = \"s\"; ";
        let mut chunk = Vec::with_capacity(CHUNK + piece.len());
        while chunk.len() < CHUNK {
            chunk.extend_from_slice(piece);
        }
        chunk.truncate(CHUNK);
        let mut counter = CodeAccumulator::for_type("javascript").expect("javascript is supported");
        let chunks = (64 * 1024 * 1024) / CHUNK;
        for _ in 0..chunks {
            counter.push(&chunk);
        }
        counter.push(b"\n");
        let peak = counter.peak_window();
        let metrics = counter.finish();
        assert_eq!((metrics.physical_lines, metrics.code_lines), (1, 1));
        assert!(
            peak <= LINE_WINDOW_BYTES + CHUNK,
            "the window held {peak} bytes of a {} MiB line",
            chunks * CHUNK / (1024 * 1024)
        );
    }

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
