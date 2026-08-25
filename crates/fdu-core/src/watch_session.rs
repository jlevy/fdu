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

use crate::engine_contract::{AppliedDelta, CommittedState, EntryKind, Op, Result, StateChange};
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
    /// The last commit this batch carries, and the position to resume from.
    ///
    /// Derived from the deltas rather than sampled from the index afterwards, which is the
    /// difference between correct and nearly correct. Sampling let a concurrent refresh
    /// commit between the apply and the sample: the batch then carried no record of that
    /// commit while naming a position past it, so resuming skipped it permanently. The
    /// clocks here were assigned under the write guard that applied them, so this capture
    /// is atomic without any new locking, and a commit landing afterwards is simply unseen
    /// and replays on the next resume.
    ///
    /// `None` when the batch carried no deltas: it names no new position, and saying so
    /// beats inventing one.
    pub cursor: Option<crate::Cursor>,
    /// Answer-affecting transitions this batch committed, in commit order.
    ///
    /// Coverage moving, a sweep verifying a subtree, a replaced run envelope, re-bound tag
    /// rules: each changes what the rows mean without changing any row, so a consumer that
    /// only watched paths would keep an answer that is wrong and was never contradicted.
    ///
    /// Each keeps the clock it committed at rather than the batch's terminal position, so
    /// a consumer can order it against `changes` and resume from either side of it.
    pub state: Vec<CommittedState>,
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
    let mut dirty_with_ancestors = |path: &PathBuf, own_key: bool| {
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
    };
    for delta in applied {
        for change in &delta.state {
            // Re-bound rules re-tag entries that were never touched, so a cached listing
            // beneath a governed directory holds tags the index no longer agrees with.
            // The other transitions move trust rather than values: they are carried in
            // `Batch::state`, and inventing roll-up dirtiness for them would make every
            // reconciliation sweep look like a mutation of the numbers.
            if let StateChange::Retagged { directories, .. } = change {
                for directory in directories {
                    dirty_with_ancestors(directory, true);
                }
            }
        }
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
            dirty_with_ancestors(path, own_key);
        }
    }
    dirty.into_iter().collect()
}

/// What an incomplete slice means for a consumer's cached state.
///
/// Two different incompletenesses reach the same conclusion by different routes, and
/// keeping them apart is why this is named rather than inlined.
///
/// An escalation says the *engine* lost precision and re-scanned: the index is right, but
/// a consumer replaying `changes` alone would be applying a suffix to state that no longer
/// matches. A truncated journal says the *consumer* fell further behind than retention
/// allows: the changes it missed are unnamed and unnameable, so enumerating what survived
/// would hand it a list it reads as complete -- the same failure `all_dirty` exists to
/// prevent one bound up. Only the second discards the dirty set, because only the second
/// makes that set a lie.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Recovery {
    reset: bool,
    all_dirty: bool,
}

impl Recovery {
    const fn of(escalated: bool, truncated: bool) -> Self {
        Self { reset: escalated || truncated, all_dirty: truncated }
    }
}

impl Drop for Session {
    /// Give the watch back, so the index stops claiming to be watched.
    ///
    /// Errors are swallowed rather than propagated, because a `Drop` has nowhere to put
    /// them and the alternative -- panicking while unwinding -- aborts the process. The
    /// only way this fails is a poisoned lock, which already means a panic happened
    /// inside a write; the count is then the least of it.
    fn drop(&mut self) {
        let _ = self.index.detach_watch();
    }
}

/// An index paired with a watcher, answering one query continuously.
pub struct Session {
    index: IndexHandle,
    watcher: Watcher,
    scan: ScanConfig,
    query: Query,
    /// The position the last batch reported, or where the session started.
    ///
    /// The consumer's resume state, and the thing each batch is built *from*: the journal
    /// since this position is everything the consumer has not seen, whoever committed it.
    /// This stream is not the index's only writer -- a caller can refresh, or ingest its
    /// own hints, against the same handle while a watch runs -- and a batch assembled from
    /// what the watcher delivered would omit those commits while advancing past them.
    resume: crate::Cursor,
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
        let resume = index.cursor()?;
        // The index reports `Phase::Watching` for as long as this session lives, so a
        // consumer reading the envelope can tell a live root from a static one without
        // being told out of band. Recorded after the watcher binds: a session that failed
        // to start never claimed to be watching.
        index.attach_watch()?;
        Ok(Self { index, watcher, scan, query, resume })
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
    /// "Nothing arrived" means the *journal* did not move, not that the watcher stayed
    /// quiet. Another producer -- a caller refreshing, ingesting hints, or rebinding rules
    /// against the same handle -- commits without producing a filesystem event, so waiting
    /// only on the watcher withheld those commits for as long as the tree stayed idle:
    /// forever, on a tree nobody was touching. The check is one clock comparison against
    /// the position already held, so an idle tick still costs no filesystem work and no
    /// journal scan.
    ///
    /// Takes `&mut self` because consuming from the event queue is a mutation: two
    /// callers draining one session would each see an arbitrary half of the stream.
    pub fn next_batch(&mut self, timeout: Duration) -> Result<Option<Batch>> {
        let mut delivered: Vec<AppliedDelta> = Vec::new();
        let outcome = self.watcher.apply_next(
            &self.index,
            &self.scan,
            timeout,
            &mut |delta: &AppliedDelta| {
                delivered.push(delta.clone());
            },
        )?;

        if outcome.is_none() && self.index.clock()? == self.resume.clock {
            return Ok(None);
        }

        // A control file that moved changes what the tag rules decide about entries this
        // batch never touched, so it is rebound before the slice below is taken -- both so
        // the tags reported are the ones the new file produces, and so the rebind's own
        // commit is inside the slice rather than after it.
        self.rebind_tags_for(&delivered)?;

        // The batch is the journal since the consumer's last position, not the deltas this
        // watcher happened to hand back. Those are not the same set, and the difference is
        // a silent loss: `apply_next` reconciles through several separately locked
        // flushes, so a direct producer -- a caller refreshing, or ingesting its own hints,
        // against the same handle -- can commit between two of them. Building from the
        // sink omitted that commit while advancing the cursor past it, and resuming from
        // the cursor skipped it for good with nothing reporting the loss.
        //
        // One guard covers the slice and its terminal position, so the cursor cannot name
        // a commit the slice does not carry.
        let since = self.index.since(self.resume)?;
        let applied = since.deltas;

        // A dropped event, an unpaired rename, or a watch registered after its
        // directory already had contents: in each case the engine re-scans and the index
        // ends up right, but a consumer replaying `changes` alone would be applying a
        // suffix to state that no longer matches. It has to re-read, and this is the only
        // thing that tells it so.
        let escalated = applied.iter().any(|delta| {
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
        let recovery = Recovery::of(escalated, since.truncated);
        // A rebind that stopped enumerating leaves no list to invalidate from, so the whole
        // cache is suspect -- the same conclusion the dirty bound reaches by its own route.
        let retagged_everything = applied.iter().any(|delta| {
            delta
                .state
                .iter()
                .any(|change| matches!(change, StateChange::Retagged { all: true, .. }))
        });
        let dirty_rollups = dirty_rollups(&applied);
        let all_dirty =
            recovery.all_dirty || retagged_everything || dirty_rollups.len() > MAX_DIRTY_ROLLUPS;
        let mut batch = Batch {
            changes: Vec::new(),
            dirty: !applied.is_empty(),
            // Past the bound the list is dropped rather than truncated. A truncated list
            // is indistinguishable from a complete one at the consumer, which is how a
            // stale row survives an invalidation that named it.
            dirty_rollups: if all_dirty { Vec::new() } else { dirty_rollups },
            all_dirty,
            reset: recovery.reset,
            // Taken with the slice rather than from its last delta, so it is the index's
            // own terminal position under the same guard. `None` only when the slice is
            // empty: a batch that carried nothing names no new place to resume from.
            cursor: (!applied.is_empty()).then_some(since.cursor),
            state: applied
                .iter()
                .flat_map(|delta| {
                    delta
                        .state
                        .iter()
                        .map(|change| CommittedState { clock: delta.clock, change: change.clone() })
                })
                .collect(),
        };
        if !applied.is_empty() {
            self.resume = since.cursor;
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
            // A re-tagged subtree is escalated even though nothing beneath it was upserted
            // or removed. That is the point -- a consumer holding rows for it has no other
            // way to learn that their tags moved, and a silently re-tagged subtree is a
            // view that is wrong without ever having been told so. Emitted in the delta's
            // own place in the sequence, so `changes` stays in commit order.
            for change in &delta.state {
                // When the rebind gave up enumerating, `directories` is empty and nothing is
                // emitted here: `all_dirty` already says the whole cache is suspect, and a
                // second copy of an oversized scope is the cost the bound exists to avoid.
                let StateChange::Retagged { directories, .. } = change else {
                    continue;
                };
                for governed in directories {
                    batch.changes.push(Change {
                        path: governed.clone(),
                        kind: ChangeKind::Invalidate,
                        entry_kind: None,
                        bytes: None,
                        allocated: None,
                        mtime_ns: None,
                        clock: delta.clock.0,
                    });
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
    fn rebind_tags_for(&self, delivered: &[AppliedDelta]) -> Result<()> {
        let moved = self.index.with_index(|index| {
            let rules = index.tag_rules();
            !rules.is_empty()
                && delivered
                    .iter()
                    .flat_map(|delta| &delta.ops)
                    .any(|op| rules.is_control_file(op.path()))
        })?;
        if !moved {
            return Ok(());
        }
        // The commit it mints is picked up by the journal slice like any other, which is
        // why this returns nothing: there is one place a batch's contents come from.
        self.index.rebind_tag_rules()?;
        Ok(())
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
        AppliedDelta::of_ops(Clock(clock), ops)
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

    /// A truncated journal is a reset *and* discards the dirty set; an escalation is not.
    ///
    /// The distinction is the whole reason this is a named decision. Both are "your view
    /// may have gaps", but only one of them makes the enumerated dirty list wrong: an
    /// escalation names the subtree it re-scanned, while a truncated journal cannot name
    /// what it dropped, so a list of what survived reads as complete and is not.
    #[test]
    fn only_a_truncated_journal_discards_the_dirty_set() {
        assert_eq!(Recovery::of(false, false), Recovery { reset: false, all_dirty: false });
        assert_eq!(
            Recovery::of(true, false),
            Recovery { reset: true, all_dirty: false },
            "an escalation names the subtree it re-scanned, so the list still means something"
        );
        assert_eq!(
            Recovery::of(false, true),
            Recovery { reset: true, all_dirty: true },
            "a truncated journal cannot name what it dropped"
        );
        assert_eq!(Recovery::of(true, true), Recovery { reset: true, all_dirty: true });
    }

    /// A re-tag dirties the directories it governs, though no path event names them.
    ///
    /// The one way a cached row goes wrong with nothing to invalidate it: the entries did
    /// not move, the rules did. A consumer holding a listing for a governed subtree has no
    /// path change to react to, so if this set does not name the directory, the row stays
    /// and is wrong.
    #[test]
    fn a_retag_dirties_the_directories_it_governs() {
        let dirty = dirty_rollups(&[AppliedDelta::of_state(
            Clock(1),
            vec![StateChange::Retagged {
                directories: vec![PathBuf::from("src/deep")],
                all: false,
            }],
        )]);

        assert!(dirty.contains(&PathBuf::from("src/deep")), "the governed directory: {dirty:?}");
        assert!(dirty.contains(&PathBuf::from("src")), "and its ancestors: {dirty:?}");
        assert!(dirty.contains(&PathBuf::new()), "including the root: {dirty:?}");
    }

    /// Trust moving is not values moving, so it does not dirty a roll-up.
    ///
    /// It still reaches the consumer -- in `Batch::state`, which is where it belongs. If a
    /// verified sweep dirtied every ancestor it touched, every reconciliation would look
    /// like a mutation of the numbers and a consumer would discard a cache that was right.
    #[test]
    fn a_trust_transition_does_not_dirty_what_it_did_not_move() {
        let dirty = dirty_rollups(&[AppliedDelta::of_state(
            Clock(1),
            vec![StateChange::Verified { path: PathBuf::from("src") }, StateChange::RunFacts],
        )]);

        assert!(dirty.is_empty(), "no roll-up value moved: {dirty:?}");
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
