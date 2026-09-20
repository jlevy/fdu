//! Sparse per-file content records and precomputed directory/group rollups.

use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use crate::stored_state::ContentTierIdentity;
use crate::{Freshness, Source};

use super::content_model::{
    AnalysisSet, AnalyzerOutcome, BasicMetrics, CodeMetrics, ContentProvenance, CoverageReason,
    FileAnalysis, WordMetrics,
};

/// Additive metrics and coverage for one analyzer unit.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct AnalyzerTally<T> {
    /// Files for which this analyzer produced a value.
    pub analyzed_files: u64,
    /// Additive values from analyzed files.
    pub metrics: T,
    /// Outcomes for every file on which the analyzer was requested.
    pub coverage: BTreeMap<CoverageReason, u64>,
}

/// Additive tally across records of one content-tier identity.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct MetricTally {
    /// Files represented, including unavailable outcomes.
    pub files: u64,
    /// Apparent bytes represented.
    pub bytes: u64,
    /// Shared lines-unit results.
    pub lines: AnalyzerTally<BasicMetrics>,
    /// Code-unit results; empty when the tier did not request code.
    pub code: AnalyzerTally<CodeMetrics>,
    /// Words-unit results; empty when the tier did not request words.
    pub words: AnalyzerTally<WordMetrics>,
}

impl MetricTally {
    fn add(&mut self, analysis: &FileAnalysis) {
        self.files = self.files.saturating_add(1);
        self.bytes = self.bytes.saturating_add(analysis.bytes);
        add_basic(&mut self.lines, analysis.lines);
        if let Some(outcome) = analysis.code {
            add_code(&mut self.code, outcome);
        }
        if let Some(outcome) = analysis.words {
            add_words(&mut self.words, outcome);
        }
    }

    fn subtract(&mut self, analysis: &FileAnalysis) {
        self.files = self.files.saturating_sub(1);
        self.bytes = self.bytes.saturating_sub(analysis.bytes);
        sub_basic(&mut self.lines, analysis.lines);
        if let Some(outcome) = analysis.code {
            sub_code(&mut self.code, outcome);
        }
        if let Some(outcome) = analysis.words {
            sub_words(&mut self.words, outcome);
        }
    }

    fn merge(&mut self, other: &Self) {
        self.files = self.files.saturating_add(other.files);
        self.bytes = self.bytes.saturating_add(other.bytes);
        merge_basic(&mut self.lines, &other.lines);
        merge_code(&mut self.code, &other.code);
        merge_words(&mut self.words, &other.words);
    }
}

fn add_coverage<T>(tally: &mut AnalyzerTally<T>, outcome: &AnalyzerOutcome<T>) {
    *tally.coverage.entry(outcome.coverage()).or_default() += 1;
    if outcome.coverage() == CoverageReason::Analyzed {
        tally.analyzed_files = tally.analyzed_files.saturating_add(1);
    }
}

fn sub_coverage<T>(tally: &mut AnalyzerTally<T>, outcome: &AnalyzerOutcome<T>) {
    if let Some(count) = tally.coverage.get_mut(&outcome.coverage()) {
        *count = count.saturating_sub(1);
        if *count == 0 {
            tally.coverage.remove(&outcome.coverage());
        }
    }
    if outcome.coverage() == CoverageReason::Analyzed {
        tally.analyzed_files = tally.analyzed_files.saturating_sub(1);
    }
}

fn merge_coverage<T>(tally: &mut AnalyzerTally<T>, other: &AnalyzerTally<T>) {
    tally.analyzed_files = tally.analyzed_files.saturating_add(other.analyzed_files);
    for (reason, count) in &other.coverage {
        let slot = tally.coverage.entry(*reason).or_default();
        *slot = slot.saturating_add(*count);
    }
}

fn add_basic(tally: &mut AnalyzerTally<BasicMetrics>, outcome: AnalyzerOutcome<BasicMetrics>) {
    add_coverage(tally, &outcome);
    let Some(value) = outcome.value() else { return };
    tally.metrics.physical_lines =
        tally.metrics.physical_lines.saturating_add(value.physical_lines);
    tally.metrics.blank_lines = tally.metrics.blank_lines.saturating_add(value.blank_lines);
    tally.metrics.nonblank_lines =
        tally.metrics.nonblank_lines.saturating_add(value.nonblank_lines);
    tally.metrics.raw_words = tally.metrics.raw_words.saturating_add(value.raw_words);
}

fn sub_basic(tally: &mut AnalyzerTally<BasicMetrics>, outcome: AnalyzerOutcome<BasicMetrics>) {
    sub_coverage(tally, &outcome);
    let Some(value) = outcome.value() else { return };
    tally.metrics.physical_lines =
        tally.metrics.physical_lines.saturating_sub(value.physical_lines);
    tally.metrics.blank_lines = tally.metrics.blank_lines.saturating_sub(value.blank_lines);
    tally.metrics.nonblank_lines =
        tally.metrics.nonblank_lines.saturating_sub(value.nonblank_lines);
    tally.metrics.raw_words = tally.metrics.raw_words.saturating_sub(value.raw_words);
}

fn merge_basic(tally: &mut AnalyzerTally<BasicMetrics>, other: &AnalyzerTally<BasicMetrics>) {
    merge_coverage(tally, other);
    tally.metrics.physical_lines =
        tally.metrics.physical_lines.saturating_add(other.metrics.physical_lines);
    tally.metrics.blank_lines = tally.metrics.blank_lines.saturating_add(other.metrics.blank_lines);
    tally.metrics.nonblank_lines =
        tally.metrics.nonblank_lines.saturating_add(other.metrics.nonblank_lines);
    tally.metrics.raw_words = tally.metrics.raw_words.saturating_add(other.metrics.raw_words);
}

fn add_code(tally: &mut AnalyzerTally<CodeMetrics>, outcome: AnalyzerOutcome<CodeMetrics>) {
    add_coverage(tally, &outcome);
    let Some(value) = outcome.value() else { return };
    tally.metrics.code_lines = tally.metrics.code_lines.saturating_add(value.code_lines);
    tally.metrics.comment_lines = tally.metrics.comment_lines.saturating_add(value.comment_lines);
    tally.metrics.code_blank_lines =
        tally.metrics.code_blank_lines.saturating_add(value.code_blank_lines);
}

fn sub_code(tally: &mut AnalyzerTally<CodeMetrics>, outcome: AnalyzerOutcome<CodeMetrics>) {
    sub_coverage(tally, &outcome);
    let Some(value) = outcome.value() else { return };
    tally.metrics.code_lines = tally.metrics.code_lines.saturating_sub(value.code_lines);
    tally.metrics.comment_lines = tally.metrics.comment_lines.saturating_sub(value.comment_lines);
    tally.metrics.code_blank_lines =
        tally.metrics.code_blank_lines.saturating_sub(value.code_blank_lines);
}

fn merge_code(tally: &mut AnalyzerTally<CodeMetrics>, other: &AnalyzerTally<CodeMetrics>) {
    merge_coverage(tally, other);
    tally.metrics.code_lines = tally.metrics.code_lines.saturating_add(other.metrics.code_lines);
    tally.metrics.comment_lines =
        tally.metrics.comment_lines.saturating_add(other.metrics.comment_lines);
    tally.metrics.code_blank_lines =
        tally.metrics.code_blank_lines.saturating_add(other.metrics.code_blank_lines);
}

fn add_words(tally: &mut AnalyzerTally<WordMetrics>, outcome: AnalyzerOutcome<WordMetrics>) {
    add_coverage(tally, &outcome);
    let Some(value) = outcome.value() else { return };
    tally.metrics.paragraphs = tally.metrics.paragraphs.saturating_add(value.paragraphs);
    tally.metrics.visible_words = tally.metrics.visible_words.saturating_add(value.visible_words);
    tally.metrics.logical_word_stats.add_assign(value.logical_word_stats);
    tally.metrics.visible_logical_word_stats.add_assign(value.visible_logical_word_stats);
}

fn sub_words(tally: &mut AnalyzerTally<WordMetrics>, outcome: AnalyzerOutcome<WordMetrics>) {
    sub_coverage(tally, &outcome);
    let Some(value) = outcome.value() else { return };
    tally.metrics.paragraphs = tally.metrics.paragraphs.saturating_sub(value.paragraphs);
    tally.metrics.visible_words = tally.metrics.visible_words.saturating_sub(value.visible_words);
    sub_word_stats(&mut tally.metrics.logical_word_stats, value.logical_word_stats);
    sub_word_stats(&mut tally.metrics.visible_logical_word_stats, value.visible_logical_word_stats);
}

fn merge_words(tally: &mut AnalyzerTally<WordMetrics>, other: &AnalyzerTally<WordMetrics>) {
    merge_coverage(tally, other);
    tally.metrics.paragraphs = tally.metrics.paragraphs.saturating_add(other.metrics.paragraphs);
    tally.metrics.visible_words =
        tally.metrics.visible_words.saturating_add(other.metrics.visible_words);
    tally.metrics.logical_word_stats.add_assign(other.metrics.logical_word_stats);
    tally.metrics.visible_logical_word_stats.add_assign(other.metrics.visible_logical_word_stats);
}

fn sub_word_stats(tally: &mut super::LogicalWordStats, value: super::LogicalWordStats) {
    tally.wide_chars = tally.wide_chars.saturating_sub(value.wide_chars);
    tally.nonwide_tokens = tally.nonwide_tokens.saturating_sub(value.nonwide_tokens);
    tally.nonwide_chars = tally.nonwide_chars.saturating_sub(value.nonwide_chars);
}

/// Content totals for one directory subtree.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ContentRollUp {
    /// All sparse file records beneath this directory.
    pub total: MetricTally,
}

impl ContentRollUp {
    fn add(&mut self, analysis: &FileAnalysis) {
        self.total.add(analysis);
    }

    fn subtract(&mut self, analysis: &FileAnalysis) {
        self.total.subtract(analysis);
    }

    fn merge(&mut self, other: &Self) {
        self.total.merge(&other.total);
    }
}

/// A relative path ordered by its bytes rather than by its components.
///
/// `PathBuf`'s own ordering compares component by component, re-parsing both sides on
/// every comparison, and a `BTreeMap` keyed by it pays that on every descent: on a warm
/// content open over 52k files, `compare_components` and `Components::next` were a third
/// of the profile. Byte order is one `memcmp`, it is just as deterministic, and every
/// record beneath a directory is still contiguous -- they share the directory's bytes and
/// a separator as a prefix -- so the prefix range that invalidation relies on survives.
/// The sidecar is written in this order and read back by key, so the order is unobservable
/// outside this module.
///
/// `Path` equality ignores which separator a component boundary uses where the platform
/// accepts more than one; bytes do not. Keys and lookups therefore pass through
/// [`normalized`], which rebuilds a path from its components -- and so with the platform's
/// own separator -- only on such a platform and only when the path carries the other one.
/// Everywhere else it borrows, and a lookup allocates nothing.
#[derive(Clone, PartialEq, Eq, Debug)]
struct PathKey(PathBuf);

impl PathKey {
    fn new(path: PathBuf) -> Self {
        match normalized(&path) {
            Cow::Borrowed(_) => Self(path),
            Cow::Owned(rebuilt) => Self(rebuilt),
        }
    }

    fn bytes(&self) -> &[u8] {
        self.0.as_os_str().as_encoded_bytes()
    }
}

/// `path` spelled the way [`PathKey`] spells it.
fn normalized(path: &Path) -> Cow<'_, Path> {
    if std::path::MAIN_SEPARATOR != '/'
        && std::path::is_separator('/')
        && path.as_os_str().as_encoded_bytes().contains(&b'/')
    {
        Cow::Owned(path.components().collect())
    } else {
        Cow::Borrowed(path)
    }
}

impl Ord for PathKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes().cmp(other.bytes())
    }
}

impl PartialOrd for PathKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Lookups borrow the key as bytes, so `get` and `remove` take a `&Path` without
// allocating. The contract `Borrow` demands -- that the borrowed form orders the same
// way as the owned one -- holds by construction: `Ord` above *is* the byte order.
impl Borrow<[u8]> for PathKey {
    fn borrow(&self) -> &[u8] {
        self.bytes()
    }
}

fn path_bytes(path: &Path) -> &[u8] {
    path.as_os_str().as_encoded_bytes()
}

/// Optional derived-data tier owned by an index only after analysis is enabled.
///
/// The tier holds records of exactly one [`ContentTierIdentity`]: preparing it for another
/// identity clears it, and a record of another identity is refused. So every record the
/// tier holds answers the request it was prepared for, and none answers any other.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ContentIndex {
    identity: Option<ContentTierIdentity>,
    state: Option<ContentTierState>,
    files: BTreeMap<PathKey, FileAnalysis>,
    rollups: HashMap<PathBuf, ContentRollUp>,
}

/// Operational provenance of the content tier, separate from its semantic identity.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ContentTierState {
    pub source: Source,
    pub freshness: Freshness,
    pub observed_at_ns: Option<i64>,
}

impl ContentIndex {
    /// Number of sparse file records.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether no analysis records are present.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// The identity every record in this tier was produced under, even when the tree is
    /// empty.
    pub fn identity(&self) -> Option<&ContentTierIdentity> {
        self.identity.as_ref()
    }

    /// Analyzer set this derived tier holds records for, even when the tree is empty.
    pub fn profile(&self) -> Option<AnalysisSet> {
        self.identity.as_ref().map(|identity| identity.analysis)
    }

    /// Analyzer, rule, and option identity every record in this derived tier carries.
    pub fn provenance(&self) -> Option<ContentProvenance> {
        self.identity.as_ref().map(ContentTierIdentity::record_provenance)
    }

    pub(crate) const fn state(&self) -> Option<ContentTierState> {
        self.state
    }

    pub(crate) fn set_state(&mut self, state: ContentTierState) {
        self.state = Some(state);
    }

    /// Borrow one file's analysis.
    pub fn file(&self, path: &Path) -> Option<&FileAnalysis> {
        self.files.get(path_bytes(&normalized(path)))
    }

    /// Borrow a directory's precomputed subtree rollup.
    pub fn rollup(&self, path: &Path) -> Option<&ContentRollUp> {
        self.rollups.get(path)
    }

    pub(crate) fn records(&self) -> impl Iterator<Item = (&Path, &FileAnalysis)> {
        self.files.iter().map(|(key, analysis)| (key.0.as_path(), analysis))
    }

    /// Commit one record, or refuse it when it was produced under another identity than
    /// the one this tier was prepared for.
    ///
    /// A refusal changes nothing. Adopting the record's identity instead would clear
    /// every record of the prepared one, and keeping both would leave a tier whose totals
    /// match neither request.
    #[must_use = "a refused record was not committed"]
    pub(crate) fn commit(&mut self, path: PathBuf, analysis: FileAnalysis) -> bool {
        self.commit_record(path, analysis, true)
    }

    /// Insert or replace a record without touching roll-ups.
    ///
    /// Sidecar restore inserts every cached file first, then rebuilds directory
    /// totals once. Incremental [`commit`] still walks ancestors per file.
    #[must_use = "a refused record was not committed"]
    pub(crate) fn commit_without_rollup(&mut self, path: PathBuf, analysis: FileAnalysis) -> bool {
        self.commit_record(path, analysis, false)
    }

    #[must_use = "a refused record was not committed"]
    fn commit_record(
        &mut self,
        path: PathBuf,
        analysis: FileAnalysis,
        update_rollups: bool,
    ) -> bool {
        let Some(identity) = &self.identity else {
            return false;
        };
        if !analysis.matches_profile(identity.analysis) {
            return false;
        }
        let key = PathKey::new(path);
        if let Some(previous) = self.files.remove(key.bytes()) {
            if update_rollups {
                self.merge_ancestors(&key.0, &previous, false);
            }
        }
        if update_rollups {
            self.merge_ancestors(&key.0, &analysis, true);
        }
        self.files.insert(key, analysis);
        true
    }

    pub(crate) fn invalidate(&mut self, path: &Path) {
        // The record at `path` itself, if it is a file, plus everything beneath it if it
        // is a directory: in byte order those are `path` and then the contiguous run of
        // keys that begin with `path` and the separator keys are spelled with. The root
        // (an empty path) has no separator form and owns every record.
        let path = normalized(path);
        let mut removed: Vec<(PathBuf, FileAnalysis)> = Vec::new();
        if let Some((key, analysis)) = self.files.get_key_value(path_bytes(&path)) {
            removed.push((key.0.clone(), analysis.clone()));
        }
        let prefix: Vec<u8> = if path.as_os_str().is_empty() {
            Vec::new()
        } else {
            let mut prefix = path_bytes(&path).to_vec();
            prefix.push(std::path::MAIN_SEPARATOR as u8);
            prefix
        };
        removed.extend(
            self.files
                .range::<[u8], _>((
                    std::ops::Bound::Included(prefix.as_slice()),
                    std::ops::Bound::Unbounded,
                ))
                .take_while(|(key, _)| key.bytes().starts_with(&prefix))
                .filter(|(key, _)| key.0 != *path)
                .map(|(key, analysis)| (key.0.clone(), analysis.clone())),
        );
        for (candidate, analysis) in removed {
            self.files.remove(path_bytes(&candidate));
            self.merge_ancestors(&candidate, &analysis, false);
        }
    }

    /// Hold records of `identity` from here on.
    ///
    /// Equality, not containment: records of any other identity answer another request,
    /// so a tier prepared for a different one is cleared, whether the stored analyzer set
    /// is wider, narrower, or produced under other rules, versions, options, or entries.
    pub(crate) fn prepare(&mut self, identity: ContentTierIdentity) {
        if self.identity.as_ref() == Some(&identity) {
            return;
        }
        self.files.clear();
        self.rollups.clear();
        self.state = None;
        self.identity = Some(identity);
    }

    fn merge_ancestors(&mut self, file: &Path, analysis: &FileAnalysis, add: bool) {
        let mut directory = file.parent();
        while let Some(path) = directory {
            if add {
                // `get_mut` before `insert`, not `entry`: `entry` needs an owned key, so
                // it allocates a `PathBuf` for every ancestor of every file even when
                // the roll-up is already there, which is the overwhelmingly common case.
                if let Some(rollup) = self.rollups.get_mut(path) {
                    rollup.add(analysis);
                } else {
                    self.rollups.entry(path.to_path_buf()).or_default().add(analysis);
                }
            } else if let Some(rollup) = self.rollups.get_mut(path) {
                rollup.subtract(analysis);
                if rollup.total.files == 0 {
                    self.rollups.remove(path);
                }
            }
            directory = path.parent();
        }
    }

    /// Rebuild every directory roll-up from the files now held.
    ///
    /// Each file is added only to its parent, then each directory merges into its
    /// parent from the deepest path first. This removes repeated per-file propagation
    /// through every ancestor; the rebuild also discovers and sorts the directory set.
    pub(crate) fn rebuild_rollups(&mut self) {
        self.rollups.clear();
        for (key, analysis) in &self.files {
            let Some(parent) = key.0.parent() else {
                continue;
            };
            if let Some(rollup) = self.rollups.get_mut(parent) {
                rollup.add(analysis);
            } else {
                self.rollups.entry(parent.to_path_buf()).or_default().add(analysis);
            }
        }

        let parents: Vec<PathBuf> = self.rollups.keys().cloned().collect();
        for dir in &parents {
            let mut ancestor = dir.parent();
            while let Some(path) = ancestor {
                if !self.rollups.contains_key(path) {
                    self.rollups.insert(path.to_path_buf(), ContentRollUp::default());
                }
                ancestor = path.parent();
            }
        }

        let mut dirs: Vec<PathBuf> = self.rollups.keys().cloned().collect();
        dirs.sort_unstable_by(|left, right| {
            right
                .components()
                .count()
                .cmp(&left.components().count())
                .then_with(|| left.as_os_str().cmp(right.as_os_str()))
        });
        for dir in dirs {
            let Some(parent) = dir.parent() else {
                continue;
            };
            let Some(child) = self.rollups.remove(&dir) else {
                continue;
            };
            if let Some(parent_rollup) = self.rollups.get_mut(parent) {
                parent_rollup.merge(&child);
            } else {
                self.rollups.entry(parent.to_path_buf()).or_default().merge(&child);
            }
            self.rollups.insert(dir, child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::classify_path;
    use crate::content::{
        AnalysisRequest, AnalysisSet, AnalyzerOutcome, BasicMetrics, ContentProvenance,
        FileAnalysis,
    };
    use crate::{AnalyzerProvenance, EntryTierIdentity, Fingerprint, ScanConfig};

    fn lines() -> AnalysisSet {
        AnalysisSet::NONE.with_lines()
    }

    fn identity_for(analysis: AnalysisSet) -> ContentTierIdentity {
        let entries = ScanConfig::default().snapshot_identity().entries;
        let records = ContentProvenance::for_request(
            AnalysisRequest { profile: analysis, ..AnalysisRequest::default() },
            entries.type_rules_fingerprint,
        );
        ContentTierIdentity::of_records(entries, analysis, &records)
            .expect("records under the entry tier's type rules")
    }

    /// A tier prepared for the `lines` identity every [`analysis`] record carries.
    fn prepared() -> ContentIndex {
        let mut index = ContentIndex::default();
        index.prepare(identity_for(lines()));
        index
    }

    fn analysis(path: &str, lines: u64) -> FileAnalysis {
        FileAnalysis {
            fingerprint: Fingerprint::default(),
            bytes: 10,
            detection: classify_path(Path::new(path)).into(),
            lines: AnalyzerOutcome::analyzed(BasicMetrics {
                physical_lines: lines,
                nonblank_lines: lines,
                ..BasicMetrics::default()
            }),
            code: None,
            words: None,
            error: None,
        }
    }

    /// Commit a record the test expects the tier to accept.
    fn commit(index: &mut ContentIndex, path: &str, record: FileAnalysis) {
        assert!(index.commit(PathBuf::from(path), record), "{path} must commit");
    }

    #[test]
    fn prepare_clears_on_any_identity_change() {
        let base = identity_for(lines());
        let mut other_version = base.clone();
        other_version.provenance.analyzers[0].1 = crate::content::AnalyzerVersion(2);
        let changes = [
            ("a wider analyzer set", identity_for(AnalysisSet::ALL)),
            ("another analyzer set", identity_for(AnalysisSet::NONE.with_code())),
            (
                "another entry tier",
                ContentTierIdentity {
                    entries: EntryTierIdentity { engine: base.entries.engine ^ 1, ..base.entries },
                    ..base.clone()
                },
            ),
            (
                "other type rules",
                ContentTierIdentity {
                    entries: EntryTierIdentity {
                        type_rules_fingerprint: base.entries.type_rules_fingerprint ^ 1,
                        ..base.entries
                    },
                    ..base.clone()
                },
            ),
            (
                "other options",
                ContentTierIdentity {
                    provenance: AnalyzerProvenance {
                        options_fingerprint: crate::content::OptionsFingerprint(
                            base.provenance.options_fingerprint.0 ^ 1,
                        ),
                        ..base.provenance.clone()
                    },
                    ..base.clone()
                },
            ),
            ("another analyzer version", other_version),
        ];
        for (name, identity) in changes {
            let mut index = prepared();
            commit(&mut index, "src/lib.rs", analysis("src/lib.rs", 2));
            index.prepare(base.clone());
            assert_eq!(index.len(), 1, "the same identity keeps its records");

            index.prepare(identity.clone());
            assert!(index.is_empty(), "{name} clears the records");
            assert!(index.rollup(Path::new("")).is_none(), "{name} clears the roll-ups");
            assert_eq!(index.identity(), Some(&identity), "{name} is the tier's identity now");
        }
    }

    #[test]
    fn commit_refuses_a_record_when_the_tier_is_unprepared() {
        let mut unprepared = ContentIndex::default();
        assert!(
            !unprepared.commit(PathBuf::from("a.rs"), analysis("a.rs", 1)),
            "a tier prepared for nothing holds no record"
        );
        assert_eq!(unprepared, ContentIndex::default());
    }

    #[test]
    fn commit_refuses_records_whose_unit_slots_do_not_match_the_prepared_profile() {
        let mut index = ContentIndex::default();
        index.prepare(identity_for(AnalysisSet::ALL));

        let record = analysis("a.rs", 1);
        assert!(
            !index.commit(PathBuf::from("a.rs"), record),
            "an all-unit tier must not admit a lines-only record"
        );
        assert!(index.is_empty());

        let mut index = prepared();
        let mut record = analysis("a.rs", 1);
        record.code = Some(AnalyzerOutcome::unavailable(CoverageReason::Unsupported));
        assert!(
            !index.commit(PathBuf::from("a.rs"), record),
            "a lines-only tier must not admit an unrequested code slot"
        );
        assert!(index.is_empty());
    }

    #[test]
    fn replacement_and_subtree_invalidation_update_every_rollup() {
        let mut index = prepared();
        commit(&mut index, "src/lib.rs", analysis("src/lib.rs", 2));
        commit(&mut index, "src/main.rs", analysis("src/main.rs", 3));
        assert_eq!(
            index.rollup(Path::new("")).expect("root").total.lines.metrics.physical_lines,
            5
        );
        assert_eq!(index.rollup(Path::new("src")).expect("src").total.files, 2);

        commit(&mut index, "src/lib.rs", analysis("src/lib.rs", 7));
        assert_eq!(
            index.rollup(Path::new("")).expect("root").total.lines.metrics.physical_lines,
            10
        );

        index.invalidate(Path::new("src"));
        assert!(index.is_empty());
        assert!(index.rollup(Path::new("")).is_none());
    }

    #[test]
    fn invalidation_by_byte_prefix_stops_at_the_separator() {
        // In byte order `src-extra/a.rs` sorts before `src/a.rs` and `src2/b.rs` after
        // it; neither is beneath `src`, and the separator in the prefix is what keeps
        // them out. The root invalidates everything, and a file path invalidates only
        // its own record.
        let mut index = prepared();
        for path in ["src/a.rs", "src/deep/b.rs", "src-extra/a.rs", "src2/b.rs", "srcfile"] {
            commit(&mut index, path, analysis(path, 1));
        }
        assert_eq!(index.len(), 5);

        // A path spelled with the other separator is the same path wherever `Path` says
        // so -- Windows -- and a different file named `src\\c.rs` at the root everywhere
        // else. Either way the map agrees with `Path::starts_with` and `Path::eq`.
        let other = PathBuf::from("src\\c.rs");
        assert!(index.commit(other.clone(), analysis("src/c.rs", 1)));
        let beneath = other.starts_with("src");
        assert_eq!(index.file(Path::new("src/c.rs")).is_some(), beneath);
        assert!(index.file(&other).is_some());

        index.invalidate(Path::new("src"));
        assert_eq!(index.len(), if beneath { 3 } else { 4 });
        assert_eq!(index.file(&other).is_none(), beneath);
        assert!(index.file(Path::new("src/a.rs")).is_none());
        assert!(index.file(Path::new("src/deep/b.rs")).is_none());
        assert!(index.file(Path::new("src-extra/a.rs")).is_some());
        assert!(index.file(Path::new("src2/b.rs")).is_some());
        assert!(index.file(Path::new("srcfile")).is_some());
        assert_eq!(
            index.rollup(Path::new("")).expect("root").total.files,
            if beneath { 3 } else { 4 }
        );
        assert!(index.rollup(Path::new("src")).is_none());

        index.invalidate(Path::new("srcfile"));
        assert_eq!(index.len(), if beneath { 2 } else { 3 });
        assert!(index.file(Path::new("srcfile")).is_none());

        index.invalidate(Path::new(""));
        assert!(index.is_empty());
        assert!(index.rollup(Path::new("")).is_none());
    }

    #[test]
    fn records_are_ordered_deterministically_by_bytes() {
        let mut index = prepared();
        for path in ["b/x.rs", "a/z.rs", "a/y.rs", "a-b/q.rs"] {
            commit(&mut index, path, analysis(path, 1));
        }
        let order: Vec<&Path> = index.records().map(|(path, _)| path).collect();
        assert_eq!(
            order,
            vec![
                Path::new("a-b/q.rs"),
                Path::new("a/y.rs"),
                Path::new("a/z.rs"),
                Path::new("b/x.rs")
            ]
        );
    }

    #[test]
    fn bottom_up_rebuild_matches_incremental_nested_rollups() {
        let paths = ["README.md", "a/keep.rs", "a/b/nested.rs", "a/b/c/deep.rs", "a/b/c/other.py"];
        let mut binary = analysis("a/b/c/image.png", 0);
        binary.coverage = CoverageReason::Binary;
        binary.metrics = MetricValues::default();
        let mut incremental = prepared();
        for path in paths {
            commit(&mut incremental, path, analysis(path, 3));
        }
        commit(&mut incremental, "a/b/c/image.png", binary.clone());

        let mut rebuilt = prepared();
        for path in paths {
            assert!(
                rebuilt.commit_without_rollup(PathBuf::from(path), analysis(path, 3)),
                "{path} must commit"
            );
        }
        assert!(
            rebuilt.commit_without_rollup(PathBuf::from("a/b/c/image.png"), binary),
            "binary coverage must commit"
        );
        assert!(rebuilt.rollup(Path::new("")).is_none(), "deferred inserts leave roll-ups empty");
        rebuilt.rebuild_rollups();

        for dir in ["", "a", "a/b", "a/b/c"] {
            assert_eq!(
                rebuilt.rollup(Path::new(dir)),
                incremental.rollup(Path::new(dir)),
                "{dir:?} roll-up"
            );
        }
        assert_eq!(rebuilt, incremental);
    }
}
