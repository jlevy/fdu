//! File-type recognition.
//!
//! Exact filenames and extensions are resolved from a [`TypeRegistry`]; the repository
//! default is compiled at build time, and callers may parse a replacement once at setup.
//! Classification may also use a caller-supplied bounded prefix for binary signatures,
//! shebangs, modelines, ambiguous headers, and origin flags. Classification never parses
//! rules or opens a file; the caller owns the optional read.
//!
//! # Extension levels
//!
//! A file name has up to three extensions, one per question, and each has its own
//! functions:
//!
//! - **Raw**: [`derive_ext`], and [`ext_bucket`], which labels its `None` as
//!   [`NO_EXTENSION`]. Any final dotted component counts if that extension is valid
//!   Unicode, with no character or length restriction; a `.tar` before it is kept.
//!   An invalid native extension has no bucketable string and maps to `None`.
//!   The extension view, per-directory extension tallies,
//!   and an unrecognized type's label use this level. It is the answer fdu gave before
//!   registries existed.
//! - **Logical**: [`logical_ext`] and [`NameClassification::logical_extension`]. This is
//!   File Rollup Format's name-owned extension: up to two trailing components, each ASCII
//!   alphanumeric and at most twelve bytes. A portable entry row reports it.
//! - **Canonical**: [`TypeRegistry::canonical_ext`] and
//!   [`NameClassification::canonical_extension`], from [`TypeRegistry::classify_name`].
//!   This is the declared extension the logical one matched, whole or by its final
//!   component. It is `None` when nothing declared matches, or when an exact filename wins
//!   first.
//!
//! Where the levels part, with the compiled registry (`none` is `None`):
//!
//! | Name             | Raw       | Bucket     | Logical   | Canonical |
//! | ---------------- | --------- | ---------- | --------- | --------- |
//! | `archive.tar.gz` | `.tar.gz` | `.tar.gz`  | `.tar.gz` | `.tar.gz` |
//! | `release.v2.zip` | `.zip`    | `.zip`     | `.v2.zip` | `.zip`    |
//! | `file.c++`       | `.c++`    | `.c++`     | none      | none      |
//! | `.gitignore`     | none      | `(none)`   | none      | none      |
//! | `notes.`         | none      | `(none)`   | none      | none      |
//!
//! A unit test reads this table and checks every cell against the functions.

use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, LazyLock};

mod file_rollup_manifest;
mod file_type_detection;

// The TOML cursor both manifest dialects read with. `include!`d by `build.rs` beside
// `type_rule_manifest`, so it follows that file's rules for sharing with a build script.
mod manifest_toml;

// Compiled into the crate and `include!`d by `build.rs`, so rules supplied at run time
// are read by exactly the code that read this repository's own manifest at build time.
// This is an implementation detail of `TypeRegistry::from_manifest`, not a second public
// configuration surface.
mod type_rule_manifest;

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

/// The engine family a manifest's `family` name selects.
///
/// The manifest carries a name because it is a text file; the engine carries an enum
/// because the analyzer set is closed. This is the one place the two meet, and
/// `validate_manifest` has already rejected any name it does not admit.
fn family_from_name(name: &str) -> Option<ContentFamily> {
    match name {
        "code" => Some(ContentFamily::Code),
        "prose" => Some(ContentFamily::Prose),
        "markup" => Some(ContentFamily::Markup),
        "data" => Some(ContentFamily::Data),
        "binary" => Some(ContentFamily::Binary),
        "unknown" => Some(ContentFamily::Unknown),
        _ => None,
    }
}

/// One rule's identity, after its keys have been indexed.
///
/// Extensions and filenames are not retained: they exist to build the two indexes, and
/// the cascade reads the indexes rather than the lists. `Cow` lets the compiled default
/// borrow all rendered rule text while a caller-supplied registry owns its text.
#[derive(Clone, PartialEq, Eq, Debug)]
struct TypeRule {
    id: Cow<'static, str>,
    family: ContentFamily,
    display_family: Option<Cow<'static, str>>,
    display_group: Option<Cow<'static, str>>,
    shebangs: Vec<Cow<'static, str>>,
    priority: u16,
}

/// One ordered browsing group from a File Rollup registry.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeGroup {
    id: String,
    label: String,
    order: u32,
}

impl TypeGroup {
    /// Stable machine identity.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Human-facing label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Registry display order.
    pub fn order(&self) -> u32 {
        self.order
    }
}

/// One ordered display family from a File Rollup registry.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeFamily {
    id: String,
    label: String,
    group_id: String,
    order: u32,
    extensions: Vec<String>,
}

impl TypeFamily {
    /// Stable machine identity.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Human-facing label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Owning browsing group identity.
    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    /// Registry display order.
    pub fn order(&self) -> u32 {
        self.order
    }

    /// Complete canonical extensions declared by member kinds, with leading dots.
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }
}

/// Registry-derived portable identity for one basename.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NameClassification {
    logical_extension: Option<String>,
    canonical_extension: Option<String>,
    kind_id: Option<String>,
    family_id: Option<String>,
    group_id: Option<String>,
    content_family: ContentFamily,
}

impl NameClassification {
    /// Name-owned logical extension.
    pub fn logical_extension(&self) -> Option<&str> {
        self.logical_extension.as_deref()
    }

    /// The declared extension that matched, or `None` when none did.
    pub fn canonical_extension(&self) -> Option<&str> {
        self.canonical_extension.as_deref()
    }

    /// Winning kind identity.
    pub fn kind_id(&self) -> Option<&str> {
        self.kind_id.as_deref()
    }

    /// Winning display family identity.
    pub fn family_id(&self) -> Option<&str> {
        self.family_id.as_deref()
    }

    /// Winning browsing group identity.
    pub fn group_id(&self) -> Option<&str> {
        self.group_id.as_deref()
    }

    /// Analyzer-oriented family.
    pub fn content_family(&self) -> ContentFamily {
        self.content_family
    }
}

/// A set of file-type rules, indexed for the classification cascade.
///
/// The registry is a *value*, not a compiled-in table, so a consumer whose taxonomy
/// differs from this repository's can supply its own without rebuilding the crate or
/// reclassifying in its own language. The compiled default stays the default and the fast
/// path: no file to find, no startup parse, and all compiled rule text is borrowed.
///
/// Its `fingerprint` is what makes a rule change safe. A snapshot and a content sidecar
/// both record the fingerprint of the registry that produced them, and both refuse a
/// cached answer when it moved — a classification change can move a file between
/// families, which invalidates the metrics rather than merely their labels.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeRegistry {
    rules: Vec<TypeRule>,
    /// Exact-basename lookup table storing indexes into `rules`.
    by_filename: HashMap<Cow<'static, str>, u32>,
    /// Extension lookup table storing indexes into `rules`.
    by_extension: HashMap<Cow<'static, str>, u32>,
    groups: Vec<TypeGroup>,
    families: Vec<TypeFamily>,
    registry_revision: Option<u32>,
    case_insensitive_filenames: bool,
    fingerprint: u64,
}

/// The registry compiled from this repository's manifest.
///
/// Built once, lazily, on first classification.
static COMPILED_REGISTRY: LazyLock<Arc<TypeRegistry>> =
    LazyLock::new(|| Arc::new(TypeRegistry::from_generated()));

impl TypeRegistry {
    /// The registry compiled from this repository's manifest.
    pub fn compiled() -> &'static Self {
        COMPILED_REGISTRY.as_ref()
    }

    /// Share the compiled registry with an owner that must retain it.
    pub(crate) fn compiled_shared() -> Arc<Self> {
        Arc::clone(&COMPILED_REGISTRY)
    }

    /// Build a registry from the `[[kind]]` manifest dialect.
    ///
    /// Validated by the same code that validates this repository's manifest at build
    /// time, so a manifest this accepts would have compiled and one it rejects would have
    /// failed the build with the same message.
    pub fn from_manifest(source: &str) -> crate::Result<Self> {
        let reject = |message: String| crate::Error::InvalidValue {
            kind: "type rules",
            value: String::new(),
            hint: message,
        };
        if file_rollup_manifest::looks_like_registry(source) {
            let parsed = file_rollup_manifest::parse(source).map_err(reject)?;
            let mut groups: Vec<_> = parsed
                .groups
                .iter()
                .map(|group| TypeGroup {
                    id: group.id.clone(),
                    label: group.label.clone(),
                    order: group.order,
                })
                .collect();
            groups.sort_by(|left, right| {
                left.order.cmp(&right.order).then_with(|| left.id.cmp(&right.id))
            });
            let group_order: HashMap<_, _> =
                groups.iter().map(|group| (group.id.as_str(), group.order)).collect();
            let mut families: Vec<_> = parsed
                .families
                .iter()
                .map(|family| TypeFamily {
                    id: family.id.clone(),
                    label: family.label.clone(),
                    group_id: family.group.clone(),
                    order: family.order,
                    extensions: parsed
                        .kinds
                        .iter()
                        .filter(|kind| kind.family == family.id)
                        .flat_map(|kind| kind.extensions.iter().map(|value| format!(".{value}")))
                        .collect(),
                })
                .collect();
            families.sort_by(|left, right| {
                group_order[&left.group_id.as_str()]
                    .cmp(&group_order[&right.group_id.as_str()])
                    .then_with(|| left.order.cmp(&right.order))
                    .then_with(|| left.id.cmp(&right.id))
            });
            let family_groups: HashMap<_, _> = parsed
                .families
                .iter()
                .map(|family| (family.id.as_str(), family.group.as_str()))
                .collect();
            let rules = parsed
                .kinds
                .iter()
                .map(|kind| TypeRule {
                    id: Cow::Owned(kind.id.clone()),
                    family: family_from_name(&kind.content_family)
                        .expect("validated content family"),
                    display_family: (!kind.family.is_empty())
                        .then(|| Cow::Owned(kind.family.clone())),
                    display_group: Some(Cow::Owned(if kind.group.is_empty() {
                        family_groups[&kind.family.as_str()].to_string()
                    } else {
                        kind.group.clone()
                    })),
                    shebangs: kind.shebangs.iter().cloned().map(Cow::Owned).collect(),
                    priority: kind.priority,
                })
                .collect();
            let mut registry = Self::indexed(
                rules,
                parsed.kinds.iter().map(|kind| kind.filenames.iter().cloned().map(Cow::Owned)),
                parsed.kinds.iter().map(|kind| kind.extensions.iter().cloned().map(Cow::Owned)),
                file_rollup_manifest::fingerprint(&parsed),
            );
            registry.groups = groups;
            registry.families = families;
            registry.registry_revision = Some(parsed.revision);
            registry.case_insensitive_filenames = true;
            return Ok(registry);
        }
        let parsed = type_rule_manifest::parse_manifest(source).map_err(reject)?;
        type_rule_manifest::validate_manifest(&parsed).map_err(reject)?;
        let rules = parsed
            .iter()
            .map(|rule| TypeRule {
                id: Cow::Owned(rule.id.clone()),
                family: family_from_name(&rule.family).expect("validated family"),
                display_family: None,
                display_group: None,
                shebangs: rule.shebangs.iter().cloned().map(Cow::Owned).collect(),
                priority: rule.priority,
            })
            .collect();
        Ok(Self::indexed(
            rules,
            parsed.iter().map(|rule| rule.filenames.iter().cloned().map(Cow::Owned)),
            parsed.iter().map(|rule| rule.extensions.iter().cloned().map(Cow::Owned)),
            type_rule_manifest::manifest_fingerprint(&parsed),
        ))
    }

    fn from_generated() -> Self {
        let rules = GENERATED_RULES
            .iter()
            .map(|rule| TypeRule {
                id: Cow::Borrowed(rule.id),
                family: rule.family,
                display_family: None,
                display_group: None,
                shebangs: rule.shebangs.iter().copied().map(Cow::Borrowed).collect(),
                priority: rule.priority,
            })
            .collect();
        Self::indexed(
            rules,
            GENERATED_RULES.iter().map(|rule| rule.filenames.iter().copied().map(Cow::Borrowed)),
            GENERATED_RULES.iter().map(|rule| rule.extensions.iter().copied().map(Cow::Borrowed)),
            TYPE_RULE_FINGERPRINT,
        )
    }

    /// Index the exact-basename and extension tiers.
    fn indexed(
        rules: Vec<TypeRule>,
        filenames: impl Iterator<Item = impl Iterator<Item = Cow<'static, str>>>,
        extensions: impl Iterator<Item = impl Iterator<Item = Cow<'static, str>>>,
        fingerprint: u64,
    ) -> Self {
        let by_filename = index_keys(filenames);
        let by_extension = index_keys(extensions);
        Self {
            rules,
            by_filename,
            by_extension,
            groups: Vec::new(),
            families: Vec::new(),
            registry_revision: None,
            case_insensitive_filenames: false,
            fingerprint,
        }
    }

    /// Identity of the rules this registry holds.
    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
    }

    /// File Rollup registry revision, absent for the compact fdu analyzer manifest.
    pub fn registry_revision(&self) -> Option<u32> {
        self.registry_revision
    }

    /// Ordered File Rollup browsing groups.
    pub fn groups(&self) -> impl Iterator<Item = &TypeGroup> {
        self.groups.iter()
    }

    /// Ordered File Rollup display families.
    pub fn families(&self) -> impl Iterator<Item = &TypeFamily> {
        self.families.iter()
    }

    /// Look up one File Rollup display family.
    pub fn family(&self, id: &str) -> Option<&TypeFamily> {
        self.families.iter().find(|family| family.id == id)
    }

    /// How many `[[kind]]` rules it holds.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Distinct exact basenames it claims.
    pub fn filename_count(&self) -> usize {
        self.by_filename.len()
    }

    /// Exact basename keys declared by this registry.
    ///
    /// The serving index uses this closed vocabulary to maintain bounded basename
    /// tallies without changing the classification cascade or retaining arbitrary file
    /// names. Callers that need case-insensitive presentation may normalize these keys;
    /// classification itself keeps its existing exact spelling contract.
    pub(crate) fn exact_filenames(&self) -> impl Iterator<Item = &str> {
        self.by_filename.keys().map(AsRef::as_ref)
    }

    /// Distinct extensions it claims.
    pub fn extension_count(&self) -> usize {
        self.by_extension.len()
    }

    /// The declared extension a name matched, per File Rollup Format: `None` when no
    /// extension declaration wins, including when an exact basename wins first.
    ///
    /// A file name owns its [`logical_ext`], which may retain two trailing components.
    /// The registry owns the canonical level: a logical extension declared whole stays
    /// whole, and otherwise its longest declared suffix matches. Thus `archive.tar.gz`
    /// remains `.tar.gz`, while `release.v2.zip` becomes `.zip` when the registry declares
    /// `zip` but not `v2.zip`. An undeclared extension has no canonical form:
    /// `release.v2.widget` answers `None` here exactly as its
    /// [`NameClassification::canonical_extension`] does, so a row and a registry lookup
    /// cannot disagree about the same name.
    pub fn canonical_ext(&self, name: &OsStr) -> Option<String> {
        if name.to_str().and_then(|name| self.by_filename(name)).is_some() {
            return None;
        }
        self.extension_match(name).map(|(extension, _)| extension.as_str().to_string())
    }

    /// The declared extension a name's logical extension matches, and its rule: the whole
    /// logical extension first, then its final component.
    ///
    /// Held inline rather than allocated, because every classified file asks and most
    /// files in a tree match nothing; see [`InlineExtension`].
    fn extension_match(&self, name: &OsStr) -> Option<(InlineExtension, &TypeRule)> {
        let logical = InlineExtension::logical(name)?;
        let key =
            logical.as_str().strip_prefix('.').expect("a logical extension starts with a dot");
        if let Some(rule) = self.by_extension(key) {
            return Some((logical, rule));
        }
        let (_, suffix) = key.rsplit_once('.')?;
        self.by_extension(suffix).map(|rule| (InlineExtension::dotted(suffix), rule))
    }

    /// Every stable type identifier it can produce, in manifest order.
    pub fn type_ids(&self) -> impl Iterator<Item = &str> {
        self.rules.iter().map(|rule| rule.id.as_ref())
    }

    /// Classify one basename into portable File Rollup identity without opening a file.
    pub fn classify_name(&self, name: &OsStr) -> NameClassification {
        let logical_extension = logical_ext(name);
        let filename_rule = name.to_str().and_then(|name| self.by_filename(name));
        let extension_match = filename_rule.is_none().then(|| self.extension_match(name));
        let (canonical_extension, rule) = match (filename_rule, extension_match.flatten()) {
            (Some(rule), _) => (None, Some(rule)),
            (None, Some((extension, rule))) => (Some(extension.as_str().to_string()), Some(rule)),
            (None, None) => (None, None),
        };
        let fallback_group = self
            .registry_revision
            .is_some()
            .then(|| self.groups.iter().find(|group| group.id == "other"))
            .flatten()
            .map(|group| group.id.clone());
        NameClassification {
            logical_extension,
            canonical_extension,
            kind_id: rule.map(|rule| rule.id.to_string()),
            family_id: rule.and_then(|rule| rule.display_family.as_ref().map(ToString::to_string)),
            group_id: rule
                .and_then(|rule| rule.display_group.as_ref().map(ToString::to_string))
                .or(fallback_group),
            content_family: rule.map_or(ContentFamily::Unknown, |rule| rule.family),
        }
    }

    fn by_filename(&self, name: &str) -> Option<&TypeRule> {
        self.by_filename
            .get(name)
            .or_else(|| {
                self.case_insensitive_filenames
                    .then(|| name.to_ascii_lowercase())
                    .and_then(|name| self.by_filename.get(name.as_str()))
            })
            .map(|index| &self.rules[*index as usize])
    }

    fn by_extension(&self, key: &str) -> Option<&TypeRule> {
        self.by_extension.get(key).map(|index| &self.rules[*index as usize])
    }

    fn by_id(&self, id: &str) -> Option<&TypeRule> {
        self.rules.iter().find(|rule| rule.id == id)
    }
}

/// Resolve one validated, collision-free key tier into rule indexes.
fn index_keys(
    keys: impl Iterator<Item = impl Iterator<Item = Cow<'static, str>>>,
) -> HashMap<Cow<'static, str>, u32> {
    let mut table: HashMap<Cow<'static, str>, u32> = HashMap::new();
    for (position, rule_keys) in keys.enumerate() {
        let index = u32::try_from(position).expect("a manifest holds fewer than 4 billion rules");
        for key in rule_keys {
            let previous = table.insert(key, index);
            debug_assert!(previous.is_none(), "validated registry keys are unique");
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

/// Classify from path metadata only, against the compiled default rules.
pub fn classify_path(path: &Path) -> Classification {
    classify_path_with_prefix(path, None)
}

/// Classify with an optional bounded content prefix, against the compiled default rules.
pub fn classify_path_with_prefix(path: &Path, prefix: Option<&[u8]>) -> Classification {
    classify_with(TypeRegistry::compiled(), path, prefix)
}

/// Classify against a specific registry, with an optional bounded content prefix.
///
/// Known exact-name and ordinary extension matches avoid deep detection. The `.h`
/// ambiguity and generated-file flag alone inspect their documented bounded prefix.
/// Callers may pass a larger buffer; all content-dependent helpers enforce smaller
/// internal limits.
pub fn classify_with(
    registry: &TypeRegistry,
    path: &Path,
    prefix: Option<&[u8]>,
) -> Classification {
    let name = path.file_name().unwrap_or_else(|| OsStr::new(""));
    // The rules table is pure ASCII, so a name that is not UTF-8 matched no rule
    // filename under the byte comparison this replaces either.
    if let Some(rule) = name.to_str().and_then(|name| registry.by_filename(name)) {
        return with_flags(
            path,
            prefix,
            classified(rule, DetectionSource::ExactFilename, DetectionConfidence::Certain),
        );
    }

    if let Some((extension, rule)) = registry.extension_match(name) {
        let key =
            extension.as_str().strip_prefix('.').expect("derived extensions start with a dot");
        let source = if key.contains('.') {
            DetectionSource::CompoundExtension
        } else {
            DetectionSource::Extension
        };
        let classification = if key == "h" {
            prefix
                .and_then(file_type_detection::resolve_c_header)
                .and_then(|id| registry.by_id(id))
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

    // An unrecognized type keeps the name's raw extension in its label, as it always has:
    // `unknown:.c++` and `unknown:.tar.lz4` are the piles the types view and the content
    // cache already know. Only declared matches follow the File Rollup eligibility rule.
    let extension = derive_ext(name);
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
            if registry.by_id(id).is_some_and(|rule| rule.family == ContentFamily::Binary) =>
        {
            let rule = registry.by_id(id).expect("guard established a registry rule");
            return with_flags(
                path,
                Some(prefix),
                classified(rule, source, DetectionConfidence::High),
            );
        }
        _ => {}
    }
    if let Some(interpreter) = file_type_detection::shebang_interpreter(prefix) {
        if let Some(rule) = registry
            .rules
            .iter()
            .filter(|rule| rule.shebangs.iter().any(|shebang| shebang == interpreter))
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
        Some(file_type_detection::PrefixMatch::Rule(id, source)) => registry
            .by_id(id)
            .map_or_else(unknown, |rule| classified(rule, source, DetectionConfidence::High)),
        Some(file_type_detection::PrefixMatch::UnknownBinary) => {
            unreachable!("returned above")
        }
        None => unknown(),
    };
    with_flags(path, Some(prefix), classification)
}

fn classified(
    rule: &TypeRule,
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

fn with_flags(
    path: &Path,
    prefix: Option<&[u8]>,
    mut classification: Classification,
) -> Classification {
    classification.flags = file_type_detection::flags(path, prefix);
    classification
}

/// Return the logical extension of a file name, lowercased and including its leading dot.
///
/// The logical level is name-owned rather than registry-owned. It retains at most two
/// trailing dotted components when each is nonempty, ASCII alphanumeric, and at most
/// twelve characters. A leading dot belongs to the basename, and an ineligible final
/// component means the name has no logical extension.
///
/// This is the value a portable entry row reports, and the one declared extensions are
/// matched against ([`TypeRegistry::canonical_ext`]). The extension view's aggregate
/// buckets and an unrecognized type's label use the raw [`derive_ext`] instead.
///
/// ```
/// use std::ffi::OsStr;
/// use fdu_core::classify::logical_ext;
///
/// assert_eq!(logical_ext(OsStr::new("archive.tar.gz")).as_deref(), Some(".tar.gz"));
/// assert_eq!(logical_ext(OsStr::new("release.v2.zip")).as_deref(), Some(".v2.zip"));
/// assert_eq!(logical_ext(OsStr::new(".gitignore")), None);
/// ```
pub fn logical_ext(name: &OsStr) -> Option<String> {
    logical_ext_native(name)
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
/// This is the raw extension, and it is deliberately not the File Rollup one: any final
/// valid-Unicode component counts without a character or length restriction, so
/// `file.c++` is `.c++` here while
/// [`logical_ext`] and [`TypeRegistry::canonical_ext`] give it none. It names the extension
/// view's piles and the label of an unrecognized type, and it keeps the answer it had
/// before registries existed for the detached and command-line callers that depend on it.
/// An invalid UTF-8 or UTF-16 extension returns `None`. On Unix and Windows an invalid
/// native stem does not prevent a valid extension from being extracted.
///
/// ```
/// use std::ffi::OsStr;
/// use fdu_core::classify::derive_ext;
///
/// assert_eq!(derive_ext(OsStr::new("archive.tar.gz")).as_deref(), Some(".tar.gz"));
/// assert_eq!(derive_ext(OsStr::new("notes.MD")).as_deref(), Some(".md"));
/// assert_eq!(derive_ext(OsStr::new("file.c++")).as_deref(), Some(".c++"));
/// assert_eq!(derive_ext(OsStr::new(".gitignore")), None);
/// assert_eq!(derive_ext(OsStr::new("README")), None);
/// ```
pub fn derive_ext(name: &OsStr) -> Option<String> {
    derive_ext_native(name)
}

/// Label of the extension bucket a file belongs to, including the one for no extension.
///
/// This is the raw-extension pile every index tallies, whatever registry it classifies
/// with: canonical extensions are a classification concept, derived per row at the read
/// boundary from [`TypeRegistry::canonical_ext`] rather than retained as a second tally.
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

#[cfg(unix)]
fn logical_ext_native(name: &OsStr) -> Option<String> {
    use std::os::unix::ffi::OsStrExt;

    logical_ext_units(name.as_bytes(), b'.', |unit| unit.to_ascii_lowercase())
        .and_then(|units| String::from_utf8(units).ok())
}

#[cfg(windows)]
fn logical_ext_native(name: &OsStr) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;

    let units: Vec<u16> = name.encode_wide().collect();
    logical_ext_units(&units, u16::from(b'.'), |unit| {
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
fn logical_ext_native(name: &OsStr) -> Option<String> {
    let units: Vec<char> = name.to_str()?.chars().collect();
    logical_ext_units(&units, '.', |unit| unit.to_ascii_lowercase())
        .map(|extension| extension.into_iter().collect())
}

const MAX_LOGICAL_EXTENSION_COMPONENT: usize = 12;

fn eligible_extension_component<T: Copy + Eq + From<u8>>(component: &[T]) -> bool {
    !component.is_empty()
        && component.len() <= MAX_LOGICAL_EXTENSION_COMPONENT
        && component.iter().all(|unit| {
            (b'0'..=b'9').chain(b'a'..=b'z').chain(b'A'..=b'Z').any(|byte| *unit == T::from(byte))
        })
}

/// The components of a name's logical extension, without their dots: the inner one when
/// it is eligible, and the final one.
fn logical_ext_components<T: Copy + Eq + From<u8>>(
    name: &[T],
    dot: T,
) -> Option<(Option<&[T]>, &[T])> {
    let searchable = if name.first() == Some(&dot) { &name[1..] } else { name };
    let dot_index = searchable.iter().rposition(|unit| *unit == dot)?;
    let (stem, last) = searchable.split_at(dot_index);
    let last = &last[1..];
    if !eligible_extension_component(last) {
        return None;
    }
    let inner = stem
        .iter()
        .rposition(|unit| *unit == dot)
        .map(|inner_dot| &stem[inner_dot + 1..])
        .filter(|inner| eligible_extension_component(inner));
    Some((inner, last))
}

fn logical_ext_units<T: Copy + Eq + From<u8>>(
    name: &[T],
    dot: T,
    lowercase: impl Fn(T) -> T,
) -> Option<Vec<T>> {
    let (inner, last) = logical_ext_components(name, dot)?;
    let mut extension = Vec::new();
    for component in inner.into_iter().chain([last]) {
        extension.push(dot);
        extension.extend(component.iter().copied().map(&lowercase));
    }
    Some(extension)
}

/// The longest logical extension: two components of at most
/// [`MAX_LOGICAL_EXTENSION_COMPONENT`] units, each with its dot.
const MAX_LOGICAL_EXTENSION_BYTES: usize = 2 * (MAX_LOGICAL_EXTENSION_COMPONENT + 1);

/// A logical or declared extension held inline: lowercase ASCII with its leading dot.
///
/// Declared extensions are matched for every classified file, and most files in a tree
/// match none. An eligible component is ASCII alphanumeric and bounded, so the key fits
/// here and matching allocates nothing: an unrecognized file pays only for the raw
/// extension its label names, and a recognized one only where a caller wants its
/// canonical extension as a `String`. Allocating the key made every unrecognized file on
/// an opened root cost one allocation more, which that route's allocation-slope guard
/// caught.
#[derive(Clone, Copy)]
struct InlineExtension {
    bytes: [u8; MAX_LOGICAL_EXTENSION_BYTES],
    len: usize,
}

impl InlineExtension {
    const EMPTY: Self = Self { bytes: [0; MAX_LOGICAL_EXTENSION_BYTES], len: 0 };

    /// The name's logical extension, spelled exactly as [`logical_ext`] spells it.
    fn logical(name: &OsStr) -> Option<Self> {
        #[cfg(unix)]
        let units = std::os::unix::ffi::OsStrExt::as_bytes(name);
        // A non-ASCII character is never a dot or an alphanumeric in any encoding, so a
        // Unicode name's UTF-8 bytes find the components its native units would. A
        // Windows name that is not Unicode is rare enough to take the allocating path.
        #[cfg(not(unix))]
        let Some(units) = name.to_str().map(str::as_bytes) else {
            return logical_ext(name).map(|extension| Self::EMPTY.with(extension.bytes()));
        };
        let (inner, last) = logical_ext_components(units, b'.')?;
        let mut extension = Self::EMPTY;
        for component in inner.into_iter().chain([last]) {
            extension = extension.with(*b".").with(component.iter().map(u8::to_ascii_lowercase));
        }
        Some(extension)
    }

    /// `.component`, for a component already taken from a logical extension.
    fn dotted(component: &str) -> Self {
        Self::EMPTY.with(*b".").with(component.bytes())
    }

    fn with(mut self, bytes: impl IntoIterator<Item = u8>) -> Self {
        for byte in bytes {
            self.bytes[self.len] = byte;
            self.len += 1;
        }
        self
    }

    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len]).expect("an eligible extension is ASCII")
    }
}

#[cfg(test)]
mod tests {
    use super::type_rule_manifest::{MANIFEST_FAMILIES, ManifestRule, parse_manifest};
    use super::{
        ContentFamily, DetectionConfidence, DetectionSource, InlineExtension, NO_EXTENSION,
        TypeRegistry, classify_path, classify_path_with_prefix, classify_with, derive_ext,
        ext_bucket, family_from_name, logical_ext, type_rule_fingerprint,
    };
    use super::{GENERATED_RULES, human_language_name};
    use std::ffi::OsStr;
    use std::path::Path;

    /// The manifest this build compiled, readable at test time as text.
    const DEFAULT_MANIFEST: &str = include_str!("../rules/file-types.toml");

    fn default_manifest_rules() -> Vec<ManifestRule> {
        parse_manifest(DEFAULT_MANIFEST).expect("the repository's own manifest parses")
    }

    #[test]
    fn every_validated_manifest_family_maps_to_an_engine_family() {
        for family in MANIFEST_FAMILIES {
            assert!(
                family_from_name(family).is_some(),
                "manifest validation admits {family:?}, but registry construction cannot map it"
            );
        }
    }

    /// Assert both lookup tiers against their complete declarative source.
    fn registry_indexes_every_claim(registry: &TypeRegistry, rules: &[ManifestRule], label: &str) {
        fn filenames(rule: &ManifestRule) -> &Vec<String> {
            &rule.filenames
        }
        fn extensions(rule: &ManifestRule) -> &Vec<String> {
            &rule.extensions
        }
        /// One tier's key accessor, named so the array below stays readable.
        type KeysOf = fn(&ManifestRule) -> &Vec<String>;
        let tiers: [(KeysOf, _); 2] =
            [(filenames, &registry.by_filename), (extensions, &registry.by_extension)];
        for (keys_of, table) in tiers {
            let mut checked = 0;
            for expected in rules {
                for key in keys_of(expected) {
                    let indexed = &registry.rules[table[key.as_str()] as usize];
                    assert_eq!(
                        indexed.id, expected.id,
                        "{label}: key {key:?} resolves to {:?}, expected {:?}",
                        indexed.id, expected.id
                    );
                    checked += 1;
                }
            }
            assert!(checked > 0, "{label}: the rules produced no keys to check");
        }
    }

    #[test]
    fn compiled_and_runtime_registries_index_every_manifest_claim() {
        let rules = default_manifest_rules();
        registry_indexes_every_claim(TypeRegistry::compiled(), &rules, "compiled");
        let parsed = TypeRegistry::from_manifest(DEFAULT_MANIFEST).expect("parses at run time");
        registry_indexes_every_claim(&parsed, &rules, "runtime");
    }

    /// Parsing the default manifest at run time must reproduce the compiled registry.
    ///
    /// The migration assertion: the compiled table is a rendering of this text, so a
    /// runtime parse that disagreed with it anywhere would mean the two readers of one
    /// dialect had drifted -- which is exactly what sharing the parser is meant to
    /// prevent, and what no other test would notice.
    #[test]
    fn the_runtime_parsed_default_matches_the_compiled_one() {
        let compiled = TypeRegistry::compiled();
        let parsed = TypeRegistry::from_manifest(DEFAULT_MANIFEST).expect("parses at run time");

        assert_eq!(parsed.fingerprint(), compiled.fingerprint(), "same text, same identity");
        assert_eq!(parsed.rule_count(), compiled.rule_count());
        assert_eq!(parsed.extension_count(), compiled.extension_count());
        assert_eq!(parsed.filename_count(), compiled.filename_count());
        assert_eq!(parsed.type_ids().collect::<Vec<_>>(), compiled.type_ids().collect::<Vec<_>>());

        let mut probes: Vec<String> = Vec::new();
        for rule in default_manifest_rules() {
            probes.extend(rule.filenames.iter().cloned());
            probes.extend(rule.extensions.iter().map(|ext| format!("probe.{ext}")));
        }
        assert!(probes.len() > 100, "the manifest should offer a wide key set");
        for probe in probes {
            let path = Path::new(&probe);
            assert_eq!(
                classify_with(&parsed, path, None),
                classify_with(compiled, path, None),
                "{probe} classifies differently under the runtime-parsed default"
            );
        }
    }

    /// Validation is tested for what it rejects: an accepted manifest proves less.
    #[test]
    fn a_registry_rejects_manifests_that_would_classify_ambiguously() {
        for (manifest, expected) in [
            ("", "at least one [[kind]] rule is required"),
            (
                "[[kind]]\nid = \"a\"\nfamily = \"code\"\n[[kind]]\nid = \"a\"\nfamily = \"code\"\n",
                "duplicate rule id",
            ),
            ("[[kind]]\nid = \"a\"\nfamily = \"pictures\"\n", "invalid family"),
            (
                "[[kind]]\nid = \"a\"\nfamily = \"code\"\nextensions = [\"q\"]\n[[kind]]\nid = \"b\"\nfamily = \"data\"\nextensions = [\"q\"]\n",
                "is assigned to both",
            ),
            ("[[kind]]\nid = \"A\"\nfamily = \"code\"\n", "invalid rule id"),
            (
                "[[kind]]\nid = \"a\"\nfamily = \"code\"\nextensions = [\".q\"]\n",
                "invalid extension",
            ),
            ("id = \"a\"\n", "field appears before [[kind]]"),
            ("[[kind]]\nid = a\n", "expected a quoted string"),
            ("[[kind]]\nid = \"a\"\nid = \"b\"\nfamily = \"code\"\n", "duplicate field \"id\""),
        ] {
            let error =
                TypeRegistry::from_manifest(manifest).expect_err("this manifest must be rejected");
            let message = error.to_string();
            assert!(message.contains(expected), "{message:?} should mention {expected:?}");
        }
    }

    /// Supplied rules classify, and are a different registry by identity.
    #[test]
    fn supplied_rules_replace_the_compiled_taxonomy() {
        let registry = TypeRegistry::from_manifest(
            "[[kind]]\nid = \"notes\"\nfamily = \"prose\"\nextensions = [\"rs\"]\n",
        )
        .expect("a minimal manifest");

        let ours = classify_with(&registry, Path::new("main.rs"), None);
        assert_eq!(ours.file_type.as_str(), "notes");
        assert_eq!(ours.family, ContentFamily::Prose);

        // The compiled default is untouched by another registry existing.
        assert_eq!(classify_path(Path::new("main.rs")).file_type.as_str(), "rust");

        // A type the supplied rules do not name falls to unknown rather than to the
        // default's answer: a registry is the whole taxonomy, not an overlay on one.
        assert_eq!(
            classify_with(&registry, Path::new("a.py"), None).source,
            DetectionSource::Unknown
        );
        assert_ne!(
            registry.fingerprint(),
            type_rule_fingerprint(),
            "different rules must invalidate a snapshot taken under the others"
        );
    }

    #[test]
    fn file_rollup_v3_registry_supplies_display_taxonomy_and_content_classification() {
        let registry = TypeRegistry::from_manifest(
            r#"
schema_version = 3
registry_revision = 7
max_extension_components = 2

[[group]]
id = "code"
label = "Code"
order = 10

[[group]]
id = "other"
label = "Other"
order = 20

[[family]]
id = "javascript"
label = "JavaScript"
group = "code"
order = 100
hue = 102.0

[[kind]]
id = "javascript"
family = "javascript"
content_family = "code"
extensions = ["js", "js.map"]
filenames = []
shebangs = ["node"]
priority = 100

[[kind]]
id = "make"
group = "other"
content_family = "code"
extensions = []
filenames = ["makefile"]
shebangs = []
priority = 100
"#,
        )
        .expect("File Rollup v3 registry parses");

        assert_eq!(registry.registry_revision(), Some(7));
        assert_eq!(
            registry.groups().map(super::TypeGroup::id).collect::<Vec<_>>(),
            vec!["code", "other"]
        );
        let family = registry.family("javascript").expect("display family retained");
        assert_eq!(family.label(), "JavaScript");
        assert_eq!(family.group_id(), "code");
        assert_eq!(family.extensions(), &[".js", ".js.map"]);

        let source_map = registry.classify_name(OsStr::new("bundle.js.map"));
        assert_eq!(source_map.kind_id(), Some("javascript"));
        assert_eq!(source_map.family_id(), Some("javascript"));
        assert_eq!(source_map.group_id(), Some("code"));
        assert_eq!(source_map.content_family(), ContentFamily::Code);
        assert_eq!(source_map.logical_extension(), Some(".js.map"));
        assert_eq!(source_map.canonical_extension(), Some(".js.map"));

        let makefile = registry.classify_name(OsStr::new("Makefile"));
        assert_eq!(makefile.kind_id(), Some("make"));
        assert_eq!(makefile.family_id(), None);
        assert_eq!(makefile.group_id(), Some("other"));
        assert_eq!(makefile.logical_extension(), None);
        assert_eq!(makefile.canonical_extension(), None);

        let unknown = registry.classify_name(OsStr::new("release.v2.widget"));
        assert_eq!(unknown.logical_extension(), Some(".v2.widget"));
        assert_eq!(unknown.canonical_extension(), None);
        // One canonical answer: the registry lookup agrees with the row for an undeclared
        // compound, a declared one, and a basename match.
        assert_eq!(registry.canonical_ext(OsStr::new("release.v2.widget")), None);
        assert_eq!(registry.canonical_ext(OsStr::new("bundle.js.map")).as_deref(), Some(".js.map"));
        assert_eq!(registry.canonical_ext(OsStr::new("Makefile")), None);
        assert_eq!(unknown.kind_id(), None);
        assert_eq!(unknown.family_id(), None);
        assert_eq!(unknown.group_id(), Some("other"));
        assert_eq!(unknown.content_family(), ContentFamily::Unknown);
    }

    #[test]
    fn registry_identity_is_derived_from_semantics_not_formatting() {
        let compact = TypeRegistry::from_manifest(
            "[[kind]]\nid = \"notes\"\nfamily = \"prose\"\nextensions = [\"txt\"]\n",
        )
        .expect("compact manifest");
        let formatted = TypeRegistry::from_manifest(
            "# Equivalent rules with presentation-only differences.\r\n\r\n[[kind]]\r\n  id = \"notes\"\r\n  family = \"prose\"\r\n  extensions = [ \"txt\" ]\r\n",
        )
        .expect("formatted manifest");
        let changed = TypeRegistry::from_manifest(
            "[[kind]]\nid = \"notes\"\nfamily = \"prose\"\nextensions = [\"text\"]\n",
        )
        .expect("different manifest");

        assert_eq!(compact.fingerprint(), formatted.fingerprint());
        assert_ne!(compact.fingerprint(), changed.fingerprint());
    }

    /// A key moved between classification tiers changes what files are, so it changes
    /// identity. Without array boundaries in the hash the two registries below shared one
    /// fingerprint, and a snapshot recorded under either was served under the other.
    #[test]
    fn file_rollup_registry_identity_sees_a_key_move_between_tiers() {
        let registry = |extensions: &str, filenames: &str| {
            TypeRegistry::from_manifest(&format!(
                "schema_version = 3\nregistry_revision = 1\nmax_extension_components = 2\n\n\
                 [[group]]\nid = \"other\"\nlabel = \"Other\"\norder = 10\n\n\
                 [[kind]]\nid = \"notes\"\ngroup = \"other\"\ncontent_family = \"prose\"\n\
                 extensions = [{extensions}]\nfilenames = [{filenames}]\nshebangs = []\n\
                 priority = 100\n"
            ))
            .expect("File Rollup v3 registry parses")
        };
        let as_extension = registry("\"md\"", "");
        let as_filename = registry("", "\"md\"");

        assert_eq!(as_extension.classify_name(OsStr::new("README.md")).kind_id(), Some("notes"));
        assert_eq!(as_filename.classify_name(OsStr::new("README.md")).kind_id(), None);
        assert_ne!(as_extension.fingerprint(), as_filename.fingerprint());
    }

    /// One registry written as schema 3 and as schema 4 is one classification. Schema 4
    /// adds icons, and the same revision repaints a family; neither changes what a file
    /// is, so a snapshot recorded under either document is served under the other.
    #[test]
    fn a_schema_three_registry_and_its_schema_four_repaint_share_one_identity() {
        let registry = |schema: u32, group_icon: &str, hue: &str, family_icon: &str| {
            TypeRegistry::from_manifest(&format!(
                "schema_version = {schema}\nregistry_revision = 3\nmax_extension_components = 2\n\n\
                 [[group]]\nid = \"code\"\nlabel = \"Code\"\norder = 10\n{group_icon}\n\
                 [[group]]\nid = \"other\"\nlabel = \"Other\"\norder = 20\n{group_icon}\n\
                 [[family]]\nid = \"swift\"\nlabel = \"Swift\"\ngroup = \"code\"\norder = 190\n\
                 linguist = \"Swift\"\nlinguist_color = \"#f05138\"\nhue = {hue}\n{family_icon}\n\
                 [[kind]]\nid = \"swift\"\nfamily = \"swift\"\ncontent_family = \"code\"\n\
                 extensions = [\"swift\"]\nfilenames = []\nshebangs = []\npriority = 100\n"
            ))
            .unwrap_or_else(|error| panic!("schema {schema}: {error}"))
        };
        let three = registry(3, "", "31.62", "");
        let four = registry(
            4,
            "icon = \"alignLeft\"",
            "52.3",
            "deviation = \"\"\"Moved off svelte's hue.\"\"\"\nicon = \"fileText\"",
        );

        assert_eq!(three.fingerprint(), four.fingerprint());
        assert_eq!(three.registry_revision(), four.registry_revision());
        let swift = OsStr::new("App.swift");
        assert_eq!(three.classify_name(swift).family_id(), Some("swift"));
        assert_eq!(four.classify_name(swift).family_id(), Some("swift"));
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

    /// The module documentation's extension-level table is read back and checked cell by
    /// cell, so a change to any level fails here instead of leaving the table wrong. The
    /// engine architecture document repeats the table and names this test as its source.
    #[test]
    fn module_documentation_extension_table_matches_the_functions() {
        let rules = TypeRegistry::compiled();
        let mut rows = 0;
        for line in include_str!("classify.rs").lines().take_while(|line| line.starts_with("//!")) {
            let row = line.trim_start_matches("//!").trim();
            if !row.starts_with("| `") {
                continue;
            }
            let cells: Vec<Option<&str>> = row
                .trim_matches('|')
                .split('|')
                .map(|cell| match cell.trim() {
                    "none" => None,
                    cell => Some(cell.trim_matches('`')),
                })
                .collect();
            let [Some(name), raw, bucket, logical, canonical] = cells.as_slice() else {
                panic!("an example row has a name and four levels: {row}");
            };
            let name = OsStr::new(name);
            assert_eq!(derive_ext(name).as_deref(), *raw, "raw {name:?}");
            assert_eq!(Some(ext_bucket(name).as_str()), *bucket, "bucket {name:?}");
            assert_eq!(logical_ext(name).as_deref(), *logical, "logical {name:?}");
            assert_eq!(rules.canonical_ext(name).as_deref(), *canonical, "canonical {name:?}");
            rows += 1;
        }
        assert_eq!(rows, 5, "every example row in the module documentation was read");
    }

    /// Three extensions answer three questions, and these names are where they part.
    ///
    /// `logical_ext` is the File Rollup observation, `canonical_ext` the declared extension
    /// that matched, and `derive_ext` the raw extension the extension view and unknown type
    /// labels have always used. The eligibility rule once leaked into `derive_ext`, moving
    /// `file.c++` and `résumé.tëxt` into `(none)` while its documentation said nothing had
    /// changed; and a registry lookup once gave an undeclared extension a canonical form
    /// that the same name's row did not have.
    #[test]
    fn logical_canonical_and_raw_extensions_answer_different_questions() {
        let rules = TypeRegistry::compiled();
        for (name, logical, canonical, raw) in [
            ("archive.tar.gz", Some(".tar.gz"), Some(".tar.gz"), Some(".tar.gz")),
            ("release.v2.zip", Some(".v2.zip"), Some(".zip"), Some(".zip")),
            ("bundle.umd.min.js", Some(".min.js"), Some(".js"), Some(".js")),
            (".eslintrc.json", Some(".json"), Some(".json"), Some(".json")),
            (".gitignore", None, None, None),
            ("trailing.", None, None, None),
            ("notes.", None, None, None),
            // Undeclared compound: no canonical form; the raw extension is its final part.
            ("release.v2.widget", Some(".v2.widget"), None, Some(".widget")),
            // Ineligible for File Rollup, yet each is still a raw extension of its own.
            ("file.c++", None, None, Some(".c++")),
            ("notes.tar.gz~", None, None, Some(".tar.gz~")),
            ("a.b-c", None, None, Some(".b-c")),
            ("x.py_", None, None, Some(".py_")),
            ("x.abcdefghijklm", None, None, Some(".abcdefghijklm")),
            ("résumé.tëxt", None, None, Some(".tëxt")),
        ] {
            let name = OsStr::new(name);
            assert_eq!(logical_ext(name).as_deref(), logical, "logical {name:?}");
            assert_eq!(
                InlineExtension::logical(name).as_ref().map(InlineExtension::as_str),
                logical,
                "inline logical {name:?}"
            );
            assert_eq!(rules.canonical_ext(name).as_deref(), canonical, "canonical {name:?}");
            assert_eq!(
                rules.classify_name(name).canonical_extension(),
                canonical,
                "a row and a registry lookup agree about {name:?}"
            );
            assert_eq!(derive_ext(name).as_deref(), raw, "raw {name:?}");
            assert_eq!(ext_bucket(name), raw.unwrap_or(NO_EXTENSION), "bucket {name:?}");
        }
    }

    /// The inline matching key spells what `logical_ext` does, at each of its bounds.
    #[test]
    fn inline_logical_extension_spells_what_logical_ext_does() {
        let twelve = "abcdefghijkl";
        let names = [
            "MAIN.RS".to_string(),
            "Archive.TAR.GZ".to_string(),
            // Both components at the length bound, so the key fills its buffer.
            format!("x.{twelve}.{}", twelve.to_uppercase()),
            format!("x.{twelve}m.{twelve}"),
            format!("x.{twelve}.{twelve}m"),
            ".hidden".to_string(),
            ".hidden.TOML".to_string(),
            "..".to_string(),
            ".".to_string(),
            String::new(),
            "a.".to_string(),
            "a..b".to_string(),
            "résumé.v2.tëxt".to_string(),
            "naïve.v2.Md".to_string(),
        ];
        let inline = |name: &OsStr| InlineExtension::logical(name).map(|e| e.as_str().to_owned());
        for name in &names {
            let name = OsStr::new(name);
            assert_eq!(inline(name), logical_ext(name), "{name:?}");
        }

        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let name = OsStr::from_bytes(b"caf\xe9.v2.TXT");
            assert_eq!(logical_ext(name).as_deref(), Some(".v2.txt"));
            assert_eq!(inline(name), logical_ext(name));
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStringExt;
            let units: Vec<u16> = [0xD800].into_iter().chain(".v2.TXT".encode_utf16()).collect();
            let name = std::ffi::OsString::from_wide(&units);
            assert_eq!(logical_ext(&name).as_deref(), Some(".v2.txt"));
            assert_eq!(inline(&name), logical_ext(&name));
        }
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
        // An unrecognized type keeps its raw extension, even one File Rollup cannot name.
        assert_eq!(classify_path(Path::new("file.c++")).file_type.as_str(), "unknown:.c++");
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
