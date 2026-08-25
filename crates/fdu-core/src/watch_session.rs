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

use crate::engine_contract::{
    AppliedDelta, CommittedState, EntryKind, Op, QueryKind, Result, StateChange,
};
use crate::index::IndexHandle;
use crate::index::Work;
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
    /// The consumer's position is gone; it must re-read rather than apply this batch.
    ///
    /// Set when the consumer fell further behind than the journal retains, and only then.
    /// It is a fact about *its* history, not about the provider's: the engine losing
    /// precision and re-scanning leaves the consumer's position perfectly resumable, and
    /// says so through `issues` and the re-scan's own changes instead.
    ///
    /// A reset replaces every other signal in the batch. There is nothing to enumerate --
    /// the changes it missed are unnamed and unnameable -- so `changes`, `dirty_rollups`
    /// and `all_dirty` are all empty beside it. A partial list here would read as complete
    /// and be applied as one.
    pub reset: bool,
    /// Projections a consumer holding one may need to re-read, sorted.
    ///
    /// `dirty_rollups` says which *paths* moved; this says which *answers* those paths were
    /// part of. A consumer caching a recency list otherwise re-derives from change paths
    /// whether its own view is affected -- work the engine can do once instead of every
    /// consumer doing it differently, and differently wrong.
    ///
    /// Absence is the guarantee: a kind here may be stale, a kind missing is not. Empty
    /// beside `reset`, which replaces every other signal; unaffected by `all_dirty`, which
    /// drops the path list and leaves the question of *which projections* untouched.
    pub dirty_queries: Vec<QueryKind>,
    /// What producing this batch cost.
    ///
    /// `wall_ns` measures the work, not the wait: an idle tree with a one-minute interval
    /// would otherwise report a minute of cost for a batch that did nothing, and the one
    /// figure an embedder compares providers on would be measuring its own patience.
    ///
    /// `entries_visited` and `dirs_visited` are the filesystem the batch actually touched
    /// -- a coalesced event costs a stat, and an escalation costs the subtree it re-scanned,
    /// which is the difference a serving loop needs to see. `rows` is what the consumer
    /// receives after the selection, and `name_bytes` what those rows cost it to hold.
    pub work: Work,
    /// Typed conditions this batch observed.
    ///
    /// An observation gap is the one that matters today: the engine lost precision, re-
    /// scanned, and the index is right -- so the rows are complete and the position is
    /// resumable, which is exactly why this is not a reset. What a consumer learns here is
    /// that the stream between its last batch and this one had a hole the engine covered,
    /// which it may want to report even though it need do nothing about it.
    pub issues: Vec<crate::Issue>,
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

impl Batch {
    /// The batch a consumer whose position has expired receives.
    ///
    /// A constructor rather than a struct literal at the call site, because the shape *is*
    /// the contract: a reset replaces every other signal, and spelling that out where it is
    /// built keeps a later field from quietly acquiring a value beside it. A consumer
    /// reading "re-read everything, and also here are the changes" and acting on the second
    /// half applies a suffix to state it has just discarded.
    fn reset_at(cursor: crate::Cursor) -> Self {
        Self { dirty: true, reset: true, cursor: Some(cursor), ..Self::default() }
    }

    /// Whether this batch says only things a consumer can act on together.
    #[cfg(test)]
    fn is_consistent(&self) -> bool {
        let reset_is_alone = !self.reset
            || (self.changes.is_empty()
                && self.dirty_rollups.is_empty()
                && self.dirty_queries.is_empty()
                && !self.all_dirty
                && self.state.is_empty()
                && self.issues.is_empty());
        let all_dirty_replaces_the_list = !self.all_dirty || self.dirty_rollups.is_empty();
        reset_is_alone && all_dirty_replaces_the_list
    }
}

/// Projections a set of applied deltas may have made stale.
///
/// Conservative by construction, and the direction of that is the whole design: naming a
/// kind that turns out unaffected costs a consumer one re-read, while omitting one that is
/// affected leaves it serving an answer nothing will ever contradict.
///
/// Two asymmetries are worth stating, because both come from what an op does *not* say.
/// `Remove` does not say what it removed, so a removed directory has to be treated as
/// having taken files with it -- the same guess `dirty_rollups` makes, for the same reason.
/// And an `Upsert` does not distinguish a created file from a modified one, so a catalog of
/// identities is named for both even though only the first can move it.
fn dirty_queries(applied: &[AppliedDelta]) -> Vec<QueryKind> {
    let mut dirty: BTreeSet<QueryKind> = BTreeSet::new();
    for delta in applied {
        for op in &delta.ops {
            // Anything at a path moves the path-shaped projections and the tallies that
            // aggregate over them.
            dirty.extend([
                QueryKind::Entry,
                QueryKind::Directory,
                QueryKind::Rollup,
                QueryKind::FilteredTree,
                QueryKind::Navigation,
            ]);
            let touches_files = match op {
                Op::Upsert { kind, .. } => !kind.is_dir(),
                // See the asymmetries above: neither of these says what it covered.
                Op::Remove { .. } | Op::InvalidateSubtree { .. } => true,
            };
            if touches_files {
                dirty.extend([QueryKind::Recent, QueryKind::Catalog]);
            }
        }
        for change in &delta.state {
            // Every transition is something diagnostics reports.
            dirty.insert(QueryKind::Diagnostics);
            if matches!(change, StateChange::Retagged { .. }) {
                // Tags reach the rows themselves and the tallies partitioned by them, so a
                // rebind moves answers no path event named.
                dirty.extend([
                    QueryKind::Entry,
                    QueryKind::Directory,
                    QueryKind::FilteredTree,
                    QueryKind::Navigation,
                    QueryKind::Catalog,
                ]);
            }
        }
    }
    dirty.into_iter().collect()
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
        // From here on is assembly, which is work. What came before was mostly waiting,
        // and the watcher reports its own applying time apart from that.
        let assembling = std::time::Instant::now();
        let mut work = Work::default();
        if let Some(report) = &outcome {
            work.entries_visited = report.reconciliation.scan.entries;
            work.dirs_visited = report.reconciliation.scan.dirs_read;
            work.wall_ns = report.applied_ns;
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

        if since.truncated {
            // The consumer fell further behind than the journal retains. Its position is
            // unresumable, which is the one thing `reset` means -- and a reset replaces
            // every other signal, because the changes it missed are unnamed and unnameable.
            // A partial list beside it would read as complete and be applied as one.
            self.resume = since.cursor;
            let mut reset = Batch::reset_at(since.cursor);
            reset.work = work;
            reset.work.wall_ns = reset
                .work
                .wall_ns
                .saturating_add(u64::try_from(assembling.elapsed().as_nanos()).unwrap_or(u64::MAX));
            return Ok(Some(reset));
        }
        let applied = since.deltas;

        // A dropped event, an unpaired rename, or a watch registered after its directory
        // already had contents. The engine re-scans, so the index is right and the rows
        // below are complete -- which is exactly why this is *not* a reset: the consumer's
        // position is fine and replaying this batch is correct. What it did not have before
        // is any way to know its view had a hole in the meantime, and that is what this
        // says.
        let issues: Vec<crate::Issue> = applied
            .iter()
            .flat_map(|delta| &delta.ops)
            .filter_map(|op| match op {
                Op::InvalidateSubtree { path, reason }
                    if matches!(
                        reason,
                        crate::InvalidateReason::WatchOverflow
                            | crate::InvalidateReason::UnpairedRename
                            | crate::InvalidateReason::WatchSetupRace
                    ) =>
                {
                    Some(crate::Issue {
                        kind: crate::IssueKind::ObservationGap,
                        path: Some(path.clone()),
                        message: format!("watch precision lost at {}: {reason:?}", path.display()),
                        os_error: None,
                    })
                }
                _ => None,
            })
            .collect();
        // A rebind that stopped enumerating leaves no list to invalidate from, so the whole
        // cache is suspect -- the same conclusion the dirty bound reaches by its own route.
        let retagged_everything = applied.iter().any(|delta| {
            delta
                .state
                .iter()
                .any(|change| matches!(change, StateChange::Retagged { all: true, .. }))
        });
        let dirty_rollups = dirty_rollups(&applied);
        let all_dirty = retagged_everything || dirty_rollups.len() > MAX_DIRTY_ROLLUPS;
        let mut batch = Batch {
            changes: Vec::new(),
            dirty_queries: dirty_queries(&applied),
            work,
            issues,
            dirty: !applied.is_empty(),
            // Past the bound the list is dropped rather than truncated. A truncated list
            // is indistinguishable from a complete one at the consumer, which is how a
            // stale row survives an invalidation that named it.
            dirty_rollups: if all_dirty { Vec::new() } else { dirty_rollups },
            all_dirty,
            reset: false,
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
        batch.work.rows = batch.changes.len() as u64;
        batch.work.name_bytes =
            batch.changes.iter().map(|change| change.path.as_os_str().len() as u64).sum();
        batch.work.wall_ns = batch
            .work
            .wall_ns
            .saturating_add(u64::try_from(assembling.elapsed().as_nanos()).unwrap_or(u64::MAX));
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

    /// A reset says one thing and nothing else; `all_dirty` replaces the list it bounds.
    ///
    /// These are the consumer contract's own invariants, and they are not decoration. A
    /// batch carrying reset *and* a change list reads as "re-read everything, and also
    /// here are the changes", and a consumer acting on the second half applies a suffix to
    /// state it has just discarded. A truncated dirty list beside `all_dirty` is the same
    /// mistake one bound down.
    #[test]
    fn a_reset_batch_carries_nothing_a_consumer_could_apply() {
        let cursor = crate::Cursor { session: crate::SessionId(1), clock: Clock(9) };
        let reset = Batch::reset_at(cursor);

        assert!(reset.reset && reset.dirty, "it happened, and the position is unresumable");
        assert_eq!(reset.cursor, Some(cursor), "and it still says where the index now is");
        assert!(reset.is_consistent());

        // The two ways to get this wrong, spelled out so the checker itself has teeth.
        let with_changes = Batch {
            changes: vec![Change {
                path: PathBuf::from("a.txt"),
                kind: ChangeKind::Upsert,
                entry_kind: Some(EntryKind::File),
                bytes: Some(1),
                allocated: Some(1),
                mtime_ns: Some(0),
                clock: 9,
            }],
            ..Batch::reset_at(cursor)
        };
        assert!(!with_changes.is_consistent(), "a reset cannot also hand over a suffix");

        let truncated_list = Batch {
            all_dirty: true,
            dirty_rollups: vec![PathBuf::from("src")],
            ..Batch::default()
        };
        assert!(!truncated_list.is_consistent(), "a partial list beside all-dirty reads as whole");
    }

    /// A batch says which projections may be stale, and which certainly are not.
    ///
    /// The direction of the conservatism is the design: naming a kind that turns out
    /// unaffected costs one re-read, and omitting one that is affected leaves a consumer
    /// serving an answer nothing will contradict. So the useful assertions are about what
    /// is *absent* -- those are the guarantees a consumer can act on.
    #[test]
    fn a_batch_names_the_projections_its_changes_could_have_moved() {
        let one_file = dirty_queries(&[delta(
            1,
            vec![Op::Upsert {
                path: PathBuf::from("src/main.rs"),
                kind: EntryKind::File,
                attrs: Attrs::default(),
            }],
        )]);

        assert!(one_file.contains(&QueryKind::Recent), "a file moved, so recency did");
        assert!(one_file.contains(&QueryKind::Catalog));
        assert!(one_file.contains(&QueryKind::Rollup));
        assert!(
            !one_file.contains(&QueryKind::Diagnostics),
            "no state moved, so nothing diagnostics reports did: {one_file:?}"
        );
        assert!(
            !one_file.contains(&QueryKind::Metadata),
            "and identity facts are fixed for an opened index: {one_file:?}"
        );

        // A directory's own attributes do not enter a list of files or a catalog of
        // identities, and saying so is what makes the answer worth having.
        let one_directory = dirty_queries(&[delta(
            1,
            vec![Op::Upsert {
                path: PathBuf::from("src"),
                kind: EntryKind::Dir,
                attrs: Attrs::default(),
            }],
        )]);
        assert!(!one_directory.contains(&QueryKind::Recent), "{one_directory:?}");
        assert!(!one_directory.contains(&QueryKind::Catalog), "{one_directory:?}");
        assert!(one_directory.contains(&QueryKind::Directory), "{one_directory:?}");

        // A removal does not say what it removed, so it is treated as having taken files
        // with it -- the same guess the dirty set makes, and for the same reason.
        let removed = dirty_queries(&[delta(1, vec![Op::Remove { path: PathBuf::from("src") }])]);
        assert!(removed.contains(&QueryKind::Recent), "{removed:?}");
        assert!(removed.contains(&QueryKind::Catalog), "{removed:?}");

        // A transition with no operation moves only what reports transitions.
        let state_only =
            dirty_queries(&[AppliedDelta::of_state(Clock(1), vec![StateChange::RunFacts])]);
        assert_eq!(state_only, vec![QueryKind::Diagnostics], "{state_only:?}");

        // Except a re-tag, which moves rows nothing touched.
        let retagged = dirty_queries(&[AppliedDelta::of_state(
            Clock(1),
            vec![StateChange::Retagged { directories: vec![PathBuf::from("src")], all: false }],
        )]);
        assert!(retagged.contains(&QueryKind::Catalog), "tags reach the rows: {retagged:?}");
        assert!(retagged.contains(&QueryKind::Navigation), "and the tallies: {retagged:?}");
        assert!(!retagged.contains(&QueryKind::Recent), "but not mtime order: {retagged:?}");
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
