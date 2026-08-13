//! Versioned content-analysis contracts shared across the engine and report layers.

use std::path::PathBuf;

use crate::classify::Classification;
use crate::{Attrs, EntryId, Fingerprint};

/// Stable analyzer identity.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct AnalyzerId(pub &'static str);

/// Version of an analyzer's counting semantics.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct AnalyzerVersion(pub u16);

/// Stable metric-slot identity within an analyzer dialect.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MetricSlotId(pub &'static str);

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

/// Requested depth of content analysis.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum AnalysisProfile {
    /// Preserve the metadata-only behavior and perform no content I/O.
    #[default]
    Disabled,
    /// Physical, blank, and nonblank lines plus raw prose words.
    Basic,
    /// Basic metrics plus common-language standard SLOC.
    Code,
    /// Basic metrics plus raw/logical/visible prose volume.
    Documents,
    /// Every shipped analyzer.
    Full,
}

impl AnalysisProfile {
    /// Whether any source file may be opened.
    pub const fn is_enabled(self) -> bool {
        !matches!(self, Self::Disabled)
    }

    /// Whether standard SLOC is requested.
    pub const fn includes_code(self) -> bool {
        matches!(self, Self::Code | Self::Full)
    }

    /// Whether logical and visible prose metrics are requested.
    pub const fn includes_documents(self) -> bool {
        matches!(self, Self::Documents | Self::Full)
    }
}

/// Bounded settings for one analysis pass.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AnalysisRequest {
    /// Analyzer bundle to run.
    pub profile: AnalysisProfile,
    /// Maximum bytes read from one file.
    pub max_file_bytes: u64,
    /// Maximum worker count; zero selects the available parallelism.
    pub workers: usize,
}

impl Default for AnalysisRequest {
    fn default() -> Self {
        const DEFAULT_MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
        Self {
            profile: AnalysisProfile::Disabled,
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
            workers: 0,
        }
    }
}

impl AnalysisRequest {
    /// Fingerprint only settings that can change a stored answer.
    pub fn options_fingerprint(self) -> OptionsFingerprint {
        const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
        const PRIME: u64 = 0x0000_0100_0000_01b3;
        let profile = match self.profile {
            AnalysisProfile::Disabled => 0_u8,
            AnalysisProfile::Basic => 1,
            AnalysisProfile::Code => 2,
            AnalysisProfile::Documents => 3,
            AnalysisProfile::Full => 4,
        };
        let hash = [profile]
            .into_iter()
            .chain(self.max_file_bytes.to_le_bytes())
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
    /// Resolve analyzer dialects implied by a request.
    pub fn for_request(request: AnalysisRequest) -> Self {
        const VERSION_ONE: AnalyzerVersion = AnalyzerVersion(1);
        let mut analyzers = Vec::new();
        if request.profile.is_enabled() {
            analyzers.push((CONTENT_BASIC, VERSION_ONE));
        }
        if request.profile.includes_code() {
            analyzers.push((CODE_SLOC, VERSION_ONE));
        }
        if request.profile.includes_documents() {
            analyzers.push((TEXT_LOGICAL, VERSION_ONE));
            analyzers.push((MARKDOWN_PROSE, VERSION_ONE));
        }
        Self {
            type_rules_fingerprint: crate::classify::type_rule_fingerprint(),
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

impl LogicalWordStats {
    /// Derive rounded logical words after aggregation.
    pub fn logical_words(self) -> u64 {
        let minimum = self.nonwide_chars.div_ceil(6);
        let maximum = self.nonwide_chars.div_ceil(3);
        let nonwide = self.nonwide_tokens.clamp(minimum, maximum.max(minimum));
        (nonwide.saturating_mul(2).saturating_add(self.wide_chars).saturating_add(1)) / 2
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

impl MetricValues {
    pub(crate) fn add_assign(&mut self, other: &Self) {
        self.physical_lines = self.physical_lines.saturating_add(other.physical_lines);
        self.blank_lines = self.blank_lines.saturating_add(other.blank_lines);
        self.nonblank_lines = self.nonblank_lines.saturating_add(other.nonblank_lines);
        self.raw_words = self.raw_words.saturating_add(other.raw_words);
        self.code_lines = self.code_lines.saturating_add(other.code_lines);
        self.comment_lines = self.comment_lines.saturating_add(other.comment_lines);
        self.code_blank_lines = self.code_blank_lines.saturating_add(other.code_blank_lines);
        self.paragraphs = self.paragraphs.saturating_add(other.paragraphs);
        self.visible_words = self.visible_words.saturating_add(other.visible_words);
        self.logical_word_stats.wide_chars =
            self.logical_word_stats.wide_chars.saturating_add(other.logical_word_stats.wide_chars);
        self.logical_word_stats.nonwide_tokens = self
            .logical_word_stats
            .nonwide_tokens
            .saturating_add(other.logical_word_stats.nonwide_tokens);
        self.logical_word_stats.nonwide_chars = self
            .logical_word_stats
            .nonwide_chars
            .saturating_add(other.logical_word_stats.nonwide_chars);
        self.visible_logical_word_stats.wide_chars = self
            .visible_logical_word_stats
            .wide_chars
            .saturating_add(other.visible_logical_word_stats.wide_chars);
        self.visible_logical_word_stats.nonwide_tokens = self
            .visible_logical_word_stats
            .nonwide_tokens
            .saturating_add(other.visible_logical_word_stats.nonwide_tokens);
        self.visible_logical_word_stats.nonwide_chars = self
            .visible_logical_word_stats
            .nonwide_chars
            .saturating_add(other.visible_logical_word_stats.nonwide_chars);
    }

    pub(crate) fn sub_assign(&mut self, other: &Self) {
        macro_rules! subtract {
            ($field:ident) => {
                self.$field = self.$field.saturating_sub(other.$field);
            };
        }
        subtract!(physical_lines);
        subtract!(blank_lines);
        subtract!(nonblank_lines);
        subtract!(raw_words);
        subtract!(code_lines);
        subtract!(comment_lines);
        subtract!(code_blank_lines);
        subtract!(paragraphs);
        subtract!(visible_words);
        self.logical_word_stats.wide_chars =
            self.logical_word_stats.wide_chars.saturating_sub(other.logical_word_stats.wide_chars);
        self.logical_word_stats.nonwide_tokens = self
            .logical_word_stats
            .nonwide_tokens
            .saturating_sub(other.logical_word_stats.nonwide_tokens);
        self.logical_word_stats.nonwide_chars = self
            .logical_word_stats
            .nonwide_chars
            .saturating_sub(other.logical_word_stats.nonwide_chars);
        self.visible_logical_word_stats.wide_chars = self
            .visible_logical_word_stats
            .wide_chars
            .saturating_sub(other.visible_logical_word_stats.wide_chars);
        self.visible_logical_word_stats.nonwide_tokens = self
            .visible_logical_word_stats
            .nonwide_tokens
            .saturating_sub(other.visible_logical_word_stats.nonwide_tokens);
        self.visible_logical_word_stats.nonwide_chars = self
            .visible_logical_word_stats
            .nonwide_chars
            .saturating_sub(other.visible_logical_word_stats.nonwide_chars);
    }
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
    /// The configured per-file read bound was exceeded.
    TooLarge,
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
    /// Stable classification used for grouping.
    pub classification: Classification,
    /// Metadata fingerprint this result describes.
    pub fingerprint: Fingerprint,
    /// Apparent bytes represented by the record.
    pub bytes: u64,
    /// Requested profile that produced the record.
    pub profile: AnalysisProfile,
    /// Analyzer, rule, and semantic-option identity.
    pub provenance: ContentProvenance,
    /// Additive metrics; zero when coverage is not `Analyzed`.
    pub metrics: MetricValues,
    /// Coverage outcome.
    pub coverage: CoverageReason,
    /// Optional path-specific failure detail.
    pub error: Option<String>,
}

/// Owned immutable candidate captured before worker execution.
#[derive(Clone, Debug)]
pub struct AnalysisCandidate {
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
    /// Requested analyzer profile.
    pub profile: AnalysisProfile,
}

/// Worker result submitted to the index's derived-data mutation boundary.
#[derive(Clone, Debug)]
pub struct AnalysisObservation {
    /// Candidate identity and expectation.
    pub candidate: AnalysisCandidate,
    /// Completed or skipped analysis record.
    pub analysis: FileAnalysis,
}

/// Result of conditionally committing one worker observation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AnalysisApplyOutcome {
    /// The sparse record and ancestor rollups changed.
    Applied,
    /// Metadata changed after candidate capture; the result was discarded.
    Stale,
}

#[cfg(test)]
mod tests {
    use super::LogicalWordStats;

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
}
