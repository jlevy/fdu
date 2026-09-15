//! Bounded, removal-aware control state used to classify retained filesystem facts.
//!
//! A control file is producer input, not something the index discovers by walking the
//! filesystem. The table stores the exact bytes a producer verified and the matcher
//! derived from them. That keeps cold discovery, refresh, observation, and snapshot load
//! on one semantic path and makes deleting the last control file an ordinary state
//! transition rather than a special rebuild.
//!
//! **Which of git's ignore inputs count.** Exactly one: every regular file named
//! [`CONTROL_FILE_NAME`] inside the scanned root, each governing its own directory and
//! everything below it, with deeper files taking precedence. Nothing else git consults is
//! read. `.git/info/exclude` and `core.excludesFile` are ignored, so a `.DS_Store` excluded
//! only globally lands in the unignored partition. A nested repository is not a boundary:
//! its `.gitignore` files join the outer ones as if the tree were one repository. And
//! unlike git, which never looks inside an ignored directory, the walk reads a
//! `.gitignore` there too; the ignored partition is still right, because git's rule that
//! an excluded parent cannot be re-included is applied when matching, but the file's
//! bytes are retained against the table bound. Matching is case-sensitive regardless of
//! `core.ignorecase`.

mod gitignore;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

/// One distinct control content, parsed once and shared by every directory holding it.
#[derive(Debug)]
struct SharedContent {
    bytes: Vec<u8>,
    identity: ControlIdentity,
    matcher: Gitignore,
    /// Charge for the exact bytes and the parsed matcher, paid once per distinct content.
    content_cost: usize,
}

/// A distinct content and how many directories of one table hold it.
///
/// The count belongs to the table, not to the `Arc`: a projected clone of a table shares
/// every content with it, so the reference count cannot say which holders one table has.
#[derive(Clone, Debug)]
struct Holding {
    content: Arc<SharedContent>,
    holders: usize,
}

/// Exact `.gitignore` sources and parsed matchers, keyed by governing directory.
///
/// Identical sources are stored and parsed once. A tree of package checkouts repeats a few
/// `.gitignore` files thousands of times, and charging every copy for its bytes and matcher
/// crossed the bound at a fraction of the distinct rules the tree holds (fdu-szkg). Each
/// directory still pays for its own key, so a tree of empty control files cannot evade the
/// bound, and removing the last holder of a content releases the content's charge.
#[derive(Clone, Debug, Default)]
pub struct ControlTable {
    by_directory: BTreeMap<PathBuf, Arc<SharedContent>>,
    /// Distinct contents by identity. A list, because an equal length and FNV-1a digest do
    /// not prove equal bytes, and a collision must never share another source's matcher.
    shared: HashMap<ControlIdentity, Vec<Holding>>,
    source_bytes: usize,
    retained_cost: usize,
}

impl ControlTable {
    /// Insert or replace one verified control source.
    ///
    /// `path` names the control file relative to the index root. The source is retained
    /// exactly, while matching state is derived once per distinct content rather than per
    /// directory or per entry.
    pub fn upsert(&mut self, path: &Path, source: Vec<u8>) -> crate::Result<bool> {
        let identity = identity(&source);
        self.upsert_identified(path, source, identity)
    }

    fn upsert_identified(
        &mut self,
        path: &Path,
        source: Vec<u8>,
        identity: ControlIdentity,
    ) -> crate::Result<bool> {
        let directory = control_directory(path)?;
        if let Some(line) =
            source.split(|byte| *byte == b'\n').find(|line| line.len() > MAX_CONTROL_PATTERN_BYTES)
        {
            return Err(crate::Error::ControlPatternLimit {
                attempted: line.len(),
                limit: MAX_CONTROL_PATTERN_BYTES,
            });
        }
        if self.by_directory.get(directory).is_some_and(|current| current.bytes == source) {
            return Ok(false);
        }
        let content_charge =
            if self.holding(identity, &source).is_some() { 0 } else { content_cost(&source) };
        let next = self
            .retained_cost
            .checked_sub(self.release_charge(directory))
            .and_then(|bytes| bytes.checked_add(directory_cost(directory)))
            .and_then(|bytes| bytes.checked_add(content_charge))
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

        self.detach(directory);
        self.attach(directory, source, identity);
        debug_assert_eq!(self.retained_cost, next);
        Ok(true)
    }

    /// Remove one control source. Missing sources are no-ops.
    pub fn remove(&mut self, path: &Path) -> crate::Result<bool> {
        let directory = control_directory(path)?;
        Ok(self.detach(directory))
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
            self.detach(&directory);
        }
    }

    /// The holding for exactly `source`, when some directory already retains it.
    fn holding(&self, identity: ControlIdentity, source: &[u8]) -> Option<&Holding> {
        self.shared
            .get(&identity)?
            .iter()
            .find(|holding| holding.content.bytes.as_slice() == source)
    }

    /// The charge that dropping `directory`'s current source would release.
    fn release_charge(&self, directory: &Path) -> usize {
        let Some(content) = self.by_directory.get(directory) else {
            return 0;
        };
        let last_holder = self.shared.get(&content.identity).is_some_and(|holdings| {
            holdings
                .iter()
                .any(|holding| Arc::ptr_eq(&holding.content, content) && holding.holders == 1)
        });
        directory_cost(directory).saturating_add(if last_holder { content.content_cost } else { 0 })
    }

    /// Drop `directory`'s source, releasing its content when this was the last holder.
    fn detach(&mut self, directory: &Path) -> bool {
        let Some(content) = self.by_directory.remove(directory) else {
            return false;
        };
        let holdings = self.shared.get_mut(&content.identity).expect("a retained content is held");
        let position = holdings
            .iter()
            .position(|holding| Arc::ptr_eq(&holding.content, &content))
            .expect("a retained content is listed under its own identity");
        holdings[position].holders -= 1;
        if holdings[position].holders == 0 {
            holdings.swap_remove(position);
            self.retained_cost -= content.content_cost;
            if holdings.is_empty() {
                self.shared.remove(&content.identity);
            }
        }
        self.retained_cost -= directory_cost(directory);
        self.source_bytes -= content.bytes.len();
        true
    }

    /// Hold `source` at `directory`, sharing an identical retained content when one exists.
    fn attach(&mut self, directory: &Path, source: Vec<u8>, identity: ControlIdentity) {
        let holdings = self.shared.entry(identity).or_default();
        let content = if let Some(holding) =
            holdings.iter_mut().find(|holding| holding.content.bytes == source)
        {
            holding.holders += 1;
            Arc::clone(&holding.content)
        } else {
            let content_cost = content_cost(&source);
            let matcher = Gitignore::parse(&source);
            let content =
                Arc::new(SharedContent { bytes: source, identity, matcher, content_cost });
            holdings.push(Holding { content: Arc::clone(&content), holders: 1 });
            self.retained_cost += content_cost;
            content
        };
        self.retained_cost += directory_cost(directory);
        self.source_bytes += content.bytes.len();
        self.by_directory.insert(directory.to_path_buf(), content);
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
                    (Some(before), Some(after)) => {
                        !Arc::ptr_eq(before, after) && before.bytes != after.bytes
                    }
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
        // The empty table answers without collecting the ancestor list. Every entry of
        // a scan that observes no control state asks this question exactly once, so the
        // allocation below would be the last per-entry cost of a machinery the scan
        // opted out of (fdu-etfj).
        if self.table.by_directory.is_empty() {
            return false;
        }
        // A deeper control source overrides every matching ancestor. Walk from the
        // immediate directory toward the root and return the first opinion instead of
        // allocating and reversing an ancestor vector for every classified entry.
        for directory in self.path.parent().into_iter().flat_map(Path::ancestors) {
            let Some(source) = self.table.by_directory.get(directory) else {
                continue;
            };
            let relative = self.path.strip_prefix(directory).unwrap_or(self.path);
            if let Some(ignored) = source.matcher.matches(relative, is_dir) {
                return ignored;
            }
        }
        false
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

/// Charge for one directory's key and identity, whatever content it holds.
fn directory_cost(directory: &Path) -> usize {
    CONTROL_SOURCE_OVERHEAD.saturating_add(directory.as_os_str().as_encoded_bytes().len())
}

/// Charge for one distinct content: its exact bytes and parsed glob bytes, plus the
/// matcher's per-pattern and per-segment shells.
fn content_cost(source: &[u8]) -> usize {
    let (newlines, segment_shells) = source.iter().fold((0usize, 0usize), |counts, byte| {
        (counts.0 + usize::from(*byte == b'\n'), counts.1 + usize::from(*byte == b'/'))
    });
    let pattern_shells = newlines.saturating_add(1);
    source
        .len()
        .saturating_mul(2)
        .saturating_add(pattern_shells.saturating_mul(64))
        .saturating_add(segment_shells.saturating_mul(24))
}

/// Charge for one source whose content no other directory holds.
#[cfg(test)]
fn retained_source_cost(directory: &Path, source: &[u8]) -> usize {
    directory_cost(directory).saturating_add(content_cost(source))
}

#[cfg(test)]
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

#[cfg(test)]
impl ControlTable {
    /// Recompute every charge and holder count from the directories alone.
    fn assert_consistent(&self) {
        let mut distinct: Vec<&Arc<SharedContent>> = Vec::new();
        let mut directory_charges = 0;
        let mut source_bytes = 0;
        for (directory, content) in &self.by_directory {
            directory_charges += directory_cost(directory);
            source_bytes += content.bytes.len();
            if !distinct.iter().any(|seen| Arc::ptr_eq(seen, content)) {
                distinct.push(content);
            }
        }
        let content_charges: usize = distinct.iter().map(|content| content.content_cost).sum();
        assert_eq!(self.retained_cost, directory_charges + content_charges, "retained cost");
        assert_eq!(self.source_bytes, source_bytes, "source bytes");
        let holdings: usize = self.shared.values().map(Vec::len).sum();
        assert_eq!(holdings, distinct.len(), "one holding per distinct content");
        for (identity, holdings) in &self.shared {
            assert!(!holdings.is_empty(), "no empty identity list survives");
            for holding in holdings {
                assert_eq!(holding.content.identity, *identity);
                let holders = self
                    .by_directory
                    .values()
                    .filter(|content| Arc::ptr_eq(content, &holding.content))
                    .count();
                assert_eq!(holding.holders, holders, "holder count");
            }
            for (index, left) in holdings.iter().enumerate() {
                for right in &holdings[index + 1..] {
                    assert_ne!(left.content.bytes, right.content.bytes, "equal bytes are shared");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic `SplitMix64`, so a failing sequence replays from its printed seed.
    struct SplitMix(u64);

    impl SplitMix {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut value = self.0;
            value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            value ^ (value >> 31)
        }

        fn below(&mut self, bound: usize) -> usize {
            usize::try_from(self.next() % u64::try_from(bound).expect("small bound")).expect("fits")
        }
    }

    #[test]
    fn shared_charges_stay_exact_through_random_upserts_and_removals() {
        const DIRECTORIES: [&str; 6] = ["", "a", "a/b", "b", "b/c/d", "c"];
        const CONTENTS: [&[u8]; 4] = [b"*.log\n", b"target/\n", b"!keep\n*.tmp\n", b""];
        for seed in 0..64 {
            let mut random = SplitMix(seed);
            let mut table = ControlTable::default();
            for step in 0..200 {
                let directory = Path::new(DIRECTORIES[random.below(DIRECTORIES.len())]);
                let path = directory.join(CONTROL_FILE_NAME);
                match random.below(8) {
                    0 => table.remove_subtree(directory),
                    1 | 2 => {
                        table.remove(&path).expect("control path");
                    }
                    _ => {
                        let content = CONTENTS[random.below(CONTENTS.len())].to_vec();
                        table.upsert(&path, content).expect("far below the bound");
                    }
                }
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    table.assert_consistent();
                }))
                .unwrap_or_else(|_| panic!("seed {seed}, step {step}: inconsistent table"));
            }
            for directory in DIRECTORIES {
                table.remove(&Path::new(directory).join(CONTROL_FILE_NAME)).expect("control path");
            }
            assert_eq!(table.retained_cost(), 0, "seed {seed}");
            assert_eq!(table.source_bytes(), 0, "seed {seed}");
            assert!(table.shared.is_empty(), "seed {seed}");
        }
    }

    #[test]
    fn a_fingerprint_collision_never_shares_a_matcher() {
        let collision = ControlIdentity { bytes: 6, fingerprint: 7 };
        let mut table = ControlTable::default();
        table
            .upsert_identified(Path::new("a/.gitignore"), b"*.log\n".to_vec(), collision)
            .expect("first");
        table
            .upsert_identified(Path::new("b/.gitignore"), b"*.tmp\n".to_vec(), collision)
            .expect("second");
        table.assert_consistent();

        assert_eq!(table.shared[&collision].len(), 2);
        assert!(table.is_ignored(Path::new("a/x.log"), false));
        assert!(!table.is_ignored(Path::new("a/x.tmp"), false));
        assert!(table.is_ignored(Path::new("b/x.tmp"), false));
        assert!(!table.is_ignored(Path::new("b/x.log"), false));
        assert_eq!(
            table.retained_cost(),
            retained_source_cost(Path::new("a"), b"*.log\n")
                + retained_source_cost(Path::new("b"), b"*.tmp\n")
        );

        table.remove(Path::new("a/.gitignore")).expect("remove");
        table.assert_consistent();
        assert!(table.is_ignored(Path::new("b/x.tmp"), false));
    }

    #[test]
    fn replacing_the_last_holder_releases_its_content_for_the_bound() {
        let mut table = ControlTable::default();
        let first = source_at_test_limit();
        table.upsert(Path::new(".gitignore"), first.clone()).expect("at the bound");
        // The same content elsewhere costs only a key, which still crosses a full table.
        let error = table
            .upsert(Path::new("copy/.gitignore"), first)
            .expect_err("a key alone crosses a table at its bound");
        assert!(matches!(error, crate::Error::ControlSourceLimit { .. }));
        table.upsert(Path::new(".gitignore"), b"small\n".to_vec()).expect("replace");
        table.assert_consistent();
        assert_eq!(table.retained_cost(), retained_source_cost(Path::new(""), b"small\n"));
    }

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
    fn identical_sources_share_one_content_charge() {
        let source = b"target/\n*.log\nnode_modules/\n".to_vec();
        let mut table = ControlTable::default();
        table.upsert(Path::new("a/.gitignore"), source.clone()).expect("first holder");
        let one = table.retained_cost();
        table.upsert(Path::new("bb/.gitignore"), source.clone()).expect("second holder");

        // The second directory pays for its key and nothing for content it shares.
        assert_eq!(table.retained_cost() - one, CONTROL_SOURCE_OVERHEAD + "bb".len());
        assert_eq!(table.source_bytes(), 2 * source.len());
        assert!(table.is_ignored(Path::new("a/debug.log"), false));
        assert!(table.is_ignored(Path::new("bb/debug.log"), false));
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
