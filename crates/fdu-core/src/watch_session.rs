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
    pub dirty_rollups: Vec<PathBuf>,
}

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
            let (path, is_directory) = match op {
                Op::Upsert { path, kind, .. } => (path, kind.is_dir()),
                // A removed entry's own roll-up is gone, so only its ancestors move. An
                // invalidated subtree is a directory whose own totals are in doubt.
                Op::Remove { path } => (path, false),
                Op::InvalidateSubtree { path, .. } => (path, true),
            };
            if is_directory {
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
        let index = self.index.snapshot()?;
        Ok(report(&index, &self.query, provenance))
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

        let mut batch = Batch {
            changes: Vec::new(),
            dirty: !applied.is_empty(),
            dirty_rollups: dirty_rollups(&applied),
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
