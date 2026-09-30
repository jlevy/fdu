//! Bounded content-dependent detection helpers.
//!
//! Every function in this module receives an already-captured prefix and applies its
//! own smaller bound. There are no regular expressions, parsers, or unbounded scans on
//! the classification path.

use std::path::Path;

use super::{ClassificationFlags, DetectionSource};

pub(super) const AMBIGUITY_PROBE_BYTES: usize = 16 * 1024;
const SHEBANG_PROBE_BYTES: usize = 200;
const MODELINE_PROBE_BYTES: usize = 1024;
const GENERATED_PROBE_BYTES: usize = 2048;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum PrefixMatch<'a> {
    Rule(&'a str, DetectionSource),
    UnknownBinary,
}

pub(super) fn resolve_c_header(prefix: &[u8]) -> Option<&'static str> {
    has_cpp_token(bounded(prefix, AMBIGUITY_PROBE_BYTES)).then_some("cpp")
}

/// Whether the source holds a C++ construct where C could not: `namespace` or
/// `template` opening a line, or `std::` or `constexpr` as a whole token, all outside
/// comments and string or character literals.
///
/// A plain substring test called kernel headers C++: `struct pid_namespace *` holds
/// `namespace `, and comments discuss namespaces and templates freely (fdu-0lo2). One
/// pass over the bounded prefix, so it costs less than the five substring scans it
/// replaces.
fn has_cpp_token(source: &[u8]) -> bool {
    const OPENING_A_LINE: [&[u8]; 3] = [b"namespace ", b"template<", b"template <"];
    const AS_A_TOKEN: [&[u8]; 2] = [b"std::", b"constexpr "];
    let is_identifier = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
    let mut at_line_start = true;
    let mut position = 0;
    while position < source.len() {
        let rest = &source[position..];
        match rest[0] {
            b'\n' => {
                at_line_start = true;
                position += 1;
                continue;
            }
            b' ' | b'\t' | b'\r' => {
                position += 1;
                continue;
            }
            b'/' if rest.starts_with(b"//") => {
                position += rest.iter().position(|byte| *byte == b'\n').unwrap_or(rest.len());
                continue;
            }
            b'/' if rest.starts_with(b"/*") => {
                let close = rest[2..].windows(2).position(|pair| pair == b"*/");
                position += close.map_or(rest.len(), |close| close + 4);
                at_line_start = false;
                continue;
            }
            quote @ (b'"' | b'\'') => {
                // A literal ends at its quote, or at the end of its line when it is
                // unterminated, as an apostrophe in an `#error` message is.
                let mut inside = 1;
                while inside < rest.len() && rest[inside] != quote && rest[inside] != b'\n' {
                    inside += if rest[inside] == b'\\' { 2 } else { 1 };
                }
                position += inside.min(rest.len()) + usize::from(rest.get(inside) == Some(&quote));
                at_line_start = false;
                continue;
            }
            _ => {}
        }
        let boundary = position == 0 || !is_identifier(source[position - 1]);
        if boundary
            && ((at_line_start && OPENING_A_LINE.iter().any(|keyword| rest.starts_with(keyword)))
                || AS_A_TOKEN.iter().any(|keyword| rest.starts_with(keyword)))
        {
            return true;
        }
        at_line_start = false;
        position += 1;
    }
    false
}

pub(super) fn probe_unresolved(prefix: &[u8]) -> Option<PrefixMatch<'static>> {
    let prefix = bounded(prefix, AMBIGUITY_PROBE_BYTES);
    if prefix.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(PrefixMatch::Rule("image", DetectionSource::FormatSignature));
    }
    if prefix.starts_with(b"%PDF-") {
        return Some(PrefixMatch::Rule("pdf", DetectionSource::FormatSignature));
    }
    if prefix.starts_with(&[0x1f, 0x8b]) {
        return Some(PrefixMatch::Rule("archive", DetectionSource::FormatSignature));
    }
    if prefix.starts_with(b"PK\x03\x04")
        || prefix.starts_with(b"PK\x05\x06")
        || prefix.starts_with(b"PK\x07\x08")
    {
        return Some(PrefixMatch::Rule("archive", DetectionSource::FormatSignature));
    }
    if prefix.contains(&0) {
        return Some(PrefixMatch::UnknownBinary);
    }
    if prefix.starts_with(b"<?xml") {
        return Some(PrefixMatch::Rule("xml", DetectionSource::FormatSignature));
    }
    if first_nonblank_line(prefix).is_some_and(|line| line.starts_with(b".TH ")) {
        return Some(PrefixMatch::Rule("manpage", DetectionSource::FormatSignature));
    }
    modeline_rule(prefix).map(|rule| PrefixMatch::Rule(rule, DetectionSource::Modeline))
}

pub(super) fn shebang_interpreter(prefix: &[u8]) -> Option<&str> {
    let line =
        bounded(prefix, SHEBANG_PROBE_BYTES).split(|byte| matches!(byte, b'\r' | b'\n')).next()?;
    let command = line.strip_prefix(b"#!")?;
    let mut tokens = std::str::from_utf8(command).ok()?.split_ascii_whitespace();
    let executable = tokens.next()?.rsplit('/').next()?;
    if executable != "env" {
        return Some(executable);
    }
    tokens.find(|token| !token.starts_with('-')).and_then(|token| token.rsplit('/').next())
}

pub(super) fn flags(path: &Path, prefix: Option<&[u8]>) -> ClassificationFlags {
    let mut flags = ClassificationFlags::default();
    for component in path.components() {
        let component = component.as_os_str().to_string_lossy();
        if ["vendor", "vendored", "third_party", "third-party", "node_modules"]
            .iter()
            .any(|name| component.eq_ignore_ascii_case(name))
        {
            flags.vendored = true;
        }
        if ["doc", "docs", "documentation"].iter().any(|name| component.eq_ignore_ascii_case(name))
        {
            flags.documentation = true;
        }
    }
    if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
        flags.documentation |= ["readme", "changelog", "contributing"]
            .iter()
            .any(|stem| name.eq_ignore_ascii_case(stem) || starts_with_stem(name, stem));
    }
    if let Some(prefix) = prefix {
        let prefix = bounded(prefix, GENERATED_PROBE_BYTES);
        flags.generated = [
            b"@generated" as &[u8],
            b"Code generated ",
            b"DO NOT EDIT",
            b"This file is generated",
            b"Automatically generated",
        ]
        .iter()
        .any(|literal| contains(prefix, literal));
    }
    flags
}

fn modeline_rule(prefix: &[u8]) -> Option<&'static str> {
    let text = std::str::from_utf8(bounded(prefix, MODELINE_PROBE_BYTES)).ok()?;
    let lower = text.to_ascii_lowercase();
    let emacs = lower.contains("-*-");
    let vim = lower.contains("vim:") || lower.contains("vi:");
    if !emacs && !vim {
        return None;
    }
    for (alias, rule) in [
        ("rust", "rust"),
        ("python", "python"),
        ("javascript", "javascript"),
        ("typescript", "typescript"),
        ("go", "go"),
        ("c++", "cpp"),
        ("cpp", "cpp"),
        ("c", "c"),
        ("ruby", "ruby"),
        ("shell", "shell"),
        ("sh", "shell"),
        ("markdown", "markdown"),
        ("xml", "xml"),
    ] {
        let names = |key| modeline_names(&lower, key).any(|name| name == alias);
        if (emacs && names("mode: ")) || (vim && (names("ft=") || names("filetype="))) {
            return Some(rule);
        }
    }
    None
}

/// Each language a modeline names after `key`, as the whole token: `mode: conf-colon`
/// names `conf-colon`, not `c`, and `ft=css` names `css`, not `cs` (fdu-d0gc). A token
/// ends at whitespace, `;`, `:`, the end of the text, or the `-*-` that closes an Emacs
/// modeline when nothing separates them.
fn modeline_names<'a>(text: &'a str, key: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    text.match_indices(key).map(move |(at, _)| {
        let value = &text[at + key.len()..];
        let end = value
            .find(|character: char| {
                character.is_ascii_whitespace() || matches!(character, ';' | ':')
            })
            .unwrap_or(value.len());
        let value = &value[..end];
        value.strip_suffix("-*-").unwrap_or(value)
    })
}

fn first_nonblank_line(prefix: &[u8]) -> Option<&[u8]> {
    prefix
        .split(|byte| matches!(byte, b'\r' | b'\n'))
        .map(<[u8]>::trim_ascii_start)
        .find(|line| !line.is_empty())
}

fn starts_with_stem(name: &str, stem: &str) -> bool {
    name.get(..stem.len()).is_some_and(|prefix| prefix.eq_ignore_ascii_case(stem))
        && name.as_bytes().get(stem.len()) == Some(&b'.')
}

fn bounded(bytes: &[u8], limit: usize) -> &[u8] {
    &bytes[..bytes.len().min(limit)]
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::{PrefixMatch, flags, probe_unresolved, resolve_c_header};
    use std::path::Path;

    #[test]
    fn ambiguity_requires_a_cpp_literal_within_the_bound() {
        assert_eq!(resolve_c_header(b"#include <stdio.h>\n"), None);
        assert_eq!(resolve_c_header(b"namespace demo { class Value {}; }"), Some("cpp"));
        let mut late = vec![b' '; super::AMBIGUITY_PROBE_BYTES];
        late.extend_from_slice(b"namespace too_late {}");
        assert_eq!(resolve_c_header(&late), None);
    }

    /// Kernel-style C headers (fdu-0lo2): a C++ keyword inside an identifier, a comment,
    /// or a string literal is not a C++ signal, and `constexpr` and `std::` count only as
    /// whole tokens, `namespace` and `template` only at the start of a line.
    #[test]
    fn c_headers_are_not_cpp_on_identifier_comment_or_string_substrings() {
        let kernel: &[&[u8]] = &[
            b"struct pid_namespace {\n\tstruct pid_namespace *parent;\n\tstruct user_namespace *user_ns;\n};\nextern struct pid_namespace init_pid_ns;\n",
            b"static inline struct pid_namespace *get_pid_ns(struct pid_namespace *ns)\n{\n\treturn ns;\n}\n",
            b"/* Each namespace holds its own template <config>; see std::cout in the docs. */\nint value;\n",
            b"// the namespace of this constexpr thing\nint value;\n",
            b"static const char *label = \"namespace \";\nstatic const char *hint = \"std::list\";\n",
            b"static const char sep = ':';\n/* std::nothing here */\nint template_count;\n",
            b"#define namespace_of(x) ((x)->ns)\n",
            b"int my_std::x; /* not a token: `my_std` */\n",
        ];
        for header in kernel {
            assert_eq!(resolve_c_header(header), None, "{}", header.escape_ascii());
        }

        let cpp: &[&[u8]] = &[
            b"namespace demo {\nclass Value {};\n}\n",
            b"  namespace nested { }\n",
            b"template <typename T>\nstruct Box { T value; };\n",
            b"\ttemplate<class T> T id(T);\n",
            b"#include <string>\nstd::string name();\n",
            b"static constexpr int limit = 4;\n",
            b"/* a comment first */\nnamespace after_comment {}\n",
            b"#include <string>\n::std::size_t count(); // `std::` in a comment alone would not do\n",
        ];
        // A mid-line `namespace` is deliberately not a signal: C may name a field or a
        // variable `namespace`, so only a line opening with it, as a C++ header does,
        // counts. `inline namespace` and `using namespace` are the cost of that.
        assert_eq!(resolve_c_header(b"inline namespace v1 {}\n"), None);
        for header in cpp {
            assert_eq!(resolve_c_header(header), Some("cpp"), "{}", header.escape_ascii());
        }
    }

    /// A modeline names a language as a whole token (fdu-d0gc): `conf-colon` is not `c`,
    /// `css` is not `cs`, and `gomod` is not `go`.
    #[test]
    fn modelines_match_an_alias_as_a_whole_token() {
        let rule = |prefix: &[u8]| match probe_unresolved(prefix) {
            Some(PrefixMatch::Rule(rule, super::DetectionSource::Modeline)) => Some(rule),
            _ => None,
        };
        assert_eq!(rule(b"# -*- coding: utf-8 mode: conf-colon -*-\n[general]\n"), None);
        assert_eq!(rule(b"# -*- mode: conf -*-\n"), None);
        assert_eq!(rule(b"# -*- mode: conf; -*-\n"), None);
        assert_eq!(rule(b"# vim: set ft=cs:\n"), None);
        assert_eq!(rule(b"# vim: set ft=css:\n"), None);
        assert_eq!(rule(b"# vim: ft=cmake\n"), None);
        assert_eq!(rule(b"# vim: filetype=gomod\n"), None);
        assert_eq!(rule(b"# -*- mode: gomod -*-\n"), None);
        assert_eq!(rule(b"# -*- mode: shell-script -*-\n"), None);

        assert_eq!(rule(b"# -*- mode: c -*-\n"), Some("c"));
        assert_eq!(rule(b"// -*- mode: c++; coding: utf-8 -*-\n"), Some("cpp"));
        assert_eq!(rule(b"# -*- mode: python; -*-\n"), Some("python"));
        assert_eq!(rule(b"# -*- mode: go -*-\n"), Some("go"));
        assert_eq!(rule(b"# vim: set ft=c:\n"), Some("c"));
        assert_eq!(rule(b"# vim: ft=go ts=4\n"), Some("go"));
        assert_eq!(rule(b"# vi: filetype=sh\n"), Some("shell"));
        assert_eq!(rule(b"# vim: set filetype=rust:\nfn main() {}\n"), Some("rust"));
        assert_eq!(rule(b"# -*- Mode: Python -*-\n"), Some("python"));
    }

    #[test]
    fn signatures_modelines_and_manpages_are_named() {
        assert_eq!(
            probe_unresolved(b"%PDF-1.7\n"),
            Some(PrefixMatch::Rule("pdf", super::DetectionSource::FormatSignature))
        );
        assert_eq!(
            probe_unresolved(b"# -*- mode: python -*-\nprint('ok')\n"),
            Some(PrefixMatch::Rule("python", super::DetectionSource::Modeline))
        );
        assert_eq!(
            probe_unresolved(b"\n.TH FDU 1\n.SH NAME\n"),
            Some(PrefixMatch::Rule("manpage", super::DetectionSource::FormatSignature))
        );
    }

    #[test]
    fn flags_are_bounded_and_path_aware() {
        let detected = flags(
            Path::new("third_party/docs/generated.rs"),
            Some(b"// Code generated by fixture; DO NOT EDIT.\n"),
        );
        assert!(detected.generated);
        assert!(detected.vendored);
        assert!(detected.documentation);
    }
}
