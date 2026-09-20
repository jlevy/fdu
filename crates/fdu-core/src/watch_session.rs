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

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

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

/// An index paired with a watcher, answering one request continuously.
pub struct Session {
    index: IndexHandle,
    watcher: Watcher,
    scan: ScanConfig,
    request: Request,
}

impl Session {
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
        let root = index.root_path()?;
        let scan = request.basis.scope.clone();
        // What no delivery can carry, before anything stored is read and before the
        // backend is bound: this is the rule each surface used to keep for itself, so a
        // library caller could watch what `--watch` has always refused.
        let delivery = Delivery {
            watch: delivery.watch.or(Some(WatchDelivery { interval: watch.settle })),
            ..delivery.clone()
        };
        request.validate_delivery(&delivery).map_err(Error::InvalidRequest)?;
        // Reject an out-of-scope watch before the backend is bound, so a rejected run
        // never leaves a watcher registered on the tree.
        scan.validate_for_scope(index.scope()?)?;
        // The scope check above proved the index was taken under exactly this scan's
        // identity, control tier included, so what remains is what this index holds.
        let held = Basis {
            root: root.clone(),
            scope: scan.clone(),
            content: index.read_with(crate::Index::content_set)?,
        };
        request.validate_read(&held).map_err(Error::InvalidRequest)?;
        // Bind observation before closing the gap from the scan that produced `index`.
        // The full reconciliation catches a mutation that completed before registration;
        // the capture drain applies every hint observed while that pass ran.
        let watcher = Watcher::new(&root, watch)?;
        Self::finish_initial_handoff(index, request, &delivery, watcher, scan)
    }

    /// Finish the two-part initial handoff after observation has been bound.
    ///
    /// Kept separate so the scripted watcher exercises the same reconciliation, drain, and
    /// acceptance boundary as an OS watcher. Once this returns, later partial observations are
    /// valid live state; `accept_partial` governs only the coherent state handed to the caller.
    fn finish_initial_handoff(
        index: IndexHandle,
        request: Request,
        delivery: &Delivery,
        watcher: Watcher,
        scan: ScanConfig,
    ) -> Result<Self> {
        let reconciliation = crate::scan::reconcile_handle(&index, &scan, &mut |_| {})?;
        if !reconciliation.scan.is_complete() && !delivery.accept_partial {
            return Err(Error::ObservationHandoffIncomplete);
        }
        drain_initial_capture(&watcher, &index, &scan)?;
        if !delivery.accept_partial
            && !index.read_with(|index| crate::query::TreeStatus::of(index, &request).complete)?
        {
            return Err(Error::ObservationHandoffIncomplete);
        }
        Ok(Self { index, watcher, scan, request })
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
                    (false, true) | (true, true) => Some(Change {
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

fn drain_initial_capture(watcher: &Watcher, index: &IndexHandle, scan: &ScanConfig) -> Result<()> {
    for _ in 0..2 {
        watcher.flush_capture()?;
        let mut drained = false;
        for _ in 0..=watcher.capture_backlog_bound() {
            if watcher.apply_next(index, scan, Duration::ZERO, &mut |_| {})?.is_none() {
                drained = true;
                break;
            }
        }
        if !drained {
            return Err(Error::ObservationHandoffIncomplete);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
                scope: scan.clone(),
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
            analysis_workers: 0,
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
                scope: scan.clone(),
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
            analysis_workers: 0,
        };

        let error = match Session::new(
            IndexHandle::new(index.clone()),
            request.clone(),
            &delivery,
            WatchConfig::default(),
        ) {
            Ok(_) => panic!("a partial handoff is refused"),
            Err(error) => error,
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
                scope: scan.clone(),
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
            analysis_workers: 0,
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

        let error = match Session::finish_initial_handoff(
            IndexHandle::new(index),
            request,
            &delivery,
            watcher,
            scan,
        ) {
            Ok(_) => panic!("a partial state created while draining is refused"),
            Err(error) => error,
        };
        assert!(matches!(error, Error::ObservationHandoffIncomplete));
        assert!(attempts.load(Ordering::SeqCst) > 1, "the drain ran after startup reconciliation");
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
}
