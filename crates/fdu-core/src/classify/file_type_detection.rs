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
    let prefix = bounded(prefix, AMBIGUITY_PROBE_BYTES);
    [b"namespace " as &[u8], b"template<", b"template <", b"std::", b"constexpr "]
        .iter()
        .any(|literal| contains(prefix, literal))
        .then_some("cpp")
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

/// Orthogonal attributes of one path, two of which are also tag rules.
///
/// `vendored` and `documentation` are decided by [`crate::tags`] rather than here, because
/// they are the same fact wearing two hats: a caller asking `--not-tag vendored` and a row
/// reporting `vendored: true` must agree about every file, and two copies of the rule are
/// how they come not to. The classification keeps reporting them unchanged -- a consumer
/// reading the flag needs no tag rule enabled -- and now there is one predicate behind both.
///
/// `generated` stays here alone, and the tag model's own tier check is why: it reads the
/// file's opening bytes, which is [`TagTier::Content`](crate::tags::TagTier::Content), and
/// `TagRules::from_names` refuses that tier rather than turning a metadata walk into a
/// content walk. It is available on the classification of a file whose bytes were read for
/// some other reason, which is the only place it could ever be free.
pub(super) fn flags(path: &Path, prefix: Option<&[u8]>) -> ClassificationFlags {
    let mut flags = ClassificationFlags {
        generated: false,
        vendored: crate::tags::is_vendored(path),
        documentation: crate::tags::is_documentation(path),
    };
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
        let emacs = format!("mode: {alias}");
        let vim_ft = format!("ft={alias}");
        let vim_filetype = format!("filetype={alias}");
        if (lower.contains("-*-") && lower.contains(&emacs))
            || ((lower.contains("vim:") || lower.contains("vi:"))
                && (lower.contains(&vim_ft) || lower.contains(&vim_filetype)))
        {
            return Some(rule);
        }
    }
    None
}

fn first_nonblank_line(prefix: &[u8]) -> Option<&[u8]> {
    prefix
        .split(|byte| matches!(byte, b'\r' | b'\n'))
        .map(<[u8]>::trim_ascii_start)
        .find(|line| !line.is_empty())
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
