//! Sparse per-file content records and precomputed directory/group rollups.

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

/// Optional derived-data tier owned by an index only after analysis is enabled.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ContentIndex {
    profile: Option<AnalysisSet>,
    provenance: Option<ContentProvenance>,
    files: BTreeMap<PathBuf, FileAnalysis>,
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
        self.files.get(path)
    }

    /// Borrow a directory's precomputed subtree rollup.
    pub fn rollup(&self, path: &Path) -> Option<&ContentRollUp> {
        self.rollups.get(path)
    }

    pub(crate) fn records(&self) -> impl Iterator<Item = (&Path, &FileAnalysis)> {
        self.files.iter().map(|(path, analysis)| (path.as_path(), analysis))
    }

    pub(crate) fn commit(&mut self, path: PathBuf, analysis: FileAnalysis) {
        self.prepare(analysis.profile, analysis.provenance.clone());
        if let Some(previous) = self.files.remove(&path) {
            self.merge_ancestors(&path, &previous, false);
        }
        self.merge_ancestors(&path, &analysis, true);
        self.files.insert(path, analysis);
    }

    pub(crate) fn invalidate(&mut self, path: &Path) {
        let removed: Vec<(PathBuf, FileAnalysis)> = self
            .files
            .range(path.to_path_buf()..)
            .take_while(|(candidate, _)| candidate.starts_with(path))
            .map(|(candidate, analysis)| (candidate.clone(), analysis.clone()))
            .collect();
        for (candidate, analysis) in removed {
            self.files.remove(&candidate);
            self.merge_ancestors(&candidate, &analysis, false);
        }
    }

    pub(crate) fn prepare(&mut self, profile: AnalysisSet, provenance: ContentProvenance) {
        // Keep a wider tier intact when a narrower request arrives: its records already
        // answer the narrower question, and overwriting the stored set with the request's
        // would discard analyzers the records still carry.
        let retained = self.profile.is_some_and(|stored| {
            self.provenance.as_ref().is_some_and(|held| held.satisfies(stored, profile))
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
            provenance: ContentProvenance::for_request(AnalysisRequest {
                profile: AnalysisSet::NONE.with_lines(),
                ..AnalysisRequest::default()
            }),
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
}
