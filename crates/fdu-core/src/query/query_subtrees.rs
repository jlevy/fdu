//! Directory values used by report selection, before positive predicates are applied.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::index::{EntryId, Index, newer};
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
/// and every row it sorts by recency, so the read is a slot load rather than a search. A
/// non-directory's slot is empty.
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
/// The reader of an index that may hold an unlisted subtree, where completeness takes a
/// pass anyway. Where every subtree was listed ([`every_subtree_listed`]) each directory
/// is complete and its activity is maintained with its roll-up, so an unfiltered tree
/// reads [`maintained_activity`] per row instead, and a test holds the two equal.
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

/// Whether every subtree of `index` was listed in full: the whole index is complete and
/// no scan depth left a directory retained but unlisted. [`measure`] and [`activity`] call
/// every directory complete exactly then, so a tree needs neither for completeness.
pub(super) fn every_subtree_listed(index: &Index) -> bool {
    index.state().coverage == Coverage::Complete && index.scope().max_depth.is_none()
}

/// [`activity`]'s newest activity of live directory `id`, read from the roll-up the index
/// maintains: its own time, the report root's excepted, and the newest activity beneath
/// it ([`Index::newest_activity_below`]).
///
/// A field read per row where [`activity`] is a pass over every entry. A retained index
/// (a library caller's [`Index`], an opened root, a watch session) answers report after
/// report, and the pass cost each one a traversal of the whole index, whose entries an
/// opened root's incrementally built arena scatters; reading only the rows a tree shows
/// keeps those reports at the cost of their rows.
pub(super) fn maintained_activity(index: &Index, id: EntryId) -> Option<i64> {
    let own = (id != EntryId::ROOT).then(|| index.attrs_of(id).map(|attrs| attrs.mtime_ns));
    newer(own.flatten(), index.newest_activity_below(id))
}

/// Assert that the activity `index` maintains is [`activity`]'s on every live directory
/// and, where the index keeps every file, [`measure`]'s, which reads no roll-up at all.
///
/// Checked whatever the index's coverage or scope: a report reads the maintained value
/// only where every subtree was listed, but the index maintains it everywhere, so a
/// partial index that later completes must already hold it exactly.
#[cfg(test)]
pub(crate) fn assert_maintained_activity(index: &Index, label: &str) {
    let table = activity(index);
    let measured =
        (!index.is_folded()).then(|| measure(index, &Selection::default(), NameIdentity::Native));
    let mut stack = vec![(PathBuf::new(), EntryId::ROOT)];
    while let Some((path, id)) = stack.pop() {
        let maintained = maintained_activity(index, id);
        let passed = table.get(id).expect("every live directory has a value").newest_ns;
        assert_eq!(maintained, passed, "{label}: maintained and pass at {}", path.display());
        if let Some(values) = &measured {
            let value = values[&id].mtime_ns;
            assert_eq!(
                maintained,
                (value != i64::MIN).then_some(value),
                "{label}: maintained and measured at {}",
                path.display()
            );
        }
        for (name, child) in index.children_of(id).expect("a live directory") {
            if index.kind_of(child) == Some(EntryKind::Dir) {
                stack.push((path.join(name), child));
            }
        }
    }
}

/// What one entry of an index says about the disk, for [`assert_same_as_cold_walk`].
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HeldEntry {
    kind: EntryKind,
    mtime_ns: i64,
    /// A regular file's apparent size; a directory's varies by reader and filesystem.
    size: Option<u64>,
    /// A directory's tree-row activity, its maintained `mtime_ns`; `None` for any other kind.
    activity: Option<i64>,
}

#[cfg(test)]
fn held_entries(index: &Index) -> BTreeMap<PathBuf, HeldEntry> {
    let mut held = BTreeMap::new();
    let mut stack = vec![(PathBuf::new(), EntryId::ROOT)];
    while let Some((path, id)) = stack.pop() {
        let kind = index.kind_of(id).expect("a live entry");
        let attrs = index.attrs_of(id).expect("a live entry");
        held.insert(
            path.clone(),
            HeldEntry {
                kind,
                mtime_ns: attrs.mtime_ns,
                size: (kind == EntryKind::File).then_some(attrs.size),
                activity: if kind.is_dir() { maintained_activity(index, id) } else { None },
            },
        );
        if kind.is_dir() {
            for (name, child) in index.children_of(id).expect("a live directory") {
                stack.push((path.join(name), child));
            }
        }
    }
    held
}

/// Assert that `index` holds what a cold walk of `root` under `scan` finds now: every
/// entry, its kind and own modification time, each file's size, and each directory's
/// tree-row activity.
///
/// The incremental routes are otherwise compared with themselves, with a pass over the
/// same index, or with a model fed the same operations, and none of those can see a fact
/// that no operation carried: a directory whose own time moved because an entry inside it
/// was removed or renamed, which no event names (B1 on #191). This holds them to the
/// disk instead.
#[cfg(test)]
pub(crate) fn assert_same_as_cold_walk(
    index: &Index,
    root: &Path,
    scan: &crate::ScanConfig,
    label: &str,
) {
    let (cold, report) = crate::scan::scan_into_index(root, scan).expect("cold walk");
    assert!(report.is_complete(), "{label}: the cold walk is complete");
    let (held, walked) = (held_entries(index), held_entries(&cold));
    let differing: Vec<_> = walked
        .keys()
        .chain(held.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .filter(|path| held.get(*path) != walked.get(*path))
        .map(|path| (path.clone(), held.get(path).copied(), walked.get(path).copied()))
        .collect();
    assert!(differing.is_empty(), "{label}: (path, held, cold walk) differ: {differing:#?}");
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

    /// The activity every directory maintains with its roll-up, by path.
    fn maintained(index: &Index) -> BTreeMap<PathBuf, Option<i64>> {
        directories(index)
            .into_iter()
            .map(|(path, id)| (path, maintained_activity(index, id)))
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

    /// The activity each directory maintains stays the pass's through every change the
    /// apply lane makes: a file written newer and older, the newest file removed, a
    /// directory's own time raised past everything and lowered back, a symlink's and an
    /// other entry's time moved both ways, kinds replaced, a subtree removed, and a rename.
    /// Each step also states the values it expects where the repair is the point.
    #[test]
    fn the_maintained_activity_follows_every_kind_of_change() {
        fn step(index: &mut Index, label: &str, ops: Vec<Op>) {
            index.apply_ok(&Observation::new(ops));
            assert_maintained_activity(index, label);
        }
        fn newest(index: &Index, path: &str) -> Option<i64> {
            let id = index.lookup(Path::new(path)).expect("a live directory");
            maintained_activity(index, id)
        }
        let mut index = Index::new("/root");
        step(
            &mut index,
            "initial",
            vec![
                upsert("src", EntryKind::Dir, attrs(0, 5)),
                upsert("src/main.rs", EntryKind::File, attrs(10, 10)),
                upsert("src/deep", EntryKind::Dir, attrs(0, 6)),
                upsert("src/deep/a.rs", EntryKind::File, attrs(10, 40)),
                upsert("src/deep/b.rs", EntryKind::File, attrs(10, 30)),
                upsert("src/empty", EntryKind::Dir, attrs(0, 20)),
                upsert("docs", EntryKind::Dir, attrs(0, 4)),
                upsert("docs/link", EntryKind::Symlink, attrs(0, 50)),
                upsert("fifo", EntryKind::Other, attrs(0, 15)),
            ],
        );
        assert_eq!(
            [newest(&index, ""), newest(&index, "src"), newest(&index, "src/empty")],
            [Some(50), Some(40), Some(20)],
            "the symlink is the root's newest, and an empty directory's own time is its"
        );

        step(&mut index, "file newer", vec![upsert("src/main.rs", EntryKind::File, attrs(10, 70))]);
        assert_eq!(newest(&index, ""), Some(70));
        step(&mut index, "file older", vec![upsert("src/main.rs", EntryKind::File, attrs(10, 1))]);
        assert_eq!([newest(&index, ""), newest(&index, "src")], [Some(50), Some(40)]);
        step(&mut index, "newest file removed", vec![Op::Remove { path: "src/deep/a.rs".into() }]);
        assert_eq!(newest(&index, "src"), Some(30));

        step(
            &mut index,
            "directory raised",
            vec![upsert("src/deep", EntryKind::Dir, attrs(0, 90))],
        );
        assert_eq!(
            [newest(&index, ""), newest(&index, "src"), newest(&index, "src/deep")],
            [Some(90), Some(90), Some(90)],
            "a directory's own time is activity in its row and in every ancestor's"
        );
        step(
            &mut index,
            "directory lowered",
            vec![upsert("src/deep", EntryKind::Dir, attrs(0, 2))],
        );
        assert_eq!([newest(&index, ""), newest(&index, "src")], [Some(50), Some(30)]);

        step(
            &mut index,
            "symlink older",
            vec![upsert("docs/link", EntryKind::Symlink, attrs(0, 3))],
        );
        assert_eq!([newest(&index, ""), newest(&index, "docs")], [Some(30), Some(4)]);
        step(
            &mut index,
            "symlink newer",
            vec![upsert("docs/link", EntryKind::Symlink, attrs(0, 100))],
        );
        step(&mut index, "other newer", vec![upsert("fifo", EntryKind::Other, attrs(0, 200))]);
        assert_eq!(newest(&index, ""), Some(200));
        step(&mut index, "other older", vec![upsert("fifo", EntryKind::Other, attrs(0, 0))]);
        assert_eq!(newest(&index, ""), Some(100));

        step(
            &mut index,
            "kinds replaced",
            vec![
                upsert("fifo", EntryKind::Dir, attrs(0, 300)),
                upsert("fifo/inner", EntryKind::File, attrs(1, 7)),
                upsert("src/deep/b.rs", EntryKind::Symlink, attrs(0, 35)),
                upsert("src/empty", EntryKind::File, attrs(1, 8)),
            ],
        );
        assert_eq!([newest(&index, ""), newest(&index, "src")], [Some(300), Some(35)]);
        step(
            &mut index,
            "subtrees removed",
            vec![Op::Remove { path: "fifo".into() }, Op::Remove { path: "docs".into() }],
        );
        assert_eq!(newest(&index, ""), Some(35));

        step(
            &mut index,
            "rename",
            vec![
                Op::Remove { path: "src/deep".into() },
                upsert("moved", EntryKind::Dir, attrs(0, 3)),
                upsert("moved/deep", EntryKind::Dir, attrs(0, 2)),
                upsert("moved/deep/b.rs", EntryKind::Symlink, attrs(0, 35)),
            ],
        );
        assert_eq!(
            [newest(&index, ""), newest(&index, "src"), newest(&index, "moved")],
            [Some(35), Some(8), Some(35)]
        );

        step(
            &mut index,
            "everything removed",
            vec![Op::Remove { path: "src".into() }, Op::Remove { path: "moved".into() }],
        );
        assert_eq!(newest(&index, ""), None, "a root holding nothing has no activity");
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
            // The roll-up a folded index maintains counts the files it folded, as the pass
            // reads them, so it is the full index's too.
            assert_maintained_activity(&folded, &format!("{label}, {largest_files} kept"));
            assert_eq!(maintained(&folded), maintained(&full), "{label}, {largest_files} kept");
        }
        assert_maintained_activity(&full, label);
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

    /// Every route that builds an index from a walk or a file maintains the activity the
    /// pass computes, and the same activity for the same tree: the detached cold walk, the
    /// scanner's apply lane, a scan-depth boundary, and a snapshot load. The folded index
    /// and the failed subtree are pinned with the folded pass above.
    #[test]
    fn every_route_that_builds_an_index_maintains_the_passs_activity() {
        let tree = folding_tree();
        let canonical = tree.path().canonicalize().expect("canonical");
        let (detached, _) = crate::scan::scan_into_index(&canonical, &crate::ScanConfig::default())
            .expect("detached walk");
        assert_maintained_activity(&detached, "detached walk");

        // A walk that leaves ignored entries out applies its listings through the apply
        // lane, as the opened root and every refresh do, rather than the detached builder.
        let scanner = crate::ScanConfig {
            population: crate::query::IgnoredEntries::Exclude,
            ..crate::ScanConfig::default()
        };
        let (applied, _) =
            crate::scan::scan_into_index(&canonical, &scanner).expect("scanner walk");
        assert!(!applied.is_folded());
        assert_maintained_activity(&applied, "scanner walk");
        assert_eq!(maintained(&applied), maintained(&detached), "same tree, same activity");

        let bounded = crate::ScanConfig { max_depth: Some(2), ..crate::ScanConfig::default() };
        let (shallow, _) =
            crate::scan::scan_into_index(&canonical, &bounded).expect("bounded walk");
        assert_maintained_activity(&shallow, "scan depth 2");

        let cache = tempfile::tempdir().expect("cache dir");
        let snapshot = cache.path().join("tree.fdu");
        crate::snapshot::save(&detached, &snapshot).expect("save");
        let loaded = crate::snapshot::load(&snapshot).expect("load").expect("present");
        assert_maintained_activity(&loaded, "snapshot load");
        assert_eq!(maintained(&loaded), maintained(&detached), "a load rebuilds it exactly");
    }
}
