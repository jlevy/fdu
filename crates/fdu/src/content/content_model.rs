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

    /// Whether every analyzer in `other` is also in `self`.
    ///
    /// This is what lets a stored record answer a narrower request: a sidecar written by
    /// a superset already holds every metric the narrower one would recompute, so reuse
    /// is containment rather than equality.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Stable on-disk and fingerprint encoding.
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Decode [`Self::bits`], rejecting any analyzer this build does not know.
    ///
    /// Unknown bits mean a record written by a newer build whose extra analyzers cannot
    /// be honored, so it is refused rather than silently under-reported.
    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits & !Self::KNOWN == 0 { Some(Self(bits)) } else { None }
    }

    /// Parse the comma-delimited vocabulary both front ends accept.
    ///
    /// Lives here rather than in either front end because it is the axis's grammar, not
    /// one surface's flag parsing: the CLI and the Python binding must accept exactly the
    /// same words or the two surfaces disagree about what a request means.
    ///
    /// `none` and `all` are totals and cannot be combined with anything, including each
    /// other — `none,code` has no coherent reading, and silently letting one win is how a
    /// caller ends up with analysis they did not ask for or did not get.
    pub fn parse(value: &str) -> Result<Self, String> {
        let mut set = Self::NONE;
        let mut seen: Vec<String> = Vec::new();
        let mut total: Option<&'static str> = None;
        for raw in value.split(',') {
            let token = raw.trim().to_ascii_lowercase();
            if token.is_empty() {
                return Err(format!("invalid analyze {value:?}: empty entry in the list"));
            }
            if seen.contains(&token) {
                return Err(format!("invalid analyze {value:?}: {token:?} appears more than once"));
            }
            seen.push(token.clone());
            match token.as_str() {
                "none" => total = Some("none"),
                "all" => {
                    total = Some("all");
                    set = Self::ALL;
                }
                "lines" => set = set.with_lines(),
                "code" => set = set.with_code(),
                "words" => set = set.with_words(),
                other => {
                    return Err(format!(
                        "invalid analyze {other:?}: expected one of none, lines, code, words, all"
                    ));
                }
            }
        }
        if let Some(total) = total {
            if seen.len() > 1 {
                return Err(format!(
                    "invalid analyze {value:?}: {total:?} names the whole axis and cannot be combined"
                ));
            }
            if total == "none" {
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
        if request.profile.includes_words() {
            analyzers.push((TEXT_LOGICAL, VERSION_ONE));
            analyzers.push((MARKDOWN_PROSE, VERSION_ONE));
        }
        Self {
            type_rules_fingerprint: crate::classify::type_rule_fingerprint(),
            options_fingerprint: request.options_fingerprint(),
            analyzers,
        }
    }

    /// Whether content produced under `self` with `stored` analyzers answers `wanted`.
    ///
    /// Containment rather than equality.  Every field here except the type-rule
    /// fingerprint is derived from the analyzer set — `options_fingerprint` hashes the
    /// set's bits and `analyzers` lists what those bits select — so comparing them by
    /// equality is the same test as set equality, spelled three times, and it is what
    /// forced a complete re-read whenever a wider cached set met a narrower request.
    ///
    /// The type-rule fingerprint still has to match exactly: a classification change can
    /// move a file between families, which invalidates the metrics themselves rather
    /// than merely how they are labelled.
    pub fn satisfies(&self, stored: AnalysisSet, wanted: AnalysisSet) -> bool {
        self.type_rules_fingerprint == crate::classify::type_rule_fingerprint()
            && stored.contains(wanted)
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
    pub profile: AnalysisSet,
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
    pub profile: AnalysisSet,
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
    use super::{AnalysisSet, LogicalWordStats};

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

    #[test]
    fn containment_is_reflexive_and_ordered_by_membership() {
        let code = AnalysisSet::NONE.with_code();
        let words = AnalysisSet::NONE.with_words();
        assert!(AnalysisSet::ALL.contains(code) && AnalysisSet::ALL.contains(words));
        assert!(!code.contains(words) && !words.contains(code));
        assert!(code.contains(code), "a set answers its own request");
        assert!(code.contains(AnalysisSet::NONE), "every set answers an empty request");
        assert!(!AnalysisSet::NONE.is_enabled(), "an empty set opens no file");
        assert!(code.contains(AnalysisSet::NONE.with_lines()), "lines ride along with code");
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
