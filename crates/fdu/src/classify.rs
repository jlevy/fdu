//! File-type recognition.
//!
//! Phase 1 ships only the cheapest tier of the planned cascade: the compound-tail
//! extension. The full design is a priority-ordered cascade of declarative rules
//! (exact filename, extension, shebang, bounded content probe) compiled to automata at
//! build time, deliberately compatible with metabrowser's `[[kind]]` manifest dialect.
//! Nothing here should grow runtime rule parsing; that is what the compiled dialect is
//! for.

use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

/// Broad analysis family for a recognized file type.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ContentFamily {
    /// Programming and configuration languages with code-like comment syntax.
    Code,
    /// Human-authored prose.
    Prose,
    /// Mixed markup whose reader-visible text may need projection.
    Markup,
    /// Structured textual or binary data.
    Data,
    /// Known binary formats that text analyzers must not open.
    Binary,
    /// No family was established by the bounded cascade.
    Unknown,
}

impl ContentFamily {
    /// Stable machine label used by reports and caches.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Prose => "prose",
            Self::Markup => "markup",
            Self::Data => "data",
            Self::Binary => "binary",
            Self::Unknown => "unknown",
        }
    }
}

/// Stable identifier for a known or preserved unknown file type.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FileTypeId(String);

impl FileTypeId {
    /// Borrow the stable machine label.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FileTypeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Which bounded step established a classification.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DetectionSource {
    /// An exact basename rule matched.
    ExactFilename,
    /// A compound extension such as `.tar.gz` matched.
    CompoundExtension,
    /// An ordinary extension matched.
    Extension,
    /// An unresolved text file's interpreter matched a shebang rule.
    Shebang,
    /// A bounded prefix established a binary family.
    ContentProbe,
    /// No known rule matched; an extension, when present, is preserved in the type id.
    Unknown,
}

/// Coarse confidence attached to a classification decision.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum DetectionConfidence {
    /// Exact filename, extension, or known binary format.
    Certain,
    /// A conventional shebang interpreter.
    High,
    /// A bounded content heuristic rather than a named format rule.
    Heuristic,
}

/// Result of the cheapest-first file-type cascade.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Classification {
    /// Stable known id or `unknown:.ext` for an unrecognized extension.
    pub file_type: FileTypeId,
    /// Broad analyzer family.
    pub family: ContentFamily,
    /// Rule tier that produced this result.
    pub source: DetectionSource,
    /// Strength of the evidence used.
    pub confidence: DetectionConfidence,
}

#[derive(Clone, Copy)]
struct GeneratedRule {
    id: &'static str,
    family: ContentFamily,
    extensions: &'static [&'static str],
    filenames: &'static [&'static str],
    shebangs: &'static [&'static str],
    priority: u16,
}

include!(concat!(env!("OUT_DIR"), "/file_type_rules.rs"));

const SHEBANG_PROBE_BYTES: usize = 200;

/// Fingerprint of the repository-owned rule manifest compiled into this build.
pub const fn type_rule_fingerprint() -> u64 {
    TYPE_RULE_FINGERPRINT
}

/// Classify from path metadata only, without opening the file.
pub fn classify_path(path: &Path) -> Classification {
    classify_path_with_prefix(path, None)
}

/// Classify with an optional bounded content prefix.
///
/// Known exact-name and extension matches return before inspecting `prefix`. Callers may
/// pass a larger buffer; only the documented shebang prefix is consulted here.
pub fn classify_path_with_prefix(path: &Path, prefix: Option<&[u8]>) -> Classification {
    let name = path.file_name().unwrap_or_else(|| OsStr::new(""));
    if let Some(rule) = GENERATED_RULES
        .iter()
        .filter(|rule| rule.filenames.iter().any(|candidate| name == OsStr::new(candidate)))
        .max_by_key(|rule| rule.priority)
    {
        return classified(rule, DetectionSource::ExactFilename, DetectionConfidence::Certain);
    }

    let extension = derive_ext(name);
    if let Some(extension) = extension.as_deref() {
        let key = extension.strip_prefix('.').expect("derived extensions start with a dot");
        if let Some(rule) = GENERATED_RULES
            .iter()
            .filter(|rule| rule.extensions.contains(&key))
            .max_by_key(|rule| rule.priority)
        {
            let source = if key.contains('.') {
                DetectionSource::CompoundExtension
            } else {
                DetectionSource::Extension
            };
            return classified(rule, source, DetectionConfidence::Certain);
        }
    }

    let unknown = || Classification {
        file_type: FileTypeId(
            extension
                .as_deref()
                .map_or_else(|| "unknown".to_string(), |ext| format!("unknown:{ext}")),
        ),
        family: ContentFamily::Unknown,
        source: DetectionSource::Unknown,
        confidence: DetectionConfidence::Heuristic,
    };
    let Some(prefix) = prefix else {
        return unknown();
    };
    let prefix = &prefix[..prefix.len().min(SHEBANG_PROBE_BYTES)];
    if prefix.contains(&0) {
        return Classification {
            family: ContentFamily::Binary,
            source: DetectionSource::ContentProbe,
            ..unknown()
        };
    }
    if let Some(interpreter) = shebang_interpreter(prefix) {
        if let Some(rule) = GENERATED_RULES
            .iter()
            .filter(|rule| rule.shebangs.contains(&interpreter))
            .max_by_key(|rule| rule.priority)
        {
            return classified(rule, DetectionSource::Shebang, DetectionConfidence::High);
        }
    }
    unknown()
}

fn classified(
    rule: &GeneratedRule,
    source: DetectionSource,
    confidence: DetectionConfidence,
) -> Classification {
    Classification {
        file_type: FileTypeId(rule.id.to_string()),
        family: rule.family,
        source,
        confidence,
    }
}

fn shebang_interpreter(prefix: &[u8]) -> Option<&str> {
    let line = prefix.split(|byte| matches!(byte, b'\r' | b'\n')).next()?;
    let command = line.strip_prefix(b"#!")?;
    let mut tokens = std::str::from_utf8(command).ok()?.split_ascii_whitespace();
    let executable = tokens.next()?.rsplit('/').next()?;
    if executable != "env" {
        return Some(executable);
    }
    tokens.find(|token| !token.starts_with('-')).and_then(|token| token.rsplit('/').next())
}

/// Extract the compound-tail extension from a file name, lowercased and including the
/// leading dot.
///
/// "Compound tail" means `archive.tar.gz` yields `.tar.gz` rather than `.gz`, because
/// the pair is what a human means by the file's type. Only `.tar` is folded this way;
/// generalizing to an arbitrary set of compound stems belongs in the rule dialect, not
/// in a hand-maintained list here.
///
/// Returns `None` for names with no usable extension, including dotfiles like
/// `.gitignore` — a leading dot marks a hidden file, it does not introduce an extension.
///
/// ```
/// use std::ffi::OsStr;
/// use fdu::classify::derive_ext;
///
/// assert_eq!(derive_ext(OsStr::new("archive.tar.gz")).as_deref(), Some(".tar.gz"));
/// assert_eq!(derive_ext(OsStr::new("notes.MD")).as_deref(), Some(".md"));
/// assert_eq!(derive_ext(OsStr::new(".gitignore")), None);
/// assert_eq!(derive_ext(OsStr::new("README")), None);
/// ```
pub fn derive_ext(name: &OsStr) -> Option<String> {
    derive_ext_native(name)
}

#[cfg(unix)]
fn derive_ext_native(name: &OsStr) -> Option<String> {
    use std::os::unix::ffi::OsStrExt;

    derive_ext_units(name.as_bytes(), b'.', |unit| unit.to_ascii_lowercase())
        .and_then(|units| String::from_utf8(units).ok())
}

#[cfg(windows)]
fn derive_ext_native(name: &OsStr) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;

    let units: Vec<u16> = name.encode_wide().collect();
    derive_ext_units(&units, u16::from(b'.'), |unit| {
        if unit <= u16::from(u8::MAX) {
            u16::from(u8::try_from(unit).expect("bounded to one byte").to_ascii_lowercase())
        } else {
            unit
        }
    })
    .and_then(|extension| String::from_utf16(&extension).ok())
}

#[cfg(not(any(unix, windows)))]
fn derive_ext_native(name: &OsStr) -> Option<String> {
    derive_ext_str(name.to_str()?)
}

fn derive_ext_units<T: Copy + Eq + From<u8>>(
    name: &[T],
    dot: T,
    lowercase: impl Fn(T) -> T,
) -> Option<Vec<T>> {
    let searchable = if name.first() == Some(&dot) { &name[1..] } else { name };
    let dot_index = searchable.iter().rposition(|unit| *unit == dot)?;
    let (stem, last) = searchable.split_at(dot_index);
    if last.len() <= 1 {
        return None;
    }

    let mut extension = Vec::new();
    if let Some(inner_dot) = stem.iter().rposition(|unit| *unit == dot) {
        let inner = &stem[inner_dot..];
        let tar = [
            dot,
            lowercase_ascii_unit(b't', &lowercase),
            lowercase_ascii_unit(b'a', &lowercase),
            lowercase_ascii_unit(b'r', &lowercase),
        ];
        if inner.len() == tar.len() && inner.iter().copied().map(&lowercase).eq(tar) {
            extension.extend(inner.iter().copied().map(&lowercase));
        }
    }
    extension.extend(last.iter().copied().map(lowercase));
    Some(extension)
}

fn lowercase_ascii_unit<T: Copy + From<u8>>(byte: u8, lowercase: &impl Fn(T) -> T) -> T {
    lowercase(T::from(byte))
}

#[cfg(not(any(unix, windows)))]
fn derive_ext_str(name: &str) -> Option<String> {
    // Skip a leading dot so dotfiles are not read as all-extension.
    let searchable = name.strip_prefix('.').unwrap_or(name);
    let dot = searchable.rfind('.')?;
    let (stem, last) = searchable.split_at(dot);
    if last.len() <= 1 {
        // A trailing dot with nothing after it is not an extension.
        return None;
    }

    if let Some(inner_dot) = stem.rfind('.') {
        if stem[inner_dot..].eq_ignore_ascii_case(".tar") {
            return Some(format!(".tar{}", last.to_ascii_lowercase()));
        }
    }

    Some(last.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::{
        ContentFamily, DetectionConfidence, DetectionSource, classify_path,
        classify_path_with_prefix, derive_ext, type_rule_fingerprint,
    };
    use std::ffi::OsStr;
    use std::path::Path;

    #[test]
    fn plain_extensions_lowercase() {
        assert_eq!(derive_ext(OsStr::new("main.RS")).as_deref(), Some(".rs"));
        assert_eq!(derive_ext(OsStr::new("Photo.JPEG")).as_deref(), Some(".jpeg"));
    }

    #[test]
    fn tar_pairs_fold_into_one_extension() {
        assert_eq!(derive_ext(OsStr::new("archive.tar.gz")).as_deref(), Some(".tar.gz"));
        assert_eq!(derive_ext(OsStr::new("archive.tar.zst")).as_deref(), Some(".tar.zst"));
        assert_eq!(derive_ext(OsStr::new("archive.TAR.BZ2")).as_deref(), Some(".tar.bz2"));
        // Only .tar folds; an unrelated inner segment is not part of the extension.
        assert_eq!(derive_ext(OsStr::new("release.v2.zip")).as_deref(), Some(".zip"));
    }

    #[test]
    fn names_without_a_usable_extension() {
        assert_eq!(derive_ext(OsStr::new("README")), None);
        assert_eq!(derive_ext(OsStr::new(".gitignore")), None);
        assert_eq!(derive_ext(OsStr::new(".bashrc")), None);
        assert_eq!(derive_ext(OsStr::new("trailing.")), None);
        assert_eq!(derive_ext(OsStr::new("")), None);
    }

    #[test]
    fn dotfiles_with_a_real_extension_keep_it() {
        assert_eq!(derive_ext(OsStr::new(".eslintrc.json")).as_deref(), Some(".json"));
    }

    #[test]
    fn compiled_rules_cover_exact_extension_compound_and_unknown_paths() {
        let makefile = classify_path(Path::new("Makefile"));
        assert_eq!(makefile.file_type.as_str(), "make");
        assert_eq!(makefile.source, DetectionSource::ExactFilename);

        let rust = classify_path(Path::new("src/lib.RS"));
        assert_eq!(rust.file_type.as_str(), "rust");
        assert_eq!(rust.family, ContentFamily::Code);
        assert_eq!(rust.source, DetectionSource::Extension);

        let archive = classify_path(Path::new("source.tar.ZST"));
        assert_eq!(archive.file_type.as_str(), "archive");
        assert_eq!(archive.source, DetectionSource::CompoundExtension);

        let unknown = classify_path(Path::new("sample.widget"));
        assert_eq!(unknown.file_type.as_str(), "unknown:.widget");
        assert_eq!(unknown.family, ContentFamily::Unknown);
        assert_ne!(type_rule_fingerprint(), 0);
    }

    #[test]
    fn bounded_prefix_resolves_shebangs_and_unknown_binary_files() {
        let python = classify_path_with_prefix(
            Path::new("script"),
            Some(b"#!/usr/bin/env -S python3 -I\nprint('ok')\n"),
        );
        assert_eq!(python.file_type.as_str(), "python");
        assert_eq!(python.source, DetectionSource::Shebang);
        assert_eq!(python.confidence, DetectionConfidence::High);

        let binary = classify_path_with_prefix(Path::new("payload.unknown"), Some(b"abc\0def"));
        assert_eq!(binary.file_type.as_str(), "unknown:.unknown");
        assert_eq!(binary.family, ContentFamily::Binary);
        assert_eq!(binary.source, DetectionSource::ContentProbe);
    }

    #[cfg(unix)]
    #[test]
    fn ascii_extension_survives_non_unicode_stem() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let name = OsString::from_vec(vec![b'n', 0xff, b'.', b'R', b'S']);
        assert_eq!(derive_ext(&name).as_deref(), Some(".rs"));
    }

    #[cfg(windows)]
    #[test]
    fn ascii_extension_survives_unpaired_wide_stem() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        let name = OsString::from_wide(&[
            u16::from(b'n'),
            0xd800,
            u16::from(b'.'),
            u16::from(b'R'),
            u16::from(b'S'),
        ]);
        assert_eq!(derive_ext(&name).as_deref(), Some(".rs"));
    }
}
