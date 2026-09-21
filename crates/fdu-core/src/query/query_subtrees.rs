//! Directory values used by report selection, before positive predicates are applied.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::index::{EntryId, Index};
use crate::query::query_selection::NameIdentity;
use crate::query::{Candidate, IgnoredEntries, Selection};
use crate::{Attrs, EntryKind};

/// Directory bytes and activity after exclusions, independent of positive predicates.
#[derive(Clone, Copy, Debug)]
pub(super) struct SubtreeValues {
    pub bytes: u64,
    pub allocated: u64,
    pub mtime_ns: i64,
    pub files: u64,
    pub dirs: u64,
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
pub(super) fn measure(
    index: &Index,
    selection: &Selection,
    identity: NameIdentity,
) -> BTreeMap<EntryId, SubtreeValues> {
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
                    }
                }
            }
            values.insert(id, total);
            continue;
        }
        let own_time = index.attrs_of(id).map_or(i64::MIN, |attrs| attrs.mtime_ns);
        let mut total =
            SubtreeValues { bytes: 0, allocated: 0, mtime_ns: own_time, files: 0, dirs: 0 };
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
