//! File-type recognition.
//!
//! Exact filenames and extensions are resolved from repository-owned declarative rules
//! compiled at build time. Callers that explicitly enabled content analysis may also
//! supply a bounded prefix for binary signatures, shebangs, modelines, ambiguous
//! headers, and origin flags. Nothing here performs runtime rule parsing or opens a
//! file; the caller owns the optional read.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt;
use std::path::Path;
use std::sync::LazyLock;

mod file_type_detection;

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

    pub(crate) fn from_cache(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for FileTypeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Which bounded step established a classification.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum DetectionSource {
    /// An exact basename rule matched.
    ExactFilename,
    /// A compound extension such as `.tar.gz` matched.
    CompoundExtension,
    /// An ordinary extension matched.
    Extension,
    /// An unresolved text file's interpreter matched a shebang rule.
    Shebang,
    /// A modeline in an unresolved text file named the language.
    Modeline,
    /// A required literal resolved an explicitly ambiguous extension.
    AmbiguousContent,
    /// A named binary or textual format signature matched.
    FormatSignature,
    /// A bounded prefix established a binary family.
    ContentProbe,
    /// No known rule matched; an extension, when present, is preserved in the type id.
    Unknown,
}

impl DetectionSource {
    /// Stable machine label used by reports and caches.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactFilename => "exact_filename",
            Self::CompoundExtension => "compound_extension",
            Self::Extension => "extension",
            Self::Shebang => "shebang",
            Self::Modeline => "modeline",
            Self::AmbiguousContent => "ambiguous_content",
            Self::FormatSignature => "format_signature",
            Self::ContentProbe => "content_probe",
            Self::Unknown => "unknown",
        }
    }
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

impl DetectionConfidence {
    /// Stable machine label used by reports and caches.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Certain => "certain",
            Self::High => "high",
            Self::Heuristic => "heuristic",
        }
    }
}

/// Orthogonal attributes discovered from bounded path and prefix checks.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ClassificationFlags {
    /// The prefix contains a conventional generated-file marker.
    pub generated: bool,
    /// A path component names a conventional vendored dependency tree.
    pub vendored: bool,
    /// The path names a conventional documentation tree or document basename.
    pub documentation: bool,
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
    /// Orthogonal origin and purpose attributes.
    pub flags: ClassificationFlags,
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

/// The exact-name and extension tiers, resolved once instead of scanned per file.
///
/// The scan these replace was
/// `GENERATED_RULES.iter().filter(..).max_by_key(..)`, and `max_by_key` consumes the
/// whole iterator, so every file paid all 65 rules and all 167 extension strings even
/// when its extension matched the first rule in the table. Classification ran twice per
/// file on a warm content open -- once to build each candidate, once again in
/// `Index::apply_analysis`'s staleness guard -- so this is charged for on both.
///
/// `Iterator::max_by_key` returns the *last* of equally-maximum elements. The builder
/// reproduces that tie-break by letting a later rule in table order win at equal
/// priority; changing it would silently reclassify the types that share a key.
static RULES_BY_FILENAME: LazyLock<HashMap<&'static str, &'static GeneratedRule>> =
    LazyLock::new(|| index_rules(|rule| rule.filenames));
static RULES_BY_EXTENSION: LazyLock<HashMap<&'static str, &'static GeneratedRule>> =
    LazyLock::new(|| index_rules(|rule| rule.extensions));

fn index_rules(
    keys: impl Fn(&'static GeneratedRule) -> &'static [&'static str],
) -> HashMap<&'static str, &'static GeneratedRule> {
    let mut table: HashMap<&'static str, &'static GeneratedRule> = HashMap::new();
    for rule in GENERATED_RULES {
        for key in keys(rule) {
            match table.get(key) {
                Some(existing) if existing.priority > rule.priority => {}
                _ => {
                    table.insert(key, rule);
                }
            }
        }
    }
    table
}

/// Fingerprint of the repository-owned rule manifest compiled into this build.
pub const fn type_rule_fingerprint() -> u64 {
    TYPE_RULE_FINGERPRINT
}

/// Human-facing name for a stable code-type identifier.
///
/// Reports and caches retain the identifier; only terminal language views use this
/// presentation layer. Unknown identifiers are returned unchanged.
pub(crate) fn human_language_name(id: &str) -> &str {
    match id {
        "rust" => "Rust",
        "python" => "Python",
        "javascript" => "JavaScript",
        "typescript" => "TypeScript",
        "go" => "Go",
        "c" => "C",
        "cpp" => "C++",
        "csharp" => "C#",
        "java" => "Java",
        "kotlin" => "Kotlin",
        "swift" => "Swift",
        "ruby" => "Ruby",
        "php" => "PHP",
        "shell" => "Shell",
        "powershell" => "PowerShell",
        "lua" => "Lua",
        "perl" => "Perl",
        "r" => "R",
        "dart" => "Dart",
        "scala" => "Scala",
        "haskell" => "Haskell",
        "elixir" => "Elixir",
        "erlang" => "Erlang",
        "clojure" => "Clojure",
        "fsharp" => "F#",
        "ocaml" => "OCaml",
        "objective-c" => "Objective-C",
        "julia" => "Julia",
        "zig" => "Zig",
        "nim" => "Nim",
        "solidity" => "Solidity",
        "assembly" => "Assembly",
        "sql" => "SQL",
        "make" => "Make",
        "dockerfile" => "Dockerfile",
        "cmake" => "CMake",
        "protobuf" => "Protocol Buffers",
        "terraform" => "Terraform",
        "nix" => "Nix",
        "css" => "CSS",
        _ => id,
    }
}

/// Classify from path metadata only, without opening the file.
pub fn classify_path(path: &Path) -> Classification {
    classify_path_with_prefix(path, None)
}

/// Classify with an optional bounded content prefix.
///
/// Known exact-name and ordinary extension matches avoid deep detection. The `.h`
/// ambiguity and generated-file flag alone inspect their documented bounded prefix.
/// Callers may pass a larger buffer; all content-dependent helpers enforce smaller
/// internal limits.
pub fn classify_path_with_prefix(path: &Path, prefix: Option<&[u8]>) -> Classification {
    let name = path.file_name().unwrap_or_else(|| OsStr::new(""));
    // The rules table is pure ASCII, so a name that is not UTF-8 matched no rule
    // filename under the byte comparison this replaces either.
    if let Some(rule) = name.to_str().and_then(|name| RULES_BY_FILENAME.get(name).copied()) {
        return with_flags(
            path,
            prefix,
            classified(rule, DetectionSource::ExactFilename, DetectionConfidence::Certain),
        );
    }

    let extension = derive_ext(name);
    if let Some(extension) = extension.as_deref() {
        let key = extension.strip_prefix('.').expect("derived extensions start with a dot");
        if let Some(rule) = RULES_BY_EXTENSION.get(key).copied() {
            let source = if key.contains('.') {
                DetectionSource::CompoundExtension
            } else {
                DetectionSource::Extension
            };
            let classification = if key == "h" {
                prefix
                    .and_then(file_type_detection::resolve_c_header)
                    .and_then(rule_by_id)
                    .map_or_else(
                        || classified(rule, source, DetectionConfidence::Certain),
                        |cpp| {
                            classified(
                                cpp,
                                DetectionSource::AmbiguousContent,
                                DetectionConfidence::High,
                            )
                        },
                    )
            } else {
                classified(rule, source, DetectionConfidence::Certain)
            };
            return with_flags(path, prefix, classification);
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
        flags: ClassificationFlags::default(),
    };
    let Some(prefix) = prefix else {
        return with_flags(path, None, unknown());
    };
    let probed = file_type_detection::probe_unresolved(prefix);
    match probed {
        Some(file_type_detection::PrefixMatch::UnknownBinary) => {
            return with_flags(
                path,
                Some(prefix),
                Classification {
                    family: ContentFamily::Binary,
                    source: DetectionSource::ContentProbe,
                    ..unknown()
                },
            );
        }
        Some(file_type_detection::PrefixMatch::Rule(id, source))
            if rule_by_id(id).is_some_and(|rule| rule.family == ContentFamily::Binary) =>
        {
            let rule = rule_by_id(id).expect("guard established a generated rule");
            return with_flags(
                path,
                Some(prefix),
                classified(rule, source, DetectionConfidence::High),
            );
        }
        _ => {}
    }
    if let Some(interpreter) = file_type_detection::shebang_interpreter(prefix) {
        if let Some(rule) = GENERATED_RULES
            .iter()
            .filter(|rule| rule.shebangs.contains(&interpreter))
            .max_by_key(|rule| rule.priority)
        {
            return with_flags(
                path,
                Some(prefix),
                classified(rule, DetectionSource::Shebang, DetectionConfidence::High),
            );
        }
    }
    let classification = match probed {
        Some(file_type_detection::PrefixMatch::Rule(id, source)) => rule_by_id(id)
            .map_or_else(unknown, |rule| classified(rule, source, DetectionConfidence::High)),
        Some(file_type_detection::PrefixMatch::UnknownBinary) => {
            unreachable!("returned above")
        }
        None => unknown(),
    };
    with_flags(path, Some(prefix), classification)
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
        flags: ClassificationFlags::default(),
    }
}

fn rule_by_id(id: &str) -> Option<&'static GeneratedRule> {
    GENERATED_RULES.iter().find(|rule| rule.id == id)
}

fn with_flags(
    path: &Path,
    prefix: Option<&[u8]>,
    mut classification: Classification,
) -> Classification {
    classification.flags = file_type_detection::flags(path, prefix);
    classification
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
/// use fdu_core::classify::derive_ext;
///
/// assert_eq!(derive_ext(OsStr::new("archive.tar.gz")).as_deref(), Some(".tar.gz"));
/// assert_eq!(derive_ext(OsStr::new("notes.MD")).as_deref(), Some(".md"));
/// assert_eq!(derive_ext(OsStr::new(".gitignore")), None);
/// assert_eq!(derive_ext(OsStr::new("README")), None);
/// ```
pub fn derive_ext(name: &OsStr) -> Option<String> {
    derive_ext_native(name)
}

/// Label of the extension bucket a file belongs to, including the one for no extension.
///
/// [`derive_ext`] answers "what is this name's extension", and `None` is the right answer
/// for `Makefile`. A roll-up asks a different question — "which pile does this file's
/// bytes go on" — and every file belongs on some pile. Dropping the `None` case meant the
/// extension view's rows did not sum to the tree it was reporting on: a 263-byte fixture
/// came back as three rows totalling 235, the missing 28 being a `Makefile`, and nothing
/// in the output said so.
///
/// [`derive_ext`] always yields a leading dot, so this label cannot collide with a real
/// extension however the tree is named.
///
/// ```
/// use std::ffi::OsStr;
/// use fdu_core::classify::{NO_EXTENSION, ext_bucket};
///
/// assert_eq!(ext_bucket(OsStr::new("archive.tar.gz")), ".tar.gz");
/// assert_eq!(ext_bucket(OsStr::new("Makefile")), NO_EXTENSION);
/// assert_eq!(ext_bucket(OsStr::new(".gitignore")), NO_EXTENSION);
/// ```
pub fn ext_bucket(name: &OsStr) -> String {
    derive_ext(name).unwrap_or_else(|| NO_EXTENSION.to_string())
}

/// Extension-view label for files whose name carries no extension.
///
/// Parenthesised so it reads as a category rather than as a filename, and dot-free so it
/// cannot be mistaken for — or collide with — an extension [`derive_ext`] produced.
pub const NO_EXTENSION: &str = "(none)";

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
        // Lowercase the units that are single bytes and leave the rest alone. Asking
        // `try_from` whether it fits says that directly, where a comparison plus an
        // `expect` made the caller argue the bound was already checked.
        match u8::try_from(unit) {
            Ok(byte) => u16::from(byte.to_ascii_lowercase()),
            Err(_) => unit,
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
        ContentFamily, DetectionConfidence, DetectionSource, RULES_BY_EXTENSION, RULES_BY_FILENAME,
        classify_path, classify_path_with_prefix, derive_ext, type_rule_fingerprint,
    };
    use super::{GENERATED_RULES, human_language_name};
    use std::ffi::OsStr;
    use std::path::Path;

    /// The indexed tiers must answer exactly what the scan they replaced answered.
    ///
    /// The scan was `filter(..).max_by_key(priority)`, and `max_by_key` returns the
    /// *last* of equally-maximum elements. Several keys are claimed by more than one
    /// rule, so a builder that took the first winner instead of the last would
    /// reclassify them silently, with no other test noticing.
    #[test]
    fn indexed_rule_tiers_agree_with_the_scan_they_replaced() {
        for (table, keys_of) in [
            (&*RULES_BY_FILENAME, (|rule: &super::GeneratedRule| rule.filenames) as fn(_) -> _),
            (&*RULES_BY_EXTENSION, (|rule: &super::GeneratedRule| rule.extensions) as fn(_) -> _),
        ] {
            let mut checked = 0;
            for key in GENERATED_RULES.iter().flat_map(keys_of) {
                let scanned = GENERATED_RULES
                    .iter()
                    .filter(|rule| keys_of(rule).contains(key))
                    .max_by_key(|rule| rule.priority)
                    .expect("the key came from a rule, so at least one rule matches");
                let indexed = table.get(*key).expect("every rule key is indexed");
                assert_eq!(
                    indexed.id, scanned.id,
                    "key {key:?} resolves to {:?} but the scan chose {:?}",
                    indexed.id, scanned.id
                );
                checked += 1;
            }
            assert!(checked > 0, "the rules table produced no keys to check");
        }
    }

    /// A name that is not valid UTF-8 must classify as unknown, as it did when the tier
    /// compared raw `OsStr` bytes against an all-ASCII rules table.
    #[test]
    fn non_utf8_names_still_reach_the_unknown_tier() {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let name = OsStr::from_bytes(b"weird\xff\xfename");
            let classification = classify_path(Path::new(name));
            assert_eq!(classification.source, DetectionSource::Unknown);
            assert_eq!(classification.family, ContentFamily::Unknown);
        }
    }

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
    fn every_code_rule_has_a_canonical_human_language_name() {
        for rule in GENERATED_RULES.iter().filter(|rule| rule.family == ContentFamily::Code) {
            assert_ne!(
                human_language_name(rule.id),
                rule.id,
                "code rule {} needs a human-facing language name",
                rule.id
            );
        }

        assert_eq!(human_language_name("css"), "CSS");
        assert_eq!(human_language_name("cpp"), "C++");
        assert_eq!(human_language_name("csharp"), "C#");
        assert_eq!(human_language_name("javascript"), "JavaScript");
        assert_eq!(human_language_name("powershell"), "PowerShell");
        assert_eq!(human_language_name("protobuf"), "Protocol Buffers");
        assert_eq!(human_language_name("unknown"), "unknown");
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

        let spoofed = classify_path_with_prefix(
            Path::new("payload"),
            Some(b"#!/usr/bin/env python3\ntext\0binary"),
        );
        assert_eq!(spoofed.family, ContentFamily::Binary);
        assert_eq!(spoofed.source, DetectionSource::ContentProbe);
    }

    #[test]
    fn bounded_deep_detection_is_explainable() {
        let c = classify_path_with_prefix(Path::new("include/value.h"), Some(b"int value;\n"));
        assert_eq!(c.file_type.as_str(), "c");
        assert_eq!(c.source, DetectionSource::Extension);

        let cpp = classify_path_with_prefix(
            Path::new("include/value.h"),
            Some(b"namespace demo { constexpr int value = 1; }\n"),
        );
        assert_eq!(cpp.file_type.as_str(), "cpp");
        assert_eq!(cpp.source, DetectionSource::AmbiguousContent);
        assert_eq!(cpp.confidence, DetectionConfidence::High);

        let modeline = classify_path_with_prefix(
            Path::new("script.unknown"),
            Some(b"# vim: set filetype=rust:\nfn main() {}\n"),
        );
        assert_eq!(modeline.file_type.as_str(), "rust");
        assert_eq!(modeline.source, DetectionSource::Modeline);

        let xml = classify_path_with_prefix(
            Path::new("document.unknown"),
            Some(b"<?xml version=\"1.0\"?><root/>"),
        );
        assert_eq!(xml.file_type.as_str(), "xml");
        assert_eq!(xml.source, DetectionSource::FormatSignature);

        let manual = classify_path_with_prefix(
            Path::new("fdu.1"),
            Some(b".TH FDU 1\n.SH NAME\nfdu - disk usage\n"),
        );
        assert_eq!(manual.file_type.as_str(), "manpage");

        let pdf = classify_path_with_prefix(Path::new("download"), Some(b"%PDF-1.7\n"));
        assert_eq!(pdf.file_type.as_str(), "pdf");
        assert_eq!(pdf.family, ContentFamily::Binary);
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
