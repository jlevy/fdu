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
//! passes throttles aggregate repaints and persistence — it plays no part in detection.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::engine_contract::{Commit, EffectiveChange, EntryKind, Error, Result};
use crate::index::IndexHandle;
use crate::query::{Basis, Delivery, Query, Report, Request, Selection, WatchDelivery, report};
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
    /// Set on an upsert, and on a removal a rule edit caused, where the new classification
    /// is why the row left the selection. `None` on every other record: a session that
    /// observes no control state never claims a classification an index without rules can
    /// make, and an ordinary removal or an invalidation has no entry left to classify. A
    /// report's rows carry the same split, and a stream that could not would be the one
    /// place a consumer had to guess.
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
    /// How ignore rules classify each touched entry the index still holds once the batch
    /// applied, or `None` when the index observed no control state and classifies nothing.
    ///
    /// A map rather than a set of the ignored: an entry a later commit in the same batch
    /// removed is in neither partition, and a set could not tell that from unignored.
    ignored: Option<BTreeMap<PathBuf, bool>>,
    /// The retained facts of each reclassified entry the selection could move, so a rule
    /// edit that admits one can stream the upsert that draws it.
    reclassified: BTreeMap<PathBuf, EntryFacts>,
}

impl BatchFacts {
    /// How rules classify a touched entry: `None` when nothing was classified, and when
    /// the entry is gone from the index, which leaves no entry to classify.
    fn is_ignored(&self, path: &std::path::Path) -> Option<bool> {
        self.ignored.as_ref()?.get(path).copied()
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

/// The result of a throttled attempt to persist a live session.
#[derive(Debug)]
pub enum SaveOutcome {
    /// The metadata snapshot reached disk.
    Written,
    /// No write was due, or the plan could not yet persist the current state.
    Skipped,
    /// Persistence failed; the live session remains usable and will retry.
    Failed(Error),
}

struct Persistence {
    pending: bool,
    last_attempt: Instant,
}

impl Persistence {
    fn persist_due(
        &mut self,
        now: Instant,
        interval: Duration,
        save: impl FnOnce() -> Result<bool>,
    ) -> SaveOutcome {
        if !save_is_due(self.pending, now.saturating_duration_since(self.last_attempt), interval) {
            return SaveOutcome::Skipped;
        }
        let outcome = match save() {
            Ok(true) => SaveOutcome::Written,
            Ok(false) => SaveOutcome::Skipped,
            Err(error) => SaveOutcome::Failed(error),
        };
        self.pending = pending_after(&outcome);
        // Skips and failures are throttled too, while retaining the pending work.
        self.last_attempt = now;
        outcome
    }
}

fn save_is_due(pending: bool, since_last_save: Duration, interval: Duration) -> bool {
    pending && since_last_save >= interval
}

fn pending_after(outcome: &SaveOutcome) -> bool {
    !matches!(outcome, SaveOutcome::Written)
}

/// An index paired with a watcher, answering one request continuously.
pub struct Session {
    index: IndexHandle,
    watcher: Watcher,
    scan: ScanConfig,
    request: Request,
    plan: crate::Plan,
    persistence: Persistence,
    startup_save_error: Option<Error>,
}

impl Session {
    /// Open a tree and bind observation under the shared execution plan.
    ///
    /// Startup persistence is joined before binding the session. A save failure is
    /// returned by the first `persist_due` call, so it does not discard a valid live
    /// answer. Filesystem and observation failures still fail startup.
    pub fn start(request: Request, delivery: Delivery) -> Result<Self> {
        Self::start_observed(request, delivery, None)
    }

    /// [`Self::start`], reporting the initial scan through `progress` as it runs.
    ///
    /// The same session as [`Self::start`]; the handle observes the start and changes
    /// nothing about it. A start is two passes: the open (cold, or a load and
    /// revalidation) with its save, if it writes one, joined under
    /// [`ProgressPhase::Saving`](crate::ProgressPhase), and then the revalidation that
    /// closes the gap between that walk and the bound watcher. The second pass begins
    /// again at [`ProgressPhase::Revalidating`](crate::ProgressPhase) with the walk
    /// counters restarted, so they end at the tree's totals once, not twice. Once this
    /// returns the session reports nothing further through the handle; its repaints are
    /// the progress from there.
    pub fn start_with_progress(
        request: Request,
        delivery: Delivery,
        progress: &crate::Progress,
    ) -> Result<Self> {
        Self::start_observed(request, delivery, Some(progress))
    }

    fn start_observed(
        request: Request,
        mut delivery: Delivery,
        progress: Option<&crate::Progress>,
    ) -> Result<Self> {
        delivery.watch.get_or_insert_with(WatchDelivery::default);
        let plan =
            crate::plan(&request, &delivery, crate::Route::Watch).map_err(Error::InvalidRequest)?;
        let (index, report, pending, _diagnostics) =
            crate::execute(&plan, &request.basis, false, progress)?;
        let startup_save_error = pending.join().err();
        let index = std::sync::Arc::into_inner(index)
            .expect("the joined writer released the only other reference");
        let mut session = Self::new_observed(
            IndexHandle::new(index),
            request,
            &delivery,
            WatchConfig::default(),
            progress,
        )?;
        session.persistence.pending |= startup_save_error.is_some() || !report.is_complete();
        session.startup_save_error = startup_save_error;
        Ok(session)
    }

    /// Persist pending changes when the watch delivery's interval has elapsed.
    ///
    /// Call this after batches and idle timeouts. Only a completed write clears pending
    /// changes; a refused or failed write is retried on a later interval. `now` is a
    /// monotonic caller-supplied clock so wall-clock corrections cannot postpone saves.
    pub fn persist_due(&mut self, now: Instant) -> SaveOutcome {
        if let Some(error) = self.startup_save_error.take() {
            self.persistence.last_attempt = now;
            return SaveOutcome::Failed(error);
        }
        let interval = self.plan.delivery().watch.expect("watch plan").interval;
        self.persistence.persist_due(now, interval, || {
            if !self.plan.delivery().cache.writes() || self.plan.delivery().cache_path.is_none() {
                return Ok(false);
            }
            let index = self.index.snapshot()?;
            crate::persist_index(&index, &self.plan)
        })
    }

    /// Start watching an already-opened index, answering `request` as the tree changes.
    ///
    /// `request` carries its own `now`, fixed when it was built: a watch answers one
    /// request as the tree changes, and a relative time window that slid under it would
    /// make two repaints answer two different questions.
    ///
    /// `delivery` is the one its caller opened the index under. It is taken rather than
    /// composed here because the cache policy is part of it: a session built against a
    /// fabricated `Delivery` read `cache: Auto` whatever the caller had asked for, so
    /// [`RequestError::WatchCacheOnly`](crate::query::RequestError::WatchCacheOnly) could
    /// not fire inside the engine at all and the rule held only at the two front doors
    /// (fdu-i18y). Starting a session is what a watch *is*, so the delivery is read as a
    /// watch whether or not the caller remembered to say so.
    ///
    /// Refusals are in the order every route publishes: what no delivery can carry first,
    /// then what this index was taken under, then what it holds. The request-level rule
    /// speaks first, so a library caller watching a depth-2 request is told the watch
    /// cannot narrow its scope rather than that the index has another scope.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidRequest`] when a watch cannot deliver the request -- a narrowed scan
    /// scope, content analysis nothing re-reads, a snapshot nothing verified -- or when
    /// this index cannot answer it, including a selection by ignored state over an index
    /// that observed no control state.
    /// [`Error::ScanScopeMismatch`] when the index was not taken under the request's scope.
    pub fn new(
        index: IndexHandle,
        request: Request,
        delivery: &Delivery,
        watch: WatchConfig,
    ) -> Result<Self> {
        Self::new_observed(index, request, delivery, watch, None)
    }

    fn new_observed(
        index: IndexHandle,
        request: Request,
        delivery: &Delivery,
        watch: WatchConfig,
        progress: Option<&crate::Progress>,
    ) -> Result<Self> {
        let root = index.root_path()?;
        let scan = request.basis.scope.scan_config(delivery);
        // What no delivery can carry, before anything stored is read and before the
        // backend is bound: this is the rule each surface used to keep for itself, so a
        // library caller could watch what `--watch` has always refused.
        let delivery =
            Delivery { watch: Some(delivery.watch.unwrap_or_default()), ..delivery.clone() };
        crate::plan(&request, &delivery, crate::Route::Watch).map_err(Error::InvalidRequest)?;
        crate::validate_basis_root(&root, &request.basis)?;
        // Reject an out-of-scope watch before the backend is bound, so a rejected run
        // never leaves a watcher registered on the tree.
        scan.validate_for_scope(index.scope()?)?;
        // The scope check above proved the index was taken under exactly this scan's
        // identity, control tier included, so what remains is what this index holds.
        let held = Basis {
            root: root.clone(),
            scope: scan.clone().into(),
            content: index.read_with(crate::Index::content_set)?,
        };
        request.validate_read(&held).map_err(Error::InvalidRequest)?;
        // Bind observation before closing the gap from the scan that produced `index`.
        // The full reconciliation catches a mutation that completed before registration;
        // the capture drain applies every hint observed while that pass ran.
        let watcher = Watcher::new(&root, watch)?;
        Self::finish_initial_handoff(index, request, &delivery, watcher, scan, progress)
    }

    /// Finish the two-part initial handoff after observation has been bound.
    ///
    /// Kept separate so the scripted watcher exercises the same reconciliation, drain, and
    /// acceptance boundary as an OS watcher. Once this returns, later partial observations are
    /// valid live state; `accept_partial` governs only the coherent state handed to the caller.
    ///
    /// `progress` observes the handoff's own walk and drain only. The session keeps
    /// `scan` without it, so nothing it reconciles later reports through a handle whose
    /// poller has long since stopped.
    fn finish_initial_handoff(
        index: IndexHandle,
        request: Request,
        delivery: &Delivery,
        watcher: Watcher,
        scan: ScanConfig,
        progress: Option<&crate::Progress>,
    ) -> Result<Self> {
        if let Some(progress) = progress {
            progress.begin_pass(crate::ProgressPhase::Revalidating);
        }
        let observed = ScanConfig { progress: progress.cloned(), ..scan.clone() };
        let mut dirty = false;
        let reconciliation = crate::scan::reconcile_handle(&index, &observed, &mut |commit| {
            dirty |= !commit.changes.is_empty();
        })?;
        if !reconciliation.scan.is_complete() && !delivery.accept_partial {
            return Err(Error::ObservationHandoffIncomplete);
        }
        dirty |= drain_initial_capture(&watcher, &index, &observed)?;
        if !delivery.accept_partial
            && !index.read_with(|index| crate::query::TreeStatus::of(index, &request).complete)?
        {
            return Err(Error::ObservationHandoffIncomplete);
        }
        let plan =
            crate::plan(&request, delivery, crate::Route::Watch).map_err(Error::InvalidRequest)?;
        Ok(Self {
            index,
            watcher,
            scan,
            request,
            plan,
            persistence: Persistence { pending: dirty, last_attempt: Instant::now() },
            startup_save_error: None,
        })
    }

    /// The request this session answers.
    pub fn request(&self) -> &Request {
        &self.request
    }

    /// The query this session answers.
    pub fn query(&self) -> &Query {
        &self.request.query
    }

    /// Render the current answer.
    ///
    /// The same `report` a one-shot run produces, from the same index, which is what
    /// makes "watch is the same query repeated" true rather than aspirational.
    pub fn report(&self, generated_at: std::time::SystemTime) -> Result<Report> {
        let index = self.index.snapshot()?;
        report(&index, &self.request, generated_at)
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
            });

        self.persistence.pending |= commits.iter().any(|commit| !commit.changes.is_empty());
        let Some(_report) = outcome? else {
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
        let mut touched: Vec<&PathBuf> = Vec::new();
        let mut reclassified: Vec<&PathBuf> = Vec::new();
        for effective in commits.iter().flat_map(|commit| &commit.changes) {
            match effective {
                EffectiveChange::Inserted { path, .. } | EffectiveChange::Updated { path, .. } => {
                    touched.push(path);
                }
                EffectiveChange::Reclassified { path, .. } => {
                    reclassified.push(path);
                }
                EffectiveChange::Removed { .. }
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
                        .filter_map(|path| match index.is_ignored(path) {
                            Ok(Some(ignored)) => Some((path.clone(), ignored)),
                            // Gone from the index, or the index reads no rules; either
                            // way there is nothing to say about it.
                            Ok(None) | Err(_) => None,
                        })
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
            EffectiveChange::Inserted { path, kind, attrs } => {
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
            EffectiveChange::Updated { path, kind, previous: _, current } => {
                let name = path.file_name()?.to_string_lossy().into_owned();
                let ignored = facts.is_ignored(path).unwrap_or(false);
                let candidate = crate::query::Candidate {
                    relative: path,
                    name: &name,
                    kind: *kind,
                    bytes: current.size,
                    allocated: current.allocated,
                    mtime_ns: current.mtime_ns,
                    ignored,
                };
                if self.selection().admits(&candidate) {
                    Some(Change {
                        path: path.clone(),
                        kind: ChangeKind::Upsert,
                        entry_kind: Some(*kind),
                        bytes: Some(current.size),
                        allocated: Some(current.allocated),
                        mtime_ns: Some(current.mtime_ns),
                        ignored: facts.is_ignored(path),
                        clock,
                    })
                } else if self.admits_by_path(path, &name) {
                    Some(Change {
                        path: path.clone(),
                        kind: ChangeKind::Remove,
                        entry_kind: None,
                        bytes: None,
                        allocated: None,
                        mtime_ns: None,
                        ignored: None,
                        clock,
                    })
                } else {
                    None
                }
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
                    (_, true) => Some(Change {
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
        &self.request.query.selection
    }
}

fn drain_initial_capture(
    watcher: &Watcher,
    index: &IndexHandle,
    scan: &ScanConfig,
) -> Result<bool> {
    let mut dirty = false;
    for _ in 0..2 {
        watcher.flush_capture()?;
        let mut drained = false;
        for _ in 0..=watcher.capture_backlog_bound() {
            if watcher
                .apply_next(index, scan, Duration::ZERO, &mut |commit| {
                    dirty |= !commit.changes.is_empty();
                })?
                .is_none()
            {
                drained = true;
                break;
            }
        }
        if !drained {
            return Err(Error::ObservationHandoffIncomplete);
        }
    }
    Ok(dirty)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The watch loop's save throttle, as a table over every state that reaches it.
    ///
    /// Two of the three defects review found on this branch were transitions in here, and
    /// the second was introduced by fixing the first. End-to-end tests could not catch
    /// either: they observe whether a file changed on disk, which cannot distinguish "not
    /// due yet" from "due and skipped", nor a cleared flag from a retained one.
    #[test]
    fn a_retained_session_rejects_a_request_for_another_root_before_binding() {
        let a = tempfile::tempdir().expect("root a");
        let b = tempfile::tempdir().expect("root b");
        let basis = Basis {
            root: a.path().into(),
            scope: crate::query::Scope::default(),
            content: crate::content::AnalysisSet::NONE,
        };
        let delivery = Delivery::new(crate::CachePolicy::Off, None);
        let (index, _) = crate::open(&basis, &delivery).expect("open a");
        let handle = IndexHandle::new(index);
        let before = handle.clock().expect("clock");
        let request = Request::new(
            Basis { root: b.path().into(), ..basis },
            Query::default(),
            std::time::SystemTime::now(),
        );
        assert!(matches!(
            Session::new(handle.clone(), request, &delivery, WatchConfig::default()),
            Err(Error::InvalidRequest(crate::query::RequestError::RootMismatch { .. }))
        ));
        assert_eq!(handle.clock().expect("clock"), before);
    }

    #[test]
    fn a_save_is_due_only_when_a_change_is_pending_and_the_throttle_has_elapsed() {
        let interval = Duration::from_secs(1);
        let cases = [
            // (pending, since last save, due, what this case is)
            (true, Duration::from_secs(2), true, "pending and past the interval"),
            (true, interval, true, "pending, exactly at the interval: inclusive"),
            // The R5 case. Not due *now* -- and the flag stays set, which is the half that
            // was missing: the idle path saves it once the interval passes.
            (true, Duration::from_millis(1), false, "pending but throttled"),
            (false, Duration::from_secs(60), false, "nothing pending, however long it has been"),
            (false, Duration::ZERO, false, "nothing pending and just saved"),
        ];

        for (pending, since, want, case) in cases {
            assert_eq!(save_is_due(pending, since, interval), want, "{case}");
        }
    }

    /// A throttled change must survive every outcome except a completed write.
    #[test]
    fn only_a_completed_write_clears_the_pending_change() {
        // The R7 case is Skipped and Failed: clearing the flag for either means the idle
        // path never retries, so on a quiet tree the change is never persisted at all.
        assert!(!pending_after(&SaveOutcome::Written), "a completed write persists the change");
        assert!(
            pending_after(&SaveOutcome::Skipped),
            "a skipped save wrote nothing, so the change is still owed to disk",
        );
        assert!(
            pending_after(&SaveOutcome::Failed(Error::Snapshot("failed".into()))),
            "a failed save must be retried, not forgotten"
        );
    }

    /// The sequence that defeated persistence in its most common shape.
    #[test]
    fn a_burst_then_a_quiet_tree_still_persists() {
        let interval = Duration::from_secs(1);

        // A change arrives too soon after the last save, so nothing is written yet.
        let mut pending = true;
        assert!(!save_is_due(pending, Duration::from_millis(50), interval));
        assert!(pending, "the throttle must not consume the change");

        // The tree goes quiet: no further batches will ever arrive. The idle path is the
        // only remaining caller, and once the interval passes the save must happen.
        assert!(save_is_due(pending, Duration::from_secs(3), interval));

        // A skip at that point keeps it pending for the next idle tick rather than
        // silently dropping the session's work.
        pending = pending_after(&SaveOutcome::Skipped);
        assert!(pending);
        pending = pending_after(&SaveOutcome::Written);
        assert!(!pending, "once written, the loop stops rewriting an unchanged index");
    }

    #[test]
    fn skips_and_failures_retry_only_after_another_interval() {
        let start = Instant::now();
        let interval = Duration::from_secs(2);
        let mut persistence = Persistence { pending: true, last_attempt: start };
        assert!(matches!(
            persistence.persist_due(start + interval / 2, interval, || panic!("throttled")),
            SaveOutcome::Skipped
        ));
        assert!(matches!(
            persistence.persist_due(start + interval, interval, || Ok(false)),
            SaveOutcome::Skipped
        ));
        assert!(persistence.pending);
        assert!(matches!(
            persistence.persist_due(start + interval, interval, || panic!("skip was throttled")),
            SaveOutcome::Skipped
        ));
        assert!(matches!(
            persistence.persist_due(start + interval * 2, interval, || {
                Err(Error::Snapshot("disk unavailable".into()))
            }),
            SaveOutcome::Failed(_)
        ));
        assert!(persistence.pending);
        assert!(matches!(
            persistence
                .persist_due(start + interval * 2, interval, || panic!("failure was throttled")),
            SaveOutcome::Skipped
        ));
        assert!(matches!(
            persistence.persist_due(start + interval * 3, interval, || Ok(true)),
            SaveOutcome::Written
        ));
        assert!(!persistence.pending);
        assert!(matches!(
            persistence.persist_due(start + interval * 4, interval, || panic!("already persisted")),
            SaveOutcome::Skipped
        ));
    }

    #[test]
    fn handoff_changes_are_persisted_after_the_tree_goes_quiet() {
        let root = tempfile::tempdir().expect("root");
        let cache = tempfile::tempdir().expect("cache");
        let cache_path = cache.path().join("snapshot");
        let scan = ScanConfig::default();
        std::fs::write(root.path().join("before.txt"), b"before").expect("before");
        let (index, _) = crate::scan::scan_into_index(root.path(), &scan).expect("scan");
        let request = Request::new(
            Basis {
                root: root.path().to_path_buf(),
                scope: scan.clone().into(),
                content: crate::content::AnalysisSet::NONE,
            },
            Query::default(),
            std::time::SystemTime::now(),
        );
        let interval = Duration::from_secs(2);
        let delivery = Delivery {
            cache: crate::CachePolicy::Auto,
            cache_path: Some(cache_path.clone()),
            accept_partial: false,
            watch: Some(WatchDelivery { interval }),
            workers: crate::query::Workers::default(),
            batch_size: ScanConfig::default().batch_size,
            order: crate::scan::ScanOrder::default(),
        };
        let script = tempfile::NamedTempFile::new().expect("script");
        let (watcher, _sender) =
            Watcher::scripted(root.path(), WatchConfig::default(), script.path()).expect("watcher");
        std::fs::write(root.path().join("during-handoff.txt"), b"handoff").expect("handoff change");
        let mut session = Session::finish_initial_handoff(
            IndexHandle::new(index),
            request,
            &delivery,
            watcher,
            scan,
            None,
        )
        .expect("handoff");
        let started = session.persistence.last_attempt;
        assert!(session.persistence.pending, "handoff changes need persistence too");
        assert!(matches!(session.persist_due(started), SaveOutcome::Skipped));
        assert!(!cache_path.exists(), "throttle delays the write");
        assert!(matches!(session.persist_due(started + interval), SaveOutcome::Written));
        let restored = crate::snapshot::load(&cache_path).expect("load").expect("saved");
        assert!(matches!(
            restored.path_state(std::path::Path::new("during-handoff.txt")),
            crate::PathState::Present { .. }
        ));
        assert!(matches!(session.persist_due(started + interval * 2), SaveOutcome::Skipped));
    }

    #[test]
    fn startup_save_failure_keeps_the_session_live_and_retries() {
        let root = tempfile::tempdir().expect("root");
        let cache = tempfile::tempdir().expect("cache");
        let cache_path = cache.path().join("blocked-snapshot");
        std::fs::create_dir(&cache_path).expect("directory blocks snapshot rename");
        std::fs::write(root.path().join("file.txt"), b"content").expect("file");
        let interval = Duration::from_secs(2);
        let request = Request::new(
            Basis {
                root: root.path().to_path_buf(),
                scope: ScanConfig::default().into(),
                content: crate::content::AnalysisSet::NONE,
            },
            Query::default(),
            std::time::SystemTime::now(),
        );
        let delivery = Delivery {
            cache: crate::CachePolicy::Refresh,
            cache_path: Some(cache_path.clone()),
            accept_partial: false,
            watch: Some(WatchDelivery { interval }),
            workers: crate::query::Workers::default(),
            batch_size: ScanConfig::default().batch_size,
            order: crate::scan::ScanOrder::default(),
        };
        let mut session = Session::start(request, delivery).expect("save failure is nonfatal");
        assert!(session.report(std::time::SystemTime::now()).expect("live report").status.complete);
        let now = Instant::now();
        assert!(matches!(session.persist_due(now), SaveOutcome::Failed(_)));
        std::fs::remove_dir(&cache_path).expect("restore writable destination");
        assert!(matches!(session.persist_due(now), SaveOutcome::Skipped));
        assert!(matches!(session.persist_due(now + interval), SaveOutcome::Written));
        assert!(crate::snapshot::load(&cache_path).expect("read snapshot").is_some());
    }

    #[test]
    fn an_update_that_leaves_attribute_selection_emits_remove() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join("file.txt"), b"12345678").expect("fixture");
        let scan = ScanConfig::default();
        let (index, report) = crate::scan::scan_into_index(root.path(), &scan).expect("scan");
        assert!(report.is_complete());
        let request = Request::new(
            Basis {
                root: root.path().to_path_buf(),
                scope: scan.clone().into(),
                content: crate::content::AnalysisSet::NONE,
            },
            Query {
                selection: Selection { min_size: Some(4), ..Selection::default() },
                ..Query::default()
            },
            std::time::SystemTime::now(),
        );
        let delivery = Delivery {
            cache: crate::CachePolicy::Off,
            cache_path: None,
            accept_partial: false,
            watch: Some(WatchDelivery { interval: Duration::from_millis(50) }),
            workers: crate::query::Workers::default(),
            batch_size: ScanConfig::default().batch_size,
            order: crate::scan::ScanOrder::default(),
        };
        let session =
            Session::new(IndexHandle::new(index), request, &delivery, WatchConfig::default())
                .expect("session");
        let path = PathBuf::from("file.txt");
        let change = session
            .change_for(
                &EffectiveChange::Updated {
                    path: path.clone(),
                    kind: EntryKind::File,
                    previous: crate::Attrs { size: 8, allocated: 8, ..crate::Attrs::default() },
                    current: crate::Attrs { size: 1, allocated: 1, ..crate::Attrs::default() },
                },
                1,
                &BatchFacts {
                    ignored: Some(BTreeMap::from([(path, false)])),
                    reclassified: BTreeMap::new(),
                },
            )
            .expect("membership transition");
        assert_eq!(change.kind, ChangeKind::Remove);
    }

    #[test]
    fn initial_handoff_drains_a_sticky_overflow_after_a_full_intent_queue() {
        let root = tempfile::tempdir().expect("root");
        let script = tempfile::NamedTempFile::new().expect("script");
        let scan = ScanConfig::default();
        let (index, report) = crate::scan::scan_into_index(root.path(), &scan).expect("scan");
        assert!(report.is_complete());
        let handle = IndexHandle::new(index);
        let config = WatchConfig {
            settle: Duration::from_millis(1),
            max_hold: Duration::from_millis(2),
            event_capacity: 8,
            batch_path_capacity: 1,
            intent_capacity: 1,
            ..WatchConfig::default()
        };
        let (watcher, sender) =
            Watcher::scripted(root.path(), config, script.path()).expect("scripted watcher");
        std::fs::write(root.path().join("a.txt"), b"a").expect("a");
        sender.send("create\ta.txt\n").expect("first event");
        watcher.flush_capture().expect("first barrier fills the intent queue");
        std::fs::write(root.path().join("b.txt"), b"b").expect("b");
        sender.send("create\tb.txt\n").expect("second event");
        watcher.flush_capture().expect("second barrier retains sticky overflow");

        drain_initial_capture(&watcher, &handle, &scan).expect("bounded handoff");

        assert!(
            handle.snapshot().expect("snapshot").lookup(std::path::Path::new("a.txt")).is_some()
        );
        assert!(
            handle.snapshot().expect("snapshot").lookup(std::path::Path::new("b.txt")).is_some()
        );
        assert!(
            watcher
                .apply_next(&handle, &scan, Duration::ZERO, &mut |_| {})
                .expect("proof poll")
                .is_none(),
            "no queued or sticky pre-handoff work remains"
        );
    }

    #[test]
    fn initial_handoff_enforces_partial_acceptance_after_its_reconciliation() {
        let root = tempfile::tempdir().expect("root");
        std::fs::write(root.path().join("kept.txt"), b"kept").expect("fixture");
        let scan = ScanConfig::default();
        let (index, report) = crate::scan::scan_into_index(root.path(), &scan).expect("scan");
        assert!(report.is_complete());
        let request = Request::new(
            Basis {
                root: root.path().to_path_buf(),
                scope: scan.clone().into(),
                content: crate::content::AnalysisSet::NONE,
            },
            Query::default(),
            std::time::SystemTime::now(),
        );
        let _fault = crate::scan::install_walk_hook(root.path(), |_| {
            Some(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "deterministic handoff refusal",
            ))
        });
        let delivery = Delivery {
            cache: crate::CachePolicy::Off,
            cache_path: None,
            accept_partial: false,
            watch: Some(WatchDelivery { interval: Duration::from_millis(50) }),
            workers: crate::query::Workers::default(),
            batch_size: ScanConfig::default().batch_size,
            order: crate::scan::ScanOrder::default(),
        };

        let Err(error) = Session::new(
            IndexHandle::new(index.clone()),
            request.clone(),
            &delivery,
            WatchConfig::default(),
        ) else {
            panic!("a partial handoff is refused");
        };
        assert!(matches!(error, Error::ObservationHandoffIncomplete));

        let accepted = Session::new(
            IndexHandle::new(index),
            request,
            &Delivery { accept_partial: true, ..delivery },
            WatchConfig::default(),
        )
        .expect("the caller explicitly accepts a partial handoff");
        assert!(
            !accepted.report(std::time::SystemTime::now()).expect("partial report").status.complete
        );
    }

    #[test]
    fn initial_handoff_rechecks_partial_acceptance_after_draining_capture() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let root = tempfile::tempdir().expect("root");
        std::fs::write(root.path().join("kept.txt"), b"kept").expect("fixture");
        let scan = ScanConfig::default();
        let (index, report) = crate::scan::scan_into_index(root.path(), &scan).expect("scan");
        assert!(report.is_complete());
        let request = Request::new(
            Basis {
                root: root.path().to_path_buf(),
                scope: scan.clone().into(),
                content: crate::content::AnalysisSet::NONE,
            },
            Query::default(),
            std::time::SystemTime::now(),
        );
        let delivery = Delivery {
            cache: crate::CachePolicy::Off,
            cache_path: None,
            accept_partial: false,
            watch: Some(WatchDelivery { interval: Duration::from_millis(50) }),
            workers: crate::query::Workers::default(),
            batch_size: ScanConfig::default().batch_size,
            order: crate::scan::ScanOrder::default(),
        };
        let script = tempfile::NamedTempFile::new().expect("script");
        let (watcher, sender) =
            Watcher::scripted(root.path(), WatchConfig::default(), script.path()).expect("watcher");
        sender.send("rescan\t.\n").expect("queue initial gap");

        // The startup reconciliation succeeds. The scripted overflow then reaches the same
        // tree during the handoff drain, where its reconciliation fails and must be refused.
        let attempts = Arc::new(AtomicUsize::new(0));
        let hook_attempts = Arc::clone(&attempts);
        let _fault = crate::scan::install_walk_hook(root.path(), move |_| {
            (hook_attempts.fetch_add(1, Ordering::SeqCst) > 0).then(|| {
                std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "deterministic drain-only refusal",
                )
            })
        });

        let Err(error) = Session::finish_initial_handoff(
            IndexHandle::new(index),
            request,
            &delivery,
            watcher,
            scan,
            None,
        ) else {
            panic!("a partial state created while draining is refused");
        };
        assert!(matches!(error, Error::ObservationHandoffIncomplete));
        assert!(attempts.load(Ordering::SeqCst) > 1, "the drain ran after startup reconciliation");
    }

    /// The handoff pass restarts the walk counters and enters `Revalidating`, whatever
    /// the first pass left in the handle: with a scripted watcher that reports nothing,
    /// the counts afterwards are exactly one walk of the tree.
    #[test]
    fn the_handoff_pass_restarts_the_counts() {
        let root = tempfile::tempdir().expect("root");
        let mut bytes = 0;
        for directory in 0..2 {
            let dir = root.path().join(format!("d{directory}"));
            std::fs::create_dir(&dir).expect("directory");
            for file in 0..3 {
                let size = directory * 3 + file + 1;
                std::fs::write(dir.join(format!("f{file}.txt")), vec![b'.'; size]).expect("file");
                bytes += size as u64;
            }
        }
        let scan = ScanConfig::default();
        let (index, report) = crate::scan::scan_into_index(root.path(), &scan).expect("scan");
        assert!(report.is_complete());
        let request = Request::new(
            Basis {
                root: root.path().to_path_buf(),
                scope: scan.clone().into(),
                content: crate::content::AnalysisSet::NONE,
            },
            Query::default(),
            std::time::SystemTime::now(),
        );
        let delivery = Delivery {
            cache: crate::CachePolicy::Off,
            cache_path: None,
            accept_partial: false,
            watch: Some(WatchDelivery { interval: Duration::from_millis(50) }),
            workers: crate::query::Workers::default(),
            batch_size: ScanConfig::default().batch_size,
            order: crate::scan::ScanOrder::default(),
        };
        let script = tempfile::NamedTempFile::new().expect("script");
        let (watcher, _sender) =
            Watcher::scripted(root.path(), WatchConfig::default(), script.path()).expect("watcher");
        let progress = crate::Progress::new();
        progress.add_walked(100, 100, 100);
        progress.enter(crate::ProgressPhase::Saving);

        Session::finish_initial_handoff(
            IndexHandle::new(index),
            request,
            &delivery,
            watcher,
            scan,
            Some(&progress),
        )
        .expect("handoff");

        let snapshot = progress.snapshot();
        assert_eq!(snapshot.phase, crate::ProgressPhase::Revalidating);
        assert_eq!(
            (snapshot.directories, snapshot.files, snapshot.bytes),
            (3, 6, bytes),
            "one walk of the root and its two directories, the first pass not added in"
        );
    }

    /// A record says what the index can be asked, and nothing more.
    ///
    /// The three answers are distinct and a consumer acts on each differently: a bit, "no
    /// rules were read", and "there is no such entry". A set of the ignored paths collapsed
    /// the last two into `false`, so a batch that created and removed one file in the same
    /// window upserted it as unignored before removing it -- a classification claim about
    /// an entry that never survived the batch.
    #[test]
    fn a_record_claims_a_classification_only_for_an_entry_the_index_still_holds() {
        let unobserved = BatchFacts { ignored: None, reclassified: BTreeMap::new() };
        assert_eq!(unobserved.is_ignored(std::path::Path::new("any.txt")), None);

        let observed = BatchFacts {
            ignored: Some(BTreeMap::from([
                (PathBuf::from("build/out.bin"), true),
                (PathBuf::from("src/main.rs"), false),
            ])),
            reclassified: BTreeMap::new(),
        };
        assert_eq!(observed.is_ignored(std::path::Path::new("build/out.bin")), Some(true));
        assert_eq!(observed.is_ignored(std::path::Path::new("src/main.rs")), Some(false));
        assert_eq!(
            observed.is_ignored(std::path::Path::new("gone.tmp")),
            None,
            "an entry the batch removed is in neither partition, not in the unignored one"
        );
    }

    /// A start is the open and then the handoff revalidation, a second pass whose
    /// counts restart, which keeps the line moving on a large tree after the save,
    /// where a frozen count would look like a hang. The session it returns is the one
    /// [`Session::start`] returns, and it reports nothing further through the handle
    /// once started. The exact reset is pinned by the scripted test below; with a real
    /// backend, which may replay the tree's own creation, only lower bounds hold.
    #[test]
    fn a_started_session_reports_its_second_pass_and_then_nothing() {
        let root = tempfile::tempdir().expect("root");
        let cache = tempfile::tempdir().expect("cache");
        let mut bytes = 0;
        for directory in 0..4 {
            let dir = root.path().join(format!("d{directory}"));
            std::fs::create_dir(&dir).expect("directory");
            for file in 0..3 {
                let size = directory * 3 + file + 1;
                std::fs::write(dir.join(format!("f{file}.txt")), vec![b'.'; size]).expect("file");
                bytes += size as u64;
            }
        }
        let request = || {
            Request::new(
                Basis {
                    root: root.path().to_path_buf(),
                    scope: ScanConfig::default().into(),
                    content: crate::content::AnalysisSet::NONE,
                },
                Query::default(),
                std::time::UNIX_EPOCH,
            )
        };
        let delivery = Delivery {
            cache: crate::CachePolicy::Auto,
            cache_path: Some(cache.path().join("snapshot")),
            accept_partial: false,
            watch: Some(WatchDelivery { interval: Duration::from_secs(2) }),
            workers: crate::query::Workers::default(),
            batch_size: 4,
            order: crate::scan::ScanOrder::default(),
        };

        let progress = crate::Progress::new();
        let session = Session::start_with_progress(request(), delivery.clone(), &progress)
            .expect("observed start");
        let after_start = progress.snapshot();
        assert_eq!(
            after_start.phase,
            crate::ProgressPhase::Revalidating,
            "the handoff revalidation follows the joined save"
        );
        // The closing pass restarts the counters, so they show its walk alone. A backend
        // that reports a file created just before the watch began can make that pass
        // read more, never less.
        assert!(after_start.directories >= 5, "the root and four children: {after_start:?}");
        assert!(after_start.files >= 12, "{after_start:?}");
        assert!(after_start.bytes >= bytes, "{after_start:?}");
        assert_eq!(after_start.analysis, None);

        let plain = Session::start(request(), delivery).expect("plain start");
        let generated_at = std::time::SystemTime::now();
        let observed_report = session.report(generated_at).expect("observed report");
        let mut plain_report = plain.report(generated_at).expect("plain report");
        plain_report.provenance = observed_report.provenance.clone();
        let json = |report: &Report| {
            crate::report_format::render(report, crate::report_format::Format::Json, false)
                .expect("render")
        };
        assert_eq!(json(&plain_report), json(&observed_report));
        assert_eq!(progress.snapshot(), after_start, "the second start was not observed");
    }
}
