//! Sparse per-file content records and precomputed directory/group rollups.

use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use crate::classify::ContentFamily;

use super::content_model::{
    AnalysisSet, ContentProvenance, CoverageReason, FileAnalysis, MetricValues,
};

/// Additive tally for one type or family group.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct MetricTally {
    /// Files represented, including skipped coverage records.
    pub files: u64,
    /// Apparent bytes represented.
    pub bytes: u64,
    /// Files with complete analyzer coverage.
    pub analyzed_files: u64,
    /// Additive metrics from analyzed files.
    pub metrics: MetricValues,
}

impl MetricTally {
    fn add(&mut self, analysis: &FileAnalysis) {
        self.files = self.files.saturating_add(1);
        self.bytes = self.bytes.saturating_add(analysis.bytes);
        if analysis.coverage == CoverageReason::Analyzed {
            self.analyzed_files = self.analyzed_files.saturating_add(1);
            self.metrics.add_assign(&analysis.metrics);
        }
    }

    fn subtract(&mut self, analysis: &FileAnalysis) {
        self.files = self.files.saturating_sub(1);
        self.bytes = self.bytes.saturating_sub(analysis.bytes);
        if analysis.coverage == CoverageReason::Analyzed {
            self.analyzed_files = self.analyzed_files.saturating_sub(1);
            self.metrics.sub_assign(&analysis.metrics);
        }
    }
}

/// Content totals for one directory subtree.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ContentRollUp {
    /// All sparse file records beneath this directory.
    pub total: MetricTally,
    /// Tallies keyed by stable file type id.
    pub by_type: BTreeMap<String, MetricTally>,
    /// Tallies keyed by broad content family.
    pub by_family: BTreeMap<ContentFamily, MetricTally>,
    /// Coverage outcomes across requested files.
    pub coverage: BTreeMap<CoverageReason, u64>,
}

impl ContentRollUp {
    fn add(&mut self, analysis: &FileAnalysis) {
        self.total.add(analysis);
        self.by_type
            .entry(analysis.classification.file_type.as_str().to_string())
            .or_default()
            .add(analysis);
        self.by_family.entry(analysis.classification.family).or_default().add(analysis);
        *self.coverage.entry(analysis.coverage).or_default() += 1;
    }

    fn subtract(&mut self, analysis: &FileAnalysis) {
        self.total.subtract(analysis);
        let type_id = analysis.classification.file_type.as_str();
        if let Some(tally) = self.by_type.get_mut(type_id) {
            tally.subtract(analysis);
            if tally.files == 0 {
                self.by_type.remove(type_id);
            }
        }
        let family = analysis.classification.family;
        if let Some(tally) = self.by_family.get_mut(&family) {
            tally.subtract(analysis);
            if tally.files == 0 {
                self.by_family.remove(&family);
            }
        }
        if let Some(count) = self.coverage.get_mut(&analysis.coverage) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.coverage.remove(&analysis.coverage);
            }
        }
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
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ContentIndex {
    profile: Option<AnalysisSet>,
    provenance: Option<ContentProvenance>,
    files: BTreeMap<PathKey, FileAnalysis>,
    rollups: HashMap<PathBuf, ContentRollUp>,
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

    /// Requested profile represented by this derived tier, even when the tree is empty.
    pub fn profile(&self) -> Option<AnalysisSet> {
        self.profile
    }

    /// Analyzer, rule, and option identity represented by this derived tier.
    pub fn provenance(&self) -> Option<&ContentProvenance> {
        self.provenance.as_ref()
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

    pub(crate) fn commit(&mut self, path: PathBuf, analysis: FileAnalysis) {
        self.prepare(analysis.profile, analysis.provenance.clone());
        let key = PathKey::new(path);
        if let Some(previous) = self.files.remove(key.bytes()) {
            self.merge_ancestors(&key.0, &previous, false);
        }
        self.merge_ancestors(&key.0, &analysis, true);
        self.files.insert(key, analysis);
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

    pub(crate) fn prepare(&mut self, profile: AnalysisSet, provenance: ContentProvenance) {
        // Keep a wider tier intact when a narrower request arrives: its records already
        // answer the narrower question, and overwriting the stored set with the request's
        // would discard analyzers the records still carry.
        //
        // The incoming provenance carries the rules now in effect, so the comparison is
        // held-against-incoming rather than held-against-a-global: different rules mean
        // the stored records answer a different question and must go.
        let retained = self.profile.is_some_and(|stored| {
            self.provenance.as_ref().is_some_and(|held| {
                held.satisfies(stored, profile, provenance.type_rules_fingerprint)
            })
        });
        if retained {
            return;
        }
        self.files.clear();
        self.rollups.clear();
        self.profile = Some(profile);
        self.provenance = Some(provenance);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Fingerprint;
    use crate::classify::classify_path;
    use crate::content::{AnalysisRequest, AnalysisSet, ContentProvenance, FileAnalysis};

    fn analysis(path: &str, lines: u64) -> FileAnalysis {
        FileAnalysis {
            classification: classify_path(Path::new(path)),
            fingerprint: Fingerprint::default(),
            bytes: 10,
            profile: AnalysisSet::NONE.with_lines(),
            provenance: ContentProvenance::for_request(
                AnalysisRequest {
                    profile: AnalysisSet::NONE.with_lines(),
                    ..AnalysisRequest::default()
                },
                crate::classify::type_rule_fingerprint(),
            ),
            metrics: MetricValues {
                physical_lines: lines,
                nonblank_lines: lines,
                ..MetricValues::default()
            },
            coverage: CoverageReason::Analyzed,
            error: None,
        }
    }

    #[test]
    fn replacement_and_subtree_invalidation_update_every_rollup() {
        let mut index = ContentIndex::default();
        index.commit(PathBuf::from("src/lib.rs"), analysis("src/lib.rs", 2));
        index.commit(PathBuf::from("src/main.rs"), analysis("src/main.rs", 3));
        assert_eq!(index.rollup(Path::new("")).expect("root").total.metrics.physical_lines, 5);
        assert_eq!(index.rollup(Path::new("src")).expect("src").total.files, 2);

        index.commit(PathBuf::from("src/lib.rs"), analysis("src/lib.rs", 7));
        assert_eq!(index.rollup(Path::new("")).expect("root").total.metrics.physical_lines, 10);

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
        let mut index = ContentIndex::default();
        for path in ["src/a.rs", "src/deep/b.rs", "src-extra/a.rs", "src2/b.rs", "srcfile"] {
            index.commit(PathBuf::from(path), analysis(path, 1));
        }
        assert_eq!(index.len(), 5);

        // A path spelled with the other separator is the same path wherever `Path` says
        // so -- Windows -- and a different file named `src\\c.rs` at the root everywhere
        // else. Either way the map agrees with `Path::starts_with` and `Path::eq`.
        let other = PathBuf::from("src\\c.rs");
        index.commit(other.clone(), analysis("src/c.rs", 1));
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
        let mut index = ContentIndex::default();
        for path in ["b/x.rs", "a/z.rs", "a/y.rs", "a-b/q.rs"] {
            index.commit(PathBuf::from(path), analysis(path, 1));
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
}
