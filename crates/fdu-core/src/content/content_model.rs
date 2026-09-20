//! Versioned content-analysis contracts shared across the engine and report layers.

use std::path::PathBuf;

use crate::classify::{
    Classification, ClassificationFlags, ContentFamily, DetectionConfidence, DetectionSource,
    FileTypeId,
};
use crate::query::Rejection;
use crate::{Attrs, EntryId, Fingerprint};

/// Stable analyzer identity.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct AnalyzerId(pub &'static str);

/// Version of an analyzer's counting semantics.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct AnalyzerVersion(pub u16);

/// Definition of one measured value exposed by content reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MetricDef {
    /// Stable report key.
    pub name: &'static str,
    /// Requestable unit that owns the value's presence.
    pub owner: AnalysisSet,
    /// Analyzer dialect that defines the value.
    pub analyzer: AnalyzerId,
    /// Short semantic definition.
    pub doc: &'static str,
}

/// Fingerprint of semantic analyzer options; operational worker count is excluded.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct OptionsFingerprint(pub u64);

/// Fused physical-line and raw-word analyzer.
pub const CONTENT_BASIC: AnalyzerId = AnalyzerId("content-basic-v1");
/// Common-language code/comment/blank analyzer.
pub const CODE_SLOC: AnalyzerId = AnalyzerId("code-sloc-v1");
/// Plain-text logical word and paragraph analyzer.
pub const TEXT_LOGICAL: AnalyzerId = AnalyzerId("text-logical-v1");
/// Reader-visible Markdown prose analyzer.
pub const MARKDOWN_PROSE: AnalyzerId = AnalyzerId("markdown-prose-v1");

/// The single registry of content metric names, owners, and definitions.
pub const METRICS: &[MetricDef] = &[
    MetricDef {
        name: "physical_lines",
        owner: AnalysisSet::LINES_ONLY,
        analyzer: CONTENT_BASIC,
        doc: "Logical physical lines across admitted text files.",
    },
    MetricDef {
        name: "blank_lines",
        owner: AnalysisSet::LINES_ONLY,
        analyzer: CONTENT_BASIC,
        doc: "Whitespace-only physical lines.",
    },
    MetricDef {
        name: "nonblank_lines",
        owner: AnalysisSet::LINES_ONLY,
        analyzer: CONTENT_BASIC,
        doc: "Physical lines containing non-whitespace text.",
    },
    MetricDef {
        name: "raw_words",
        owner: AnalysisSet::LINES_ONLY,
        analyzer: CONTENT_BASIC,
        doc: "Whitespace-delimited words before document projection.",
    },
    MetricDef {
        name: "code_lines",
        owner: AnalysisSet::CODE_ONLY,
        analyzer: CODE_SLOC,
        doc: "Code-bearing lines in supported source languages.",
    },
    MetricDef {
        name: "comment_lines",
        owner: AnalysisSet::CODE_ONLY,
        analyzer: CODE_SLOC,
        doc: "Comment-only lines in supported source languages.",
    },
    MetricDef {
        name: "code_blank_lines",
        owner: AnalysisSet::CODE_ONLY,
        analyzer: CODE_SLOC,
        doc: "Blank lines under the code analyzer's syntax.",
    },
    MetricDef {
        name: "logical_words",
        owner: AnalysisSet::WORDS_ONLY,
        analyzer: TEXT_LOGICAL,
        doc: "Normalized logical word volume.",
    },
    MetricDef {
        name: "paragraphs",
        owner: AnalysisSet::WORDS_ONLY,
        analyzer: TEXT_LOGICAL,
        doc: "Plain-text runs or reader-visible Markdown paragraphs.",
    },
    MetricDef {
        name: "visible_words",
        owner: AnalysisSet::WORDS_ONLY,
        analyzer: MARKDOWN_PROSE,
        doc: "Reader-visible Markdown words.",
    },
    MetricDef {
        name: "visible_logical_words",
        owner: AnalysisSet::WORDS_ONLY,
        analyzer: MARKDOWN_PROSE,
        doc: "Normalized reader-visible Markdown words.",
    },
    MetricDef {
        name: "document_words",
        owner: AnalysisSet::WORDS_ONLY,
        analyzer: TEXT_LOGICAL,
        doc: "Logical words after the document-type projection.",
    },
];

/// The set of content analyzers a request enables.
///
/// A set rather than a ladder, because the analyzers are independent: `code` and `words`
/// measure different things over different families and either is useful without the
/// other.  An ordered enum could name only the combinations somebody thought to
/// enumerate — four of the eight this registry already permits — and it made
/// `text-logical-v1` without `markdown-prose-v1` unreachable.
///
/// `lines` is the base every analyzer shares: any analyzer that runs has already
/// streamed the file, so line counts cost nothing extra.  It is therefore implicit in
/// every non-empty set rather than something a caller must remember to request, which is
/// why each `with_*` constructor sets it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct AnalysisSet(u8);

impl AnalysisSet {
    const LINES: u8 = 1 << 0;
    const CODE: u8 = 1 << 1;
    const WORDS: u8 = 1 << 2;
    const KNOWN: u8 = Self::LINES | Self::CODE | Self::WORDS;

    /// Open no file; preserve the metadata-only behavior.
    pub const NONE: Self = Self(0);
    /// Physical-line and raw-word unit.
    pub const LINES_ONLY: Self = Self(Self::LINES);
    /// Code unit, including its shared line pass.
    pub const CODE_ONLY: Self = Self(Self::LINES | Self::CODE);
    /// Word unit, including its shared line pass.
    pub const WORDS_ONLY: Self = Self(Self::LINES | Self::WORDS);
    /// Every registered analyzer.
    pub const ALL: Self = Self(Self::KNOWN);

    /// Add physical, blank, and nonblank line counts.
    #[must_use]
    pub const fn with_lines(self) -> Self {
        Self(self.0 | Self::LINES)
    }

    /// Add common-language standard SLOC over the `code` family.
    #[must_use]
    pub const fn with_code(self) -> Self {
        Self(self.0 | Self::LINES | Self::CODE)
    }

    /// Add raw, normalized, and reader-visible word volume.
    #[must_use]
    pub const fn with_words(self) -> Self {
        Self(self.0 | Self::LINES | Self::WORDS)
    }

    /// Whether any source file may be opened.
    pub const fn is_enabled(self) -> bool {
        self.0 != 0
    }

    /// Whether standard SLOC is requested.
    pub const fn includes_code(self) -> bool {
        self.0 & Self::CODE != 0
    }

    /// Whether logical and visible word metrics are requested.
    pub const fn includes_words(self) -> bool {
        self.0 & Self::WORDS != 0
    }

    /// Whether this request includes every unit in `other`.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Stable on-disk and fingerprint encoding.
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Decode [`Self::bits`], rejecting any analyzer this build does not know.
    ///
    /// How the grammar spells the empty set, and the one spelling of an analyzer set a
    /// `const` can state: every other set is a list [`Self::labels`] builds.
    ///
    /// Named because a surface whose help text states the default analyzer set must read
    /// that spelling rather than write the word again.
    pub const NONE_LABEL: &'static str = "none";

    /// Unknown bits mean a record written by a newer build whose extra analyzers cannot
    /// be honored, so it is refused rather than silently under-reported.
    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits & !Self::KNOWN == 0 { Some(Self(bits)) } else { None }
    }

    /// Parse the comma-delimited vocabulary both front ends accept, naming the axis as
    /// the library and the Python API spell it.
    ///
    /// Lives here rather than in either front end because it is the axis's grammar, not
    /// one surface's flag parsing: the CLI and the Python binding must accept exactly the
    /// same words or the two surfaces disagree about what a request means.
    ///
    /// `none` and `all` are totals and cannot be combined with anything, including each
    /// other — `none,code` has no coherent reading, and silently letting one win is how a
    /// caller ends up with analysis they did not ask for or did not get.
    pub fn parse(value: &str) -> Result<Self, String> {
        Self::parse_labeled(value, "analyze")
    }

    /// `label` is how the calling surface names this axis in its diagnostics: `--analyze`
    /// for the CLI, `analyze` for the Python API. Passed in rather than rewritten
    /// afterwards, because the CLI used to relabel by substring replace and that hit the
    /// user's own token: `--analyze analyzer` reported `invalid --analyze "--analyzer"`,
    /// misquoting the very value it was rejecting (fdu-7j6z).
    pub fn parse_labeled(value: &str, label: &str) -> Result<Self, String> {
        Self::parse_rejecting(value).map_err(|rejection| rejection.labeled(label))
    }

    /// [`Self::parse_labeled`], refusing with the value and expectation rather than a
    /// sentence, so the request model can name the axis in a typed refusal.
    pub(crate) fn parse_rejecting(value: &str) -> Result<Self, Rejection> {
        let mut set = Self::NONE;
        let mut seen: Vec<String> = Vec::new();
        let mut total: Option<&'static str> = None;
        for raw in value.split(',') {
            let token = raw.trim().to_ascii_lowercase();
            if token.is_empty() {
                return Err(Rejection::new(value, "empty entry in the list"));
            }
            if seen.contains(&token) {
                return Err(Rejection::new(value, format!("{token:?} appears more than once")));
            }
            seen.push(token.clone());
            match token.as_str() {
                "none" => total = Some(Self::NONE_LABEL),
                "all" => {
                    total = Some("all");
                    set = Self::ALL;
                }
                "lines" => set = set.with_lines(),
                "code" => set = set.with_code(),
                "words" => set = set.with_words(),
                other => {
                    return Err(Rejection::new(
                        other,
                        "expected one of none, lines, code, words, all",
                    ));
                }
            }
        }
        if let Some(total) = total {
            if seen.len() > 1 {
                return Err(Rejection::new(
                    value,
                    format!("{total:?} names the whole axis and cannot be combined"),
                ));
            }
            if total == Self::NONE_LABEL {
                return Ok(Self::NONE);
            }
        }
        Ok(set)
    }

    /// Requested analyzers in canonical order, as the CLI and reports spell them.
    pub fn labels(self) -> Vec<&'static str> {
        let mut labels = Vec::new();
        if self.0 & Self::LINES != 0 {
            labels.push("lines");
        }
        if self.includes_code() {
            labels.push("code");
        }
        if self.includes_words() {
            labels.push("words");
        }
        labels
    }
}

/// Content-derived classification evidence retained separately from name grouping.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ContentDetection {
    /// Type suggested by the bounded content probe.
    pub file_type: FileTypeId,
    /// Broad family suggested by the bounded content probe.
    pub family: ContentFamily,
    /// Evidence source.
    pub source: DetectionSource,
    /// Strength of the evidence.
    pub confidence: DetectionConfidence,
    /// Orthogonal generated, vendored, and documentation markers.
    pub flags: ClassificationFlags,
}

impl From<Classification> for ContentDetection {
    fn from(value: Classification) -> Self {
        Self {
            file_type: value.file_type,
            family: value.family,
            source: value.source,
            confidence: value.confidence,
            flags: value.flags,
        }
    }
}

impl From<ContentDetection> for Classification {
    fn from(value: ContentDetection) -> Self {
        Self {
            file_type: value.file_type,
            family: value.family,
            source: value.source,
            confidence: value.confidence,
            flags: value.flags,
        }
    }
}

/// Settings for one analysis pass.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AnalysisRequest {
    /// Analyzer bundle to run.
    pub profile: AnalysisSet,
    /// Maximum worker count; zero selects the available parallelism.
    pub workers: usize,
}

impl Default for AnalysisRequest {
    fn default() -> Self {
        Self { profile: AnalysisSet::NONE, workers: 0 }
    }
}

impl AnalysisRequest {
    /// Fingerprint only settings that can change a stored answer.
    pub fn options_fingerprint(self) -> OptionsFingerprint {
        const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
        const PRIME: u64 = 0x0000_0100_0000_01b3;
        let hash = [self.profile.bits()]
            .into_iter()
            .fold(OFFSET, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(PRIME));
        OptionsFingerprint(hash)
    }
}

/// Analyzer/rule/options identity attached to cached and reported content.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ContentProvenance {
    /// Compiled file-type rule identity.
    pub type_rules_fingerprint: u64,
    /// Semantic option identity.
    pub options_fingerprint: OptionsFingerprint,
    /// Analyzer dialects enabled by the profile.
    pub analyzers: Vec<(AnalyzerId, AnalyzerVersion)>,
}

impl ContentProvenance {
    /// Resolve analyzer dialects implied by a request, under a given set of type rules.
    ///
    /// The fingerprint is passed rather than read from a global: a caller may run two
    /// indexes under different taxonomies in one process, and a record must record the
    /// rules that actually produced it.
    pub fn for_request(request: AnalysisRequest, type_rules_fingerprint: u64) -> Self {
        const VERSION_ONE: AnalyzerVersion = AnalyzerVersion(1);
        let mut analyzers = Vec::new();
        if request.profile.is_enabled() {
            analyzers.push((CONTENT_BASIC, VERSION_ONE));
        }
        if request.profile.includes_code() {
            analyzers.push((CODE_SLOC, VERSION_ONE));
        }
        if request.profile.includes_words() {
            analyzers.push((TEXT_LOGICAL, VERSION_ONE));
            analyzers.push((MARKDOWN_PROSE, VERSION_ONE));
        }
        Self {
            type_rules_fingerprint,
            options_fingerprint: request.options_fingerprint(),
            analyzers,
        }
    }
}

/// Additive sufficient statistics for FlexDoc-style logical word volume.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct LogicalWordStats {
    /// Non-whitespace wide/fullwidth characters, each worth half a logical word.
    pub wide_chars: u64,
    /// Whitespace-delimited non-wide tokens.
    pub nonwide_tokens: u64,
    /// Non-whitespace non-wide characters used by the 3..6 clamp.
    pub nonwide_chars: u64,
}

/// Metrics owned by the always-present line analyzer unit.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct BasicMetrics {
    /// Logical physical lines across admitted text files.
    pub physical_lines: u64,
    /// Whitespace-only lines.
    pub blank_lines: u64,
    /// Lines containing at least one non-whitespace character.
    pub nonblank_lines: u64,
    /// Whitespace-delimited words before document projection.
    pub raw_words: u64,
}

/// Metrics owned by the code analyzer unit.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct CodeMetrics {
    /// Code-bearing lines.
    pub code_lines: u64,
    /// Comment-only lines.
    pub comment_lines: u64,
    /// Blank lines under the code analyzer's syntax.
    pub code_blank_lines: u64,
}

/// Metrics owned by the word analyzer unit.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct WordMetrics {
    /// Plain-text paragraph runs or visible Markdown paragraphs.
    pub paragraphs: u64,
    /// Reader-visible Markdown words.
    pub visible_words: u64,
    /// Additive logical-word sufficient statistics.
    pub logical_word_stats: LogicalWordStats,
    /// Reader-visible Markdown logical-word sufficient statistics.
    pub visible_logical_word_stats: LogicalWordStats,
}

/// One analyzer unit's explicit coverage and optional successful value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AnalyzerOutcome<T> {
    /// Why the unit did or did not produce a value.
    coverage: CoverageReason,
    /// Successful measured value; absent for every non-analyzed outcome.
    value: Option<T>,
}

impl<T> AnalyzerOutcome<T> {
    /// A successful analyzer result.
    pub(crate) const fn analyzed(value: T) -> Self {
        Self { coverage: CoverageReason::Analyzed, value: Some(value) }
    }

    /// A unit that could not produce a value for the named reason.
    pub(crate) fn unavailable(coverage: CoverageReason) -> Self {
        assert!(
            !matches!(coverage, CoverageReason::Analyzed),
            "an analyzed outcome must carry a value"
        );
        Self { coverage, value: None }
    }

    /// Coverage outcome for this unit.
    pub const fn coverage(&self) -> CoverageReason {
        self.coverage
    }

    /// Successful measured value, absent for every unavailable outcome.
    pub const fn value(self) -> Option<T>
    where
        T: Copy,
    {
        self.value
    }

    pub(crate) fn from_parts(coverage: CoverageReason, value: Option<T>) -> Option<Self> {
        if matches!(coverage, CoverageReason::Analyzed) == value.is_some() {
            Some(Self { coverage, value })
        } else {
            None
        }
    }

    /// Operational failures must be retried rather than treated as cache hits.
    pub const fn is_reusable(&self) -> bool {
        !matches!(self.coverage, CoverageReason::IoError | CoverageReason::ChangedDuringRead)
    }
}

impl LogicalWordStats {
    pub(crate) fn add_assign(&mut self, other: Self) {
        self.wide_chars = self.wide_chars.saturating_add(other.wide_chars);
        self.nonwide_tokens = self.nonwide_tokens.saturating_add(other.nonwide_tokens);
        self.nonwide_chars = self.nonwide_chars.saturating_add(other.nonwide_chars);
    }

    /// Derive rounded logical words after aggregation.
    pub fn logical_words(self) -> u64 {
        let chars = u128::from(self.nonwide_chars);
        let tokens = u128::from(self.nonwide_tokens);
        let wide = u128::from(self.wide_chars);
        let (numerator, denominator) = if tokens.saturating_mul(6) < chars {
            (chars.saturating_add(wide.saturating_mul(3)), 6)
        } else if tokens.saturating_mul(3) > chars {
            (chars.saturating_mul(2).saturating_add(wide.saturating_mul(3)), 6)
        } else {
            (tokens.saturating_mul(2).saturating_add(wide), 2)
        };
        let rounded = numerator.saturating_add(denominator / 2) / denominator;
        u64::try_from(rounded).unwrap_or(u64::MAX)
    }
}

/// Fixed additive metric slots shipped by the first content schema.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MetricValues {
    /// Logical physical lines across accepted text files.
    pub physical_lines: u64,
    /// Whitespace-only lines.
    pub blank_lines: u64,
    /// Lines containing at least one non-whitespace character.
    pub nonblank_lines: u64,
    /// Whitespace-delimited prose words before markup projection.
    pub raw_words: u64,
    /// Code-bearing lines under `code-sloc-v1`.
    pub code_lines: u64,
    /// Comment-only lines under `code-sloc-v1`.
    pub comment_lines: u64,
    /// Blank lines under `code-sloc-v1`, distinct from whitespace-only source lines.
    pub code_blank_lines: u64,
    /// Plain-text paragraph runs.
    pub paragraphs: u64,
    /// Reader-visible Markdown words.
    pub visible_words: u64,
    /// Additive logical-word sufficient statistics.
    pub logical_word_stats: LogicalWordStats,
    /// Reader-visible Markdown logical-word sufficient statistics.
    pub visible_logical_word_stats: LogicalWordStats,
}

/// Why a requested file did or did not produce metrics.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum CoverageReason {
    /// Requested analyzers completed.
    Analyzed,
    /// Known binary type or a NUL byte made text metrics inapplicable.
    Binary,
    /// Input was not valid UTF-8.
    InvalidUtf8,
    /// No shipped analyzer accepts this type.
    Unsupported,
    /// File I/O failed; the human error is retained separately.
    IoError,
    /// Metadata changed while the file was being read.
    ChangedDuringRead,
}

/// Sparse analysis record for one regular file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FileAnalysis {
    /// Metadata fingerprint this result describes.
    pub fingerprint: Fingerprint,
    /// Apparent bytes represented by the record.
    pub bytes: u64,
    /// Bounded content evidence, kept separate from name-based grouping.
    pub detection: ContentDetection,
    /// Shared physical-line and raw-word outcome.
    pub lines: AnalyzerOutcome<BasicMetrics>,
    /// Code outcome when the request included the code unit.
    pub code: Option<AnalyzerOutcome<CodeMetrics>>,
    /// Word outcome when the request included the word unit.
    pub words: Option<AnalyzerOutcome<WordMetrics>>,
    /// Optional path-specific failure detail.
    pub error: Option<String>,
}

impl FileAnalysis {
    /// Whether optional unit slots exactly match the tier's requested analyzer set.
    pub const fn matches_profile(&self, profile: AnalysisSet) -> bool {
        self.code.is_some() == profile.includes_code()
            && self.words.is_some() == profile.includes_words()
    }

    /// File-level operational failure, counted once even though it affects every unit.
    pub const fn operational_failure(&self) -> Option<CoverageReason> {
        match self.lines.coverage() {
            CoverageReason::IoError => Some(CoverageReason::IoError),
            CoverageReason::ChangedDuringRead => Some(CoverageReason::ChangedDuringRead),
            CoverageReason::Analyzed
            | CoverageReason::Binary
            | CoverageReason::InvalidUtf8
            | CoverageReason::Unsupported => None,
        }
    }

    /// Whether every retained requested unit is safe to reuse.
    pub const fn is_reusable(&self) -> bool {
        self.operational_failure().is_none()
            && self.lines.is_reusable()
            && match self.code {
                Some(outcome) => outcome.is_reusable(),
                None => true,
            }
            && match self.words {
                Some(outcome) => outcome.is_reusable(),
                None => true,
            }
    }
}

/// Owned immutable candidate captured before worker execution.
///
/// Crate-private with [`Index::analysis_candidates`] and [`Index::apply_analysis`] until
/// the request model (P1.3) decides whether an out-of-crate analyzer is a supported
/// surface (`fdu-5upj`): the tier must be prepared for a candidate's identity before a
/// result for it can commit, and preparation is crate-private.
///
/// [`Index::analysis_candidates`]: crate::Index::analysis_candidates
/// [`Index::apply_analysis`]: crate::Index::apply_analysis
#[derive(Clone, Debug)]
pub(crate) struct AnalysisCandidate {
    /// Generation-safe index identity.
    pub entry_id: EntryId,
    /// Entry revision at capture time.
    pub revision: u64,
    /// Path relative to the index root.
    pub relative_path: PathBuf,
    /// Absolute filesystem path.
    pub absolute_path: PathBuf,
    /// Last observed attributes.
    pub attrs: Attrs,
    /// Metadata-only classification.
    pub classification: Classification,
}

/// Live file identity used to match a sidecar record on cache-only restore.
///
/// Restore does not classify and does not open the file. The sidecar already stores the
/// classification that `apply_analysis` would have committed, and the apply-path
/// classify self-check is a crate-private consistency guard, not part of the answer.
#[derive(Clone, Debug)]
pub(crate) struct RestoreCandidate {
    /// Generation-safe index identity.
    pub entry_id: EntryId,
    /// Entry revision at capture time.
    pub revision: u64,
    /// Path relative to the index root.
    pub relative_path: PathBuf,
    /// Last observed attributes.
    pub attrs: Attrs,
}

/// Worker result submitted to the index's derived-data mutation boundary.
#[derive(Clone, Debug)]
pub(crate) struct AnalysisObservation {
    /// Candidate identity and expectation.
    pub candidate: AnalysisCandidate,
    /// Analyzer set whose tier may accept the result.
    pub profile: AnalysisSet,
    /// Analyzer identity whose tier may accept the result.
    pub provenance: ContentProvenance,
    /// Completed or skipped analysis record.
    pub analysis: FileAnalysis,
}

/// Result of conditionally committing one worker observation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum AnalysisApplyOutcome {
    /// The sparse record and ancestor rollups changed.
    Applied,
    /// The result was discarded: metadata changed after candidate capture, or the content
    /// tier holds another identity than the one the result was produced under, which is
    /// the same answer because both mean the result describes something else.
    Stale,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{AnalysisSet, AnalyzerOutcome, CoverageReason, LogicalWordStats, METRICS};

    #[test]
    #[should_panic(expected = "an analyzed outcome must carry a value")]
    fn unavailable_outcome_cannot_claim_success() {
        let _: AnalyzerOutcome<()> = AnalyzerOutcome::unavailable(CoverageReason::Analyzed);
    }

    #[test]
    fn metric_registry_has_unique_names_and_one_requestable_owner_each() {
        let mut names = HashSet::new();
        for metric in METRICS {
            assert!(names.insert(metric.name), "duplicate metric name {}", metric.name);
            assert!(
                matches!(
                    metric.owner,
                    AnalysisSet::LINES_ONLY | AnalysisSet::CODE_ONLY | AnalysisSet::WORDS_ONLY
                ),
                "{} has a non-unit owner {:?}",
                metric.name,
                metric.owner
            );
        }
        assert_eq!(names.len(), 12);
    }

    #[test]
    fn logical_words_derive_only_after_additive_stats_are_combined() {
        let first = LogicalWordStats { wide_chars: 3, nonwide_tokens: 1, nonwide_chars: 12 };
        let second = LogicalWordStats { wide_chars: 1, nonwide_tokens: 9, nonwide_chars: 6 };
        let combined = LogicalWordStats {
            wide_chars: first.wide_chars + second.wide_chars,
            nonwide_tokens: first.nonwide_tokens + second.nonwide_tokens,
            nonwide_chars: first.nonwide_chars + second.nonwide_chars,
        };
        assert_eq!(combined.logical_words(), 8);
    }

    #[test]
    fn logical_words_match_the_pinned_rational_clamp_and_half_up_rounding() {
        let logical = |wide_chars, nonwide_tokens, nonwide_chars| {
            LogicalWordStats { wide_chars, nonwide_tokens, nonwide_chars }.logical_words()
        };
        assert_eq!(logical(0, 0, 0), 0);
        assert_eq!(logical(0, 2, 9), 2, "ordinary prose passes through");
        assert_eq!(logical(0, 1, 12), 2, "long tokens use the six-character floor");
        assert_eq!(logical(0, 4, 4), 1, "short tokens use the three-character ceiling");
        assert_eq!(logical(3, 0, 0), 2, "wide halves round up once");
        assert_eq!(logical(1, 1, 1), 1, "mixed fractions combine before rounding");
    }

    #[test]
    fn the_analyzer_vocabulary_parses_every_accepted_spelling() {
        let cases = [
            ("none", AnalysisSet::NONE),
            ("lines", AnalysisSet::NONE.with_lines()),
            ("code", AnalysisSet::NONE.with_code()),
            ("words", AnalysisSet::NONE.with_words()),
            ("all", AnalysisSet::ALL),
            // Order is irrelevant: a set has no order, so neither spelling is preferred.
            ("code,words", AnalysisSet::ALL),
            ("words,code", AnalysisSet::ALL),
            // `lines` is already implied by any analyzer, so naming it adds nothing.
            ("lines,code", AnalysisSet::NONE.with_code()),
            // Whitespace and case are incidental, as in every other list flag.
            (" CODE , Words ", AnalysisSet::ALL),
        ];
        for (input, expected) in cases {
            assert_eq!(AnalysisSet::parse(input), Ok(expected), "parsing {input:?}");
        }
    }

    #[test]
    fn the_analyzer_vocabulary_rejects_every_incoherent_request() {
        // Each rejection names what was wrong; a set flag that silently drops a token is
        // how a caller ends up believing it measured something it did not.
        let cases = [
            ("", "empty entry"),
            ("code,,words", "empty entry"),
            ("code,code", "more than once"),
            ("basic", "expected one of"),
            ("documents", "expected one of"),
            ("full", "expected one of"),
            ("none,code", "cannot be combined"),
            ("all,code", "cannot be combined"),
            ("none,all", "cannot be combined"),
        ];
        for (input, needle) in cases {
            let error = AnalysisSet::parse(input).expect_err(&format!("{input:?} must fail"));
            assert!(error.contains(needle), "parsing {input:?} said {error:?}, wanted {needle:?}");
        }
    }

    /// The CLI used to relabel by substring replace, which rewrote the user's own token:
    /// `--analyze analyzer` reported `invalid --analyze "--analyzer"`, misquoting the very
    /// value it was rejecting. The label is a parameter now, so the value is untouched.
    #[test]
    fn a_label_never_rewrites_the_value_it_is_reporting() {
        for value in ["analyzer", "reanalyze", "analyze-all"] {
            let error = AnalysisSet::parse_labeled(value, "--analyze")
                .expect_err("must reject an unknown analyzer");
            assert!(
                error.contains(&format!("{value:?}")),
                "{error} must quote {value:?} exactly as typed"
            );
            assert!(error.starts_with("invalid --analyze "), "{error} must carry the label");
        }
    }

    #[test]
    fn the_on_disk_encoding_round_trips_and_refuses_unknown_analyzers() {
        for set in [
            AnalysisSet::NONE,
            AnalysisSet::NONE.with_lines(),
            AnalysisSet::NONE.with_code(),
            AnalysisSet::NONE.with_words(),
            AnalysisSet::ALL,
        ] {
            assert_eq!(AnalysisSet::from_bits(set.bits()), Some(set));
        }
        // A record written by a build with an analyzer this one lacks cannot be honored,
        // so it is refused rather than silently under-reported as absent metrics.
        assert_eq!(AnalysisSet::from_bits(0b1000_0000), None);
    }

    #[test]
    fn labels_are_the_vocabulary_parse_accepts() {
        for set in [
            AnalysisSet::NONE.with_lines(),
            AnalysisSet::NONE.with_code(),
            AnalysisSet::NONE.with_words(),
            AnalysisSet::ALL,
        ] {
            let spelled = set.labels().join(",");
            assert_eq!(AnalysisSet::parse(&spelled), Ok(set), "round trip through {spelled:?}");
        }
        assert!(AnalysisSet::NONE.labels().is_empty());
    }
}
