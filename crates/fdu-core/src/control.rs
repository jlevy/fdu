//! Bounded, removal-aware control state used to classify retained filesystem facts.
//!
//! A control file is producer input, not something the index discovers by walking the
//! filesystem. The table stores the exact bytes a producer verified and the matcher
//! derived from them. That keeps cold discovery, refresh, observation, and snapshot load
//! on one semantic path and makes deleting the last control file an ordinary state
//! transition rather than a special rebuild.

#[cfg(feature = "gitignore")]
mod gitignore;

#[cfg(not(feature = "gitignore"))]
mod gitignore {
    use std::path::Path;

    #[derive(Clone, Debug, Default)]
    pub(super) struct Gitignore;

    impl Gitignore {
        pub(super) const fn matches(&self, _relative: &Path, _is_dir: bool) -> Option<bool> {
            let _ = self;
            None
        }
    }
}

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use gitignore::Gitignore;

/// Name of the fixed control file understood by the first engine version.
pub const CONTROL_FILE_NAME: &str = ".gitignore";

/// Maximum retained control-table cost for one index.
///
/// The limit includes a fixed charge per source as well as its exact bytes, so a hostile
/// tree of empty control files cannot evade the bound. Four MiB is far above ordinary
/// repositories while remaining small relative to the inventory it governs.
pub const MAX_CONTROL_TABLE_BYTES: usize = 4 * 1024 * 1024;

/// Maximum bytes in one parsed ignore pattern line.
///
/// A control file may contain many ordinary rules up to the shared table bound, but a
/// single adversarial rule must not impose unbounded matching work on every entry.
pub const MAX_CONTROL_PATTERN_BYTES: usize = 16 * 1024;

/// Conservative retained charge for one key, identity, and matcher shell.
pub(crate) const CONTROL_SOURCE_OVERHEAD: usize = 64;

/// Stable, non-sensitive identity of one retained control source.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ControlIdentity {
    /// Exact source length.
    pub bytes: u64,
    /// Stable FNV-1a digest of the source bytes.
    pub fingerprint: u64,
}

#[derive(Clone, Debug)]
struct ControlSource {
    bytes: Vec<u8>,
    identity: ControlIdentity,
    matcher: Gitignore,
    retained_cost: usize,
}

impl ControlSource {
    #[cfg(feature = "gitignore")]
    fn new(bytes: Vec<u8>, retained_cost: usize) -> Self {
        let identity = identity(&bytes);
        let matcher = Gitignore::parse(&bytes);
        Self { bytes, identity, matcher, retained_cost }
    }
}

/// Exact `.gitignore` sources and parsed matchers, keyed by governing directory.
#[derive(Clone, Debug, Default)]
pub struct ControlTable {
    by_directory: BTreeMap<PathBuf, ControlSource>,
    source_bytes: usize,
    retained_cost: usize,
}

impl ControlTable {
    /// Insert or replace one verified control source.
    ///
    /// `path` names the control file relative to the index root. The source is retained
    /// exactly, while matching state is derived once here rather than per entry.
    pub fn upsert(&mut self, path: &Path, source: Vec<u8>) -> crate::Result<bool> {
        #[cfg(not(feature = "gitignore"))]
        {
            let _ = (path, source);
            Err(crate::Error::UnsupportedScanConfig(
                "control observations require the fdu-core `gitignore` feature",
            ))
        }
        #[cfg(feature = "gitignore")]
        {
            let directory = control_directory(path)?;
            if let Some(line) = source
                .split(|byte| *byte == b'\n')
                .find(|line| line.len() > MAX_CONTROL_PATTERN_BYTES)
            {
                return Err(crate::Error::ControlPatternLimit {
                    attempted: line.len(),
                    limit: MAX_CONTROL_PATTERN_BYTES,
                });
            }
            let replaced = self.by_directory.get(directory).map_or(0, |value| value.retained_cost);
            let incoming_cost = retained_source_cost(directory, &source);
            let next = self
                .retained_cost
                .checked_sub(replaced)
                .and_then(|bytes| bytes.checked_add(incoming_cost))
                .ok_or(crate::Error::ControlSourceLimit {
                    attempted: usize::MAX,
                    limit: MAX_CONTROL_TABLE_BYTES,
                })?;
            if next > MAX_CONTROL_TABLE_BYTES {
                return Err(crate::Error::ControlSourceLimit {
                    attempted: next,
                    limit: MAX_CONTROL_TABLE_BYTES,
                });
            }

            let incoming = ControlSource::new(source, incoming_cost);
            if self
                .by_directory
                .get(directory)
                .is_some_and(|current| current.bytes == incoming.bytes)
            {
                return Ok(false);
            }
            let previous = self.by_directory.insert(directory.to_path_buf(), incoming);
            self.source_bytes = self
                .source_bytes
                .saturating_sub(previous.as_ref().map_or(0, |value| value.bytes.len()))
                .saturating_add(self.by_directory[directory].bytes.len());
            self.retained_cost = next;
            Ok(true)
        }
    }

    /// Remove one control source. Missing sources are no-ops.
    pub fn remove(&mut self, path: &Path) -> crate::Result<bool> {
        let directory = control_directory(path)?;
        let Some(removed) = self.by_directory.remove(directory) else {
            return Ok(false);
        };
        self.source_bytes -= removed.bytes.len();
        self.retained_cost -= removed.retained_cost;
        Ok(true)
    }

    /// Remove every control file at or below `subtree`.
    pub(crate) fn remove_subtree(&mut self, subtree: &Path) {
        let directories: Vec<PathBuf> = self
            .by_directory
            .keys()
            .filter(|directory| directory.starts_with(subtree))
            .cloned()
            .collect();
        for directory in directories {
            if let Some(removed) = self.by_directory.remove(&directory) {
                self.source_bytes -= removed.bytes.len();
                self.retained_cost -= removed.retained_cost;
            }
        }
    }

    /// Matcher view for one retained path.
    pub fn matcher_for<'a>(&'a self, path: &'a Path) -> ControlMatcher<'a> {
        ControlMatcher { table: self, path }
    }

    /// Evaluate complete ignore semantics without relying on retained parent facts.
    ///
    /// The index hot path uses [`ControlMatcher::is_ignored`] with the parent's stored
    /// classification. This standalone form evaluates each directory prefix so callers
    /// and tests receive the same answer even without an index entry in hand.
    pub fn is_ignored(&self, path: &Path, is_dir: bool) -> bool {
        let components: Vec<_> = path.components().collect();
        let mut current = PathBuf::new();
        let mut parent_ignored = false;
        for (position, component) in components.iter().enumerate() {
            current.push(component.as_os_str());
            if parent_ignored {
                return true;
            }
            let current_is_dir = position + 1 < components.len() || is_dir;
            parent_ignored = self.matcher_for(&current).is_ignored(current_is_dir);
        }
        parent_ignored
    }

    /// Relative subtree whose classification may move when `path` changes.
    pub fn affected_subtree(path: &Path) -> crate::Result<PathBuf> {
        Ok(control_directory(path)?.to_path_buf())
    }

    /// Stable identities changed between two complete table states.
    pub(crate) fn changes_from(
        &self,
        previous: &Self,
    ) -> Vec<(PathBuf, Option<ControlIdentity>, Option<ControlIdentity>)> {
        let directories: BTreeSet<&Path> = previous
            .by_directory
            .keys()
            .chain(self.by_directory.keys())
            .map(PathBuf::as_path)
            .collect();
        directories
            .into_iter()
            .filter_map(|directory| {
                let before = previous.by_directory.get(directory);
                let after = self.by_directory.get(directory);
                let changed = match (before, after) {
                    (Some(before), Some(after)) => before.bytes != after.bytes,
                    (None, None) => false,
                    (Some(_), None) | (None, Some(_)) => true,
                };
                changed.then(|| {
                    (
                        control_path(directory),
                        before.map(|source| source.identity),
                        after.map(|source| source.identity),
                    )
                })
            })
            .collect()
    }

    /// Exact sources in deterministic governing-directory order.
    pub(crate) fn sources(&self) -> impl ExactSizeIterator<Item = (PathBuf, &[u8])> {
        self.by_directory
            .iter()
            .map(|(directory, source)| (control_path(directory), source.bytes.as_slice()))
    }

    /// Exact retained source bytes across the whole table.
    pub const fn source_bytes(&self) -> usize {
        self.source_bytes
    }

    /// Bounded retained charge for the complete table.
    pub const fn retained_cost(&self) -> usize {
        self.retained_cost
    }

    /// Whether `path` already retains exactly `source`.
    pub fn source_is(&self, path: &Path, source: &[u8]) -> bool {
        control_directory(path)
            .ok()
            .and_then(|directory| self.by_directory.get(directory))
            .is_some_and(|current| current.bytes == source)
    }

    /// Whether an exact control source is retained at `path`.
    pub(crate) fn contains(&self, path: &Path) -> bool {
        control_directory(path)
            .ok()
            .is_some_and(|directory| self.by_directory.contains_key(directory))
    }

    /// Number of retained control files.
    pub fn len(&self) -> usize {
        self.by_directory.len()
    }

    /// Whether no control file is retained.
    pub fn is_empty(&self) -> bool {
        self.by_directory.is_empty()
    }
}

/// A path-bound view over the controls that may govern it.
pub struct ControlMatcher<'a> {
    table: &'a ControlTable,
    path: &'a Path,
}

impl ControlMatcher<'_> {
    /// Decide this path assuming its retained parent is not ignored.
    ///
    /// A caller with retained facts already knows the parent's effective classification,
    /// so it can stop immediately when that parent is ignored. Otherwise every control
    /// directory on this path is active and the deepest matching opinion wins.
    pub fn is_ignored(&self, is_dir: bool) -> bool {
        let mut verdict = false;
        let mut directories: Vec<&Path> =
            self.path.parent().into_iter().flat_map(Path::ancestors).collect();
        directories.reverse();
        for directory in directories {
            let Some(source) = self.table.by_directory.get(directory) else {
                continue;
            };
            let relative = self.path.strip_prefix(directory).unwrap_or(self.path);
            if let Some(ignored) = source.matcher.matches(relative, is_dir) {
                verdict = ignored;
            }
        }
        verdict
    }
}

/// Whether a relative path names the fixed control file.
pub fn is_control_file(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == CONTROL_FILE_NAME)
}

fn control_directory(path: &Path) -> crate::Result<&Path> {
    if !is_control_file(path) {
        return Err(crate::Error::InvalidControlPath(path.to_path_buf()));
    }
    Ok(path.parent().unwrap_or_else(|| Path::new("")))
}

fn control_path(directory: &Path) -> PathBuf {
    directory.join(CONTROL_FILE_NAME)
}

#[cfg(feature = "gitignore")]
fn identity(bytes: &[u8]) -> ControlIdentity {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x100_0000_01b3;

    let mut fingerprint = FNV_OFFSET_BASIS;
    for byte in bytes {
        fingerprint ^= u64::from(*byte);
        fingerprint = fingerprint.wrapping_mul(FNV_PRIME);
    }
    ControlIdentity { bytes: u64::try_from(bytes.len()).unwrap_or(u64::MAX), fingerprint }
}

#[cfg(feature = "gitignore")]
fn retained_source_cost(directory: &Path, source: &[u8]) -> usize {
    let (newlines, segment_shells) = source.iter().fold((0usize, 0usize), |counts, byte| {
        (counts.0 + usize::from(*byte == b'\n'), counts.1 + usize::from(*byte == b'/'))
    });
    let pattern_shells = newlines.saturating_add(1);
    CONTROL_SOURCE_OVERHEAD
        .saturating_add(directory.as_os_str().as_encoded_bytes().len())
        // Exact source and the parsed glob bytes are both retained.
        .saturating_add(source.len().saturating_mul(2))
        .saturating_add(pattern_shells.saturating_mul(64))
        .saturating_add(segment_shells.saturating_mul(24))
}

#[cfg(all(test, feature = "gitignore"))]
pub(crate) fn source_at_test_limit() -> Vec<u8> {
    let mut source = Vec::new();
    loop {
        let previous_len = source.len();
        source.extend(std::iter::repeat_n(b'a', MAX_CONTROL_PATTERN_BYTES));
        source.push(b'\n');
        if retained_source_cost(Path::new(""), &source) > MAX_CONTROL_TABLE_BYTES {
            source.truncate(previous_len);
            break;
        }
    }
    let remaining = MAX_CONTROL_TABLE_BYTES - retained_source_cost(Path::new(""), &source);
    source.extend(std::iter::repeat_n(b'a', (remaining / 2).min(MAX_CONTROL_PATTERN_BYTES)));
    assert_eq!(retained_source_cost(Path::new(""), &source), MAX_CONTROL_TABLE_BYTES);
    source
}

#[cfg(all(test, feature = "gitignore"))]
mod tests {
    use super::*;

    #[test]
    fn creation_edit_and_last_removal_are_exact() {
        let mut table = ControlTable::default();
        assert!(table.upsert(Path::new(".gitignore"), b"*.log\n".to_vec()).expect("insert"));
        let original = table.clone();
        assert!(!table.upsert(Path::new(".gitignore"), b"*.log\n".to_vec()).expect("no-op"));
        assert!(table.upsert(Path::new(".gitignore"), b"*.tmp\n".to_vec()).expect("edit"));
        assert_eq!(table.changes_from(&original).len(), 1);
        assert!(table.remove(Path::new(".gitignore")).expect("remove"));
        assert!(table.is_empty());
        assert_eq!(table.source_bytes(), 0);
        assert!(!table.remove(Path::new(".gitignore")).expect("missing is a no-op"));
    }

    #[test]
    fn total_source_bound_is_shared_and_replacement_gets_its_bytes_back() {
        let mut table = ControlTable::default();
        let first = source_at_test_limit();
        assert!(table.upsert(Path::new(".gitignore"), first).expect("at the shared bound"));
        let error = table
            .upsert(Path::new("nested/.gitignore"), b"ab".to_vec())
            .expect_err("the table, not each source, is bounded");
        assert!(matches!(error, crate::Error::ControlSourceLimit { .. }));
        assert!(table.upsert(Path::new(".gitignore"), b"small\n".to_vec()).expect("replace"));
        assert!(
            table
                .upsert(Path::new("nested/.gitignore"), b"now it fits\n".to_vec())
                .expect("freed bytes are reusable")
        );
    }

    #[test]
    fn one_pattern_line_has_an_independent_work_bound() {
        let source = vec![b'a'; MAX_CONTROL_PATTERN_BYTES + 1];
        let error = ControlTable::default()
            .upsert(Path::new(".gitignore"), source)
            .expect_err("one hostile rule is rejected before parsing");
        let crate::Error::ControlPatternLimit { attempted, limit } = error else {
            panic!("unexpected control error: {error}");
        };
        assert_eq!(attempted, MAX_CONTROL_PATTERN_BYTES + 1);
        assert_eq!(limit, MAX_CONTROL_PATTERN_BYTES);
    }

    #[test]
    fn control_identity_uses_standard_fnv1a_vectors() {
        assert_eq!(identity(b"").fingerprint, 0xcbf2_9ce4_8422_2325);
        assert_eq!(identity(b"a").fingerprint, 0xaf63_dc4c_8601_ec8c);
        assert_eq!(identity(b"foobar").fingerprint, 0x8594_4171_f739_67e8);
    }

    #[test]
    fn nested_negation_and_control_removal_change_the_governed_subtree() {
        let mut table = ControlTable::default();
        table.upsert(Path::new(".gitignore"), b"*.log\n".to_vec()).expect("root");
        table.upsert(Path::new("docs/.gitignore"), b"!keep.log\n".to_vec()).expect("nested");

        assert!(table.is_ignored(Path::new("debug.log"), false));
        assert!(table.is_ignored(Path::new("docs/other.log"), false));
        assert!(!table.is_ignored(Path::new("docs/keep.log"), false));
        assert_eq!(
            ControlTable::affected_subtree(Path::new("docs/.gitignore")).expect("scope"),
            Path::new("docs")
        );

        table.remove(Path::new("docs/.gitignore")).expect("remove nested");
        assert!(table.is_ignored(Path::new("docs/keep.log"), false));
    }

    #[test]
    fn ignored_parent_cannot_be_reincluded_from_inside_it() {
        let mut table = ControlTable::default();
        table.upsert(Path::new(".gitignore"), b"vendor/\n".to_vec()).expect("root");
        table
            .upsert(Path::new("vendor/.gitignore"), b"!keep.txt\n".to_vec())
            .expect("retained but inactive nested control");

        assert!(table.is_ignored(Path::new("vendor"), true));
        assert!(table.is_ignored(Path::new("vendor/keep.txt"), false));
    }
}

#[cfg(all(test, not(feature = "gitignore")))]
mod disabled_tests {
    use super::*;

    #[test]
    fn control_input_fails_closed_when_the_capability_is_absent() {
        let error = ControlTable::default()
            .upsert(Path::new(".gitignore"), b"*.log\n".to_vec())
            .expect_err("a disabled capability must not mean an empty answer");
        assert!(matches!(error, crate::Error::UnsupportedScanConfig(_)));

        let mut index = crate::Index::new("/root");
        let error = index
            .apply(&crate::Observation::new(vec![crate::Op::ControlRemove {
                path: PathBuf::from(".gitignore"),
            }]))
            .expect_err("removal input also requires the capability");
        assert!(matches!(error, crate::Error::UnsupportedScanConfig(_)));
    }
}
