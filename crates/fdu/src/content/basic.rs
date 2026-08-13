//! Fused streaming admission, line, word, paragraph, and logical-word statistics.

use super::MetricValues;

/// Outcome of the basic text-admission pass.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextAdmission {
    /// Valid UTF-8 without a NUL byte.
    Accepted(MetricValues),
    /// A NUL byte appeared anywhere in the stream.
    Binary,
    /// The byte stream was not valid UTF-8.
    InvalidUtf8,
}

/// Stateful fused counter that accepts arbitrary byte chunks.
#[derive(Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct BasicAccumulator {
    metrics: MetricValues,
    utf8_carry: Vec<u8>,
    collect_logical: bool,
    binary: bool,
    invalid_utf8: bool,
    first_character: bool,
    current_line_exists: bool,
    current_line_nonblank: bool,
    previous_cr: bool,
    raw_word_open: bool,
    nonwide_word_open: bool,
    paragraph_open: bool,
}

impl BasicAccumulator {
    /// Create an empty streaming counter.
    pub fn new() -> Self {
        Self::with_logical_metrics(false)
    }

    /// Select the optional logical-word and paragraph collectors.
    pub(crate) fn with_logical_metrics(collect_logical: bool) -> Self {
        Self { collect_logical, first_character: true, ..Self::default() }
    }

    /// Consume another byte chunk.
    pub fn push(&mut self, chunk: &[u8]) {
        if self.binary || self.invalid_utf8 {
            return;
        }
        if chunk.contains(&0) {
            self.binary = true;
            self.utf8_carry.clear();
            return;
        }

        let mut joined = Vec::with_capacity(self.utf8_carry.len() + chunk.len());
        joined.extend_from_slice(&self.utf8_carry);
        joined.extend_from_slice(chunk);
        self.utf8_carry.clear();

        match std::str::from_utf8(&joined) {
            Ok(text) => self.push_text(text),
            Err(error) => {
                let valid = error.valid_up_to();
                if let Ok(text) = std::str::from_utf8(&joined[..valid]) {
                    self.push_text(text);
                }
                if error.error_len().is_some() {
                    self.invalid_utf8 = true;
                } else {
                    self.utf8_carry.extend_from_slice(&joined[valid..]);
                }
            }
        }
    }

    /// Finish the stream, accounting for an unterminated final line and word.
    pub fn finish(mut self) -> TextAdmission {
        if self.binary {
            return TextAdmission::Binary;
        }
        if self.invalid_utf8 || !self.utf8_carry.is_empty() {
            return TextAdmission::InvalidUtf8;
        }
        if self.current_line_exists {
            self.finish_line();
        }
        if self.raw_word_open {
            self.metrics.raw_words = self.metrics.raw_words.saturating_add(1);
        }
        TextAdmission::Accepted(self.metrics)
    }

    fn push_text(&mut self, text: &str) {
        for character in text.chars() {
            if self.first_character {
                self.first_character = false;
                if character == '\u{feff}' {
                    self.current_line_exists = true;
                    continue;
                }
            }

            if self.previous_cr {
                self.previous_cr = false;
                if character == '\n' {
                    continue;
                }
            }
            match character {
                '\r' => {
                    self.current_line_exists = true;
                    self.finish_line();
                    self.previous_cr = true;
                }
                '\n' => {
                    self.current_line_exists = true;
                    self.finish_line();
                }
                _ => self.push_character(character),
            }
        }
    }

    fn push_character(&mut self, character: char) {
        self.current_line_exists = true;
        if character.is_whitespace() {
            if self.raw_word_open {
                self.metrics.raw_words = self.metrics.raw_words.saturating_add(1);
                self.raw_word_open = false;
            }
            self.nonwide_word_open = false;
            return;
        }

        self.current_line_nonblank = true;
        self.raw_word_open = true;
        if !self.collect_logical {
            return;
        }
        if is_wide(character) {
            self.metrics.logical_word_stats.wide_chars =
                self.metrics.logical_word_stats.wide_chars.saturating_add(1);
            self.nonwide_word_open = false;
        } else {
            self.metrics.logical_word_stats.nonwide_chars =
                self.metrics.logical_word_stats.nonwide_chars.saturating_add(1);
            if !self.nonwide_word_open {
                self.metrics.logical_word_stats.nonwide_tokens =
                    self.metrics.logical_word_stats.nonwide_tokens.saturating_add(1);
                self.nonwide_word_open = true;
            }
        }
    }

    fn finish_line(&mut self) {
        if self.raw_word_open {
            self.metrics.raw_words = self.metrics.raw_words.saturating_add(1);
            self.raw_word_open = false;
        }
        self.metrics.physical_lines = self.metrics.physical_lines.saturating_add(1);
        if self.current_line_nonblank {
            self.metrics.nonblank_lines = self.metrics.nonblank_lines.saturating_add(1);
            if self.collect_logical && !self.paragraph_open {
                self.metrics.paragraphs = self.metrics.paragraphs.saturating_add(1);
                self.paragraph_open = true;
            }
        } else {
            self.metrics.blank_lines = self.metrics.blank_lines.saturating_add(1);
            if self.collect_logical {
                self.paragraph_open = false;
            }
        }
        self.current_line_exists = false;
        self.current_line_nonblank = false;
        self.nonwide_word_open = false;
    }
}

fn is_wide(character: char) -> bool {
    matches!(
        character as u32,
        0x1100..=0x115f
            | 0x2329..=0x232a
            | 0x2e80..=0xa4cf
            | 0xac00..=0xd7a3
            | 0xf900..=0xfaff
            | 0xfe10..=0xfe19
            | 0xfe30..=0xfe6f
            | 0xff00..=0xff60
            | 0xffe0..=0xffe6
            | 0x1f300..=0x1faff
            | 0x20000..=0x3fffd
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accepted(chunks: &[&[u8]]) -> MetricValues {
        let mut accumulator = BasicAccumulator::with_logical_metrics(true);
        for chunk in chunks {
            accumulator.push(chunk);
        }
        match accumulator.finish() {
            TextAdmission::Accepted(metrics) => metrics,
            other => panic!("expected accepted text, got {other:?}"),
        }
    }

    #[test]
    fn line_endings_and_terminal_boundaries_have_one_contract() {
        for input in ["a\nb\n", "a\r\nb\r\n", "a\rb\r", "a\r\nb\rc\n"] {
            let metrics = accepted(&[input.as_bytes()]);
            let expected = if input.contains('c') { 3 } else { 2 };
            assert_eq!(metrics.physical_lines, expected, "{input:?}");
            assert_eq!(metrics.nonblank_lines, expected, "{input:?}");
            assert_eq!(metrics.blank_lines, 0, "{input:?}");
        }
        assert_eq!(accepted(&[b""]).physical_lines, 0);
        assert_eq!(accepted(&[b"one line"]).physical_lines, 1);
        assert_eq!(accepted(&[b"one line\n"]).physical_lines, 1);
        assert_eq!(accepted(&[b"\n"]).blank_lines, 1);
    }

    #[test]
    fn every_chunk_boundary_matches_the_one_chunk_result() {
        let input = "\u{feff}alpha\r\n\u{3000}\r中文 beta\nlast".as_bytes();
        let expected = accepted(&[input]);
        for split in 0..=input.len() {
            assert_eq!(accepted(&[&input[..split], &input[split..]]), expected, "split {split}");
        }
        for first in 0..=input.len() {
            for second in first..=input.len() {
                assert_eq!(
                    accepted(&[&input[..first], &input[first..second], &input[second..]]),
                    expected,
                    "splits {first}, {second}"
                );
            }
        }
    }

    #[test]
    fn unicode_whitespace_words_paragraphs_and_logical_stats_are_additive() {
        let metrics = accepted(&["one two\n\n中文\u{3000}longtoken\n".as_bytes()]);
        assert_eq!(metrics.physical_lines, 3);
        assert_eq!(metrics.blank_lines, 1);
        assert_eq!(metrics.nonblank_lines, 2);
        assert_eq!(metrics.raw_words, 4);
        assert_eq!(metrics.paragraphs, 2);
        assert_eq!(
            metrics.logical_word_stats,
            super::super::LogicalWordStats { wide_chars: 2, nonwide_tokens: 3, nonwide_chars: 15 }
        );
    }

    #[test]
    fn early_or_late_nul_and_invalid_utf8_discard_provisional_metrics() {
        let mut early = BasicAccumulator::new();
        early.push(b"\0text");
        assert_eq!(early.finish(), TextAdmission::Binary);

        let mut late = BasicAccumulator::new();
        late.push(b"valid\nlines\n");
        late.push(b"late\0binary");
        assert_eq!(late.finish(), TextAdmission::Binary);

        let mut invalid = BasicAccumulator::new();
        invalid.push(&[b'a', 0xff]);
        assert_eq!(invalid.finish(), TextAdmission::InvalidUtf8);
    }

    #[test]
    fn basic_mode_keeps_raw_words_but_omits_deeper_document_metrics() {
        let mut accumulator = BasicAccumulator::new();
        accumulator.push(b"one two\n\nthree\n");
        let TextAdmission::Accepted(metrics) = accumulator.finish() else {
            panic!("expected accepted text");
        };
        assert_eq!(metrics.raw_words, 3);
        assert_eq!(metrics.paragraphs, 0);
        assert_eq!(metrics.logical_word_stats, super::super::LogicalWordStats::default());
    }
}
