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

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::Duration;

use crate::engine_contract::{Commit, EffectiveChange, EntryKind, Error, Result};
use crate::index::IndexHandle;
use crate::query::{IgnoredEntries, Provenance, Query, Report, ReportSource, Selection, report};
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
    /// Whether `.gitignore` rules ignore this entry after the commit.
    ///
    /// `None` when the session observes no control state, so a record never claims a
    /// classification an index without rules cannot make. A report's rows carry the same
    /// split, and a stream that could not would be the one place a consumer had to guess.
    pub ignored: Option<bool>,
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
}

/// What one batch of commits needs from the index, read once under one lock.
struct BatchFacts {
    /// The touched entries ignore rules ignore once the batch applied, or `None` when the
    /// index observed no control state and so classifies nothing.
    ignored: Option<BTreeSet<PathBuf>>,
    /// The retained facts of each reclassified entry the selection could move, so a rule
    /// edit that admits one can stream the upsert that draws it.
    reclassified: BTreeMap<PathBuf, EntryFacts>,
}

impl BatchFacts {
    /// Whether rules ignore a touched entry, or `None` when nothing was classified.
    fn is_ignored(&self, path: &std::path::Path) -> Option<bool> {
        Some(self.ignored.as_ref()?.contains(path))
    }
}

/// One reclassified entry's retained facts.
#[derive(Clone, Copy)]
struct EntryFacts {
    kind: EntryKind,
    bytes: u64,
    allocated: u64,
    mtime_ns: i64,
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
    ///
    /// # Errors
    ///
    /// [`Error::ControlStateNotObserved`] when the query selects by ignored state and the
    /// index observed no control state, as [`Query::validate_controls`] refuses it.
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
        query
            .validate_controls(index.scope()?.observes_controls())
            .map_err(|_refused| Error::ControlStateNotObserved)?;
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
        report(&index, &self.query, provenance)
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
        let mut commits: Vec<Commit> = Vec::new();
        let outcome =
            self.watcher.apply_next(&self.index, &self.scan, timeout, &mut |commit: &Commit| {
                commits.push(commit.clone());
            })?;

        let Some(_report) = outcome else {
            return Ok(None);
        };

        let mut batch = Batch {
            changes: Vec::new(),
            dirty: commits.iter().any(|commit| !commit.changes.is_empty()),
        };
        let facts = self.batch_facts(&commits)?;
        for commit in &commits {
            for effective in &commit.changes {
                if let Some(change) = self.change_for(effective, commit.clock.0, &facts) {
                    batch.changes.push(change);
                }
            }
        }
        Ok(Some(batch))
    }

    /// What one batch needs from the index, read once under one lock rather than per
    /// change.
    ///
    /// Two things: the ignore classification of every entry the batch touched, which each
    /// record carries and an ignored-state selection filters on, and the retained facts of
    /// every reclassified entry, which is what lets a rule edit that moves an entry into
    /// the selection be streamed as the upsert a consumer needs to draw the row.
    fn batch_facts(&self, commits: &[Commit]) -> Result<BatchFacts> {
        let filters_by_ignored = self.selection().ignored != IgnoredEntries::Include;
        let mut touched: Vec<&PathBuf> = Vec::new();
        let mut reclassified: Vec<&PathBuf> = Vec::new();
        for effective in commits.iter().flat_map(|commit| &commit.changes) {
            match effective {
                EffectiveChange::Inserted { path, .. } | EffectiveChange::Updated { path, .. } => {
                    touched.push(path);
                }
                // A reclassified entry is read only when the selection can move it: under
                // `Include` both partitions are in the stream already, so nothing about
                // its row changed and the facts would be fetched for nobody.
                EffectiveChange::Reclassified { path, .. } if filters_by_ignored => {
                    reclassified.push(path);
                }
                EffectiveChange::Reclassified { .. }
                | EffectiveChange::Removed { .. }
                | EffectiveChange::Invalidated { .. }
                | EffectiveChange::ControlUpdated { .. }
                | EffectiveChange::ControlRefusalUpdated { .. } => {}
            }
        }

        self.index.read_with(|index| {
            let observed = index.observes_controls();
            let entries = reclassified
                .into_iter()
                .filter_map(|path| {
                    let id = index.lookup(path)?;
                    let attrs = index.attrs_of(id)?;
                    Some((
                        path.clone(),
                        EntryFacts {
                            kind: index.kind_of(id)?,
                            bytes: attrs.size,
                            allocated: attrs.allocated,
                            mtime_ns: attrs.mtime_ns,
                        },
                    ))
                })
                .collect();
            BatchFacts {
                ignored: observed.then(|| {
                    touched
                        .into_iter()
                        .filter(|path| matches!(index.is_ignored(path), Ok(Some(true))))
                        .cloned()
                        .collect()
                }),
                reclassified: entries,
            }
        })
    }

    /// Translate one exact effective change into the legacy change view.
    fn change_for(
        &self,
        effective: &EffectiveChange,
        clock: u64,
        facts: &BatchFacts,
    ) -> Option<Change> {
        match effective {
            EffectiveChange::Inserted { path, kind, attrs }
            | EffectiveChange::Updated { path, kind, current: attrs, .. } => {
                let name = path.file_name()?.to_string_lossy().into_owned();
                let candidate = crate::query::Candidate {
                    relative: path,
                    name: &name,
                    kind: *kind,
                    bytes: attrs.size,
                    allocated: attrs.allocated,
                    mtime_ns: attrs.mtime_ns,
                    ignored: facts.is_ignored(path).unwrap_or(false),
                };
                self.selection().admits(&candidate).then(|| Change {
                    path: path.clone(),
                    kind: ChangeKind::Upsert,
                    entry_kind: Some(*kind),
                    bytes: Some(attrs.size),
                    allocated: Some(attrs.allocated),
                    mtime_ns: Some(attrs.mtime_ns),
                    ignored: facts.is_ignored(path),
                    clock,
                })
            }
            // A removal carries no attributes to filter on, so only the path-shaped parts
            // of a selection can apply. Filtering it out entirely on a size, time, or
            // ignored-state bound would hide the disappearance of something the caller was
            // watching, and a removed entry has no classification left to read.
            EffectiveChange::Removed { path, .. } => {
                let name = path.file_name()?.to_string_lossy().into_owned();
                self.admits_by_path(path, &name).then(|| Change {
                    path: path.clone(),
                    kind: ChangeKind::Remove,
                    entry_kind: None,
                    bytes: None,
                    allocated: None,
                    mtime_ns: None,
                    ignored: None,
                    clock,
                })
            }
            // Escalations are never filtered: they say the consumer's view may have gaps,
            // and that is true regardless of what the selection asked for.
            EffectiveChange::Invalidated { path, .. } => Some(Change {
                path: path.clone(),
                kind: ChangeKind::Invalidate,
                entry_kind: None,
                bytes: None,
                allocated: None,
                mtime_ns: None,
                ignored: None,
                clock,
            }),
            // A rule edit changes what an ignored-state selection contains without
            // anything on disk changing for the entry, so the entry set the flag promises
            // is maintained here rather than left to the aggregates: a row that left is
            // removed and a row that arrived is upserted with the facts to draw it.
            EffectiveChange::Reclassified { path, previous_ignored, current_ignored } => {
                let name = path.file_name()?.to_string_lossy().into_owned();
                // Absent in two cases, each meaning there is nothing to emit: a selection
                // that admits both partitions, whose membership no edit can change, and an
                // entry that left the index after the commit, whose removal is already in
                // this batch.
                let entry = facts.reclassified.get(path)?;
                let admits = |ignored: bool| {
                    self.selection().admits(&crate::query::Candidate {
                        relative: path,
                        name: &name,
                        kind: entry.kind,
                        bytes: entry.bytes,
                        allocated: entry.allocated,
                        mtime_ns: entry.mtime_ns,
                        ignored,
                    })
                };
                match (admits(*previous_ignored), admits(*current_ignored)) {
                    (true, false) => Some(Change {
                        path: path.clone(),
                        kind: ChangeKind::Remove,
                        entry_kind: None,
                        bytes: None,
                        allocated: None,
                        mtime_ns: None,
                        ignored: Some(*current_ignored),
                        clock,
                    }),
                    (false, true) => Some(Change {
                        path: path.clone(),
                        kind: ChangeKind::Upsert,
                        entry_kind: Some(entry.kind),
                        bytes: Some(entry.bytes),
                        allocated: Some(entry.allocated),
                        mtime_ns: Some(entry.mtime_ns),
                        ignored: Some(*current_ignored),
                        clock,
                    }),
                    _ => None,
                }
            }
            // The legacy watch surface repaints the complete query when `dirty` is true,
            // so it needs no second row-change vocabulary for control-file effects.
            // Opened-root consumers read these exact commit variants directly.
            EffectiveChange::ControlUpdated { .. }
            | EffectiveChange::ControlRefusalUpdated { .. } => None,
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
