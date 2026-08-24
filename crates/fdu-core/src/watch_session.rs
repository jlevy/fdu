//! A live session: an index, a watcher, and the query they answer together.
//!
//! A watch run is the same query as a one-shot run, re-evaluated as changes arrive —
//! there is no separate watch grammar. This module owns that composition so the CLI loop
//! and the Python iterator are both thin consumers rather than two implementations of
//! the same coordination.
//!
//! # Detection is event-driven
//!
//! Changes arrive from the operating system's own notification backend, never from
//! polling: `FSEvents` on macOS, inotify on Linux, `ReadDirectoryChangesW` on Windows. An
//! idle tree costs no filesystem work at all. Events are hints, so each coalesced path is
//! verified with one fresh stat before it becomes a delta, and the interval a caller
//! passes throttles only how often aggregate views are re-rendered — it plays no part in
//! detection.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Duration;

use crate::engine_contract::{AppliedDelta, EntryKind, Op, Result};
use crate::index::IndexHandle;
use crate::query::{Provenance, Query, Report, ReportSource, Selection, report};
use crate::scan::ScanConfig;
use crate::watch::{WatchConfig, Watcher};

/// One effective change, already filtered through the run's selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// Path relative to the index root.
    pub path: PathBuf,
    /// What happened to it.
    pub kind: ChangeKind,
    /// What the entry is, when it still exists.
    pub entry_kind: Option<EntryKind>,
    /// Apparent bytes, when the entry still exists.
    pub bytes: Option<u64>,
    /// Allocated bytes, when the entry still exists.
    pub allocated: Option<u64>,
    /// Modification time, when the entry still exists.
    pub mtime_ns: Option<i64>,
    /// The index clock at which this change was committed.
    pub clock: u64,
}

/// What happened to a path.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChangeKind {
    /// The entry appeared or its metadata changed.
    Upsert,
    /// The entry is gone.
    Remove,
    /// A producer could not describe the change precisely; the subtree was re-scanned.
    ///
    /// Surfaced rather than swallowed: this is the signal that a consumer's own view of
    /// the subtree may have gaps, and dropping it is how an index silently diverges.
    Invalidate,
}

/// One batch of applied changes.
#[derive(Clone, Debug, Default)]
pub struct Batch {
    /// Changes the selection admitted, in commit order.
    pub changes: Vec<Change>,
    /// Whether anything was applied at all, before selection filtering.
    ///
    /// A batch can be non-empty and still yield no changes, when everything it carried
    /// was filtered out. Aggregate views re-render on this rather than on `changes`,
    /// because a filtered-out change still moves the totals a tree view reports.
    pub dirty: bool,
    /// Directories whose roll-up values this batch may have moved, root first.
    ///
    /// Every ancestor of every applied path, plus a changed directory itself: exactly
    /// the chain `merge_upward` walks when it applies the batch. A consumer caching a
    /// per-directory answer invalidates these and keeps the rest, instead of re-deriving
    /// the set from change paths itself or dropping every cached row.
    ///
    /// Sorted and deduplicated, and never filtered by the selection: a change the
    /// selection hides still moves the totals its ancestors report, which is the same
    /// reason `dirty` is not computed from `changes`.
    ///
    /// Empty when `all_dirty` is set: past a bound, listing the paths costs more than the
    /// consumer saves by having them.
    pub dirty_rollups: Vec<PathBuf>,
    /// Every cached roll-up should be discarded; `dirty_rollups` is not the answer.
    ///
    /// A bounded carrier has to say when it gave up enumerating, or a consumer reads a
    /// truncated list as a complete one and keeps rows it should have dropped. This is
    /// that signal, and it is why the list above can be empty while `dirty` is true.
    pub all_dirty: bool,
    /// The consumer's position is gone; it must re-read rather than apply these changes.
    ///
    /// Set when the kernel dropped events, when a rename could not be paired, or when a
    /// directory's watch was registered too late to see what was created inside it. The
    /// engine recovers by re-scanning the affected subtree, so the index is right either
    /// way -- but a consumer replaying `changes` alone would be applying a suffix to
    /// state that no longer matches, which is the one recovery it cannot do.
    pub reset: bool,
    /// The version this batch leaves the index at, and the position to resume from.
    ///
    /// Captured after the batch is applied, under the index's own guard, so it names
    /// exactly the commits this batch carries -- not a later sample that would skip
    /// whatever landed in between.
    pub cursor: crate::Cursor,
}

/// How many dirty directories a batch enumerates before it says "all of them".
///
/// A rename near the root, or a reconciliation sweep, touches every ancestor of every
/// path -- so the honest list is sometimes the whole tree, and shipping it costs a
/// `PathBuf` per directory to tell a consumer something one bit says better.
const MAX_DIRTY_ROLLUPS: usize = 1024;

/// Directories whose roll-up values a set of applied deltas may have moved.
///
/// The ancestors of every touched path, plus a touched directory itself, which is the
/// chain the index walks upward as it applies. Derived from the ops rather than reported
/// out of the reducer because the two are the same set by construction, and deriving it
/// keeps the index from having to learn what a consumer's cache wants.
fn dirty_rollups(applied: &[AppliedDelta]) -> Vec<PathBuf> {
    let mut dirty: BTreeSet<PathBuf> = BTreeSet::new();
    for delta in applied {
        for op in &delta.ops {
            let (path, own_key) = match op {
                // A file's own key is not a roll-up, so naming it would invalidate
                // something no consumer caches.
                Op::Upsert { path, kind, .. } => (path, kind.is_dir()),
                // A removal names its own key whatever it removed, and an invalidated
                // subtree is a directory whose totals are in doubt. `Remove` does not say
                // what was there, and the two ways of guessing are not symmetric: a
                // removed *file* has no cached roll-up, so naming its key costs a consumer
                // nothing, while a removed *directory* had one and would otherwise keep it
                // forever -- stale, with no later event ever naming it again, because the
                // entry is gone. Guessing in the cheap direction is the whole trade.
                Op::Remove { path } | Op::InvalidateSubtree { path, .. } => (path, true),
            };
            if own_key {
                dirty.insert(path.clone());
            }
            let mut ancestor = path.parent();
            while let Some(current) = ancestor {
                dirty.insert(current.to_path_buf());
                ancestor = current.parent();
            }
            // The root is an ancestor of everything and its totals always move.
            dirty.insert(PathBuf::new());
        }
    }
    dirty.into_iter().collect()
}

/// An index paired with a watcher, answering one query continuously.
pub struct Session {
    index: IndexHandle,
    watcher: Watcher,
    scan: ScanConfig,
    query: Query,
}

impl Session {
    /// Start watching an already-opened index.
    pub fn new(
        index: IndexHandle,
        scan: ScanConfig,
        query: Query,
        watch: WatchConfig,
    ) -> Result<Self> {
        let root = index.root_path()?;
        // Reject an out-of-scope watch before the backend is bound, so a rejected run
        // never leaves a watcher registered on the tree.
        scan.validate_for_watch_scope(index.scope()?)?;
        let watcher = Watcher::new(&root, watch)?;
        Ok(Self { index, watcher, scan, query })
    }

    /// The query this session answers.
    pub fn query(&self) -> &Query {
        &self.query
    }

    /// Render the current answer.
    ///
    /// The same `report` a one-shot run produces, from the same index, which is what
    /// makes "watch is the same query repeated" true rather than aspirational.
    pub fn report(&self, provenance: &Provenance) -> Result<Report> {
        // Under the read guard, not over a copy. `snapshot` deep-clones every entry, so a
        // repaint of a large tree paid for the whole index each time it drew -- the same
        // regression `with_index` exists to prevent, and the one the read path was already
        // fixed for. A report is a read; it takes a reader's lock like any other.
        self.index.with_index(|index| report(index, &self.query, provenance))
    }

    /// A consistent copy of the current index.
    ///
    /// Used to persist a live session without holding a lock across the write.
    pub fn index_snapshot(&self) -> Result<crate::Index> {
        self.index.snapshot()
    }

    /// Wait for the next batch of changes, up to `timeout`.
    ///
    /// Returns `None` when nothing arrived in the window, which is the idle case and
    /// costs no filesystem work.
    ///
    /// Takes `&mut self` because consuming from the event queue is a mutation: two
    /// callers draining one session would each see an arbitrary half of the stream.
    pub fn next_batch(&mut self, timeout: Duration) -> Result<Option<Batch>> {
        let mut applied: Vec<AppliedDelta> = Vec::new();
        let outcome = self.watcher.apply_next(
            &self.index,
            &self.scan,
            timeout,
            &mut |delta: &AppliedDelta| {
                applied.push(delta.clone());
            },
        )?;

        let Some(_report) = outcome else {
            return Ok(None);
        };

        // A dropped event, an unpaired rename, or a watch registered after its
        // directory already had contents: in each case the engine re-scans and the index
        // ends up right, but a consumer replaying `changes` alone would be applying a
        // suffix to state that no longer matches. It has to re-read, and this is the only
        // thing that tells it so.
        let reset = applied.iter().any(|delta| {
            delta.ops.iter().any(|op| {
                matches!(
                    op,
                    Op::InvalidateSubtree {
                        reason: crate::InvalidateReason::WatchOverflow
                            | crate::InvalidateReason::UnpairedRename
                            | crate::InvalidateReason::WatchSetupRace,
                        ..
                    }
                )
            })
        });
        let dirty_rollups = dirty_rollups(&applied);
        let all_dirty = dirty_rollups.len() > MAX_DIRTY_ROLLUPS;
        let mut batch = Batch {
            changes: Vec::new(),
            dirty: !applied.is_empty(),
            // Past the bound the list is dropped rather than truncated. A truncated list
            // is indistinguishable from a complete one at the consumer, which is how a
            // stale row survives an invalidation that named it.
            dirty_rollups: if all_dirty { Vec::new() } else { dirty_rollups },
            all_dirty,
            reset,
            cursor: self.index.cursor()?,
        };
        // A control file that moved changes what the tag rules decide about entries the
        // batch never touched, so it is handled before the rows are read: rebinding first
        // means the tags this batch reports are the ones the new file produces, not the
        // ones the old one did.
        //
        // The escalations go out even though nothing beneath those directories was
        // upserted or removed. That is the point -- a consumer holding rows for a subtree
        // has no other way to learn that its tags moved, and a silently re-tagged subtree
        // is a view that is wrong without ever having been told it changed.
        for governed in self.rebind_tags_for(&applied)? {
            batch.changes.push(Change {
                path: governed,
                kind: ChangeKind::Invalidate,
                entry_kind: None,
                bytes: None,
                allocated: None,
                mtime_ns: None,
                clock: applied.last().map_or(0, |delta| delta.clock.0),
            });
        }

        // Tags are read from the index, which is where they were computed, so this needs a
        // read guard -- one for the batch rather than one per op, since a batch can carry
        // thousands. Skipped entirely when nothing filters on tags, which is the default.
        let tagged = !self.selection().tags.is_unconstrained();
        for delta in &applied {
            for op in &delta.ops {
                let tags = if tagged {
                    self.index
                        .with_index(|index| {
                            index.lookup(op.path()).map_or(0, |id| index.tag_bits_of(id))
                        })
                        .unwrap_or(0)
                } else {
                    0
                };
                if let Some(change) = self.change_for(op, delta.clock.0, tags) {
                    batch.changes.push(change);
                }
            }
        }
        Ok(Some(batch))
    }

    /// Re-read the tag rules' control files when this batch moved one.
    ///
    /// Returns the directories whose tags may have changed, or nothing at all -- the
    /// common case, and one cheap `any` over the batch -- when no control file was
    /// touched or no enabled rule reads one.
    fn rebind_tags_for(&self, applied: &[AppliedDelta]) -> Result<Vec<PathBuf>> {
        let moved = self.index.with_index(|index| {
            let rules = index.tag_rules();
            !rules.is_empty()
                && applied
                    .iter()
                    .flat_map(|delta| &delta.ops)
                    .any(|op| rules.is_control_file(op.path()))
        })?;
        if !moved {
            return Ok(Vec::new());
        }
        self.index.rebind_tag_rules()
    }

    /// Translate one applied op into a change, when the selection admits it.
    fn change_for(&self, op: &Op, clock: u64, tags: crate::tags::TagBits) -> Option<Change> {
        match op {
            Op::Upsert { path, kind, attrs } => {
                let name = path.file_name()?.to_string_lossy().into_owned();
                let candidate = crate::query::Candidate {
                    relative: path,
                    name: &name,
                    kind: *kind,
                    bytes: attrs.size,
                    allocated: attrs.allocated,
                    mtime_ns: attrs.mtime_ns,
                    tags,
                };
                self.selection().admits(&candidate).then(|| Change {
                    path: path.clone(),
                    kind: ChangeKind::Upsert,
                    entry_kind: Some(*kind),
                    bytes: Some(attrs.size),
                    allocated: Some(attrs.allocated),
                    mtime_ns: Some(attrs.mtime_ns),
                    clock,
                })
            }
            // A removal carries no attributes to filter on, so only the path-shaped parts
            // of a selection can apply. Filtering it out entirely on a size or time bound
            // would hide the disappearance of something the caller was watching. Tags fall
            // on the same side of that line: the entry is gone, so its bits are gone with
            // it, and a tag filter cannot speak for what is no longer there.
            Op::Remove { path } => {
                let name = path.file_name()?.to_string_lossy().into_owned();
                self.admits_by_path(path, &name).then(|| Change {
                    path: path.clone(),
                    kind: ChangeKind::Remove,
                    entry_kind: None,
                    bytes: None,
                    allocated: None,
                    mtime_ns: None,
                    clock,
                })
            }
            // Escalations are never filtered: they say the consumer's view may have gaps,
            // and that is true regardless of what the selection asked for.
            Op::InvalidateSubtree { path, .. } => Some(Change {
                path: path.clone(),
                kind: ChangeKind::Invalidate,
                entry_kind: None,
                bytes: None,
                allocated: None,
                mtime_ns: None,
                clock,
            }),
        }
    }

    /// Whether the path-shaped parts of the selection admit a path.
    fn admits_by_path(&self, path: &std::path::Path, name: &str) -> bool {
        let selection = self.selection();
        if selection.exclude.iter().any(|pattern| pattern.matches(path, name)) {
            return false;
        }
        selection.include.is_empty()
            || selection.include.iter().any(|pattern| pattern.matches(path, name))
    }

    fn selection(&self) -> &Selection {
        &self.query.selection
    }

    /// Provenance for a live report, which is always warm by construction.
    pub fn live_provenance(&self, generated_at: std::time::SystemTime) -> Provenance {
        Provenance {
            scan_started_at: None,
            generated_at,
            source: ReportSource::WarmRevalidate,
            complete: true,
            errors: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_contract::{Attrs, Clock, EntryKind};

    fn delta(clock: u64, ops: Vec<Op>) -> AppliedDelta {
        AppliedDelta { clock: Clock(clock), ops }
    }

    /// A removed directory's own cached roll-up has to be invalidated too.
    ///
    /// The set used to be the ancestors of every touched path plus a touched *directory*,
    /// and a `Remove` was classified as a non-directory because the op does not say what
    /// it removed. So deleting a directory invalidated everything above it and never the
    /// key a consumer had actually cached for it -- which no later event would ever name
    /// again, because the entry is gone. The row survives forever, stale.
    ///
    /// Guessing in the cheap direction is the fix: a removed *file* has no cached roll-up,
    /// so naming its key costs a consumer nothing, while a removed directory's key is the
    /// whole bug. The two errors are not symmetric, so the conservative one wins.
    #[test]
    fn a_removed_directory_invalidates_its_own_rollup_and_not_only_its_ancestors() {
        let dirty =
            dirty_rollups(&[delta(1, vec![Op::Remove { path: PathBuf::from("src/deep") }])]);

        assert!(
            dirty.contains(&PathBuf::from("src/deep")),
            "the removed directory's own key must be invalidated: {dirty:?}"
        );
        assert!(dirty.contains(&PathBuf::from("src")), "and its ancestors: {dirty:?}");
        assert!(dirty.contains(&PathBuf::new()), "including the root: {dirty:?}");
    }

    /// The set is sorted, deduplicated, and rooted, whatever order the ops arrived in.
    #[test]
    fn the_dirty_set_is_a_set_rather_than_a_transcript() {
        let dirty = dirty_rollups(&[delta(
            1,
            vec![
                Op::Upsert {
                    path: PathBuf::from("a/b/one.txt"),
                    kind: EntryKind::File,
                    attrs: Attrs::default(),
                },
                Op::Upsert {
                    path: PathBuf::from("a/b/two.txt"),
                    kind: EntryKind::File,
                    attrs: Attrs::default(),
                },
            ],
        )]);

        let mut sorted = dirty.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(dirty, sorted, "a consumer iterates this; it should not have to clean it");
        assert_eq!(
            dirty,
            vec![PathBuf::new(), PathBuf::from("a"), PathBuf::from("a/b")],
            "two files in one directory move three roll-ups, not six"
        );
    }
}
