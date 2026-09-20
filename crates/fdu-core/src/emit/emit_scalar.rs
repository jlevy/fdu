//! String policies shared by the JSON and YAML sinks.

use std::fmt;

/// Whether YAML 1.1 and 1.2 both resolve `value` as the same string.
pub(crate) fn is_plain_safe(value: &str) -> bool {
    let dot_safe = !value.starts_with('.')
        || value.starts_with("./")
        || value.starts_with("../")
        || value.as_bytes().get(1).is_some_and(u8::is_ascii_alphabetic);
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '/' | '-' | '+'))
        && !value.starts_with(|ch: char| ch.is_ascii_digit() || matches!(ch, '-' | '+'))
        && dot_safe
        && !matches!(
            value.to_ascii_lowercase().as_str(),
            "true"
                | "false"
                | "null"
                | "yes"
                | "no"
                | "on"
                | "off"
                | "y"
                | "n"
                | "~"
                | ".inf"
                | ".nan"
        )
}

/// Append one JSON string, escaping the YAML-forbidden characters too.
pub(crate) fn write_json_string(out: &mut impl fmt::Write, text: &str) {
    let _ = out.write_char('"');
    for ch in text.chars() {
        match ch {
            '"' => {
                let _ = out.write_str("\\\"");
            }
            '\\' => {
                let _ = out.write_str("\\\\");
            }
            '\u{8}' => {
                let _ = out.write_str("\\b");
            }
            '\u{c}' => {
                let _ = out.write_str("\\f");
            }
            '\n' => {
                let _ = out.write_str("\\n");
            }
            '\r' => {
                let _ = out.write_str("\\r");
            }
            '\t' => {
                let _ = out.write_str("\\t");
            }
            ch if (ch as u32) < 0x20
                || ('\u{7f}'..='\u{9f}').contains(&ch)
                || matches!(ch, '\u{2028}' | '\u{2029}' | '\u{feff}' | '\u{fffe}' | '\u{ffff}') =>
            {
                let _ = write!(out, "\\u{:04x}", ch as u32);
            }
            ch => {
                let _ = out.write_char(ch);
            }
        }
    }
    let _ = out.write_char('"');
}

/// Append one scalar that round-trips as a string in strict YAML 1.1 and 1.2.
pub(crate) fn write_yaml_scalar(out: &mut impl fmt::Write, text: &str) {
    if is_plain_safe(text) {
        let _ = out.write_str(text);
    } else {
        write_json_string(out, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emit::{Event, Scalar, Shape, Sink, YamlSink};

    #[test]
    fn json_escapes_every_character_yaml_forbids() {
        let mut out = String::new();
        write_json_string(&mut out, "a\u{0}\u{7f}\u{85}\u{2028}\u{2029}\u{feff}\u{fffe}\u{ffff}z");
        assert_eq!(out, "\"a\\u0000\\u007f\\u0085\\u2028\\u2029\\ufeff\\ufffe\\uffffz\"");
    }

    #[test]
    fn strict_plain_policy_quotes_yaml_implicit_types() {
        for value in [
            "",
            "0",
            "+1",
            "-1",
            "0x10",
            "1_000",
            "0b101",
            "0o17",
            ".inf",
            ".NaN",
            "true",
            "Y",
            "n",
            "yes",
            "OFF",
            "null",
            "~",
            "has space",
            "2024-01-01",
        ] {
            assert!(!is_plain_safe(value), "{value:?} must be quoted");
        }
        for value in ["./src", "../src", "src", "main.rs", ".hidden", "a-b_c/d.e"] {
            assert!(is_plain_safe(value), "{value:?} may stay plain");
        }
        for value in [".", ".1", "._1", "..", "..."] {
            assert!(!is_plain_safe(value), "{value:?} is conservatively quoted");
        }
    }

    #[test]
    fn conformance_corpus_uses_only_plain_safe_or_escaped_scalars() {
        const CORPUS: &[&str] = &[
            "0x10",
            "1_000",
            "0b101",
            "0o17",
            "012",
            ".inf",
            "-.inf",
            ".NaN",
            "1e3",
            "y",
            "n",
            "Y",
            "yes",
            "on",
            "off",
            "~",
            "null",
            "true",
            "2024-01-01",
            "12:30:00",
            "",
            " lead",
            "trail ",
            "a: b",
            "#hash",
            "- dash",
            "?q",
            "*star",
            "&amp",
            "!bang",
            "%pct",
            "@at",
            "`tick",
            "'quote",
            "\"dq",
            "multi\nline",
            "tab\tx",
            "del\u{7f}name",
            "nel\u{85}name",
            "c1\u{9b}name",
            "ls\u{2028}name",
            "bom\u{feff}",
            "nonchar\u{fffe}",
            "é unicode",
            "emoji 🙂",
            "plain-name.txt",
            ".",
            ".hidden",
            "src/main.rs",
            "._1",
            "._",
            ".1_0",
            ".5",
            "1.",
            "+1",
            "0",
            "-1",
            "1:30",
            "190:20:30",
            "0755",
            "1e+3",
            "1E3",
            "6.8523015e+5",
            "0.1.0",
            "True",
            "TRUE",
            "No",
            "NO",
            "Off",
            "N",
            "Null",
            "NULL",
            "=",
            "<<",
            "inf",
            "nan",
            "Infinity",
            "2001-12-14t21:59:43.10-05:00",
            "2002-12-14",
            "...",
            "---",
            "-",
            "?",
            ":",
            "!",
            "|",
            ">",
            "{a}",
            "[a]",
            "a,b",
            "a #b",
            "a:b",
            "key:",
            "a\\b",
            "back\\n",
            "%",
            "\ttab-lead",
            "trail\t",
            "x\r\ny",
            "cr\rx",
            "multi\nline\n",
            "a\n\nb",
            "\nlead-nl",
            "trail \nx",
            "  \n  ",
            " ",
            "\u{a0}nbsp",
            "ps\u{2029}name",
            "vt\u{b}x",
            "nul\0x",
            "esc\u{1b}x",
            "\u{feff}lead-bom",
            "nonchar\u{ffff}",
            "astral 𝔘",
            "combining e\u{301}",
            "rtl \u{202e}",
            "zwsp\u{200b}",
            "é",
            "ñ",
            "long path long path long path long path long path long path long path long path long path long path long path long path ",
            "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        ];
        assert_eq!(CORPUS.len(), 121);
        for value in CORPUS {
            let mut rendered = String::new();
            write_yaml_scalar(&mut rendered, value);
            if is_plain_safe(value) {
                assert_eq!(rendered, *value, "plain scalar changed {value:?}");
            } else {
                assert!(rendered.starts_with('"') && rendered.ends_with('"'), "{value:?}");
            }
            assert!(
                !rendered.chars().any(|ch| ('\u{7f}'..='\u{9f}').contains(&ch)
                    || matches!(
                        ch,
                        '\u{2028}' | '\u{2029}' | '\u{feff}' | '\u{fffe}' | '\u{ffff}'
                    )),
                "forbidden YAML character survived in {value:?}"
            );
        }

        let mut sink = YamlSink::new();
        sink.event(Event::BeginSeq(Shape::Block));
        for value in CORPUS {
            sink.event(Event::Scalar(Scalar::Str(value)));
        }
        sink.event(Event::EndSeq);
        let yaml = sink.finish();
        assert_eq!(yaml, include_str!("../testdata/yaml-scalar-corpus.yaml"));
    }
}
