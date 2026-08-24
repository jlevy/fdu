//! File-type recognition.
//!
//! Exact filenames and extensions are resolved from repository-owned declarative rules
//! compiled at build time. Callers that explicitly enabled content analysis may also
//! supply a bounded prefix for binary signatures, shebangs, modelines, ambiguous
//! headers, and origin flags. Nothing here performs runtime rule parsing or opens a
//! file; the caller owns the optional read.

use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, LazyLock};

mod file_type_detection;

/// The `[[kind]]` manifest dialect, parsed and validated by one implementation.
///
/// Compiled into the crate and `include!`d by `build.rs`, so rules supplied at run time
/// are read by exactly the code that read this repository's own manifest at build time.
/// Two parsers for one dialect is how a manifest comes to mean one thing to the compiler
/// and another to a consumer, with neither wrong on its own terms.
pub mod type_rule_manifest;

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
    /// Browsing group, as an index into the registry that produced this classification.
    ///
    /// An index rather than a name because this is built per file on the analysis path;
    /// resolving it costs a `String` per file for something most callers never read.
    /// Like an interned extension id, it is meaningful only with the registry that issued
    /// it -- [`TypeRegistry::group`] turns it back into a name.
    pub group: Option<GroupId>,
}

#[derive(Clone, Copy)]
struct GeneratedRule {
    id: &'static str,
    family: ContentFamily,
    group: &'static str,
    extensions: &'static [&'static str],
    filenames: &'static [&'static str],
    shebangs: &'static [&'static str],
    priority: u16,
}

#[derive(Clone, Copy)]
struct GeneratedGroup {
    id: &'static str,
    label: &'static str,
    order: u32,
}

/// Index of a browsing group within its registry.
///
/// An index rather than a name because it is stored on every classified file: a `u16`
/// beside the interned extension id costs what a second interner would have cost,
/// without the interner.
pub type GroupId = u16;

/// One browsing bucket a reader chooses among.
///
/// A second axis, not a renaming of [`ContentFamily`]. `family` answers an analysis
/// question -- which analyzer may open this file -- so every image, video, PDF, and
/// archive is `Binary` under it, and a families view over a photo directory is one row
/// reading "binary 100%". A group answers the browsing question instead, and the two are
/// maintained side by side rather than one being derived from the other.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeGroup {
    /// Stable machine identifier.
    pub id: Cow<'static, str>,
    /// Human-facing name.
    pub label: Cow<'static, str>,
    /// Display rank; lower sorts first.
    pub order: u32,
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
/// the cascade reads the indexes rather than the lists. `Cow` rather than `String` so the
/// compiled default borrows the rendered `&'static` data and allocates nothing per rule.
#[derive(Clone, PartialEq, Eq, Debug)]
struct TypeRule {
    id: Cow<'static, str>,
    family: ContentFamily,
    /// Browsing group, or `None` when the registry declares none.
    group: Option<GroupId>,
    shebangs: Vec<Cow<'static, str>>,
    priority: u16,
}

/// A set of file-type rules, indexed for the classification cascade.
///
/// The registry is a *value*, not a compiled-in table, so a consumer whose taxonomy
/// differs from this repository's can supply its own without rebuilding the crate or
/// reclassifying in its own language. The compiled default stays the default and the fast
/// path: no file to find, no startup parse, and the borrowed form allocates nothing per
/// rule.
///
/// Its `fingerprint` is what makes a rule change safe. A snapshot and a content sidecar
/// both record the fingerprint of the registry that produced them, and both refuse a
/// cached answer when it moved — a classification change can move a file between
/// families, which invalidates the metrics rather than merely their labels.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeRegistry {
    rules: Vec<TypeRule>,
    /// Browsing groups, in declaration order; empty when the manifest declares none.
    groups: Vec<TypeGroup>,
    /// Exact-basename tier, resolved once instead of scanned per file.
    ///
    /// The scan this replaces was `rules.iter().filter(..).max_by_key(..)`, and
    /// `max_by_key` consumes the whole iterator, so every file paid all 65 rules and all
    /// 167 extension strings even when its extension matched the first rule in the table.
    /// Classification runs twice per file on a warm content open -- once to build each
    /// candidate, once again in `Index::apply_analysis`'s staleness guard -- so this is
    /// charged for on both.
    by_filename: HashMap<Cow<'static, str>, u32>,
    /// Extension tier, indexed on the same terms.
    by_extension: HashMap<Cow<'static, str>, u32>,
    /// Type id to rule, for the tiers that name a type rather than match a key.
    ///
    /// Indexed rather than scanned because two paths reach it per file: deep detection
    /// resolves a probe's named rule, and restoring a content sidecar resolves every
    /// cached record's stored type id.
    by_type_id: HashMap<Cow<'static, str>, u32>,
    fingerprint: u64,
}

/// The registry compiled from this repository's manifest.
///
/// Built once, lazily, on first classification -- the same one-time index build the two
/// `LazyLock` tables it replaces performed, in one value instead of two.
static COMPILED_REGISTRY: LazyLock<Arc<TypeRegistry>> =
    LazyLock::new(|| Arc::new(TypeRegistry::from_generated()));

impl TypeRegistry {
    /// The registry compiled from this repository's manifest.
    pub fn compiled() -> &'static Arc<Self> {
        &COMPILED_REGISTRY
    }

    /// Build a registry from the `[[kind]]` manifest dialect.
    ///
    /// Validated by the same code that validates this repository's manifest at build
    /// time, so a manifest this accepts would have compiled and one it rejects would have
    /// failed the build with the same message.
    pub fn from_manifest(source: &str) -> crate::Result<Self> {
        Self::from_manifest_expecting(source, None)
    }

    /// [`TypeRegistry::from_manifest`], refusing a packet whose identity is not the one
    /// the supplier says it exported.
    ///
    /// The supplier computes a fingerprint when it writes the packet; this recomputes one
    /// from what was actually parsed and indexed. A disagreement means the bytes are not
    /// the rules the supplier believes it sent -- a truncated file, an edit nobody meant
    /// to ship, or a dialect this parser reads differently than the writer wrote.
    ///
    /// Failing here rather than returning the mismatch is the point. Both fingerprints
    /// were already readable, so a caller could always have compared them; what it could
    /// not do was be *unable* to skip the comparison. A registry that silently classifies
    /// under rules nobody chose invalidates every cached answer keyed on it, and does so
    /// without an error anywhere.
    pub fn from_manifest_expecting(source: &str, expected: Option<u64>) -> crate::Result<Self> {
        let registry = Self::parse_manifest(source)?;
        if let Some(expected) = expected
            && registry.fingerprint != expected
        {
            return Err(crate::Error::TypeRules(format!(
                "type rules identity mismatch: supplied packet indexes to {:#018x}, caller \
                     expected {expected:#018x}",
                registry.fingerprint
            )));
        }
        Ok(registry)
    }

    fn parse_manifest(source: &str) -> crate::Result<Self> {
        let manifest =
            type_rule_manifest::parse_manifest(source).map_err(crate::Error::TypeRules)?;
        type_rule_manifest::validate_manifest(&manifest).map_err(crate::Error::TypeRules)?;
        let groups: Vec<TypeGroup> = manifest
            .groups
            .iter()
            .map(|group| TypeGroup {
                id: Cow::Owned(group.id.clone()),
                label: Cow::Owned(group.label.clone()),
                order: group.order,
            })
            .collect();
        let rules = manifest
            .rules
            .iter()
            .map(|rule| TypeRule {
                id: Cow::Owned(rule.id.clone()),
                family: family_from_name(&rule.family).expect("validated family"),
                group: group_index(&groups, &rule.group),
                shebangs: rule.shebangs.iter().cloned().map(Cow::Owned).collect(),
                priority: rule.priority,
            })
            .collect();
        Ok(Self::indexed(
            rules,
            groups,
            manifest.rules.iter().map(|rule| rule.filenames.iter().map(String::as_str)),
            manifest.rules.iter().map(|rule| rule.extensions.iter().map(String::as_str)),
            type_rule_manifest::manifest_fingerprint(source),
        ))
    }

    fn from_generated() -> Self {
        let groups: Vec<TypeGroup> = GENERATED_GROUPS
            .iter()
            .map(|group| TypeGroup {
                id: Cow::Borrowed(group.id),
                label: Cow::Borrowed(group.label),
                order: group.order,
            })
            .collect();
        let rules = GENERATED_RULES
            .iter()
            .map(|rule| TypeRule {
                id: Cow::Borrowed(rule.id),
                family: rule.family,
                group: group_index(&groups, rule.group),
                shebangs: rule.shebangs.iter().copied().map(Cow::Borrowed).collect(),
                priority: rule.priority,
            })
            .collect();
        Self::indexed(
            rules,
            groups,
            GENERATED_RULES.iter().map(|rule| rule.filenames.iter().copied()),
            GENERATED_RULES.iter().map(|rule| rule.extensions.iter().copied()),
            TYPE_RULE_FINGERPRINT,
        )
    }

    /// Index both key tiers, reproducing the scan's tie-break exactly.
    ///
    /// `Iterator::max_by_key` returns the *last* of equally-maximum elements, so a later
    /// rule in manifest order wins at equal priority. Changing that would silently
    /// reclassify the types that share a key.
    fn indexed<'a>(
        rules: Vec<TypeRule>,
        groups: Vec<TypeGroup>,
        filenames: impl Iterator<Item = impl Iterator<Item = &'a str>>,
        extensions: impl Iterator<Item = impl Iterator<Item = &'a str>>,
        fingerprint: u64,
    ) -> Self {
        let by_filename = index_keys(&rules, filenames);
        let by_extension = index_keys(&rules, extensions);
        let by_type_id = rules
            .iter()
            .enumerate()
            .map(|(position, rule)| (rule.id.clone(), u32::try_from(position).expect("few rules")))
            .collect();
        Self { rules, groups, by_filename, by_extension, by_type_id, fingerprint }
    }

    /// Browsing groups this registry declares, in declaration order.
    pub fn groups(&self) -> &[TypeGroup] {
        &self.groups
    }

    /// One group by index, or `None` when the index is not this registry's.
    pub fn group(&self, id: GroupId) -> Option<&TypeGroup> {
        self.groups.get(id as usize)
    }

    /// The key a name is matched and bucketed under: its logical extension when a rule
    /// claims that, otherwise its trailing component.
    ///
    /// The format's second level. `.tar.gz` is claimed by a rule and stays whole;
    /// `.v2.zip` is claimed by none and falls back to `.zip`, which is why a release
    /// archive is an archive rather than an `unknown:.v2.zip`. That fallback replaced a
    /// hand-maintained fold of `.tar`, and it generalises: any two-component extension a
    /// registry declares wins whole, and any it does not falls to its last component.
    ///
    /// Registry-dependent by construction, which is why it lives here rather than beside
    /// [`logical_ext`]: what counts as canonical is exactly what some rule claims.
    pub fn canonical_ext(&self, name: &OsStr) -> Option<String> {
        let logical = logical_ext(name)?;
        let key = logical.strip_prefix('.').expect("a logical extension starts with a dot");
        if self.by_extension.contains_key(key) {
            return Some(logical);
        }
        // On a component boundary, never mid-component: `.min.js` falls to `.js`, and a
        // name with one component has nothing to fall back to and keeps what it has.
        match key.rsplit_once('.') {
            Some((_, last)) => Some(format!(".{last}")),
            None => Some(logical),
        }
    }

    /// The browsing group a name falls in, from path metadata alone.
    ///
    /// The insert path's cut of the cascade: the two index lookups that decide a group,
    /// without building a [`Classification`] or scanning for origin flags. A file's group
    /// is resolved once when it enters the index, beside its interned extension, so the
    /// reducer that maintains group totals never classifies.
    ///
    /// Metadata-only by design: the shebang and content-probe tiers need a file's bytes,
    /// and a roll-up maintained over a metadata walk cannot wait for them. A file those
    /// tiers would have named falls in no group here, exactly as it falls in no type row
    /// without content analysis.
    pub fn group_of_name(&self, name: &OsStr) -> Option<GroupId> {
        if self.groups.is_empty() {
            return None;
        }
        if let Some(rule) = name.to_str().and_then(|name| self.by_filename(name)) {
            return rule.group;
        }
        let extension = self.canonical_ext(name)?;
        let key = extension.strip_prefix('.').expect("a canonical extension starts with a dot");
        self.by_extension(key)?.group
    }

    /// Identity of the rules this registry holds.
    pub fn fingerprint(&self) -> u64 {
        self.fingerprint
    }

    /// How many `[[kind]]` rules it holds.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Distinct exact basenames it claims.
    pub fn filename_count(&self) -> usize {
        self.by_filename.len()
    }

    /// Distinct extensions it claims.
    pub fn extension_count(&self) -> usize {
        self.by_extension.len()
    }

    /// Every stable type identifier it can produce, in manifest order.
    pub fn type_ids(&self) -> impl Iterator<Item = &str> {
        self.rules.iter().map(|rule| rule.id.as_ref())
    }

    fn by_filename(&self, name: &str) -> Option<&TypeRule> {
        self.by_filename.get(name).map(|index| &self.rules[*index as usize])
    }

    fn by_extension(&self, key: &str) -> Option<&TypeRule> {
        self.by_extension.get(key).map(|index| &self.rules[*index as usize])
    }

    fn by_id(&self, id: &str) -> Option<&TypeRule> {
        self.by_type_id.get(id).map(|index| &self.rules[*index as usize])
    }

    /// The browsing group a stable type identifier belongs to.
    ///
    /// What a restored content record needs: the sidecar stores the type id it was
    /// written with, and the group is this registry's answer for it. Safe because a
    /// sidecar written under other rules is rejected on the fingerprint before any record
    /// is read.
    pub fn group_of_type(&self, id: &str) -> Option<GroupId> {
        self.by_id(id)?.group
    }
}

/// Resolve a group name to its index, or `None` when the manifest declares no groups.
///
/// Validation has already rejected a name no group declares, so a `None` here means the
/// manifest declared none at all.
fn group_index(groups: &[TypeGroup], name: &str) -> Option<GroupId> {
    groups
        .iter()
        .position(|group| group.id == name)
        .map(|index| GroupId::try_from(index).expect("a manifest declares few groups"))
}

/// Resolve one key tier into rule indexes, later-wins at equal priority.
fn index_keys<'a>(
    rules: &[TypeRule],
    keys: impl Iterator<Item = impl Iterator<Item = &'a str>>,
) -> HashMap<Cow<'static, str>, u32> {
    let mut table: HashMap<Cow<'static, str>, u32> = HashMap::new();
    for (position, rule_keys) in keys.enumerate() {
        let index = u32::try_from(position).expect("a manifest holds fewer than 4 billion rules");
        for key in rule_keys {
            match table.get(key) {
                Some(existing) if rules[*existing as usize].priority > rules[position].priority => {
                }
                _ => {
                    table.insert(Cow::Owned(key.to_string()), index);
                }
            }
        }
    }
    table
}

/// Fingerprint of the repository-owned rule manifest compiled into this build.
pub fn type_rule_fingerprint() -> u64 {
    TypeRegistry::compiled().fingerprint()
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

    // The canonical level, which is where a rule can match: `.tar.gz` is claimed whole,
    // `.v2.zip` falls to `.zip`. The unknown arm below reports the *logical* level instead,
    // because `unknown:.v2.zip` says more about a file nothing claimed than `unknown:.zip`
    // would -- the second would name a bucket that other files legitimately occupy.
    let extension = registry.canonical_ext(name);
    if let Some(extension) = extension.as_deref() {
        let key = extension.strip_prefix('.').expect("a canonical extension starts with a dot");
        if let Some(rule) = registry.by_extension(key) {
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
    }

    // The logical level here, not the canonical one: `unknown:.v2.zip` says what this file
    // actually is, where `unknown:.zip` would name a bucket that recognised files occupy.
    // This is the format's `remaining_types` key.
    let unclaimed = logical_ext(name);
    let unknown = || Classification {
        file_type: FileTypeId(
            unclaimed
                .as_deref()
                .map_or_else(|| "unknown".to_string(), |ext| format!("unknown:{ext}")),
        ),
        family: ContentFamily::Unknown,
        source: DetectionSource::Unknown,
        confidence: DetectionConfidence::Heuristic,
        flags: ClassificationFlags::default(),
        group: None,
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
    if let Some(interpreter) = file_type_detection::shebang_interpreter(prefix)
        && let Some(rule) = registry
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
        group: rule.group,
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

/// The *logical* extension of a file name: at most its final two dotted components,
/// lowercased, with the leading dot.
///
/// This is the raw level of the shared File Rollup Format's two, and it is deliberately
/// not the level rules match on. `release.v2.zip` is `.v2.zip` here and an `archive` by
/// type; `bundle.umd.min.js` is `.min.js` here and `javascript` by type. A consumer
/// showing a file's extension, filtering on a literal one, or bucketing the types no rule
/// claims wants this level -- the one a person would read off the name.
///
/// The rule, from the format:
///
/// - only the final basename;
/// - a leading dot belongs to the basename, so `.gitignore` has no extension while
///   `.eslintrc.json` has `.json`;
/// - at most the final two dotted components;
/// - each retained component is nonempty, alphanumeric, and at most twelve characters;
/// - no extension at all when the final component is ineligible.
///
/// The cap is what controls vocabulary size, and the format forbids widening it with
/// filename-specific exceptions -- which is what this replaced: a hand-maintained fold of
/// `.tar` alone, whose own comment asked for exactly this.
///
/// ```
/// use std::ffi::OsStr;
/// use fdu_core::classify::logical_ext;
///
/// assert_eq!(logical_ext(OsStr::new("archive.tar.gz")).as_deref(), Some(".tar.gz"));
/// assert_eq!(logical_ext(OsStr::new("bundle.umd.min.js")).as_deref(), Some(".min.js"));
/// assert_eq!(logical_ext(OsStr::new("release.v2.zip")).as_deref(), Some(".v2.zip"));
/// assert_eq!(logical_ext(OsStr::new("notes.MD")).as_deref(), Some(".md"));
/// assert_eq!(logical_ext(OsStr::new(".gitignore")), None);
/// assert_eq!(logical_ext(OsStr::new("README")), None);
/// ```
pub fn logical_ext(name: &OsStr) -> Option<String> {
    derive_ext_native(name)
}

/// Label of the extension bucket a file belongs to, including the one for no extension.
///
/// [`logical_ext`] answers "what is this name's extension", and `None` is the right answer
/// for `Makefile`. A roll-up asks a different question — "which pile does this file's
/// bytes go on" — and every file belongs on some pile. Dropping the `None` case meant the
/// extension view's rows did not sum to the tree it was reporting on: a 263-byte fixture
/// came back as three rows totalling 235, the missing 28 being a `Makefile`, and nothing
/// in the output said so.
///
/// The *canonical* level, not the logical one: a bucket is a pile files share, and
/// `release.v2.zip` belongs on the `.zip` pile beside `plain.zip` rather than on one of
/// its own. Registry-dependent for the same reason canonicalisation is.
///
/// A bucket always yields a leading dot, so [`NO_EXTENSION`] cannot collide with one
/// however the tree is named.
///
/// ```
/// use std::ffi::OsStr;
/// use fdu_core::classify::{NO_EXTENSION, TypeRegistry, ext_bucket};
///
/// let rules = TypeRegistry::compiled();
/// assert_eq!(ext_bucket(rules, OsStr::new("archive.tar.gz")), ".tar.gz");
/// assert_eq!(ext_bucket(rules, OsStr::new("release.v2.zip")), ".zip");
/// assert_eq!(ext_bucket(rules, OsStr::new("Makefile")), NO_EXTENSION);
/// assert_eq!(ext_bucket(rules, OsStr::new(".gitignore")), NO_EXTENSION);
/// ```
pub fn ext_bucket(registry: &TypeRegistry, name: &OsStr) -> String {
    registry.canonical_ext(name).unwrap_or_else(|| NO_EXTENSION.to_string())
}

/// Extension-view label for files whose name carries no extension.
///
/// Parenthesised so it reads as a category rather than as a filename, and dot-free so it
/// cannot be mistaken for — or collide with — an extension [`logical_ext`] produced.
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
    // No byte or wide view to borrow, so decode and run the same rule over `char`s. The
    // component cap counts units, and an eligible component is ASCII alphanumeric on every
    // platform, so bytes, wide units, and chars agree on every name any of them accepts.
    let units: Vec<char> = name.to_str()?.chars().collect();
    derive_ext_units(&units, '.', |unit| unit.to_ascii_lowercase())
        .map(|extension| extension.into_iter().collect())
}

/// Longest a retained component may be, from the format. The cap is what keeps the
/// vocabulary bounded: without it `release.candidate-final.zip` invents a bucket nobody
/// asked for.
const MAX_EXTENSION_COMPONENT: usize = 12;

/// A component is eligible when it is nonempty, alphanumeric, and within the cap.
///
/// Checked on the units rather than on a decoded string so the Windows and Unix paths
/// share one rule: a name that is not valid Unicode still has to be judged, and judging it
/// as ineligible is the answer either way.
fn eligible_component<T: Copy + Eq + From<u8>>(component: &[T]) -> bool {
    if component.is_empty() || component.len() > MAX_EXTENSION_COMPONENT {
        return false;
    }
    component.iter().all(|unit| {
        (b'0'..=b'9').chain(b'a'..=b'z').chain(b'A'..=b'Z').any(|byte| *unit == T::from(byte))
    })
}

fn derive_ext_units<T: Copy + Eq + From<u8>>(
    name: &[T],
    dot: T,
    lowercase: impl Fn(T) -> T,
) -> Option<Vec<T>> {
    let searchable = if name.first() == Some(&dot) { &name[1..] } else { name };
    let dot_index = searchable.iter().rposition(|unit| *unit == dot)?;
    let (stem, last) = searchable.split_at(dot_index);
    let last = &last[1..];
    if !eligible_component(last) {
        // "Return no extension if the final component is ineligible" -- a trailing
        // component that is not extension-shaped means the name has no extension at all,
        // not that the one before it is promoted.
        return None;
    }

    let mut extension = Vec::new();
    if let Some(inner_dot) = stem.iter().rposition(|unit| *unit == dot) {
        let inner = &stem[inner_dot + 1..];
        if eligible_component(inner) {
            extension.push(dot);
            extension.extend(inner.iter().copied().map(&lowercase));
        }
    }
    extension.push(dot);
    extension.extend(last.iter().copied().map(lowercase));
    Some(extension)
}

#[cfg(test)]
mod tests {
    use super::type_rule_manifest::{ManifestRule, parse_manifest};
    use super::{
        ContentFamily, DetectionConfidence, DetectionSource, TypeRegistry, classify_path,
        classify_path_with_prefix, classify_with, ext_bucket, logical_ext, type_rule_fingerprint,
    };
    use super::{GENERATED_RULES, human_language_name};
    use std::ffi::OsStr;
    use std::path::Path;

    /// The manifest this build compiled, readable at test time as text.
    const DEFAULT_MANIFEST: &str = include_str!("../rules/file-types.toml");

    fn default_manifest_rules() -> Vec<ManifestRule> {
        parse_manifest(DEFAULT_MANIFEST).expect("the repository's own manifest parses").rules
    }

    /// The indexed tiers must answer exactly what the scan they replaced answered.
    ///
    /// The scan was `filter(..).max_by_key(priority)`, and `max_by_key` returns the
    /// *last* of equally-maximum elements. Several keys are claimed by more than one
    /// rule, so a builder that took the first winner instead of the last would
    /// reclassify them silently, with no other test noticing.
    ///
    /// Stated over a registry rather than over the compiled tables, so it holds for rules
    /// a consumer supplies as well as for this repository's.
    fn tie_break_matches_a_scan(registry: &TypeRegistry, rules: &[ManifestRule], label: &str) {
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
            for key in rules.iter().flat_map(keys_of) {
                let scanned = rules
                    .iter()
                    .filter(|rule| keys_of(rule).contains(key))
                    .max_by_key(|rule| rule.priority)
                    .expect("the key came from a rule, so at least one rule matches");
                let indexed = &registry.rules[table[key.as_str()] as usize];
                assert_eq!(
                    indexed.id, scanned.id,
                    "{label}: key {key:?} resolves to {:?} but the scan chose {:?}",
                    indexed.id, scanned.id
                );
                checked += 1;
            }
            assert!(checked > 0, "{label}: the rules produced no keys to check");
        }
    }

    #[test]
    fn indexed_rule_tiers_agree_with_the_scan_they_replaced() {
        let rules = default_manifest_rules();
        tie_break_matches_a_scan(TypeRegistry::compiled(), &rules, "compiled");
        let parsed = TypeRegistry::from_manifest(DEFAULT_MANIFEST).expect("parses at run time");
        tie_break_matches_a_scan(&parsed, &rules, "runtime");
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
            ("id = \"a\"\n", "field appears before [[group]] or [[kind]]"),
            ("[[kind]]\nid = a\n", "expected a quoted string"),
            // Groups are all-or-nothing: a manifest that declares them and leaves one
            // kind out would drop that kind from every group breakdown, silently.
            (
                "[[group]]\nid = \"g\"\nlabel = \"G\"\n[[kind]]\nid = \"a\"\nfamily = \"code\"\n",
                "names no group",
            ),
            (
                "[[group]]\nid = \"g\"\nlabel = \"G\"\n[[kind]]\nid = \"a\"\nfamily = \"code\"\ngroup = \"h\"\n",
                "unknown group",
            ),
            (
                "[[kind]]\nid = \"a\"\nfamily = \"code\"\ngroup = \"g\"\n",
                "no [[group]] is declared",
            ),
            (
                "[[group]]\nid = \"g\"\nlabel = \"G\"\n[[group]]\nid = \"g\"\nlabel = \"G\"\n[[kind]]\nid = \"a\"\nfamily = \"code\"\ngroup = \"g\"\n",
                "duplicate group id",
            ),
            (
                "[[group]]\nid = \"g\"\n[[kind]]\nid = \"a\"\nfamily = \"code\"\ngroup = \"g\"\n",
                "has no label",
            ),
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
        assert_eq!(logical_ext(OsStr::new("main.RS")).as_deref(), Some(".rs"));
        assert_eq!(logical_ext(OsStr::new("Photo.JPEG")).as_deref(), Some(".jpeg"));
    }

    /// A packet whose identity is not the one its supplier recorded is refused, and the
    /// message names both numbers so the disagreement is diagnosable rather than merely
    /// reported.
    ///
    /// The check exists because both fingerprints were already readable and a caller
    /// could therefore always skip comparing them. Silently classifying under rules
    /// nobody chose invalidates every cached answer keyed on the registry, with no error
    /// anywhere.
    #[test]
    fn a_registry_refuses_a_packet_that_is_not_the_one_its_supplier_recorded() {
        let manifest = "[[kind]]\nid = \"rust\"\nfamily = \"code\"\nextensions = [\"rs\"]\n";
        let registry = TypeRegistry::from_manifest(manifest).expect("parses");
        let identity = registry.fingerprint();

        // The identity it actually has is accepted, and is the one echoed back.
        let checked = TypeRegistry::from_manifest_expecting(manifest, Some(identity))
            .expect("its own identity is accepted");
        assert_eq!(checked.fingerprint(), identity);

        let error = TypeRegistry::from_manifest_expecting(manifest, Some(identity ^ 1))
            .expect_err("a mismatch must fail the open");
        let message = error.to_string();
        assert!(message.contains("identity mismatch"), "{message}");
        assert!(message.contains(&format!("{identity:#018x}")), "names what it got: {message}");

        // A manifest that is merely different is caught by the same check rather than
        // classifying quietly under rules the caller did not choose.
        let edited =
            "[[kind]]\nid = \"rust\"\nfamily = \"code\"\nextensions = [\"rs\", \"rlib\"]\n";
        assert!(
            TypeRegistry::from_manifest_expecting(edited, Some(identity)).is_err(),
            "an edited packet does not pass its predecessor's identity"
        );
    }

    #[test]
    fn the_logical_level_is_the_formats_table() {
        // Verbatim from the File Rollup Format's own derivation table, because this is a
        // shared format and agreeing with it is the point.
        for (name, expected) in [
            ("bundle.js.map", Some(".js.map")),
            ("bundle.umd.min.js.map", Some(".js.map")),
            ("types.d.ts.map", Some(".ts.map")),
            ("bundle.umd.min.js", Some(".min.js")),
            ("archive.tar.gz", Some(".tar.gz")),
            ("release.v2.zip", Some(".v2.zip")),
            ("Photo.JPEG", Some(".jpeg")),
            (".gitignore", None),
            (".eslintrc.json", Some(".json")),
        ] {
            assert_eq!(logical_ext(OsStr::new(name)).as_deref(), expected, "{name}");
        }

        // Eligibility bounds the vocabulary. A component that is not extension-shaped is
        // not retained, and one that is not extension-shaped in the *final* position means
        // the name has no extension at all rather than promoting the one before it.
        assert_eq!(logical_ext(OsStr::new("v1.2.3.tar.gz")).as_deref(), Some(".tar.gz"));
        assert_eq!(logical_ext(OsStr::new("report.final-draft.pdf")).as_deref(), Some(".pdf"));
        assert_eq!(logical_ext(OsStr::new("thirteencharsx.zip")).as_deref(), Some(".zip"));
        assert_eq!(logical_ext(OsStr::new("archive.tar.gz-old")), None);
    }

    /// The two levels, and the property that adopting the raw one moved nothing.
    ///
    /// The trap this bead was written around: `derive_ext` returning the raw value alone
    /// would send key `v2.zip` at a rule table with no suffix fallback, and
    /// `release.v2.zip` would become `unknown:.v2.zip` while its `.zip` bucket split in
    /// two. One edit, two regressions, in exactly the names the change is for.
    #[test]
    fn the_canonical_level_falls_back_on_a_component_boundary() {
        let rules = TypeRegistry::compiled();
        for (name, logical, canonical) in [
            // Claimed whole, so both levels agree.
            ("archive.tar.gz", ".tar.gz", ".tar.gz"),
            // Claimed by nobody at two components, so the last one decides.
            ("release.v2.zip", ".v2.zip", ".zip"),
            ("bundle.umd.min.js", ".min.js", ".js"),
            // One component has nothing to fall back to.
            ("plain.zip", ".zip", ".zip"),
            ("app.js", ".js", ".js"),
        ] {
            assert_eq!(logical_ext(OsStr::new(name)).as_deref(), Some(logical), "{name} logical");
            assert_eq!(
                rules.canonical_ext(OsStr::new(name)).as_deref(),
                Some(canonical),
                "{name} canonical"
            );
            assert_eq!(ext_bucket(rules, OsStr::new(name)), canonical, "{name} bucket");
        }

        // The regression the bead names, stated as classification rather than as strings.
        for (name, file_type) in [
            ("release.v2.zip", "archive"),
            ("plain.zip", "archive"),
            ("archive.tar.gz", "archive"),
            ("bundle.umd.min.js", "javascript"),
            ("app.js", "javascript"),
        ] {
            assert_eq!(
                classify_path(Path::new(name)).file_type.as_str(),
                file_type,
                "{name} must still classify as {file_type}"
            );
        }
    }

    /// A type nothing claims is reported at the logical level, not the canonical one.
    #[test]
    fn an_unclaimed_type_keeps_the_extension_a_person_would_read() {
        // `.zip` is claimed, so a two-component name falling back to it is an archive.
        // `.frobnicate` is claimed by nobody at either level, and naming it `unknown:.zzz`
        // would file it under a bucket recognised files occupy.
        assert_eq!(
            classify_path(Path::new("thing.v2.frobnicate")).file_type.as_str(),
            "unknown:.v2.frobnicate"
        );
        assert_eq!(
            classify_path(Path::new("thing.frobnicate")).file_type.as_str(),
            "unknown:.frobnicate"
        );
    }

    #[test]
    fn names_without_a_usable_extension() {
        assert_eq!(logical_ext(OsStr::new("README")), None);
        assert_eq!(logical_ext(OsStr::new(".gitignore")), None);
        assert_eq!(logical_ext(OsStr::new(".bashrc")), None);
        assert_eq!(logical_ext(OsStr::new("trailing.")), None);
        assert_eq!(logical_ext(OsStr::new("")), None);
    }

    #[test]
    fn dotfiles_with_a_real_extension_keep_it() {
        assert_eq!(logical_ext(OsStr::new(".eslintrc.json")).as_deref(), Some(".json"));
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
        assert_eq!(logical_ext(&name).as_deref(), Some(".rs"));
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
        assert_eq!(logical_ext(&name).as_deref(), Some(".rs"));
    }
}
