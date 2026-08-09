//! The in-memory hierarchical index.
//!
//! The index is a parent-pointer tree in a flat arena. Entries store their **name only**
//! and paths are reconstructed by walking parents, so a path like
//! `srv/data/project/src/lib/utils.rs` costs six name strings across six entries with no
//! duplication — the fsearch/ncdu layout, deliberately not dut's full-path-per-entry.
//!
//! Every directory carries pre-computed roll-up state for its whole subtree, so a query
//! reads a field and never traverses. Applying a [`Delta`] re-merges that state up the
//! ancestor chain only.
//!
//! Reducers split into two classes and the split is visible in the code, because it
//! decides the cost of an update:
//!
//! - **Invertible** (counts, byte sums, per-extension tallies) apply differentially in
//!   O(depth): add the new contribution, subtract the old one.
//! - **Non-invertible** ([`RollUp::newest_mtime_ns`]) absorb *additions* in O(depth) by
//!   taking a max, but a *removal* may need the directory's value rebuilt from its direct
//!   children — standard incremental-view-maintenance behaviour. Metabrowser's
//!   per-parent newest-mtime heaps are exactly this workaround, hand-written for one
//!   metric.
//!
//! # Concurrency
//!
//! This type is a single-writer structure. The intended deployment is one writer
//! applying deltas behind a `RwLock` with readers taking the read side: writes are short
//! (O(depth) applies) and reads are field lookups rather than queries that walk. The
//! delta contract being the only mutation path means escalating later to epoch or
//! arc-swap snapshots stays contained rather than becoming a rewrite.

use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::path::{Component, Path, PathBuf};

use crate::classify::derive_ext;
use crate::types::{Attrs, Clock, Delta, EntryKind, InvalidateReason, Op};

/// How many applied deltas the index retains for [`Index::since`].
///
/// Bounded on purpose: an unbounded journal is a memory leak in a long-lived server. A
/// consumer that falls further behind than this is told so ([`Since::truncated`]) and is
/// expected to re-read state rather than silently miss changes.
const JOURNAL_CAPACITY: usize = 4096;

/// Identifier for an entry within an [`Index`] arena.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct EntryId(u32);

impl EntryId {
    /// The root entry. Always present, never removed.
    pub const ROOT: EntryId = EntryId(0);

    #[inline]
    const fn idx(self) -> usize {
        self.0 as usize
    }
}

/// Per-extension tally within a roll-up.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ExtTally {
    pub files: u64,
    pub bytes: u64,
}

/// Pre-computed aggregate state for one directory's entire subtree.
///
/// # What is counted
///
/// `bytes` and `allocated` sum **files only**. Directories contribute their own subtree
/// plus one to `dirs`, but their own inode block usage is not added — unlike `du`, which
/// counts directory blocks. The difference is small and constant per directory, and
/// making it configurable is deferred rather than guessed at.
///
/// `newest_mtime_ns` is the newest modification time among descendant **files**.
/// Directory mtimes are excluded because they change on every child add or remove, which
/// makes "what changed recently" answer with directories instead of the edits a user
/// actually made.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct RollUp {
    /// Descendant files.
    pub files: u64,
    /// Descendant directories, not counting the directory that owns this roll-up.
    pub dirs: u64,
    /// Apparent bytes across descendant files.
    pub bytes: u64,
    /// Allocated bytes across descendant files.
    pub allocated: u64,
    /// Newest mtime among descendant files, or 0 when there are none.
    pub newest_mtime_ns: i64,
    /// Per-extension file and byte tallies across the subtree.
    pub by_ext: BTreeMap<String, ExtTally>,
}

impl RollUp {
    /// Fold another roll-up into this one. Commutative and associative, which is what
    /// lets the walk merge subtrees in whatever order threads finish them.
    fn merge(&mut self, other: &RollUp) {
        self.files += other.files;
        self.dirs += other.dirs;
        self.bytes += other.bytes;
        self.allocated += other.allocated;
        self.newest_mtime_ns = self.newest_mtime_ns.max(other.newest_mtime_ns);
        for (ext, tally) in &other.by_ext {
            let slot = self.by_ext.entry(ext.clone()).or_default();
            slot.files += tally.files;
            slot.bytes += tally.bytes;
        }
    }

    /// Remove another roll-up's contribution from this one.
    ///
    /// Only the invertible reducers are corrected here. `newest_mtime_ns` is left stale
    /// on purpose and repaired by [`Index::recompute_newest_upward`], because a max
    /// cannot be un-merged without knowing what else contributed it.
    fn unmerge(&mut self, other: &RollUp) {
        self.files = self.files.saturating_sub(other.files);
        self.dirs = self.dirs.saturating_sub(other.dirs);
        self.bytes = self.bytes.saturating_sub(other.bytes);
        self.allocated = self.allocated.saturating_sub(other.allocated);
        for (ext, tally) in &other.by_ext {
            if let Some(slot) = self.by_ext.get_mut(ext) {
                slot.files = slot.files.saturating_sub(tally.files);
                slot.bytes = slot.bytes.saturating_sub(tally.bytes);
                if slot.files == 0 && slot.bytes == 0 {
                    self.by_ext.remove(ext);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Entry {
    parent: Option<EntryId>,
    name: String,
    kind: EntryKind,
    attrs: Attrs,
    /// Populated for directories only.
    children: BTreeMap<String, EntryId>,
    /// Meaningful for directories only.
    rollup: RollUp,
}

#[derive(Clone, Debug)]
enum Slot {
    Occupied(Box<Entry>),
    Free { next_free: Option<EntryId> },
}

/// Result of [`Index::since`].
#[derive(Debug)]
pub struct Since<'a> {
    /// Deltas applied strictly after the requested clock, oldest first.
    pub deltas: Vec<&'a Delta>,
    /// True when the requested clock is older than the retained journal, meaning the
    /// caller has missed changes and must re-read state rather than trust `deltas`.
    pub truncated: bool,
}

/// Summary of what one [`Index::apply`] call did.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ApplyStats {
    /// Entries created.
    pub inserted: u64,
    /// Entries whose attributes changed.
    pub updated: u64,
    /// Entries removed, including cascaded descendants.
    pub removed: u64,
    /// Upserts whose fingerprint already matched, so nothing changed.
    pub unchanged: u64,
    /// Subtrees escalated for re-scan.
    pub invalidated: u64,
}

/// The in-memory hierarchical index.
#[derive(Debug)]
pub struct Index {
    root_path: PathBuf,
    arena: Vec<Slot>,
    free_head: Option<EntryId>,
    live: u64,
    clock: Clock,
    journal: VecDeque<Delta>,
    /// Oldest clock still represented in `journal`.
    journal_floor: Clock,
    pending_invalidations: Vec<(PathBuf, InvalidateReason)>,
}

impl Index {
    /// Create an empty index rooted at `root_path`.
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        let root = Entry {
            parent: None,
            name: String::new(),
            kind: EntryKind::Dir,
            attrs: Attrs::default(),
            children: BTreeMap::new(),
            rollup: RollUp::default(),
        };
        Self {
            root_path: root_path.into(),
            arena: vec![Slot::Occupied(Box::new(root))],
            free_head: None,
            live: 1,
            clock: Clock::ZERO,
            journal: VecDeque::new(),
            journal_floor: Clock::ZERO,
            pending_invalidations: Vec::new(),
        }
    }

    /// The absolute path this index is rooted at.
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// The clock of the most recently applied delta.
    pub fn clock(&self) -> Clock {
        self.clock
    }

    /// Number of live entries, including the root.
    pub fn len(&self) -> u64 {
        self.live
    }

    /// True when the index holds nothing but its root.
    pub fn is_empty(&self) -> bool {
        self.live <= 1
    }

    /// Roll-up state for the whole tree.
    pub fn total(&self) -> &RollUp {
        &self.entry(EntryId::ROOT).rollup
    }

    /// Apply a delta, stamping it with the next clock.
    ///
    /// Idempotent: an [`Op::Upsert`] whose fingerprint already matches the stored entry
    /// is counted in [`ApplyStats::unchanged`] and changes nothing.
    pub fn apply(&mut self, delta: &Delta) -> ApplyStats {
        let mut stats = ApplyStats::default();
        for op in &delta.ops {
            match op {
                Op::Upsert { path, kind, attrs } => {
                    self.apply_upsert(path, *kind, *attrs, &mut stats);
                }
                Op::Remove { path } => self.apply_remove(path, &mut stats),
                Op::InvalidateSubtree { path, reason } => {
                    self.pending_invalidations.push((path.clone(), *reason));
                    stats.invalidated += 1;
                }
            }
        }

        self.clock = self.clock.next();
        let stamped = Delta { clock: self.clock, ops: delta.ops.clone() };
        self.journal.push_back(stamped);
        while self.journal.len() > JOURNAL_CAPACITY {
            if let Some(dropped) = self.journal.pop_front() {
                self.journal_floor = dropped.clock;
            }
        }
        stats
    }

    /// Deltas applied since `clock`, oldest first.
    pub fn since(&self, clock: Clock) -> Since<'_> {
        Since {
            deltas: self.journal.iter().filter(|d| d.clock > clock).collect(),
            truncated: clock < self.journal_floor,
        }
    }

    /// Take the subtrees that producers escalated for re-scan.
    ///
    /// The caller is expected to hand these to the scan layer, which turns them back
    /// into precise deltas. Escalation is closed-loop: draining this list without
    /// re-scanning is what makes an index silently diverge.
    pub fn take_pending_invalidations(&mut self) -> Vec<(PathBuf, InvalidateReason)> {
        std::mem::take(&mut self.pending_invalidations)
    }

    /// Look up an entry id by path relative to the root.
    pub fn lookup(&self, path: &Path) -> Option<EntryId> {
        let mut current = EntryId::ROOT;
        for part in normalize(path)? {
            current = *self.entry(current).children.get(&part)?;
        }
        Some(current)
    }

    /// Roll-up state for a directory, by relative path. The empty path is the root.
    pub fn rollup(&self, path: &Path) -> Option<&RollUp> {
        let id = self.lookup(path)?;
        let entry = self.entry(id);
        entry.kind.is_dir().then_some(&entry.rollup)
    }

    /// Attributes for any entry, by relative path.
    pub fn attrs(&self, path: &Path) -> Option<&Attrs> {
        Some(&self.entry(self.lookup(path)?).attrs)
    }

    /// Kind of an entry, by relative path.
    pub fn kind(&self, path: &Path) -> Option<EntryKind> {
        Some(self.entry(self.lookup(path)?).kind)
    }

    /// Direct children of a directory, as `(name, id)` pairs in name order.
    pub fn children(&self, path: &Path) -> Option<Vec<(&str, EntryId)>> {
        let id = self.lookup(path)?;
        let entry = self.entry(id);
        entry
            .kind
            .is_dir()
            .then(|| entry.children.iter().map(|(n, id)| (n.as_str(), *id)).collect())
    }

    /// Direct children of an entry id, as `(name, id)` pairs in name order.
    ///
    /// Empty for anything that is not a directory.
    pub fn children_of(&self, id: EntryId) -> Vec<(&str, EntryId)> {
        self.entry(id).children.iter().map(|(name, child)| (name.as_str(), *child)).collect()
    }

    /// Reconstruct an entry's path relative to the root by walking parent pointers.
    pub fn path_of(&self, id: EntryId) -> PathBuf {
        let mut parts = Vec::new();
        let mut current = Some(id);
        while let Some(node) = current {
            let entry = self.entry(node);
            if entry.parent.is_some() {
                parts.push(entry.name.as_str());
            }
            current = entry.parent;
        }
        parts.reverse();
        parts.iter().collect()
    }

    /// Roll-up state for an entry id, if it is a directory.
    pub fn rollup_of(&self, id: EntryId) -> Option<&RollUp> {
        let entry = self.entry(id);
        entry.kind.is_dir().then_some(&entry.rollup)
    }

    /// Attributes for an entry id.
    pub fn attrs_of(&self, id: EntryId) -> &Attrs {
        &self.entry(id).attrs
    }

    /// Kind for an entry id.
    pub fn kind_of(&self, id: EntryId) -> EntryKind {
        self.entry(id).kind
    }

    /// Name for an entry id. The root's name is empty.
    pub fn name_of(&self, id: EntryId) -> &str {
        &self.entry(id).name
    }

    // ---- internals ----

    fn entry(&self, id: EntryId) -> &Entry {
        match &self.arena[id.idx()] {
            Slot::Occupied(entry) => entry,
            Slot::Free { .. } => panic!("entry {id:?} was freed while still referenced"),
        }
    }

    fn entry_mut(&mut self, id: EntryId) -> &mut Entry {
        match &mut self.arena[id.idx()] {
            Slot::Occupied(entry) => entry,
            Slot::Free { .. } => panic!("entry {id:?} was freed while still referenced"),
        }
    }

    fn alloc(&mut self, entry: Entry) -> EntryId {
        self.live += 1;
        if let Some(free) = self.free_head {
            let next = match &self.arena[free.idx()] {
                Slot::Free { next_free } => *next_free,
                Slot::Occupied(_) => unreachable!("free list pointed at a live slot"),
            };
            self.free_head = next;
            self.arena[free.idx()] = Slot::Occupied(Box::new(entry));
            return free;
        }
        let id =
            EntryId(u32::try_from(self.arena.len()).expect("index arena exceeded u32 capacity"));
        self.arena.push(Slot::Occupied(Box::new(entry)));
        id
    }

    fn free(&mut self, id: EntryId) {
        self.arena[id.idx()] = Slot::Free { next_free: self.free_head };
        self.free_head = Some(id);
        self.live -= 1;
    }

    /// What an entry contributes to each of its ancestors.
    fn contribution(&self, id: EntryId) -> RollUp {
        let entry = self.entry(id);
        match entry.kind {
            EntryKind::Dir => {
                let mut roll = entry.rollup.clone();
                roll.dirs += 1;
                roll
            }
            EntryKind::File | EntryKind::Symlink | EntryKind::Other => {
                let mut roll = RollUp {
                    files: 1,
                    dirs: 0,
                    bytes: entry.attrs.size,
                    allocated: entry.attrs.allocated,
                    newest_mtime_ns: entry.attrs.mtime_ns,
                    by_ext: BTreeMap::new(),
                };
                if let Some(ext) = derive_ext(&entry.name) {
                    roll.by_ext.insert(ext, ExtTally { files: 1, bytes: entry.attrs.size });
                }
                roll
            }
        }
    }

    fn merge_upward(&mut self, from_parent: Option<EntryId>, contribution: &RollUp) {
        let mut current = from_parent;
        while let Some(id) = current {
            let entry = self.entry_mut(id);
            entry.rollup.merge(contribution);
            current = entry.parent;
        }
    }

    fn unmerge_upward(&mut self, from_parent: Option<EntryId>, contribution: &RollUp) {
        let mut current = from_parent;
        while let Some(id) = current {
            let entry = self.entry_mut(id);
            entry.rollup.unmerge(contribution);
            current = entry.parent;
        }
    }

    /// Rebuild `newest_mtime_ns` from direct children, walking to the root.
    ///
    /// Stops early once a directory's value is unchanged, since nothing above it can
    /// change either. That early exit is what keeps the common removal O(depth) instead
    /// of O(depth x children).
    fn recompute_newest_upward(&mut self, from: Option<EntryId>) {
        let mut current = from;
        while let Some(id) = current {
            let mut newest = 0i64;
            for child in self.entry(id).children.values() {
                let child_entry = self.entry(*child);
                let candidate = if child_entry.kind.is_dir() {
                    child_entry.rollup.newest_mtime_ns
                } else {
                    child_entry.attrs.mtime_ns
                };
                newest = newest.max(candidate);
            }
            let entry = self.entry_mut(id);
            if entry.rollup.newest_mtime_ns == newest {
                return;
            }
            entry.rollup.newest_mtime_ns = newest;
            current = entry.parent;
        }
    }

    /// Resolve a relative path to a directory id, creating missing ancestors.
    ///
    /// Watch events do not arrive parent-first the way a walk does, so an upsert deep in
    /// a tree may name ancestors the index has never seen. Creating them as directories
    /// with default attributes keeps the delta applicable; a later upsert or the
    /// revalidation sweep fills in their real attributes.
    fn ensure_dir_chain(&mut self, parts: &[String]) -> EntryId {
        let mut current = EntryId::ROOT;
        for part in parts {
            if let Some(existing) = self.entry(current).children.get(part) {
                current = *existing;
                continue;
            }
            let child = self.alloc(Entry {
                parent: Some(current),
                name: part.clone(),
                kind: EntryKind::Dir,
                attrs: Attrs::default(),
                children: BTreeMap::new(),
                rollup: RollUp::default(),
            });
            self.entry_mut(current).children.insert(part.clone(), child);
            // A new empty directory contributes one to `dirs` all the way up.
            let contribution = RollUp { dirs: 1, ..RollUp::default() };
            self.merge_upward(Some(current), &contribution);
            current = child;
        }
        current
    }

    fn apply_upsert(&mut self, path: &Path, kind: EntryKind, attrs: Attrs, stats: &mut ApplyStats) {
        let Some(parts) = normalize(path) else {
            return;
        };
        let Some((name, ancestors)) = parts.split_last() else {
            // The root itself: only its own attributes can change.
            self.entry_mut(EntryId::ROOT).attrs = attrs;
            stats.updated += 1;
            return;
        };

        let parent = self.ensure_dir_chain(ancestors);
        let existing = self.entry(parent).children.get(name).copied();

        if let Some(id) = existing {
            let entry = self.entry(id);
            if entry.kind == kind {
                if entry.attrs.fingerprint() == attrs.fingerprint() {
                    stats.unchanged += 1;
                    return;
                }
                if kind.is_dir() {
                    // A directory's own attributes do not reach its ancestors' roll-ups,
                    // so there is nothing to re-merge.
                    self.entry_mut(id).attrs = attrs;
                    stats.updated += 1;
                    return;
                }
                let old = self.contribution(id);
                self.unmerge_upward(Some(parent), &old);
                self.entry_mut(id).attrs = attrs;
                let new = self.contribution(id);
                self.merge_upward(Some(parent), &new);
                if new.newest_mtime_ns < old.newest_mtime_ns {
                    self.recompute_newest_upward(Some(parent));
                }
                stats.updated += 1;
                return;
            }
            // The kind changed (a file became a directory, say). Remove and re-insert
            // rather than trying to mutate one shape into the other.
            self.remove_entry(id, stats);
        }

        let id = self.alloc(Entry {
            parent: Some(parent),
            name: name.clone(),
            kind,
            attrs,
            children: BTreeMap::new(),
            rollup: RollUp::default(),
        });
        self.entry_mut(parent).children.insert(name.clone(), id);
        let contribution = self.contribution(id);
        self.merge_upward(Some(parent), &contribution);
        stats.inserted += 1;
    }

    fn apply_remove(&mut self, path: &Path, stats: &mut ApplyStats) {
        let Some(id) = self.lookup(path) else {
            return;
        };
        if id == EntryId::ROOT {
            return;
        }
        self.remove_entry(id, stats);
    }

    fn remove_entry(&mut self, id: EntryId, stats: &mut ApplyStats) {
        let parent = self.entry(id).parent;
        let name = self.entry(id).name.clone();
        let contribution = self.contribution(id);

        self.unmerge_upward(parent, &contribution);
        if let Some(parent) = parent {
            self.entry_mut(parent).children.remove(&name);
        }

        // Free the subtree iteratively; a recursive drop would blow the stack on deep
        // trees, which is exactly the shape this engine is built for.
        let mut queue = vec![id];
        while let Some(node) = queue.pop() {
            let children: Vec<EntryId> = self.entry(node).children.values().copied().collect();
            queue.extend(children);
            self.free(node);
            stats.removed += 1;
        }

        // The max may have lived in what was just removed.
        self.recompute_newest_upward(parent);
    }
}

/// Split a relative path into its normal components, rejecting anything that escapes.
///
/// Returns `None` for paths containing `..`, a root, or a prefix — an index keyed by
/// relative path has no way to represent those, and silently normalizing them away would
/// let a delta write outside the tree it claims to describe.
fn normalize(path: &Path) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file_attrs(size: u64, mtime_ns: i64) -> Attrs {
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

    fn index_with_sample_tree() -> Index {
        let mut index = Index::new("/root");
        index.apply(&Delta::new(vec![
            upsert("src", EntryKind::Dir, Attrs::default()),
            upsert("src/main.rs", EntryKind::File, file_attrs(100, 10)),
            upsert("src/lib.rs", EntryKind::File, file_attrs(200, 20)),
            upsert("docs", EntryKind::Dir, Attrs::default()),
            upsert("docs/guide.md", EntryKind::File, file_attrs(300, 30)),
        ]));
        index
    }

    #[test]
    fn rollups_aggregate_up_the_tree() {
        let index = index_with_sample_tree();

        let total = index.total();
        assert_eq!(total.files, 3);
        assert_eq!(total.dirs, 2);
        assert_eq!(total.bytes, 600);
        assert_eq!(total.newest_mtime_ns, 30);

        let src = index.rollup(Path::new("src")).expect("src is a directory");
        assert_eq!(src.files, 2);
        assert_eq!(src.dirs, 0);
        assert_eq!(src.bytes, 300);
        assert_eq!(src.newest_mtime_ns, 20);
    }

    #[test]
    fn per_extension_tallies_roll_up_hierarchically() {
        let index = index_with_sample_tree();

        let total = index.total();
        assert_eq!(total.by_ext[".rs"], ExtTally { files: 2, bytes: 300 });
        assert_eq!(total.by_ext[".md"], ExtTally { files: 1, bytes: 300 });

        // Per-directory breakdown, which no surveyed tool provides.
        let src = index.rollup(Path::new("src")).expect("src is a directory");
        assert_eq!(src.by_ext[".rs"], ExtTally { files: 2, bytes: 300 });
        assert!(!src.by_ext.contains_key(".md"));
    }

    #[test]
    fn upsert_with_matching_fingerprint_is_a_no_op() {
        let mut index = index_with_sample_tree();
        let before = index.total().clone();

        let stats = index.apply(&Delta::new(vec![upsert(
            "src/main.rs",
            EntryKind::File,
            file_attrs(100, 10),
        )]));

        assert_eq!(stats.unchanged, 1);
        assert_eq!(stats.updated, 0);
        assert_eq!(index.total(), &before);
    }

    #[test]
    fn replaying_the_same_delta_twice_changes_nothing() {
        let mut index = Index::new("/root");
        let delta = Delta::new(vec![
            upsert("a", EntryKind::Dir, Attrs::default()),
            upsert("a/f.txt", EntryKind::File, file_attrs(10, 1)),
        ]);
        index.apply(&delta);
        let after_first = index.total().clone();
        let stats = index.apply(&delta);

        assert_eq!(stats.unchanged, 2);
        assert_eq!(index.total(), &after_first);
        assert_eq!(index.len(), 3);
    }

    #[test]
    fn changed_size_updates_every_ancestor() {
        let mut index = index_with_sample_tree();
        index.apply(&Delta::new(vec![upsert("src/main.rs", EntryKind::File, file_attrs(150, 11))]));

        assert_eq!(index.rollup(Path::new("src")).expect("dir").bytes, 350);
        assert_eq!(index.total().bytes, 650);
        assert_eq!(index.total().by_ext[".rs"], ExtTally { files: 2, bytes: 350 });
    }

    #[test]
    fn removing_a_file_corrects_sums_and_rebuilds_the_max() {
        let mut index = index_with_sample_tree();
        // guide.md holds the newest mtime for the whole tree.
        let stats =
            index.apply(&Delta::new(vec![Op::Remove { path: PathBuf::from("docs/guide.md") }]));

        assert_eq!(stats.removed, 1);
        let total = index.total();
        assert_eq!(total.files, 2);
        assert_eq!(total.bytes, 300);
        assert_eq!(total.newest_mtime_ns, 20, "max must fall back to src/lib.rs");
        assert!(!total.by_ext.contains_key(".md"), "emptied tallies are dropped");
    }

    #[test]
    fn removing_a_directory_cascades_to_descendants() {
        let mut index = index_with_sample_tree();
        let stats = index.apply(&Delta::new(vec![Op::Remove { path: PathBuf::from("src") }]));

        assert_eq!(stats.removed, 3, "the directory and both files");
        let total = index.total();
        assert_eq!(total.files, 1);
        assert_eq!(total.dirs, 1);
        assert_eq!(total.bytes, 300);
        assert!(index.lookup(Path::new("src/main.rs")).is_none());
        assert!(!total.by_ext.contains_key(".rs"));
    }

    #[test]
    fn freed_slots_are_reused() {
        let mut index = index_with_sample_tree();
        let before = index.len();
        index.apply(&Delta::new(vec![Op::Remove { path: PathBuf::from("src") }]));
        index.apply(&Delta::new(vec![
            upsert("other", EntryKind::Dir, Attrs::default()),
            upsert("other/x.rs", EntryKind::File, file_attrs(1, 1)),
            upsert("other/y.rs", EntryKind::File, file_attrs(1, 1)),
        ]));
        assert_eq!(index.len(), before, "three freed slots, three new entries");
    }

    #[test]
    fn missing_ancestors_are_created_for_out_of_order_upserts() {
        let mut index = Index::new("/root");
        // A watch event can name a deep path the index has never seen.
        index.apply(&Delta::new(vec![upsert(
            "deep/nested/tree/file.txt",
            EntryKind::File,
            file_attrs(42, 7),
        )]));

        assert_eq!(index.total().files, 1);
        assert_eq!(index.total().dirs, 3);
        assert_eq!(index.total().bytes, 42);
        assert_eq!(index.rollup(Path::new("deep/nested")).expect("created").files, 1);
    }

    #[test]
    fn kind_change_replaces_the_entry() {
        let mut index = Index::new("/root");
        index.apply(&Delta::new(vec![upsert("thing", EntryKind::File, file_attrs(50, 5))]));
        assert_eq!(index.total().files, 1);

        index.apply(&Delta::new(vec![upsert("thing", EntryKind::Dir, Attrs::default())]));
        let total = index.total();
        assert_eq!(total.files, 0);
        assert_eq!(total.dirs, 1);
        assert_eq!(total.bytes, 0);
    }

    #[test]
    fn paths_are_reconstructed_from_parent_pointers() {
        let index = index_with_sample_tree();
        let id = index.lookup(Path::new("src/main.rs")).expect("present");
        assert_eq!(index.path_of(id), PathBuf::from("src/main.rs"));
        assert_eq!(index.path_of(EntryId::ROOT), PathBuf::new());
    }

    #[test]
    fn since_returns_deltas_after_a_clock() {
        let mut index = Index::new("/root");
        index.apply(&Delta::new(vec![upsert("a.txt", EntryKind::File, file_attrs(1, 1))]));
        let mark = index.clock();
        index.apply(&Delta::new(vec![upsert("b.txt", EntryKind::File, file_attrs(2, 2))]));

        let since = index.since(mark);
        assert!(!since.truncated);
        assert_eq!(since.deltas.len(), 1);
        assert_eq!(since.deltas[0].ops[0].path(), Path::new("b.txt"));

        assert_eq!(index.since(index.clock()).deltas.len(), 0);
    }

    #[test]
    fn invalidations_are_queued_for_the_scan_layer() {
        let mut index = Index::new("/root");
        let stats = index.apply(&Delta::new(vec![Op::InvalidateSubtree {
            path: PathBuf::from("src"),
            reason: InvalidateReason::WatchOverflow,
        }]));

        assert_eq!(stats.invalidated, 1);
        let pending = index.take_pending_invalidations();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, PathBuf::from("src"));
        assert_eq!(pending[0].1, InvalidateReason::WatchOverflow);
        assert!(index.take_pending_invalidations().is_empty(), "drained once");
    }

    #[test]
    fn paths_escaping_the_root_are_rejected() {
        assert!(normalize(Path::new("../escape")).is_none());
        assert!(normalize(Path::new("/absolute")).is_none());
        assert_eq!(
            normalize(Path::new("./a/b")).expect("relative"),
            vec!["a".to_string(), "b".to_string()]
        );

        let mut index = Index::new("/root");
        index.apply(&Delta::new(vec![upsert("../escape", EntryKind::File, file_attrs(1, 1))]));
        assert_eq!(index.total().files, 0, "escaping upserts are dropped");
    }
}
