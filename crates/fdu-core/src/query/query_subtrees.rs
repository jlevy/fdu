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
    /// throughout a one-shot index whose walk finished with errors, which records that
    /// the walk was partial but not where. A lower-bound maximum is not an age, so the
    /// reader that consumes this turns the mtime into an unknown age rather than
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
/// The scan root is a traversal boundary, not a selectable entry.
///
/// This is the one owner of the directory-metric definition, and it runs once per
/// report: the selection walk looks these values up by id and never measures a subtree
/// itself, which is what keeps a report at one pass here plus one pass there however
/// many directories match.
pub(super) fn measure(
    index: &Index,
    selection: &Selection,
    identity: NameIdentity,
) -> BTreeMap<EntryId, SubtreeValues> {
    // A directory is listed in full when the index marks it so, or when the whole index
    // is complete: a one-shot walk marks every directory only when it finished without
    // error, and an index assembled from observations alone never marks one, so coverage
    // has to decide for both. A partial one-shot index therefore holds no complete
    // directory at all, which is the truth it records: the walk was partial, and nothing
    // says where.
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
        // directory itself. Under `--only-ignored` an unignored ancestor is traversed for
        // the ignored matches beneath it, and its own activity is not theirs. Every
        // descendant of an ignored directory is ignored today, so no admitted directory
        // sits under a rejected one and this changes no answer; it is here so a change
        // to negation handling cannot make the seed silently wrong.
        let own_ignored = index.ignored_bit_of(id).unwrap_or(false);
        let own_time = if selection.ignored.admits(own_ignored) {
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
                let ignored = index.ignored_bit_of(child).unwrap_or(false);
                if with_candidate(&child_path, kind, *attrs, ignored, identity, |candidate| {
                    pruned(selection, &candidate)
                }) {
                    continue;
                }
                if kind == EntryKind::Dir {
                    if selection.ignored.admits(ignored) {
                        total.dirs += 1;
                    }
                    stack.push((child, child_path, false));
                } else if selection.ignored.admits(ignored) {
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
