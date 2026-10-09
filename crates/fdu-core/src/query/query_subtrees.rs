//! Directory values used by report selection, before positive predicates are applied.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::index::{EntryId, Index};
use crate::query::query_selection::NameIdentity;
use crate::query::{Candidate, IgnoredEntries, Selection};
use crate::{Attrs, Coverage, EntryKind};

/// Directory bytes and activity after exclusions, independent of positive predicates.
#[derive(Clone, Copy, Debug)]
pub(super) struct SubtreeValues {
    pub bytes: u64,
    pub allocated: u64,
    pub mtime_ns: i64,
    pub files: u64,
    pub dirs: u64,
    /// Whether every eligible directory in the subtree, this one included, was listed in
    /// full, so the values above are exact rather than lower bounds.
    ///
    /// False at the scan-depth boundary, where a directory was retained but never
    /// listed; in an opened root, for a directory discovery has not listed yet; and
    /// under a failed cold-walk boundary. Verified siblings retain their completeness.
    /// An unscoped walk failure leaves the whole tree incomplete. A lower-bound maximum
    /// is not an age, so the reader turns the mtime into an unknown age rather than
    /// reporting a directory as old because its newest activity was never seen.
    pub complete: bool,
}

/// Construct a predicate in the identity the caller uses; directory values replace
/// inode metadata only when the report reader supplies a measured subtree.
pub(super) fn with_candidate<T>(
    path: &Path,
    kind: EntryKind,
    attrs: Attrs,
    ignored: bool,
    identity: NameIdentity,
    evaluate: impl FnOnce(Candidate<'_>) -> T,
) -> T {
    let portable =
        (identity == NameIdentity::Portable).then(|| crate::opened::read::portable_path(path));
    let relative = portable.as_ref().map_or(path, |path| Path::new(path.as_str()));
    let name = relative.file_name().unwrap_or_default().to_string_lossy();
    evaluate(Candidate {
        relative,
        name: &name,
        kind,
        bytes: attrs.size,
        allocated: attrs.allocated,
        mtime_ns: attrs.mtime_ns,
        ignored,
    })
}

/// Excluded directories prune their descendants. `Only` still traverses unignored
/// ancestors so ignored descendants remain discoverable.
pub(super) fn pruned(selection: &Selection, candidate: &Candidate<'_>) -> bool {
    (selection.ignored == IgnoredEntries::Exclude && candidate.ignored)
        || selection
            .exclude
            .iter()
            .any(|pattern| pattern.matches(candidate.relative, candidate.name))
}

/// One iterative post-order traversal computes all directory candidates. Sizes count
/// regular files; recency also observes empty directories, symlinks, and other entries.
/// The scan root is a traversal boundary, not a selectable entry, so its own timestamp
/// never counts: no list row shows the root, and the root's value is read only by a
/// tree, whose root row counts what lies beneath it.
///
/// This is the one owner of the directory-metric definition, and it runs once per
/// report: the selection walk looks these values up by id and never measures a subtree
/// itself, which is what keeps a report at one pass here plus one pass there however
/// many directories match. [`activity`] is the same recency and completeness for an
/// unfiltered selection, computed without paths, and a test holds the two equal.
pub(super) fn measure(
    index: &Index,
    selection: &Selection,
    identity: NameIdentity,
) -> BTreeMap<EntryId, SubtreeValues> {
    // A directory is listed in full when the index marks it so, or when the whole index
    // is complete. Cold walks mark successful listings outside their failure boundaries,
    // so a partial index can still prove a healthy sibling complete. An index assembled
    // from observations alone has no per-listing marks, so whole-index coverage remains
    // a sufficient fallback; scan-depth boundaries below still withdraw completeness.
    let coverage_complete = index.state().coverage == Coverage::Complete;
    let boundary = index.scope().max_depth;
    let mut values: BTreeMap<EntryId, SubtreeValues> = BTreeMap::new();
    let mut stack = vec![(EntryId::ROOT, PathBuf::new(), false)];
    while let Some((id, path, expanded)) = stack.pop() {
        if expanded {
            let mut total = values[&id];
            if let Some(children) = index.children_of(id) {
                for (_, child) in children {
                    if let Some(subtree) = values.get(&child) {
                        total.files += subtree.files;
                        total.dirs += subtree.dirs;
                        total.bytes += subtree.bytes;
                        total.allocated += subtree.allocated;
                        total.mtime_ns = total.mtime_ns.max(subtree.mtime_ns);
                        total.complete &= subtree.complete;
                    }
                }
            }
            values.insert(id, total);
            continue;
        }
        // The directory's own timestamp counts only when the selection admits the
        // directory itself, which it never does for the root, the traversal boundary.
        // Under `--only-ignored` an unignored ancestor is traversed for
        // the ignored matches beneath it, and its own activity is not theirs. Every
        // descendant of an ignored directory is ignored today, so no admitted directory
        // sits under a rejected one and this changes no answer; it is here so a change
        // to negation handling cannot make the seed silently wrong.
        let own_ignored = index.ignored_classification(&path);
        let own_admitted = id != EntryId::ROOT
            && (own_ignored.is_some()
                || selection.ignored == crate::query::IgnoredEntries::Include)
            && selection.ignored.admits(own_ignored.unwrap_or(false));
        let own_time = if own_admitted {
            index.attrs_of(id).map_or(i64::MIN, |attrs| attrs.mtime_ns)
        } else {
            i64::MIN
        };
        // A directory at the scan-depth boundary was retained but never listed, so its
        // children are unknown whatever the index says about coverage.
        let listed = coverage_complete || index.directory_complete_of(id).unwrap_or(false);
        let at_boundary = boundary.is_some_and(|depth| path.components().count() >= depth);
        let mut total = SubtreeValues {
            bytes: 0,
            allocated: 0,
            mtime_ns: own_time,
            files: 0,
            dirs: 0,
            complete: listed && !at_boundary,
        };
        stack.push((id, path.clone(), true));
        if let Some(children) = index.children_of(id) {
            for (name, child) in children {
                let (Some(kind), Some(attrs)) = (index.kind_of(child), index.attrs_of(child))
                else {
                    continue;
                };
                let child_path = path.join(name);
                let classification = index.ignored_classification(&child_path);
                let ignored = classification.unwrap_or(false);
                let admitted = (classification.is_some()
                    || selection.ignored == crate::query::IgnoredEntries::Include)
                    && selection.ignored.admits(ignored);
                if with_candidate(&child_path, kind, *attrs, ignored, identity, |candidate| {
                    pruned(selection, &candidate)
                }) {
                    continue;
                }
                if kind == EntryKind::Dir {
                    if admitted {
                        total.dirs += 1;
                    }
                    stack.push((child, child_path, false));
                } else if admitted {
                    total.mtime_ns = total.mtime_ns.max(attrs.mtime_ns);
                    if kind == EntryKind::File {
                        total.files += 1;
                        total.bytes += attrs.size;
                        total.allocated += attrs.allocated;
                    }
                }
            }
        }
        values.insert(id, total);
    }
    values
}

/// One directory's newest activity and listing completeness, as [`measure`] defines them
/// for a selection that admits everything.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DirectoryActivity {
    /// The newest modification time of any entry in the subtree, of any kind, the
    /// directory's own included unless it is the report root; `None` when the subtree
    /// holds no entry at all.
    pub newest_ns: Option<i64>,
    /// Whether every directory in the subtree, this one included, was listed in full, by
    /// [`SubtreeValues::complete`]'s rule.
    pub complete: bool,
}

/// Every directory's [`DirectoryActivity`] for an unfiltered selection, one row per arena
/// slot.
///
/// Dense rather than keyed by id: an unfiltered tree reads it for every row it admits
/// and every row it sorts by recency, and a retained index pays for this table on every
/// report, so the read is a slot load rather than a search. A non-directory's slot is
/// empty.
pub(super) struct ActivityTable {
    slots: Vec<Option<DirectoryActivity>>,
}

impl ActivityTable {
    /// The activity of live directory `id`, or `None` for any other entry.
    pub(super) fn get(&self, id: EntryId) -> Option<DirectoryActivity> {
        self.slots.get(id.slot()).copied().flatten()
    }

    /// A table holding exactly `values` over `index`, for a test that states each
    /// directory's activity rather than deriving it.
    #[cfg(test)]
    pub(super) fn of(
        index: &Index,
        values: impl IntoIterator<Item = (EntryId, DirectoryActivity)>,
    ) -> Self {
        let mut slots = vec![None; index.slots()];
        for (id, value) in values {
            slots[id.slot()] = Some(value);
        }
        Self { slots }
    }
}

/// Newest activity and completeness of every directory, for a selection that admits
/// every entry: [`measure`]'s recency and completeness without its sizes, its paths, or
/// its selection.
///
/// One iterative post-order traversal by entry id and depth. A directory's activity is
/// the newest of its own time (the report root's excepted), its non-directory children's
/// times, its subdirectories' activity, and its roll-up's newest regular file. The
/// roll-up is read only when it counts files, since `0` there means none rather than the
/// epoch, and it is what makes this pass exact on a folded index: that index counts
/// every file in its directories' roll-ups but keeps only the largest as entries, so the
/// files it folded have no entry for [`measure`] to read. On a full index the roll-up
/// adds nothing the children did not.
///
/// Completeness follows [`measure`]: a directory is listed in full when the index marks
/// it so or the whole index is complete, a directory at the scan-depth boundary was
/// retained but never listed, and a subtree is complete only when every directory in it
/// is. A folded index keeps every directory, so its completeness is exact too.
///
/// Each directory is pushed twice and its children read once: a finished directory folds
/// its value into its parent's slot, which post-order guarantees has not finished yet.
pub(super) fn activity(index: &Index) -> ActivityTable {
    let coverage_complete = index.state().coverage == Coverage::Complete;
    let boundary = index.scope().max_depth;
    let mut slots: Vec<Option<DirectoryActivity>> = vec![None; index.slots()];
    // (id, parent, depth, finished)
    let mut stack = vec![(EntryId::ROOT, None::<EntryId>, 0_usize, false)];
    while let Some((id, parent, depth, finished)) = stack.pop() {
        if finished {
            let Some(parent) = parent else { continue };
            let done = slots[id.slot()].expect("a finished directory has its value");
            let into = slots[parent.slot()].as_mut().expect("a parent finishes after its children");
            into.newest_ns = newer(into.newest_ns, done.newest_ns);
            into.complete &= done.complete;
            continue;
        }
        let own = (id != EntryId::ROOT).then(|| index.attrs_of(id).map(|attrs| attrs.mtime_ns));
        let mut newest = newer(own.flatten(), index.newest_file_of(id));
        let listed = coverage_complete || index.directory_complete_of(id).unwrap_or(false);
        let at_boundary = boundary.is_some_and(|limit| depth >= limit);
        stack.push((id, parent, depth, true));
        if let Some(children) = index.children_of(id) {
            for (_, child) in children {
                match index.kind_of(child) {
                    Some(EntryKind::Dir) => stack.push((child, Some(id), depth + 1, false)),
                    Some(_) => {
                        newest = newer(newest, index.attrs_of(child).map(|attrs| attrs.mtime_ns));
                    }
                    None => {}
                }
            }
        }
        slots[id.slot()] =
            Some(DirectoryActivity { newest_ns: newest, complete: listed && !at_boundary });
    }
    ActivityTable { slots }
}

/// The later of two optional instants.
pub(super) fn newer(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (left, right) => left.or(right),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_contract::{Observation, Op};
    use crate::execution::TreeRetention;
    use crate::query::SizeMetric;
    use std::fs;
    use std::time::{Duration, UNIX_EPOCH};

    fn attrs(size: u64, mtime_ns: i64) -> Attrs {
        Attrs {
            size,
            allocated: size.div_ceil(512) * 512,
            mtime_ns,
            ctime_ns: mtime_ns,
            inode: size.wrapping_mul(31).wrapping_add(mtime_ns.unsigned_abs()),
            dev: 1,
        }
    }

    fn upsert(path: &str, kind: EntryKind, attrs: Attrs) -> Op {
        Op::Upsert { path: PathBuf::from(path), kind, attrs }
    }

    /// Every live directory of `index`, by path, root first.
    fn directories(index: &Index) -> Vec<(PathBuf, EntryId)> {
        let mut found = Vec::new();
        let mut stack = vec![(PathBuf::new(), EntryId::ROOT)];
        while let Some((path, id)) = stack.pop() {
            if let Some(children) = index.children_of(id) {
                for (name, child) in children {
                    if index.kind_of(child) == Some(EntryKind::Dir) {
                        stack.push((path.join(name), child));
                    }
                }
            }
            found.push((path, id));
        }
        found.sort();
        found
    }

    /// [`measure`]'s recency and completeness of every directory under the admit-all
    /// selection, by path, with its "nothing" sentinel read as `None`.
    fn measured(index: &Index) -> BTreeMap<PathBuf, DirectoryActivity> {
        let values = measure(index, &Selection::default(), NameIdentity::Native);
        directories(index)
            .into_iter()
            .map(|(path, id)| {
                let value = values[&id];
                let newest_ns = (value.mtime_ns != i64::MIN).then_some(value.mtime_ns);
                (path, DirectoryActivity { newest_ns, complete: value.complete })
            })
            .collect()
    }

    /// The fast pass's value for every directory, by path.
    fn fast(index: &Index) -> BTreeMap<PathBuf, DirectoryActivity> {
        let table = activity(index);
        directories(index)
            .into_iter()
            .map(|(path, id)| (path, table.get(id).expect("every directory has a value")))
            .collect()
    }

    fn at(newest_ns: i64, complete: bool) -> DirectoryActivity {
        DirectoryActivity { newest_ns: Some(newest_ns), complete }
    }

    /// Empty directories, a symlink, an other entry, and a directory whose own time is
    /// newer than anything in it: every kind counts toward recency, and the root's own
    /// time does not.
    #[test]
    fn the_fast_pass_is_measure_on_every_directory_of_a_full_index() {
        let mut index = Index::new("/root");
        index.apply_ok(&Observation::new(vec![
            upsert("src", EntryKind::Dir, attrs(0, 5)),
            upsert("src/main.rs", EntryKind::File, attrs(100, 10)),
            upsert("src/empty", EntryKind::Dir, attrs(0, 60)),
            upsert("src/link", EntryKind::Symlink, attrs(9, 70)),
            upsert("fresh", EntryKind::Dir, attrs(0, 90)),
            upsert("fresh/old.txt", EntryKind::File, attrs(3, 1)),
            upsert("fifo", EntryKind::Other, attrs(0, 80)),
            upsert("void", EntryKind::Dir, attrs(0, -10)),
        ]));
        index.set_initial_freshness(true);
        let expected = BTreeMap::from([
            (PathBuf::new(), at(90, true)),
            (PathBuf::from("fresh"), at(90, true)),
            (PathBuf::from("src"), at(70, true)),
            (Path::new("src").join("empty"), at(60, true)),
            (PathBuf::from("void"), at(-10, true)),
        ]);
        assert_eq!(fast(&index), expected);
        assert_eq!(measured(&index), expected);

        // Removing the newest file leaves a stale maximum in the roll-up until it is
        // repaired upward; the fast pass reads the roll-up, so it must see the repair.
        index.apply_ok(&Observation::new(vec![
            upsert("deep", EntryKind::Dir, attrs(0, 2)),
            upsert("deep/a", EntryKind::Dir, attrs(0, 2)),
            upsert("deep/a/newest.bin", EntryKind::File, attrs(1, 500)),
            upsert("deep/a/older.bin", EntryKind::File, attrs(1, 300)),
        ]));
        assert_eq!(fast(&index)[Path::new("deep")], at(500, true));
        index.apply_ok(&Observation::new(vec![Op::Remove {
            path: Path::new("deep").join("a").join("newest.bin"),
        }]));
        assert_eq!(fast(&index)[Path::new("deep")], at(300, true));
        assert_eq!(fast(&index), measured(&index));

        // A root holding nothing has no activity: its own time is not one.
        let mut empty = Index::new("/empty");
        empty.set_initial_freshness(true);
        let root = BTreeMap::from([(
            PathBuf::new(),
            DirectoryActivity { newest_ns: None, complete: true },
        )]);
        assert_eq!(fast(&empty), root);
        assert_eq!(measured(&empty), root);
    }

    /// A directory at the scan-depth boundary was never listed, an unscoped partial marker
    /// withdraws every listing, and an opened root lists one directory at a time; in each,
    /// the fast pass withdraws completeness exactly where [`measure`] does.
    #[test]
    fn the_fast_pass_withdraws_completeness_where_measure_does() {
        let mut bounded = Index::new_with_scope(
            "/root",
            crate::ScanScope { max_depth: Some(2), ..crate::ScanScope::default() },
        );
        bounded.apply_ok(&Observation::new(vec![
            upsert("env", EntryKind::Dir, attrs(0, 5)),
            upsert("env/lib", EntryKind::Dir, attrs(0, 7)),
            upsert("env/a.bin", EntryKind::File, attrs(100, 40)),
            upsert("docs", EntryKind::Dir, attrs(0, 5)),
            upsert("docs/guide.md", EntryKind::File, attrs(30, 50)),
        ]));
        bounded.set_initial_freshness(true);
        let completeness = fast(&bounded)
            .into_iter()
            .map(|(path, value)| (path, value.complete))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            completeness,
            BTreeMap::from([
                (PathBuf::new(), false),
                (PathBuf::from("docs"), true),
                (PathBuf::from("env"), false),
                (Path::new("env").join("lib"), false),
            ])
        );
        assert_eq!(fast(&bounded), measured(&bounded));

        let mut partial = Index::new("/root");
        partial.apply_ok(&Observation::new(vec![
            upsert("a", EntryKind::Dir, attrs(0, 5)),
            upsert("a/b", EntryKind::Dir, attrs(0, 6)),
        ]));
        partial.set_initial_freshness(false);
        assert!(fast(&partial).values().all(|value| !value.complete));
        assert_eq!(fast(&partial), measured(&partial));

        let handle = crate::index::IndexHandle::new(Index::new("/root"));
        handle
            .transition_discovery(crate::index::DiscoveryTransition::Begin)
            .expect("begin discovery");
        handle
            .apply(&Observation::new(vec![
                upsert("known", EntryKind::Dir, attrs(0, 5)),
                upsert("pending", EntryKind::Dir, attrs(0, 5)),
            ]))
            .expect("seed directories");
        handle
            .apply_discovery(
                &Observation::new(Vec::new()),
                crate::index::DiscoveryCommit {
                    directory_complete: Some(PathBuf::from("known")),
                    transition: None,
                },
            )
            .expect("list one directory");
        handle
            .read_with(|index| {
                let values = fast(index);
                assert!(values[Path::new("known")].complete);
                assert!(!values[Path::new("pending")].complete);
                assert_eq!(values, measured(index));
            })
            .expect("read");
    }

    /// Write `bytes` bytes to `path`, creating its parents, stamped `seconds` after the
    /// epoch.
    fn stamped_file(path: &Path, bytes: usize, seconds: u64) {
        fs::create_dir_all(path.parent().expect("parent")).expect("parents");
        fs::write(path, vec![b'x'; bytes]).expect("file");
        fs::File::options()
            .write(true)
            .open(path)
            .and_then(|file| file.set_modified(UNIX_EPOCH + Duration::from_secs(seconds)))
            .expect("stamp file");
    }

    /// A tree whose newest file in each directory is its smallest, so a folded index that
    /// keeps only the largest files keeps none of the newest: the fast pass has to find
    /// them in the roll-ups. With empty directories, a symlink, and directories stamped
    /// newer than their contents where the host can stamp a directory.
    fn folding_tree() -> tempfile::TempDir {
        let tree = tempfile::tempdir().expect("tempdir");
        let root = tree.path();
        for (index, directory) in ["a", "a/b", "a/b/c", "d", "e/f"].into_iter().enumerate() {
            let offset = u64::try_from(index).expect("small") * 100;
            stamped_file(&root.join(directory).join("large.bin"), 50_000, 1_000 + offset);
            stamped_file(&root.join(directory).join("medium.bin"), 20_000, 2_000 + offset);
            stamped_file(&root.join(directory).join("tiny-newest"), 3, 9_000 + offset);
        }
        stamped_file(&root.join("top-newest"), 1, 50_000);
        fs::create_dir_all(root.join("empty/inner")).expect("empty directories");
        #[cfg(unix)]
        std::os::unix::fs::symlink("a", root.join("link")).expect("symlink");
        // Every directory is stamped once nothing more is written in it, since a write in
        // a directory moves its time.
        #[cfg(unix)]
        for (directory, seconds) in [
            ("a", 10),
            ("a/b", 11),
            ("a/b/c", 12),
            ("d", 99_000),
            ("e", 13),
            ("e/f", 14),
            ("empty/inner", 7),
            ("empty", 8),
        ] {
            fs::File::open(root.join(directory))
                .and_then(|file| file.set_modified(UNIX_EPOCH + Duration::from_secs(seconds)))
                .expect("stamp directory");
        }
        crate::test_support::settle_allocations(root);
        tree
    }

    /// The fast pass over the folded index the one-shot tree tier builds equals
    /// [`measure`] over the full index of the same walk, on every directory: the files
    /// the fold counted without keeping still count, through the roll-ups.
    fn assert_folded_matches_full(root: &Path, scan: &crate::ScanConfig, label: &str) {
        let canonical = root.canonicalize().expect("canonical root");
        let (full, _) = crate::scan::scan_into_index(&canonical, scan).expect("full index");
        for largest_files in [1, 2, 5] {
            let retention = TreeRetention { largest_files, size: SizeMetric::Apparent };
            let (folded, _, _) =
                crate::scan::scan_into_folded_index(&canonical, scan, retention, false)
                    .expect("folded index");
            assert!(folded.is_folded() && folded.len() < full.len(), "{label}: it folds");
            assert_eq!(fast(&folded), measured(&full), "{label}, {largest_files} kept");
            assert_eq!(fast(&full), measured(&full), "{label}");
        }
    }

    #[test]
    fn the_fast_pass_over_a_folded_index_is_measure_over_the_full_one() {
        let tree = folding_tree();
        assert_folded_matches_full(tree.path(), &crate::ScanConfig::default(), "whole tree");
        let bounded = crate::ScanConfig { max_depth: Some(2), ..crate::ScanConfig::default() };
        assert_folded_matches_full(tree.path(), &bounded, "scan depth 2");
        // Only where the fixture could stamp its directories older than their contents.
        #[cfg(unix)]
        {
            let canonical = tree.path().canonicalize().expect("canonical");
            let (full, _) = crate::scan::scan_into_index(&canonical, &crate::ScanConfig::default())
                .expect("full index");
            let values = fast(&full);
            let newest = |path: &str| values[Path::new(path)].newest_ns.expect("activity");
            assert_eq!(newest("a"), 9_200 * 1_000_000_000, "a/b/c's tiny file is newest in a");
            assert_eq!(newest("d"), 99_000 * 1_000_000_000, "d's own time is its newest");
        }

        // A listing that fails leaves its directory, and every ancestor, incomplete.
        #[cfg(unix)]
        if crate::test_support::require_permission_bits() {
            use std::os::unix::fs::PermissionsExt;
            let locked = tree.path().join("a").join("b");
            fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).expect("lock");
            let restore = || {
                fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).expect("unlock");
            };
            let outcome = std::panic::catch_unwind(|| {
                let canonical = tree.path().canonicalize().expect("canonical");
                let (full, _) =
                    crate::scan::scan_into_index(&canonical, &crate::ScanConfig::default())
                        .expect("a partial index");
                assert_ne!(full.state().coverage, Coverage::Complete, "the listing failed");
                let values = fast(&full);
                assert!(!values[Path::new("a")].complete && !values[Path::new("")].complete);
                assert!(values[Path::new("d")].complete, "a healthy sibling stays complete");
                assert_folded_matches_full(
                    tree.path(),
                    &crate::ScanConfig::default(),
                    "failed subtree",
                );
            });
            restore();
            if let Err(panic) = outcome {
                std::panic::resume_unwind(panic);
            }
        }
    }
}
