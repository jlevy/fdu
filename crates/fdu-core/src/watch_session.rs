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

/// Write the exact activity of every row whose age `format` shows into a repaint's
/// identity: each tree row's `mtime_ns` and `complete`, and a flat row's under `--long`,
/// the one flat format that prints an age. A rendered age has a unit's resolution, so a
/// touch that leaves `3m` at `3m` would otherwise repaint nothing; the other formats show
/// the exact time or none. Whether a row counts, and so what each row shows, already
/// follows from the rendering.
///
/// Each row adds its exact fields as fixed-width bytes rather than formatted text (review
/// C10 on #191). The machine formats add nothing here: their rendering, which the identity
/// already holds, carries both fields on every row.
fn write_shown_activity(
    identity: &mut impl std::io::Write,
    report: &Report,
    format: crate::report_format::Format,
) -> std::io::Result<()> {
    use crate::report_format::Format;

    if format.is_machine() {
        return Ok(());
    }
    // Text renders whichever presentation the query asked for.
    let shown = if format == Format::Text { report.format } else { format };
    for section in &report.sections {
        match section {
            crate::query::Section::Tree { root: Some(root), .. } => {
                let mut stack = vec![&**root];
                while let Some(node) = stack.pop() {
                    write_activity(identity, node.mtime_ns, node.complete)?;
                    stack.extend(node.children.iter());
                }
            }
            crate::query::Section::Files { rows, .. } if shown == Format::Long => {
                for row in rows {
                    write_activity(identity, Some(row.mtime_ns), row.complete)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// One row's exact activity, as ten bytes: whether it has a time, the time, and its
/// completeness, which is absent, false, or true.
fn write_activity(
    identity: &mut impl std::io::Write,
    mtime_ns: Option<i64>,
    complete: Option<bool>,
) -> std::io::Result<()> {
    let mut record = [0_u8; 10];
    if let Some(mtime_ns) = mtime_ns {
        record[0] = 1;
        record[1..9].copy_from_slice(&mtime_ns.to_le_bytes());
    }
    record[9] = match complete {
        None => 0,
        Some(false) => 1,
        Some(true) => 2,
    };
    identity.write_all(&record)
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
    /// The digest of the identity of the answer [`Self::changed_report`] last handed out
    /// ([`RepaintDigest`]).
    presented: Option<u128>,
}

/// A 128-bit FNV-1a digest of a repaint's identity, written section by section.
///
/// A digest rather than the identity itself: the identity is the whole rendered answer,
/// and a session keeping it would hold a second copy of a `--view full --limit all`
/// report for its whole life. FNV-1a is the hash the engine's fingerprints already use,
/// at its 128-bit width, so two identities a session compares collide with a chance no
/// repaint rule has to consider, and it needs no dependency. Each section's length is
/// mixed in where it ends, so no two splits of the same bytes into sections digest alike.
struct RepaintDigest {
    hash: u128,
    section: u64,
}

impl RepaintDigest {
    const OFFSET_BASIS: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
    const PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;

    const fn new() -> Self {
        Self { hash: Self::OFFSET_BASIS, section: 0 }
    }

    fn mix(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= u128::from(*byte);
            self.hash = self.hash.wrapping_mul(Self::PRIME);
        }
    }

    /// Close the section written so far.
    fn end_section(&mut self) {
        let length = std::mem::take(&mut self.section);
        self.mix(&length.to_le_bytes());
    }

    fn finish(mut self) -> u128 {
        self.end_section();
        self.hash
    }
}

impl std::io::Write for RepaintDigest {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.mix(bytes);
        self.section = self.section.wrapping_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX));
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
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
    /// returns, the handle observes one more thing: the first answer's build, asked for
    /// through [`Self::changed_report_with_progress`]. The session's repaints are the
    /// progress from there.
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
            if !self.plan.persists() || self.plan.delivery().cache_path.is_none() {
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
    /// make two repaints answer two different questions. The windows were resolved to
    /// absolute bounds when the request was built, so each repaint can still measure its
    /// ages from its own instant ([`Self::report`]) without moving them.
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
            presented: None,
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
    ///
    /// Its ages are measured from `generated_at`, the instant the answer is generated,
    /// not from the instant the session's request was built. A session lives for hours,
    /// and an age measured from its start makes a file written since read as modified in
    /// the future. The windows the request selects by do not move: they were resolved to
    /// absolute bounds when it was built, and only the age reference is re-read.
    pub fn report(&self, generated_at: std::time::SystemTime) -> Result<Report> {
        let index = self.index.snapshot()?;
        report(&index, &self.request_at(generated_at), generated_at)
    }

    /// The session's request, read at `at`: the same question, with its ages measured
    /// from `at`.
    fn request_at(&self, at: std::time::SystemTime) -> Request {
        let mut request = self.request.clone();
        request.now = at;
        request
    }

    /// The current answer, unless a reader of `format` would see nothing new in it.
    ///
    /// A tree changes more often than its answer does. A touch that leaves a file's
    /// size alone, or a write to an entry the selection leaves out, moves the index and
    /// marks its batch dirty, and a caller that repaints on every dirty batch then prints
    /// the rows it printed a moment ago under a new timestamp (fdu-wb5n). The answer's
    /// identity is what `format` renders of it with its generation instant held fixed,
    /// together with its tree status, its source and freshness, and the diagnostics a
    /// frontend writes beside it: its notes and tips
    /// ([`diagnostic_lines`](crate::report_format::diagnostic_lines)) and its warnings
    /// ([`report_warnings`](crate::report_format::report_warnings)). So a tree that shows
    /// sizes repaints when a size moves, a listing that shows dates repaints when a date
    /// does, machine output repaints when any field it carries does, and a retained
    /// observation gap, a coverage change, a freshness change, or a new note such as a
    /// `.gitignore` refused for its limits repaints whether or not `format` shows it,
    /// while the instant a repaint is generated at never counts on its own. A caller that
    /// prints no notes, as a quiet one does, may repaint once more than it had to, never
    /// once fewer. A session keeps a 128-bit digest of the identity, not the identity.
    /// The change records of [`Self::next_batch`] are never deduplicated; only this
    /// repaint is. The first call always answers, and [`Self::report`] always answers.
    ///
    /// Ages are the one rendered value that moves while the tree stands still: each
    /// answer measures them from its own instant, so a tree's `3m` becomes `4m` with no
    /// change at all. The identity measures them from one fixed reference instead, the
    /// instant the session's request was built, which leaves each row's age a function
    /// of its activity alone, and adds the exact `mtime_ns` and `complete` of every row
    /// whose age `format` shows. So an idle tree repaints nothing while its ages roll
    /// over, and any change in a row's activity repaints, a touch that leaves its age in
    /// the same unit included.
    ///
    /// # Errors
    ///
    /// As [`Self::report`], and [`Error::Io`] when `format` cannot render the answer.
    pub fn changed_report(
        &mut self,
        generated_at: std::time::SystemTime,
        format: crate::report_format::Format,
        options: crate::report_format::RenderOptions,
    ) -> Result<Option<Report>> {
        use std::io::Write as _;

        let mut report = self.report(generated_at)?;
        let pinned = std::time::SystemTime::UNIX_EPOCH;
        let stamped = std::mem::replace(&mut report.provenance.generated_at, pinned);
        let measured_from = report.age_reference_ns;
        report.measure_ages_from(crate::query::system_time_to_nanos(self.request.now));
        let mut identity = RepaintDigest::new();
        let rendered =
            crate::report_format::write_with_options(&report, format, options, &mut identity)
                .and_then(|()| {
                    identity.end_section();
                    write!(
                        identity,
                        "{:?}\n{:?}\n{:?}",
                        report.status, report.provenance.source, report.provenance.freshness
                    )?;
                    identity.end_section();
                    let notes = crate::report_format::diagnostic_lines(&report).into_lines();
                    for line in notes.iter().chain(&crate::report_format::report_warnings(&report))
                    {
                        writeln!(identity, "{line}")?;
                    }
                    identity.end_section();
                    write_shown_activity(&mut identity, &report, format)
                });
        report.measure_ages_from(measured_from);
        report.provenance.generated_at = stamped;
        rendered.map_err(|error| Error::io(&self.request.basis.root, error))?;
        let identity = identity.finish();
        if self.presented == Some(identity) {
            return Ok(None);
        }
        self.presented = Some(identity);
        Ok(Some(report))
    }

    /// [`Self::changed_report`], reporting the answer's construction through `progress`.
    ///
    /// The same answer as [`Self::changed_report`]; the handle observes the build and
    /// changes nothing about it. The build enters
    /// [`ProgressPhase::Summarizing`](crate::ProgressPhase), as a one-shot report's does,
    /// so a caller that drew the start through [`Self::start_with_progress`] can keep
    /// drawing until the first answer exists: a heavy view over a large tree takes
    /// seconds to build, and a line stopped when the start returned said nothing about
    /// them (fdu-wku3). Later repaints are the progress from there and take
    /// [`Self::changed_report`].
    ///
    /// # Errors
    ///
    /// As [`Self::changed_report`].
    pub fn changed_report_with_progress(
        &mut self,
        generated_at: std::time::SystemTime,
        format: crate::report_format::Format,
        options: crate::report_format::RenderOptions,
        progress: &crate::Progress,
    ) -> Result<Option<Report>> {
        progress.enter(crate::ProgressPhase::Summarizing);
        self.changed_report(generated_at, format, options)
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
        let mut removed: Vec<(&PathBuf, crate::EntryKind)> = Vec::new();
        let mut reclassified: Vec<&PathBuf> = Vec::new();
        for effective in commits.iter().flat_map(|commit| &commit.changes) {
            match effective {
                EffectiveChange::Inserted { path, .. } | EffectiveChange::Updated { path, .. } => {
                    touched.push(path);
                }
                EffectiveChange::Reclassified { path, .. } => {
                    reclassified.push(path);
                }
                EffectiveChange::Removed { path, kind, .. } => removed.push((path, *kind)),
                EffectiveChange::Invalidated { .. }
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
                    let mut ignored = touched
                        .into_iter()
                        .filter_map(|path| match index.is_ignored(path) {
                            Ok(Some(ignored)) => Some((path.clone(), ignored)),
                            // Gone from the index, or the index reads no rules; either
                            // way there is nothing to say about it.
                            Ok(None) | Err(_) => None,
                        })
                        .collect::<BTreeMap<_, _>>();
                    for (path, kind) in removed {
                        if index.control_classification_known(path) {
                            ignored.insert(
                                path.clone(),
                                index.control_table().is_ignored(path, kind.is_dir()),
                            );
                        }
                    }
                    ignored
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
            // watching. Its last known kind lets current control state classify it
            // when the governing rules are known.
            EffectiveChange::Removed { path, .. } => {
                let name = path.file_name()?.to_string_lossy().into_owned();
                self.admits_by_path(path, &name).then(|| Change {
                    path: path.clone(),
                    kind: ChangeKind::Remove,
                    entry_kind: None,
                    bytes: None,
                    allocated: None,
                    mtime_ns: None,
                    ignored: facts.is_ignored(path),
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
            stale_ok: false,
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
        // A session reads before it writes, so the fixture must read as no snapshot yet
        // refuse the write: a parent that cannot become a directory. On Unix that is a
        // symbolic link to a directory that does not exist yet, since a path through a
        // regular file fails to open rather than reading as absent; Windows spells a
        // path through a regular file as not found, so a file serves there.
        let parent = cache.path().join("blocked");
        #[cfg(unix)]
        std::os::unix::fs::symlink(cache.path().join("missing"), &parent)
            .expect("dangling parent blocks the write");
        #[cfg(not(unix))]
        std::fs::write(&parent, b"").expect("file parent blocks the write");
        let restore = || {
            #[cfg(unix)]
            std::fs::create_dir(cache.path().join("missing")).expect("restore the parent");
            #[cfg(not(unix))]
            std::fs::remove_file(&parent).expect("restore the parent");
        };
        let cache_path = parent.join("snapshot");
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
            stale_ok: false,
            cache: crate::CachePolicy::On,
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
        restore();
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
            stale_ok: false,
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
            stale_ok: false,
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
            stale_ok: false,
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
    /// the counts afterwards are exactly one walk of the tree. Identical event streams
    /// also make the observed and unobserved reports exactly comparable.
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
            Query { views: vec![crate::query::ViewSpec::Summary], ..Query::default() },
            std::time::SystemTime::now(),
        );
        let delivery = Delivery {
            stale_ok: false,
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
        progress.add_walked(100, 100, 100, 100);
        progress.enter(crate::ProgressPhase::Saving);

        let session = Session::finish_initial_handoff(
            IndexHandle::new(index.clone()),
            request.clone(),
            &delivery,
            watcher,
            scan.clone(),
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
        assert_eq!(snapshot.allocated, report.allocated_walked, "allocated restarts with them");

        let (plain_watcher, _plain_sender) =
            Watcher::scripted(root.path(), WatchConfig::default(), script.path()).expect("watcher");
        let plain = Session::finish_initial_handoff(
            IndexHandle::new(index),
            request,
            &delivery,
            plain_watcher,
            scan,
            None,
        )
        .expect("plain handoff");
        let generated_at = std::time::SystemTime::now();
        let observed_report = session.report(generated_at).expect("observed report");
        let mut plain_report = plain.report(generated_at).expect("plain report");
        plain_report.provenance = observed_report.provenance.clone();
        let json = |report: &Report| {
            crate::report_format::render(report, crate::report_format::Format::Json, false)
                .expect("render")
        };
        assert_eq!(json(&plain_report), json(&observed_report));
        assert_eq!(progress.snapshot(), snapshot, "the second handoff and reads were not observed");
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
    /// once started. The exact reset and report equivalence are pinned by the scripted
    /// test above; independent native watchers may replay different creation hints and
    /// record different legitimate setup-gap diagnostics, so only lower bounds hold.
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
            stale_ok: false,
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
        // A native watcher registered a moment after a directory was created may still
        // report it, and the engine records that truthfully as a setup-race observation
        // gap beside the facts it re-verified (fdu-21ns). That diagnostic is the only
        // way the watched answer may differ from the plain one; anything else is a real
        // difference, and the facts both report are compared with it set aside.
        //
        // An incomplete answer is allowed only when it retains at least one issue, every
        // one a setup-race gap, and none omitted: an incomplete answer with no issue, or
        // with any other, is a regression. The facts it re-verified are as fresh as the
        // plain answer's, and with no issue its coverage is the plain answer's too; both
        // are compared before the status and provenance are set aside.
        let setup_race = format!("{:?}", crate::InvalidateReason::WatchSetupRace);
        let errors = &observed_report.status.errors;
        let setup_race_only = !errors.is_empty()
            && errors.iter().all(|issue| {
                issue.kind == crate::IssueKind::ObservationGap
                    && issue.message.ends_with(&setup_race)
            });
        assert!(
            observed_report.status.complete || setup_race_only,
            "the watched answer is complete or carries only setup-race gaps: {:?}",
            observed_report.status
        );
        assert_eq!(observed_report.status.errors_omitted, 0, "{:?}", observed_report.status);
        assert!(plain_report.status.complete, "{:?}", plain_report.status);
        assert_eq!(
            observed_report.provenance.freshness, plain_report.provenance.freshness,
            "the watched answer is as fresh as the plain one"
        );
        if errors.is_empty() {
            assert_eq!(
                observed_report.status.coverage, plain_report.status.coverage,
                "with no retained issue, the coverage is the plain answer's"
            );
        }
        plain_report.provenance = observed_report.provenance.clone();
        plain_report.status = observed_report.status.clone();
        let json = |report: &Report| {
            crate::report_format::render(report, crate::report_format::Format::Json, false)
                .expect("render")
        };
        assert_eq!(json(&plain_report), json(&observed_report), "the same facts either way");
        assert_eq!(progress.snapshot(), after_start, "the second start was not observed");
    }

    /// The first answer's build is the last thing a start's progress covers (fdu-wku3):
    /// asked for through the handle, it enters `Summarizing`, answers as the plain call
    /// does, and is the answer later repaints are measured against.
    #[test]
    fn the_first_answer_is_built_under_the_summarizing_phase() {
        let root = tempfile::tempdir().expect("root");
        std::fs::write(root.path().join("a.txt"), b"alpha").expect("file");
        std::fs::create_dir(root.path().join("d")).expect("directory");
        std::fs::write(root.path().join("d/b.txt"), b"beta").expect("nested file");
        let (mut session, _sender) = scripted_session(root.path(), Query::default());
        let format = crate::report_format::Format::Json;
        let options = crate::report_format::RenderOptions { color: false, bar_size: 0 };
        let generated_at = std::time::SystemTime::now();
        let plain_answer = session.report(generated_at).expect("plain answer");

        let progress = crate::Progress::new();
        progress.enter(crate::ProgressPhase::Revalidating);
        let observed = session
            .changed_report_with_progress(generated_at, format, options, &progress)
            .expect("observed first answer")
            .expect("a session's first answer is always given");
        assert_eq!(
            progress.snapshot().phase,
            crate::ProgressPhase::Summarizing,
            "the build is reported as the answer's construction"
        );

        let json =
            |report: &Report| crate::report_format::render(report, format, false).expect("render");
        assert_eq!(json(&observed), json(&plain_answer), "the same answer either way");
        assert!(
            session.changed_report(generated_at, format, options).expect("repaint").is_none(),
            "nothing changed since the observed build, so nothing repaints"
        );
    }

    /// A scripted session over `root` answering `query`, and the sender that scripts its
    /// events.
    fn scripted_session(
        root: &std::path::Path,
        query: Query,
    ) -> (Session, crate::watch::ScriptedSender) {
        scripted_session_under(root, query, ScanConfig::default())
    }

    /// [`scripted_session`] under `scan`, such as one with tighter control limits.
    fn scripted_session_under(
        root: &std::path::Path,
        query: Query,
        scan: ScanConfig,
    ) -> (Session, crate::watch::ScriptedSender) {
        scripted_session_built(root, query, scan, std::time::SystemTime::now())
    }

    /// [`scripted_session_under`], with its request built at `built`: the instant its
    /// windows resolve against and its repaint identity measures ages from.
    fn scripted_session_built(
        root: &std::path::Path,
        query: Query,
        scan: ScanConfig,
        built: std::time::SystemTime,
    ) -> (Session, crate::watch::ScriptedSender) {
        let (index, report) = crate::scan::scan_into_index(root, &scan).expect("scan");
        assert!(report.is_complete());
        let request = Request::new(
            Basis {
                root: root.to_path_buf(),
                scope: scan.clone().into(),
                content: crate::content::AnalysisSet::NONE,
            },
            query,
            built,
        );
        let delivery = Delivery {
            stale_ok: false,
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
            Watcher::scripted(root, WatchConfig::default(), script.path()).expect("watcher");
        let session = Session::finish_initial_handoff(
            IndexHandle::new(index),
            request,
            &delivery,
            watcher,
            scan,
            None,
        )
        .expect("handoff");
        (session, sender)
    }

    /// Move a file's modification time forward without touching its bytes.
    fn touch(path: &std::path::Path) {
        let file = std::fs::File::options().write(true).open(path).expect("open for touch");
        let modified = file.metadata().expect("metadata").modified().expect("mtime");
        file.set_modified(modified + Duration::from_secs(5)).expect("set mtime");
    }

    /// A tree changes more often than its answer (fdu-wb5n): an idle tree, a touch that
    /// leaves a size alone, and a change to an entry the selection leaves out all leave
    /// the aggregate a reader sees as it was, and only a visible change repaints it.
    #[test]
    fn an_aggregate_repaint_is_skipped_when_a_reader_would_see_no_change() {
        use crate::report_format::{Format, RenderOptions};

        let root = tempfile::tempdir().expect("root");
        let big = root.path().join("big.txt");
        std::fs::write(&big, vec![b'x'; 4096]).expect("big");
        std::fs::write(root.path().join("small.txt"), b"small").expect("small");
        let query = Query {
            views: vec![crate::query::ViewSpec::Summary],
            selection: Selection {
                size: crate::query::SizeMetric::Apparent,
                min_size: Some(4096),
                ..Selection::default()
            },
            ..Query::default()
        };
        let (mut session, sender) = scripted_session(root.path(), query);
        let now = std::time::SystemTime::now;
        let text = |session: &mut Session| {
            session.changed_report(now(), Format::Text, RenderOptions::default()).expect("report")
        };
        let summary = |report: &Report| match report.sections.first() {
            Some(crate::query::Section::Summary(row)) => (row.files, row.bytes),
            other => panic!("expected a summary, got {other:?}"),
        };

        let first = text(&mut session).expect("the first answer is always given");
        assert_eq!(summary(&first), (1, 4096));

        // Idle: nothing arrived, nothing to say.
        assert!(session.next_batch(Duration::from_millis(50)).expect("idle").is_none());
        assert!(text(&mut session).is_none(), "an idle tree repaints nothing");

        // A touch moves the index, so its batch is dirty, but no size a reader of the
        // summary sees has moved.
        touch(&big);
        sender.send("modify\tbig.txt\n").expect("script a touch");
        let batch = session.next_batch(Duration::from_secs(10)).expect("touch").expect("observed");
        assert!(batch.dirty, "the index records the new modification time");
        assert!(text(&mut session).is_none(), "a touch that changes no size repaints nothing");

        // A change to an entry the selection leaves out is a change to the tree, and
        // still not a change to the answer.
        std::fs::write(root.path().join("small.txt"), b"still small").expect("grow small");
        sender.send("modify\tsmall.txt\n").expect("script a filtered change");
        let batch = session.next_batch(Duration::from_secs(10)).expect("small").expect("observed");
        assert!(batch.dirty);
        assert!(text(&mut session).is_none(), "a filtered change repaints nothing");

        // A size the summary shows moves: repaint, with the new answer.
        std::fs::write(&big, vec![b'x'; 8192]).expect("grow big");
        sender.send("modify\tbig.txt\n").expect("script a visible change");
        let batch = session.next_batch(Duration::from_secs(10)).expect("big").expect("observed");
        assert!(batch.dirty);
        let repainted = text(&mut session).expect("a visible change repaints");
        assert_eq!(summary(&repainted), (1, 8192));
        assert!(text(&mut session).is_none(), "and only once");

        // The plain report always answers, and never counts as a repaint.
        assert_eq!(summary(&session.report(now()).expect("report")), (1, 8192));
        assert!(text(&mut session).is_none());
    }

    /// The identity follows the format a reader sees: machine output carries the newest
    /// modification time, so the touch that text ignores repaints JSON.
    #[test]
    fn a_repaint_identity_is_what_the_format_renders() {
        use crate::report_format::{Format, RenderOptions};

        let root = tempfile::tempdir().expect("root");
        let big = root.path().join("big.txt");
        std::fs::write(&big, vec![b'x'; 4096]).expect("big");
        let query = Query { views: vec![crate::query::ViewSpec::Summary], ..Query::default() };
        let (mut session, sender) = scripted_session(root.path(), query);
        let json = |session: &mut Session| {
            session
                .changed_report(
                    std::time::SystemTime::now(),
                    Format::Json,
                    RenderOptions::default(),
                )
                .expect("report")
        };
        assert!(json(&mut session).is_some());
        assert!(json(&mut session).is_none(), "a second generation instant alone is no change");

        touch(&big);
        sender.send("modify\tbig.txt\n").expect("script a touch");
        assert!(session.next_batch(Duration::from_secs(10)).expect("touch").is_some());
        assert!(json(&mut session).is_some(), "JSON shows the newest modification time");
    }

    /// The default tree, every row shown, in text.
    fn tree_query() -> Query {
        Query {
            views: vec![crate::query::ViewSpec::Tree],
            selection: Selection {
                min_share: Some(crate::query::ShareThreshold::parse("0%").expect("share")),
                ..Selection::default()
            },
            ..Query::default()
        }
    }

    /// Each answer measures its ages from its own instant, not the one the session's
    /// request was built at: a file written since the session started is not modified in
    /// the future, while the request, and every window it resolved, stays as built.
    #[test]
    fn a_repaint_measures_ages_from_its_own_instant() {
        use crate::report_format::{Format, RenderOptions};

        let root = tempfile::tempdir().expect("root");
        std::fs::write(root.path().join("a.txt"), b"alpha").expect("a");
        // Built an hour before anything below happens, as a long session's request is.
        let built = std::time::SystemTime::now() - Duration::from_secs(3_600);
        let (mut session, sender) =
            scripted_session_built(root.path(), tree_query(), ScanConfig::default(), built);
        assert!(session.changed_report(built, Format::Text, RenderOptions::default()).is_ok());

        std::fs::write(root.path().join("b.txt"), b"bravo").expect("b");
        sender.send("create\tb.txt\n").expect("script the write");
        assert!(session.next_batch(Duration::from_secs(10)).expect("write").is_some());
        let repainted_at = std::time::SystemTime::now();
        let answer = session
            .changed_report(repainted_at, Format::Json, RenderOptions::default())
            .expect("repaint")
            .expect("a new file repaints");
        assert_eq!(answer.age_reference_ns, crate::query::system_time_to_nanos(repainted_at));
        let Some(crate::query::Section::Tree { root: Some(tree), .. }) = answer.sections.first()
        else {
            panic!("a tree")
        };
        let written = tree.children.iter().find(|row| row.name == "b.txt").expect("the new file");
        assert!(written.age_ns.is_some_and(|age| age >= 0), "{:?}", written.age_ns);
        assert!(tree.age_ns.is_some_and(|age| age >= 0), "{:?}", tree.age_ns);
        assert_eq!(session.request().now, built, "the request stays as it was built");
        let plain = session.report(repainted_at).expect("report");
        assert_eq!(plain.age_reference_ns, answer.age_reference_ns, "and so does the plain read");
    }

    /// An idle tree repaints nothing while its ages roll over, and any change in a row's
    /// activity repaints, even a touch that leaves its rendered age in the same unit.
    #[test]
    fn ages_rolling_over_repaint_nothing_and_activity_always_repaints() {
        use crate::report_format::{Format, RenderOptions};

        let root = tempfile::tempdir().expect("root");
        let file = root.path().join("a.txt");
        std::fs::write(&file, b"alpha").expect("a");
        let written = std::time::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
        std::fs::File::options()
            .write(true)
            .open(&file)
            .and_then(|handle| handle.set_modified(written))
            .expect("stamp");
        // The identity measures ages from the request's instant: a day and an hour after
        // the file, so a five-second touch leaves it at `1d`.
        let built = written + Duration::from_secs(25 * 3_600);
        let (mut session, sender) =
            scripted_session_built(root.path(), tree_query(), ScanConfig::default(), built);
        let at = |session: &mut Session, instant| {
            session.changed_report(instant, Format::Text, RenderOptions::default()).expect("report")
        };
        let first = at(&mut session, built).expect("the first answer is always given");
        let rendered = |report: &Report| {
            crate::report_format::render(report, Format::Text, false).expect("render")
        };
        assert!(rendered(&first).contains("  1d  "), "{}", rendered(&first));

        // Idle, and later: the ages a reader sees have rolled over, the tree has not.
        let later = built + Duration::from_secs(40 * 86_400);
        assert!(at(&mut session, later).is_none(), "an idle tree repaints nothing");
        assert!(rendered(&session.report(later).expect("report")).contains("  1mo  "));

        // A touch inside one unit of the age the identity measures still repaints.
        touch(&file);
        sender.send("modify\ta.txt\n").expect("script a touch");
        assert!(session.next_batch(Duration::from_secs(10)).expect("touch").is_some());
        let mut held = session.report(later).expect("report");
        held.measure_ages_from(crate::query::system_time_to_nanos(built));
        assert!(
            rendered(&held).contains("  1d  "),
            "the touch stays in the unit: {}",
            rendered(&held)
        );
        assert!(at(&mut session, later).is_some(), "a change in activity repaints");
        assert!(at(&mut session, later).is_none(), "and only once");
    }

    /// The repaint digest is FNV-1a at 128 bits, and a section boundary is part of what
    /// it digests.
    #[test]
    fn a_repaint_digest_is_fnv1a_128_framed_by_section() {
        use std::io::Write as _;

        let mut raw = RepaintDigest::new();
        raw.mix(b"a");
        assert_eq!(raw.hash, 0xd228_cb69_6f1a_8caf_7891_2b70_4e4a_8964, "the published vector");
        let digest = |sections: &[&[u8]]| {
            let mut digest = RepaintDigest::new();
            for (at, section) in sections.iter().enumerate() {
                if at > 0 {
                    digest.end_section();
                }
                digest.write_all(section).expect("digest");
            }
            digest.finish()
        };
        assert_eq!(digest(&[b"ab", b"c"]), digest(&[b"ab", b"c"]));
        assert_ne!(digest(&[b"ab", b"c"]), digest(&[b"a", b"bc"]));
        assert_ne!(digest(&[b"abc", b""]), digest(&[b"", b"abc"]));
    }

    /// A reader of a text report reads its diagnostics too (R164-4): a `.gitignore` edited
    /// past the line limit adds a refusal note while the tree, its sizes, and its status
    /// stay as they were, and that note alone repaints.
    #[test]
    fn a_diagnostic_that_appears_on_an_unchanged_tree_repaints() {
        use crate::report_format::{Format, RenderOptions};

        let root = tempfile::tempdir().expect("root");
        std::fs::write(root.path().join("big.txt"), vec![b'x'; 4096]).expect("big");
        let control = root.path().join(".gitignore");
        // Ten bytes either way, so no size a reader sees moves; only the longest line does.
        std::fs::write(&control, b"a\nb\nc\nd\ne\n").expect("short lines");
        let scan = ScanConfig {
            control_limits: crate::control::ControlLimits {
                line_limit: Some(4),
                ..crate::control::ControlLimits::default()
            },
            ..ScanConfig::default()
        };
        let (mut session, sender) = scripted_session_under(root.path(), Query::default(), scan);
        let text = |session: &mut Session| {
            session
                .changed_report(
                    std::time::SystemTime::now(),
                    Format::Text,
                    RenderOptions::default(),
                )
                .expect("report")
        };
        let first = text(&mut session).expect("the first answer is always given");
        let notes = |report: &Report| crate::report_format::diagnostic_lines(report).into_lines();
        assert!(
            !notes(&first).iter().any(|line| line.contains("line limit")),
            "{:?}",
            notes(&first)
        );

        std::fs::write(&control, b"abcdefghi\n").expect("one long line");
        sender.send("modify\t.gitignore\n").expect("script the edit");
        let batch = session.next_batch(Duration::from_secs(10)).expect("edit").expect("observed");
        assert!(batch.dirty);
        let repainted = text(&mut session).expect("a new refusal note repaints");
        assert_eq!(
            format!("{:?}", repainted.status),
            format!("{:?}", first.status),
            "the status alone would not have repainted"
        );
        assert!(
            notes(&repainted).iter().any(|line| line.contains("line limit")),
            "{:?}",
            notes(&repainted)
        );
        assert!(text(&mut session).is_none(), "and only once");
    }

    /// An invalidation is never deduplicated as a change record, and the answer it leaves
    /// repaints exactly when a reader would see its status or its rows differ.
    #[test]
    fn an_invalidation_keeps_its_change_record_and_repaints_by_the_answer() {
        use crate::report_format::{Format, RenderOptions};

        let root = tempfile::tempdir().expect("root");
        std::fs::write(root.path().join("a.txt"), b"aaaa").expect("a");
        let query = Query { views: vec![crate::query::ViewSpec::Summary], ..Query::default() };
        let (mut session, sender) = scripted_session(root.path(), query);
        let text = |session: &mut Session| {
            session
                .changed_report(
                    std::time::SystemTime::now(),
                    Format::Text,
                    RenderOptions::default(),
                )
                .expect("report")
        };
        let first = text(&mut session).expect("first answer");

        sender.send("rescan\t.\n").expect("script an invalidation");
        let batch =
            session.next_batch(Duration::from_secs(10)).expect("invalidation").expect("observed");
        assert!(
            batch.changes.iter().any(|change| change.kind == ChangeKind::Invalidate),
            "the invalidation reaches the change stream: {:?}",
            batch.changes
        );
        // The closed loop re-verified the tree, which is as it was; whether the answer
        // repaints is decided by its status, which the identity carries explicitly.
        let after = session.report(std::time::SystemTime::now()).expect("report");
        let repainted = text(&mut session);
        assert_eq!(
            repainted.is_some(),
            format!("{:?}", after.status) != format!("{:?}", first.status),
            "a repaint follows a status change and nothing else: {:?}",
            after.status
        );

        // The same invalidation again leaves the same status, so no second repaint.
        sender.send("rescan\t.\n").expect("script another invalidation");
        let batch =
            session.next_batch(Duration::from_secs(10)).expect("invalidation").expect("observed");
        assert!(batch.changes.iter().any(|change| change.kind == ChangeKind::Invalidate));
        assert!(text(&mut session).is_none());
    }

    /// Apply every hint scripted so far, however the worker batched them: the flush is a
    /// barrier behind which each one is queued as an intent, and each intent is applied.
    fn drain(session: &mut Session) -> Vec<Change> {
        session.watcher.flush_capture().expect("flush the scripted hints");
        let mut changes = Vec::new();
        while let Some(batch) = session.next_batch(Duration::ZERO).expect("apply a batch") {
            changes.extend(batch.changes);
        }
        changes
    }

    /// Hold the session's index to a cold walk of the same tree, now.
    fn assert_matches_cold_walk(session: &Session, root: &std::path::Path, label: &str) {
        session
            .index
            .read_with(|index| {
                crate::query::assert_same_as_cold_walk(index, root, &session.scan, label);
            })
            .expect("read the session's index");
    }

    /// An old file and a newer one in `logs/`, and `logs/`'s own time behind the clock.
    fn logs_fixture() -> tempfile::TempDir {
        use crate::test_support::{stamped_file, wait_past_modification};

        let root = tempfile::tempdir().expect("root");
        stamped_file(&root.path().join("logs/a.log"), b"a", 1_000_000_000);
        stamped_file(&root.path().join("logs/b.log"), b"b", 1_020_000_000);
        std::fs::create_dir(root.path().join("archive")).expect("archive");
        wait_past_modification(&root.path().join("logs"));
        wait_past_modification(&root.path().join("archive"));
        root
    }

    /// A directory's own time moves when an entry inside it is removed, and no backend
    /// names the directory in that event; a cold walk reads the new time, so the watched
    /// tree must too (B1 on #191). Removing a directory's newest file makes it young.
    #[test]
    fn a_removal_ages_its_directory_as_a_cold_walk_does() {
        let root = logs_fixture();
        let (mut session, sender) = scripted_session(root.path(), tree_query());
        std::fs::remove_file(root.path().join("logs/b.log")).expect("remove the newest file");
        sender.send("remove\tlogs/b.log\n").expect("script the removal");
        let changes = drain(&mut session);
        assert_matches_cold_walk(&session, root.path(), "the newest file removed");
        assert!(
            changes.iter().any(|change| change.path == std::path::Path::new("logs")
                && change.kind == ChangeKind::Upsert
                && change.entry_kind == Some(EntryKind::Dir)),
            "the directory's new time is a change record of the same shape: {changes:?}"
        );
    }

    /// A rename inside the tree moves both directories' own times: the one the name left
    /// and the one it arrived in, which keeps the file's old time (B1 on #191).
    #[test]
    fn a_rename_ages_both_directories_as_a_cold_walk_does() {
        let root = logs_fixture();
        let (mut session, sender) = scripted_session(root.path(), tree_query());
        std::fs::rename(root.path().join("logs/b.log"), root.path().join("archive/b.log"))
            .expect("rename across directories");
        sender
            .send("rename-from\tlogs/b.log\nrename-to\tarchive/b.log\n")
            .expect("script both sides of the rename");
        drain(&mut session);
        assert_matches_cold_walk(&session, root.path(), "a file renamed across directories");
    }

    /// A file moved in from outside the tree keeps its archive time, and its directory
    /// reads as just changed, as `mv ~/Downloads/report.pdf docs/` leaves it (B1 on #191).
    #[test]
    fn a_move_in_ages_its_directory_as_a_cold_walk_does() {
        let root = logs_fixture();
        let outside = tempfile::tempdir().expect("outside the watched tree");
        let report = outside.path().join("report.pdf");
        crate::test_support::stamped_file(&report, b"pdf", 990_000_000);
        let (mut session, sender) = scripted_session(root.path(), tree_query());
        std::fs::rename(&report, root.path().join("archive/report.pdf")).expect("move it in");
        sender.send("rename-to\tarchive/report.pdf\n").expect("script the arrival");
        drain(&mut session);
        assert_matches_cold_walk(&session, root.path(), "an old file moved in");
    }

    /// Deterministic `SplitMix64`, so a failing sequence replays from its printed seed.
    struct SplitMix(u64);

    impl SplitMix {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut mixed = self.0;
            mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            mixed ^ (mixed >> 31)
        }

        fn below(&mut self, bound: usize) -> usize {
            usize::try_from(self.next() % u64::try_from(bound).expect("bound")).expect("index")
        }
    }

    /// A tree a seeded sequence changes one step at a time, and what it holds.
    struct GeneratedTree {
        root: tempfile::TempDir,
        outside: tempfile::TempDir,
        random: SplitMix,
        /// Every directory, the root first; the first [`Self::PERMANENT`] are never removed.
        directories: Vec<PathBuf>,
        files: Vec<PathBuf>,
        names: u32,
    }

    impl GeneratedTree {
        const PERMANENT: usize = 4;

        fn new(seed: u64) -> Self {
            let mut tree = Self {
                root: tempfile::tempdir().expect("root"),
                outside: tempfile::tempdir().expect("outside the watched tree"),
                random: SplitMix(seed),
                directories: vec![PathBuf::new(), "a".into(), "a/b".into(), "c".into()],
                files: Vec::new(),
                names: 0,
            };
            for directory in &tree.directories[1..] {
                std::fs::create_dir_all(tree.root.path().join(directory)).expect("directory");
            }
            for _ in 0..4 {
                let path = tree.fresh_name("f");
                let stamp = tree.stamp();
                crate::test_support::stamped_file(&tree.root.path().join(&path), b"seed", stamp);
                tree.files.push(path);
            }
            tree
        }

        /// An old modification time, a few years after 2001.
        fn stamp(&mut self) -> u64 {
            1_000_000_000 + self.random.next() % 100_000_000
        }

        /// A name no entry has had, in a directory chosen at random.
        fn fresh_name(&mut self, prefix: &str) -> PathBuf {
            self.names += 1;
            let directory = &self.directories[self.random.below(self.directories.len())];
            directory.join(format!("{prefix}{}", self.names))
        }

        /// A generated directory with nothing beneath it, which a step may remove.
        fn removable_directory(&self) -> Option<usize> {
            (Self::PERMANENT..self.directories.len()).find(|&at| {
                let directory = &self.directories[at];
                !self.files.iter().any(|file| file.starts_with(directory))
                    && !self
                        .directories
                        .iter()
                        .any(|other| other != directory && other.starts_with(directory))
            })
        }

        /// Make one change on disk and return the events for it that a backend naming
        /// each side of a rename delivers.
        fn step(&mut self) -> String {
            let line = |verb: &str, path: &std::path::Path| {
                format!("{verb}\t{}\n", path.to_string_lossy().replace('\\', "/"))
            };
            let on_disk = |tree: &Self, path: &std::path::Path| tree.root.path().join(path);
            match self.random.below(7) {
                0 | 1 => {
                    let path = self.fresh_name("f");
                    let bytes = vec![b'x'; self.random.below(64)];
                    let stamp = self.stamp();
                    crate::test_support::stamped_file(&on_disk(self, &path), &bytes, stamp);
                    self.files.push(path.clone());
                    line("create", &path)
                }
                2 if !self.files.is_empty() => {
                    let path = self.files.swap_remove(self.random.below(self.files.len()));
                    std::fs::remove_file(on_disk(self, &path)).expect("remove");
                    line("remove", &path)
                }
                3 if !self.files.is_empty() => {
                    let at = self.random.below(self.files.len());
                    let to = self.fresh_name("f");
                    let from = std::mem::replace(&mut self.files[at], to.clone());
                    std::fs::rename(on_disk(self, &from), on_disk(self, &to))
                        .expect("rename inside the tree");
                    line("rename-from", &from) + &line("rename-to", &to)
                }
                4 => {
                    let to = self.fresh_name("f");
                    let staged = self.outside.path().join("staged");
                    let stamp = self.stamp();
                    crate::test_support::stamped_file(&staged, b"moved in", stamp);
                    std::fs::rename(&staged, on_disk(self, &to)).expect("move in");
                    self.files.push(to.clone());
                    line("rename-to", &to)
                }
                5 if !self.files.is_empty() => {
                    let path = self.files[self.random.below(self.files.len())].clone();
                    let stamp = self.stamp();
                    std::fs::File::options()
                        .write(true)
                        .open(on_disk(self, &path))
                        .and_then(|file| {
                            file.set_modified(std::time::UNIX_EPOCH + Duration::from_secs(stamp))
                        })
                        .expect("restamp");
                    line("modify", &path)
                }
                // A new directory, or the removal of an empty one this sequence made:
                // either moves its parent's own time.
                _ => {
                    if let Some(at) =
                        self.removable_directory().filter(|_| self.random.below(2) == 0)
                    {
                        let directory = self.directories.remove(at);
                        std::fs::remove_dir(on_disk(self, &directory)).expect("remove a directory");
                        line("remove", &directory)
                    } else {
                        let directory = self.fresh_name("d");
                        std::fs::create_dir(on_disk(self, &directory)).expect("make a directory");
                        self.directories.push(directory.clone());
                        line("create-dir", &directory)
                    }
                }
            }
        }
    }

    /// B2 on #191: a watched tree equals a cold walk of the same tree after every step of
    /// a seeded sequence of creates, removals, renames inside the tree, moves in from
    /// outside it, restamps, and new and removed directories, each file stamped with an
    /// old time. Each step is scripted with the events a backend that names each side of
    /// a rename delivers, and takes the same worker, coalescing, verification, and apply
    /// path a real backend's events take, so the sequence is the same on every run.
    ///
    /// Every other maintenance test compares the index with itself, with a pass over it,
    /// or with a model fed the same operations, and none of those can see a fact that no
    /// operation carried. This one compares it with the disk.
    #[test]
    fn a_watched_tree_equals_a_cold_walk_after_every_generated_step() {
        const SEEDS: [u64; 3] = [1, 0x5eed, 191];
        const STEPS: usize = 24;
        for seed in SEEDS {
            let mut tree = GeneratedTree::new(seed);
            let (mut session, sender) = scripted_session(tree.root.path(), tree_query());
            let mut trace = Vec::new();
            for step in 0..STEPS {
                let script = tree.step();
                trace.push(script.trim_end().replace('\n', "; "));
                sender.send(&script).expect("script the step");
                drain(&mut session);
                assert_matches_cold_walk(
                    &session,
                    tree.root.path(),
                    &format!("seed {seed:#x}, step {step}, trace {trace:?}"),
                );
            }
        }
    }
}
