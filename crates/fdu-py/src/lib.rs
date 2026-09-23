//! Python bindings for fdu.
//!
//! # API shape
//!
//! Every method here is **bulk**: it returns a whole structured result in one call
//! rather than exposing a cursor that Python iterates. A million small zero-copy calls
//! lose comfortably to one large call, because the per-call boundary cost dominates once
//! the native work per item is a field read.
//!
//! `open()`, `scan()`, and the native reconciliation phase of `Index.refresh()` release
//! the GIL. One `PyIndex` still owns one ordinary Rust [`fdu_core::Index`]: `refresh()` keeps
//! `PyO3`'s exclusive object borrow for the whole detached reconciliation, so an
//! overlapping call on that same Python object is rejected by `PyO3`'s runtime borrow
//! check rather than becoming an unsynchronized shared-index read. Calls on independent
//! indexes may run concurrently. Python dictionary/list conversion happens after native
//! work returns and therefore runs with the GIL held.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use pyo3::exceptions::{PyOSError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use fdu_core::content::{AnalysisRequest, AnalysisSet};
use fdu_core::query::{
    AxisNames, Basis, Delivery, ReadSpec, Report, Request, RequestError, RequestSpec, TreeStatus,
    ViewSpec, WatchDelivery, parse_cache_policy, parse_kind,
};
use fdu_core::watch::WatchConfig;
use fdu_core::watch_session::{ChangeKind, Session};
use fdu_core::{CachePolicy, EntryKind, Freshness, IndexHandle, RollUp};
use std::time::{Duration, SystemTime};

mod opened_binding;

fn to_py_err(err: fdu_core::Error) -> PyErr {
    match err {
        fdu_core::Error::Io { path, source } => PyOSError::new_err((
            source.raw_os_error(),
            source.to_string(),
            path.as_os_str().to_os_string(),
        )),

        // The caller asked for something the grammar or the scope does not allow. These
        // are argument errors, and `except InvalidArgumentError` should catch exactly
        // them.
        error @ (fdu_core::Error::PathEscapesRoot(_)
        | fdu_core::Error::UnsupportedScanConfig(_)
        | fdu_core::Error::ScanScopeMismatch { .. }
        | fdu_core::Error::SubtreeOutsideScanScope { .. }
        | fdu_core::Error::InvalidValue { .. }
        | fdu_core::Error::JournalCapacityTooSmall { .. }
        | fdu_core::Error::WatchRootMismatch { .. }) => PyValueError::new_err(error.to_string()),

        // A request no holder of its basis can answer is an argument error too, and it
        // already renders itself in this API's field names.
        fdu_core::Error::InvalidRequest(refusal) => value_error(&refusal),

        // Everything else is the operation failing on its own terms: the cache had no
        // usable snapshot, a lock was poisoned, a watch worker stopped. The arguments were
        // fine, so calling these ValueError told a caller to look in the wrong place -- and
        // it made `--cache only` exit 2 as a usage error where the command line exits 1
        // (fdu-4msv).
        operational => PyRuntimeError::new_err(operational.to_string()),
    }
}

fn rollup_dict<'py>(py: Python<'py>, roll: &RollUp) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("files", roll.files)?;
    dict.set_item("dirs", roll.dirs)?;
    dict.set_item("bytes", roll.bytes)?;
    dict.set_item("allocated", roll.allocated)?;
    dict.set_item("newest_mtime_ns", roll.newest_mtime_ns)?;

    let by_ext = PyDict::new(py);
    for (ext, tally) in &roll.by_ext {
        let entry = PyDict::new(py);
        entry.set_item("files", tally.files)?;
        entry.set_item("bytes", tally.bytes)?;
        entry.set_item("allocated", tally.allocated)?;
        by_ext.set_item(ext, entry)?;
    }
    dict.set_item("by_extension", by_ext)?;
    Ok(dict)
}

fn entry_kind_label(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::File => "file",
        EntryKind::Dir => "dir",
        EntryKind::Symlink => "symlink",
        EntryKind::Other => "other",
    }
}

fn freshness_label(freshness: Freshness) -> &'static str {
    match freshness {
        Freshness::Fresh => "fresh",
        Freshness::Reconciling => "reconciling",
        Freshness::Stale => "stale",
        Freshness::Partial => "partial",
    }
}

/// A live index over one directory tree.
#[pyclass(name = "Index", module = "fdu._native")]
pub struct PyIndex {
    inner: fdu_core::Index,
    /// Root, scope, and analyzers: what this index holds for its lifetime, and what every
    /// read of it is validated against.
    basis: Basis,
    /// How this index was delivered: the cache policy it was opened under, which decides
    /// whether it may be watched, and the worker count a later `refresh` re-runs its
    /// analyzers with.
    delivery: Delivery,
}

#[pymethods]
impl PyIndex {
    /// The absolute root this index covers.
    #[getter]
    fn root(&self) -> OsString {
        self.inner.root_path().as_os_str().to_os_string()
    }

    /// The current logical clock. Pass it to `since()` later to get what changed.
    #[getter]
    fn clock(&self) -> u64 {
        self.inner.clock().0
    }

    /// Whether every path in this index's configured scope is currently trustworthy.
    #[getter]
    fn complete(&self) -> bool {
        self.tree_status().complete
    }

    /// Current trust state: fresh, reconciling, stale, or partial.
    #[getter]
    fn freshness(&self) -> &'static str {
        freshness_label(self.inner.freshness())
    }

    /// Error details from the most recent scan or refresh.
    #[getter]
    fn errors(&self) -> Vec<String> {
        self.tree_status().errors.into_iter().map(|issue| issue.message).collect()
    }

    /// Coverage, currency, origin, and structured non-fatal errors.
    fn status<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        status_dict(py, self)
    }

    /// Number of entries held, including the root.
    fn __len__(&self) -> usize {
        usize::try_from(self.inner.len()).unwrap_or(usize::MAX)
    }

    /// Build one report and hand back the finished value.
    ///
    /// Returns a handle rather than rendered bytes so every renderer a caller reaches for
    /// answers from the same report. Re-projecting the index per format instead would make
    /// one `Report` disagree with itself the moment the index moved: `as_dict` would hold
    /// the values the call was answered with and `render` would quietly return newer ones
    /// (fdu-4gno). The one-shot and the watch already return their report this way; this is
    /// the third and last producer to.
    #[pyo3(signature = (
        *,
        views = None,
        include = None,
        exclude = None,
        min_size = None,
        modified_since = None,
        modified_before = None,
        kind = None,
        ignored = None,
        depth = None,
        limit = None,
        sort = None,
        reverse = false,
        size = None,
        words_per_page = fdu_core::query::Request::DEFAULTS.words_per_page
    ))]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    fn report_handle(
        &self,
        views: Option<Vec<String>>,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        min_size: Option<&str>,
        modified_since: Option<&str>,
        modified_before: Option<&str>,
        kind: Option<Vec<String>>,
        ignored: Option<&str>,
        depth: Option<&str>,
        limit: Option<&str>,
        sort: Option<&str>,
        reverse: bool,
        size: Option<&str>,
        words_per_page: u64,
    ) -> PyResult<PyOneShot> {
        let report = self.build_report(
            views,
            include,
            exclude,
            min_size,
            modified_since,
            modified_before,
            kind,
            ignored,
            depth,
            limit,
            sort,
            reverse,
            size,
            words_per_page,
        )?;
        Ok(PyOneShot { report })
    }

    /// Watch this tree, yielding batches of changes as they arrive.
    ///
    /// Detection is event-driven, so an idle tree costs nothing; `interval` bounds how
    /// long a single wait blocks before yielding an empty batch.
    #[pyo3(signature = (
        *,
        interval = 2.0,
        views = None,
        include = None,
        exclude = None,
        min_size = None,
        modified_since = None,
        modified_before = None,
        kind = None,
        ignored = None,
        depth = None,
        limit = None,
        sort = None,
        reverse = false,
        size = None,
        words_per_page = fdu_core::query::Request::DEFAULTS.words_per_page
    ))]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    fn watch(
        &self,
        interval: f64,
        views: Option<Vec<String>>,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        min_size: Option<&str>,
        modified_since: Option<&str>,
        modified_before: Option<&str>,
        kind: Option<Vec<String>>,
        ignored: Option<&str>,
        depth: Option<&str>,
        limit: Option<&str>,
        sort: Option<&str>,
        reverse: bool,
        size: Option<&str>,
        words_per_page: u64,
    ) -> PyResult<PyWatch> {
        // No default view of its own: a watch serves what a report of the same basis serves,
        // which for an index with no analyzers is the tree. Passing `files` here was the
        // last place a request meant one thing at this door and another at the next.
        let request = build_request(
            SystemTime::now(),
            &self.basis,
            views,
            include,
            exclude,
            min_size,
            modified_since,
            modified_before,
            kind,
            ignored,
            depth,
            limit,
            sort,
            reverse,
            size,
            words_per_page,
        )?;

        // Refused here, in this API's own names, rather than as the session's typed error:
        // an index that holds analyzers, or one opened from a snapshot nothing verified,
        // cannot be watched, exactly as `--watch` refuses both.
        let interval = watch_duration(interval)?;
        let delivery =
            Delivery { watch: Some(WatchDelivery { interval }), ..self.delivery.clone() };
        request.validate_delivery(&delivery).map_err(|error| value_error(&error))?;

        // The index is cloned into the session: a watcher owns its own handle, so closing
        // the feed cannot disturb the caller's index.
        let handle = IndexHandle::new(self.inner.clone());
        let session =
            Session::new(handle, request, &delivery, WatchConfig::default()).map_err(to_py_err)?;

        Ok(PyWatch { session: Some(session), timeout: interval })
    }

    /// Roll-up totals for the whole tree.
    fn total<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        rollup_dict(py, &self.inner.total())
    }

    /// Roll-up totals for one directory, or `None` if it is absent or not a directory.
    #[pyo3(signature = (path))]
    #[allow(clippy::needless_pass_by_value)]
    fn rollup<'py>(&self, py: Python<'py>, path: PathBuf) -> PyResult<Option<Bound<'py, PyDict>>> {
        match self.inner.rollup(&path) {
            Some(roll) => Ok(Some(rollup_dict(py, &roll)?)),
            None => Ok(None),
        }
    }

    /// Every direct child of a directory, with its roll-up, in one call.
    ///
    /// Returns `None` when the path is absent or is not a directory — distinct from an
    /// empty list, which means a directory with no children.
    #[pyo3(signature = (path = None))]
    fn children<'py>(
        &self,
        py: Python<'py>,
        path: Option<PathBuf>,
    ) -> PyResult<Option<Bound<'py, PyList>>> {
        let path = path.unwrap_or_default();
        let Some(children) = self.inner.children(&path) else {
            return Ok(None);
        };

        let out = PyList::empty(py);
        for (name, id) in children {
            let entry = PyDict::new(py);
            entry.set_item("name", name)?;
            let kind = self.inner.kind_of(id).expect("child handle is live");
            entry.set_item("kind", entry_kind_label(kind))?;
            let child_path = path.join(name);
            let provenance = self.inner.provenance(&child_path).expect("child handle is live");
            entry.set_item("provenance", provenance_dict(py, provenance)?)?;
            if let Some(roll) = self.inner.rollup_of(id) {
                entry.set_item("rollup", rollup_dict(py, &roll)?)?;
            } else {
                let attrs = self.inner.attrs_of(id).expect("child handle is live");
                entry.set_item("bytes", attrs.size)?;
                entry.set_item("allocated", attrs.allocated)?;
                entry.set_item("mtime_ns", attrs.mtime_ns)?;
            }
            out.append(entry)?;
        }
        Ok(Some(out))
    }

    /// Provenance for one retained path, or `None` when it is absent.
    #[pyo3(signature = (path = None))]
    fn provenance<'py>(
        &self,
        py: Python<'py>,
        path: Option<PathBuf>,
    ) -> PyResult<Option<Bound<'py, PyDict>>> {
        let path = path.unwrap_or_default();
        let Some(provenance) = self.inner.provenance(&path) else {
            return Ok(None);
        };
        Ok(Some(provenance_dict(py, provenance)?))
    }

    /// Reconcile the index against the filesystem and return what changed.
    ///
    /// This is the revalidation tier: unchanged entries cost a stat and nothing more,
    /// because an upsert whose complete observed state already matches is a no-op.
    fn refresh<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let report = py
            .detach(|| fdu_core::refresh(&mut self.inner, &self.basis, &self.delivery))
            .map_err(to_py_err)?;
        let applied = report.apply;
        let status = self.tree_status();
        let out = PyDict::new(py);
        out.set_item("inserted", applied.inserted)?;
        out.set_item("updated", applied.updated)?;
        out.set_item("removed", applied.removed)?;
        out.set_item("unchanged", applied.unchanged)?;
        out.set_item("stale", applied.stale)?;
        set_tree_status(py, &out, &status)?;
        out.set_item("ignore_rules", ignore_rules_value(py, &self.inner.control_coverage())?)?;
        out.set_item("clock", self.inner.clock().0)?;
        Ok(out)
    }

    /// Changes applied after `clock`.
    ///
    /// `truncated` is the field that matters: when it is true the caller has fallen
    /// further behind than the retained journal and must re-read state instead of
    /// trusting the returned ops. Ignoring it is how an index silently diverges.
    #[pyo3(signature = (clock))]
    fn since<'py>(&self, py: Python<'py>, clock: u64) -> PyResult<Bound<'py, PyDict>> {
        let since = self.inner.since(fdu_core::Clock(clock));
        let ops = PyList::empty(py);
        for commit in &since.commits {
            for change in &commit.changes {
                let item = match change {
                    fdu_core::EffectiveChange::Inserted { path, kind, attrs }
                    | fdu_core::EffectiveChange::Updated { path, kind, current: attrs, .. } => {
                        let item = PyDict::new(py);
                        item.set_item("clock", commit.clock.0)?;
                        item.set_item("path", path.as_os_str())?;
                        item.set_item("op", "upsert")?;
                        item.set_item("kind", entry_kind_label(*kind))?;
                        item.set_item("bytes", attrs.size)?;
                        item.set_item("mtime_ns", attrs.mtime_ns)?;
                        item
                    }
                    fdu_core::EffectiveChange::Removed { path, .. } => {
                        let item = PyDict::new(py);
                        item.set_item("clock", commit.clock.0)?;
                        item.set_item("path", path.as_os_str())?;
                        item.set_item("op", "remove")?;
                        item
                    }
                    fdu_core::EffectiveChange::Invalidated { path, reason } => {
                        let item = PyDict::new(py);
                        item.set_item("clock", commit.clock.0)?;
                        item.set_item("path", path.as_os_str())?;
                        item.set_item("op", "invalidate_subtree")?;
                        item.set_item("reason", format!("{reason:?}"))?;
                        item
                    }
                    fdu_core::EffectiveChange::ControlUpdated { .. }
                    | fdu_core::EffectiveChange::ControlRefusalUpdated { .. }
                    | fdu_core::EffectiveChange::Reclassified { .. } => continue,
                };
                ops.append(item)?;
            }
        }

        let out = PyDict::new(py);
        out.set_item("truncated", since.truncated)?;
        out.set_item("clock", self.inner.clock().0)?;
        out.set_item("ops", ops)?;
        Ok(out)
    }
}

impl PyIndex {
    fn tree_status(&self) -> TreeStatus {
        let request =
            Request::new(self.basis.clone(), fdu_core::query::Query::default(), SystemTime::now());
        TreeStatus::of(&self.inner, &request)
    }

    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    fn build_report(
        &self,
        views: Option<Vec<String>>,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        min_size: Option<&str>,
        modified_since: Option<&str>,
        modified_before: Option<&str>,
        kind: Option<Vec<String>>,
        ignored: Option<&str>,
        depth: Option<&str>,
        limit: Option<&str>,
        sort: Option<&str>,
        reverse: bool,
        size: Option<&str>,
        words_per_page: u64,
    ) -> PyResult<Report> {
        // `now` is the report's own generated_at as well as the clock the time bounds are
        // resolved against, so both come from one reading rather than two.
        let now = SystemTime::now();
        let request = build_request(
            now,
            &self.basis,
            views,
            include,
            exclude,
            min_size,
            modified_since,
            modified_before,
            kind,
            ignored,
            depth,
            limit,
            sort,
            reverse,
            size,
            words_per_page,
        )?;
        fdu_core::query::report(&self.inner, &request, now).map_err(to_py_err)
    }

    /// The analysis pass this index's basis asks for, with the worker count it was opened
    /// under.
    fn analysis_request(&self) -> AnalysisRequest {
        AnalysisRequest { profile: self.basis.content, workers: self.delivery.workers.analysis }
    }
}

/// The request model's refusal, in the Python API's words.
///
/// A `ValueError`, like every other refusal of an argument: the caller asked for something
/// the grammar does not allow.
fn value_error(error: &RequestError) -> PyErr {
    PyValueError::new_err(error.message(&AxisNames::FIELDS))
}

fn watch_duration(interval: f64) -> PyResult<Duration> {
    let duration = Duration::try_from_secs_f64(interval).map_err(|_| {
        PyValueError::new_err("interval must be finite, positive, and within the supported range")
    })?;
    if duration.is_zero() || interval < 1e-9 {
        return Err(PyValueError::new_err(
            "interval must be finite, positive, and within the supported range",
        ));
    }
    Ok(duration)
}

/// Largest finite binary64 interval that `Duration` can represent.
///
/// `Duration::MAX.as_secs_f64()` rounds upward to exactly 2^64 seconds, which the
/// fallible constructor correctly refuses. The preceding binary64 value is the public
/// upper bound so Python and the native boundary accept exactly the same domain.
const MAX_WATCH_INTERVAL_SECONDS: f64 = f64::from_bits(0x43ef_ffff_ffff_ffff);

/// The default analyzer set, as the grammar spells it.
///
/// Read from the model the same way the command line does. The empty set is the one
/// analyzer set a `const` can spell, so the assertion is what makes this a reading of
/// the table rather than a guess about it.
const ANALYZE_DEFAULT: &str = AnalysisSet::NONE_LABEL;
const _: () = assert!(
    !Request::DEFAULTS.content.is_enabled(),
    "Python signatures print the default analyzer set; a table that enables one needs a spelling here"
);

/// The basis an `open`, a `scan`, or a one-shot holds, built by the request model.
///
/// Root, scope, and analyzers are the axes a holder is fixed with, and the model parses
/// them from the words the caller wrote: `max_depth` is typed by this API and reaches it
/// through `Display`, as the command line's typed flags do.
#[allow(clippy::too_many_arguments)]
fn build_basis(
    root: &Path,
    max_depth: Option<usize>,
    one_filesystem: bool,
    read_controls: bool,
    control_budget: Option<&str>,
    control_line_limit: Option<&str>,
    analyze: &str,
) -> PyResult<Basis> {
    let scan_depth = max_depth.map(|depth| depth.to_string());
    let spec = RequestSpec {
        scan_depth: scan_depth.as_deref(),
        one_filesystem,
        read_controls: Some(read_controls),
        control_budget,
        control_line_limit,
        analyze: Some(analyze),
        ..RequestSpec::new(root)
    };
    Basis::build(&spec, &AxisNames::FIELDS).map_err(|error| value_error(&error))
}

fn status_dict<'py>(py: Python<'py>, index: &PyIndex) -> PyResult<Bound<'py, PyDict>> {
    let tree = index.tree_status();
    let status = PyDict::new(py);
    set_tree_status(py, &status, &tree)?;
    status.set_item("ignore_rules", ignore_rules_value(py, &index.inner.control_coverage())?)?;
    Ok(status)
}

fn set_tree_status(
    py: Python<'_>,
    target: &Bound<'_, PyDict>,
    status: &TreeStatus,
) -> PyResult<()> {
    target.set_item("complete", status.complete)?;
    let coverage = PyDict::new(py);
    match status.coverage {
        fdu_core::Coverage::Complete => coverage.set_item("kind", "complete")?,
        fdu_core::Coverage::Partial(reason) => {
            coverage.set_item("kind", "partial")?;
            coverage.set_item("reason", structural_coverage_reason_label(reason))?;
        }
    }
    target.set_item("coverage", coverage)?;
    let errors = PyList::empty(py);
    for issue in &status.errors {
        let item = PyDict::new(py);
        if let Some(path) = &issue.path {
            item.set_item("path", path.as_os_str())?;
        }
        item.set_item("kind", issue_kind_label(issue.kind))?;
        item.set_item("message", &issue.message)?;
        if let Some(os_error) = issue.os_error {
            item.set_item("os_error", os_error)?;
        }
        errors.append(item)?;
    }
    target.set_item("errors", errors)?;
    target.set_item("errors_omitted", status.errors_omitted)?;
    Ok(())
}

fn structural_coverage_reason_label(reason: fdu_core::CoverageReason) -> &'static str {
    match reason {
        fdu_core::CoverageReason::Building => "building",
        fdu_core::CoverageReason::Budget => "budget",
        fdu_core::CoverageReason::Cancelled => "cancelled",
        fdu_core::CoverageReason::Inaccessible => "inaccessible",
        fdu_core::CoverageReason::Failed => "failed",
    }
}

fn issue_kind_label(kind: fdu_core::IssueKind) -> &'static str {
    match kind {
        fdu_core::IssueKind::Permission => "permission",
        fdu_core::IssueKind::Disappeared => "disappeared",
        fdu_core::IssueKind::InvalidMetadata => "invalid_metadata",
        fdu_core::IssueKind::ResourceBudget => "resource_budget",
        fdu_core::IssueKind::ObservationGap => "observation_gap",
        fdu_core::IssueKind::ProviderFailure => "provider_failure",
    }
}

/// `None` when no control file was read, else the shape a report's `ignore_rules` carries.
fn ignore_rules_value<'py>(
    py: Python<'py>,
    coverage: &fdu_core::control::ControlCoverage,
) -> PyResult<Bound<'py, PyAny>> {
    match coverage {
        fdu_core::control::ControlCoverage::NotObserved => Ok(py.None().into_bound(py)),
        fdu_core::control::ControlCoverage::Observed(observed) => {
            Ok(opened_binding::control_observation_dict(py, observed)?.into_any())
        }
    }
}

fn value_source_label(source: fdu_core::Source) -> &'static str {
    match source {
        fdu_core::Source::Scanned => "scanned",
        fdu_core::Source::Revalidated => "revalidated",
        fdu_core::Source::JournalScoped => "journal_scoped",
        fdu_core::Source::Cached => "cached",
    }
}

fn coverage_label_value(status: fdu_core::Status) -> &'static str {
    match status {
        fdu_core::Status::Complete => "complete",
        _ => "partial",
    }
}

fn provenance_dict(
    py: Python<'_>,
    provenance: fdu_core::Provenance,
) -> PyResult<Bound<'_, PyDict>> {
    let value = PyDict::new(py);
    value.set_item("source", value_source_label(provenance.source))?;
    value.set_item("observed_at_ns", provenance.observed_at_ns)?;
    value.set_item("status", coverage_label_value(provenance.status))?;
    Ok(value)
}

/// Name a cache tier for Python callers, matching the CLI's machine output.
/// Parse a serialization name.
///
/// The package can render fdu's own output, not only structured values: a caller who wants
/// what the command line prints should not have to shell out to the binary to get it.
fn parse_format(value: &str) -> PyResult<fdu_core::report_format::Format> {
    match value.trim().to_ascii_lowercase().as_str() {
        "text" => Ok(fdu_core::report_format::Format::Text),
        "json" => Ok(fdu_core::report_format::Format::Json),
        "jsonl" => Ok(fdu_core::report_format::Format::Jsonl),
        "yaml" => Ok(fdu_core::report_format::Format::Yaml),
        other => Err(PyValueError::new_err(format!(
            "invalid format {other:?}: expected one of text, json, jsonl, yaml"
        ))),
    }
}

/// Parse the `control_budget` and `control_line_limit` tokens with the engine's grammar,
/// each on its own; an absent token keeps that limit's default.
pub(crate) fn parse_control_limits(
    budget: Option<&str>,
    line_limit: Option<&str>,
) -> PyResult<fdu_core::ControlLimits> {
    let defaults = fdu_core::ControlLimits::default();
    Ok(fdu_core::ControlLimits {
        budget: budget.map_or(Ok(defaults.budget), |value| {
            fdu_core::query::parse_control_budget(value).map_err(to_py_err)
        })?,
        line_limit: line_limit.map_or(Ok(defaults.line_limit), |value| {
            fdu_core::query::parse_control_line_limit(value).map_err(to_py_err)
        })?,
    })
}

/// A live change feed over one index.
///
/// Iteration yields one list of changes per batch. A tick with no changes yields an
/// empty list rather than blocking forever, so a caller can break out on its own terms
/// and the interpreter can always exit — an iterator that blocks indefinitely inside a
/// GIL-holding call is how a Python process becomes unkillable.
// Unsendable because the event queue belongs to the thread that created it: sharing one
// feed across threads would give each an arbitrary half of the stream, and Python is
// better told that at the boundary than left to discover it.
#[pyclass(name = "Watch", module = "fdu._native", unsendable)]
struct PyWatch {
    session: Option<Session>,
    timeout: Duration,
}

#[pymethods]
impl PyWatch {
    /// The live answer, as of now, from the index this session has been updating.
    ///
    /// A watch run has no final answer: the aggregates are only true until the next
    /// change, so a caller redrawing them needs the session's own index rather than the
    /// one it was opened from. Reporting the opened index instead repaints numbers that
    /// stopped being true at the first event, which looks like a working display and is
    /// not one (fdu-m66a).
    ///
    /// Returns a snapshot, so rendering it twice gives the same answer both times.
    fn report(&self) -> PyResult<PyOneShot> {
        let session =
            self.session.as_ref().ok_or_else(|| PyRuntimeError::new_err("this watch is closed"))?;
        let report = session.report(SystemTime::now()).map_err(to_py_err)?;
        Ok(PyOneShot { report })
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Wait for the next batch, yielding a possibly empty list of changes.
    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyList>>> {
        let Some(session) = self.session.as_mut() else {
            // A closed feed is exhausted rather than an error, so `for ... in` ends
            // cleanly after close().
            return Ok(None);
        };

        // The GIL is released for the whole wait: this blocks for up to `timeout`, and
        // holding the GIL across it would freeze every other Python thread.
        let batch = py.detach(|| session.next_batch(self.timeout)).map_err(to_py_err)?;

        let list = PyList::empty(py);
        if let Some(batch) = batch {
            for change in &batch.changes {
                let dict = PyDict::new(py);
                dict.set_item("path", change.path.as_os_str())?;
                dict.set_item(
                    "op",
                    match change.kind {
                        ChangeKind::Upsert => "upsert",
                        ChangeKind::Remove => "remove",
                        ChangeKind::Invalidate => "invalidate",
                    },
                )?;
                dict.set_item("clock", change.clock)?;
                dict.set_item("kind", change.entry_kind.map(entry_kind_label))?;
                dict.set_item("bytes", change.bytes)?;
                dict.set_item("allocated", change.allocated)?;
                dict.set_item("mtime_ns", change.mtime_ns)?;
                dict.set_item("ignored", change.ignored)?;
                list.append(dict)?;
            }
        }
        Ok(Some(list.unbind()))
    }

    /// Stop watching and release the backend registration.
    fn close(&mut self) {
        self.session = None;
    }

    fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    #[pyo3(signature = (*_args))]
    fn __exit__(&mut self, _args: &Bound<'_, pyo3::types::PyTuple>) -> bool {
        self.close();
        false
    }
}

/// Build one request from the keyword arguments every report path accepts.
///
/// `basis` is what the index or the one-shot holds -- root, scope, and analyzers -- and the
/// arguments are what this read names; the model owns every grammar, every default, and
/// every rule that relates one axis to another. Extracted so the session path, the index
/// path, and the one-shot cannot disagree about what a request means, which they did when
/// each parsed its own.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
fn build_request(
    now: SystemTime,
    basis: &Basis,
    views: Option<Vec<String>>,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    min_size: Option<&str>,
    modified_since: Option<&str>,
    modified_before: Option<&str>,
    kind: Option<Vec<String>>,
    ignored: Option<&str>,
    depth: Option<&str>,
    limit: Option<&str>,
    sort: Option<&str>,
    reverse: bool,
    size: Option<&str>,
    words_per_page: u64,
) -> PyResult<Request> {
    // A closed vocabulary is a comma list to the model, as it is on the command line; this
    // API takes a sequence and joins it, so ["tree", "tree"] is the typo the one grammar
    // calls it rather than a silent no-op. A raw spec arrives as a single element and
    // survives the join intact.
    let views = views.map(|values| values.join(","));
    // An empty sequence names no kind at all, as it always has here; only a list with an
    // entry in it is a list the grammar reads.
    let kinds = kind.filter(|values| !values.is_empty()).map(|values| values.join(","));
    let include = include.unwrap_or_default();
    let exclude = exclude.unwrap_or_default();
    // Typed by this API and read back through `Display`, exactly as the command line's
    // typed flags are, so both doors hand the model the same words.
    let words_per_page = words_per_page.to_string();
    let spec = ReadSpec {
        views: views.as_deref(),
        words_per_page: Some(&words_per_page),
        include: &include,
        exclude: &exclude,
        min_size,
        modified_since,
        modified_before,
        kinds: kinds.as_deref(),
        ignored,
        depth,
        limit,
        sort,
        reverse,
        size,
    };
    // The basis is the holder's, never the caller's: an index was opened with its root,
    // scope, and analyzers, and a read of it names only what this read asks. Handing it to
    // the model up front is what lets the view axis default from the analyzers the holder
    // already holds -- a request that paid to read files displays what it read -- rather
    // than spelling that typed set back into the grammar to ask.
    Request::read(basis.clone(), &spec, now, &AxisNames::FIELDS)
        .map_err(|error| value_error(&error))
}

/// One report, holding only what the request needed.
///
/// Owns the finished report so a caller can render it in more than one format without
/// paying for the walk again. A one-shot retains no index, so re-rendering from a query
/// would mean rescanning -- which is the cost the one-shot exists to avoid.
#[pyclass(name = "OneShot", module = "fdu._native", frozen)]
struct PyOneShot {
    report: fdu_core::query::Report,
}

#[pymethods]
impl PyOneShot {
    fn render(&self, format: &str, color: bool) -> PyResult<String> {
        Ok(fdu_core::report_format::render(&self.report, parse_format(format)?, color))
    }

    /// What the report says about itself, as values rather than as rendered text.
    ///
    /// The wire envelope excludes these deliberately, so a caller reading the typed report
    /// would otherwise have to scrape them out of the text rendering to learn that a view
    /// was dropped -- which is the gap on the library side that carrying them on `Report`
    /// closed in the first place (fdu-7wd1).
    fn notes(&self) -> Vec<String> {
        self.report.notes.clone()
    }
}

/// Produce one report the way the command line does, retaining the least state it needs.
///
/// `open` takes the session path: it retains an index and writes a snapshot, which is
/// right for a caller asking many questions and wrong for one asking a single question.
/// An unfiltered summary that reads no `.gitignore` is answered by a transient tier that
/// retains nothing, so writing a snapshot for it caches state the walk never saved -- and a
/// Python caller therefore left cache state on a tree that the same command would not
/// have, which a later cache-only read could see (fdu-4msv).
///
/// `read_controls`, `control_budget`, and `control_line_limit` are the engine's, with the
/// defaults [`open`] has: on, 4 MiB, and 16 KiB. The report observes `.gitignore` as they
/// say (fdu-elnn).
#[pyfunction]
#[pyo3(signature = (
    root,
    *,
    cache = "auto",
    max_depth = None,
    one_filesystem = false,
    read_controls = fdu_core::query::Request::DEFAULTS.read_controls,
    control_budget = None,
    control_line_limit = None,
    analyze = ANALYZE_DEFAULT,
    analysis_workers = 0,
    views = None,
    include = None,
    exclude = None,
    min_size = None,
    modified_since = None,
    modified_before = None,
    kind = None,
    ignored = None,
    depth = None,
    limit = None,
    sort = None,
    reverse = false,
    size = None,
    words_per_page = fdu_core::query::Request::DEFAULTS.words_per_page,
))]
#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::fn_params_excessive_bools
)]
fn report_once(
    py: Python<'_>,
    root: PathBuf,
    cache: &str,
    max_depth: Option<usize>,
    one_filesystem: bool,
    read_controls: bool,
    control_budget: Option<&str>,
    control_line_limit: Option<&str>,
    analyze: &str,
    analysis_workers: usize,
    views: Option<Vec<String>>,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    min_size: Option<&str>,
    modified_since: Option<&str>,
    modified_before: Option<&str>,
    kind: Option<Vec<String>>,
    ignored: Option<&str>,
    depth: Option<&str>,
    limit: Option<&str>,
    sort: Option<&str>,
    reverse: bool,
    size: Option<&str>,
    words_per_page: u64,
) -> PyResult<PyOneShot> {
    // One instant for the whole request: the report's `generated_at` and the clock its time
    // bounds resolve against come from one reading rather than two.
    let now = SystemTime::now();
    let basis = build_basis(
        &root,
        max_depth,
        one_filesystem,
        read_controls,
        control_budget,
        control_line_limit,
        analyze,
    )?;
    let delivery = Delivery {
        cache: parse_cache_policy(cache, AxisNames::FIELDS.cache)
            .map_err(|error| value_error(&error))?,
        cache_path: fdu_core::default_cache_path(&root),
        workers: fdu_core::query::Workers { analysis: analysis_workers, ..Default::default() },
        batch_size: fdu_core::ScanConfig::default().batch_size,
        order: fdu_core::ScanOrder::default(),
        // A one-shot report is neither partial-tolerant nor repeated: this function
        // returns one complete answer or raises.
        accept_partial: false,
        watch: None,
    };
    // Refused before any scan, in the API's own names; the engine refuses the same request
    // with the same typed value for a Rust caller.
    let request = build_request(
        now,
        &basis,
        views,
        include,
        exclude,
        min_size,
        modified_since,
        modified_before,
        kind,
        ignored,
        depth,
        limit,
        sort,
        reverse,
        size,
        words_per_page,
    )?;

    let prepared = py.detach(|| fdu_core::prepare_report(&request, &delivery));
    let (report, pending_save, _performance) = prepared.map_err(to_py_err)?;
    // Joined before returning: the command line overlaps the write with rendering, but a
    // caller who gets a value back should not still owe the filesystem a write.
    pending_save.join().map_err(to_py_err)?;
    Ok(PyOneShot { report })
}

/// The rule that separates one watch repaint from the one before it.
///
/// A watch run has no final answer and so no performance footer, which leaves consecutive
/// text repaints with nothing between them: the last row of one and the first row of the
/// next are adjacent lines. A blank line will not do -- that is already what separates two
/// views inside a single report -- so the rule carries the instant it was drawn, which is
/// also the one fact distinguishing two repaints whose numbers happen to match.
#[pyfunction]
fn watch_rule(at_nanos: i64) -> String {
    // Nanoseconds because that is what a Change already carries, so a caller repainting
    // after a batch has the instant to hand without converting through a platform clock.
    // Windows SystemTime has 100-nanosecond precision and would truncate the last two
    // digits of a value that this API promises to render exactly.
    fdu_core::report_format::watch_rule_nanos(at_nanos)
}

/// Render one watch record the way the CLI streams it, in any format.
///
/// `Index.watch` yields the facts of a change and nothing turned them into fdu's bytes, so
/// a caller streaming changes had to invent a format that would drift from the one the
/// command line emits (fdu-m66a). This is the renderer `--watch` uses.
///
/// Takes the record's fields rather than a reconstructed value, because the fields ARE the
/// record: `Change` carries exactly these, and a parity session pins that the two surfaces
/// emit the same line.
#[pyfunction]
#[pyo3(signature = (path, op, clock, kind = None, bytes = None, allocated = None, mtime_ns = None, ignored = None, format = "jsonl"))]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn render_change(
    path: PathBuf,
    op: &str,
    clock: u64,
    kind: Option<&str>,
    bytes: Option<u64>,
    allocated: Option<u64>,
    mtime_ns: Option<i64>,
    ignored: Option<bool>,
    format: &str,
) -> PyResult<String> {
    let change = fdu_core::Change {
        path,
        kind: match op {
            "upsert" => fdu_core::ChangeKind::Upsert,
            "remove" => fdu_core::ChangeKind::Remove,
            "invalidate" => fdu_core::ChangeKind::Invalidate,
            other => {
                return Err(PyValueError::new_err(format!(
                    "invalid op {other:?}: expected upsert, remove, or invalidate"
                )));
            }
        },
        entry_kind: kind
            .map(|value| parse_kind(value, AxisNames::FIELDS.kind))
            .transpose()
            .map_err(|error| value_error(&error))?,
        bytes,
        allocated,
        mtime_ns,
        ignored,
        clock,
    };
    Ok(fdu_core::report_format::render_change(&change, parse_format(format)?))
}

/// Render cache statuses the way the CLI does, in any format.
///
/// The human layout lives in `report_format` beside every other human layout, so this is
/// the same renderer `--cache-status` uses rather than a second copy. Without it a caller
/// holding `CacheStatus` values had no way to print them as fdu prints them, and the
/// parity shim fell back to `repr()`.
///
/// `scope` is the request the statuses answer, `root` or `all`. It decides only which
/// clearing command the text rendering names.
///
/// Takes the statuses as paths rather than as reconstructed values: re-reading the files
/// is cheap, keeps one definition of what a status *is*, and means a caller cannot hand
/// the renderer a status the engine never produced.
#[pyfunction]
#[pyo3(signature = (paths, scope, format = "text"))]
#[allow(clippy::needless_pass_by_value)]
fn render_cache_status(paths: Vec<PathBuf>, scope: &str, format: &str) -> PyResult<String> {
    let scope = fdu_core::CacheScope::parse(scope).ok_or_else(|| {
        PyValueError::new_err(format!(
            "invalid cache scope {:?}: expected one of {}",
            scope.trim(),
            fdu_core::CacheScope::LABELS.join(", ")
        ))
    })?;
    let format = parse_format(format)?;
    let statuses = paths
        .iter()
        .map(|path| fdu_core::cache_status(path).map_err(to_py_err))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(fdu_core::report_format::render_cache_status(&statuses, scope, format))
}

/// Decode the authoritative cache wire row instead of maintaining a second schema.
fn cache_status_dict<'py>(
    py: Python<'py>,
    status: &fdu_core::CacheStatus,
) -> PyResult<Bound<'py, PyDict>> {
    let document = fdu_core::report_format::render_cache_status(
        std::slice::from_ref(status),
        fdu_core::CacheScope::Root,
        fdu_core::report_format::Format::Json,
    );
    let parsed = py.import("json")?.call_method1("loads", (document,))?;
    Ok(parsed.get_item("caches")?.get_item(0)?.cast_into::<PyDict>()?)
}

/// The cache directory this build would use for a root.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn cache_path(root: PathBuf) -> Option<PathBuf> {
    fdu_core::default_cache_path(&root)
}

/// Status of the snapshot for one root.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn cache_status(py: Python<'_>, root: PathBuf) -> PyResult<Option<Bound<'_, PyDict>>> {
    let Some(path) = fdu_core::default_cache_path(&root) else {
        return Ok(None);
    };
    let status = fdu_core::cache_status(&path).map_err(to_py_err)?;
    Ok(Some(cache_status_dict(py, &status)?))
}

/// Every cache file this build can see, recognized or not.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn list_caches(py: Python<'_>, root: PathBuf) -> PyResult<Bound<'_, PyList>> {
    let list = PyList::empty(py);
    let Some(dir) =
        fdu_core::default_cache_path(&root).and_then(|p| p.parent().map(Path::to_path_buf))
    else {
        return Ok(list);
    };
    for status in fdu_core::list_caches(&dir).map_err(to_py_err)? {
        list.append(cache_status_dict(py, &status)?)?;
    }
    Ok(list)
}

/// Remove the snapshot for one root, current or stale. Returns whether one was removed.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn clear_cache(root: PathBuf) -> PyResult<bool> {
    match fdu_core::default_cache_path(&root) {
        Some(path) => fdu_core::clear_cache(&path).map_err(to_py_err),
        None => Ok(false),
    }
}

/// Remove every fdu snapshot, current or stale, and reclaim the files fdu left behind,
/// leaving unrecognized files alone. Returns the two counts as a dict.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn clear_all_caches(py: Python<'_>, root: PathBuf) -> PyResult<Bound<'_, PyDict>> {
    let summary =
        match fdu_core::default_cache_path(&root).and_then(|p| p.parent().map(Path::to_path_buf)) {
            Some(dir) => fdu_core::clear_all_caches(&dir).map_err(to_py_err)?,
            None => fdu_core::ClearSummary::default(),
        };
    let dict = PyDict::new(py);
    dict.set_item("snapshots", summary.snapshots)?;
    dict.set_item("leftovers", summary.leftovers)?;
    Ok(dict)
}

/// Open a directory tree, using the snapshot cache according to `cache`.
///
/// `read_controls` is the engine's [`ScanConfig::read_controls`], on by default as it is
/// there. On, the index shares its snapshot scope with a one-shot report that observes
/// too, under the same control limits. Off, it observes no `.gitignore` control state,
/// which is a snapshot scope of its own, shared only with a report or scan that also
/// turned observation off.
#[pyfunction]
#[pyo3(signature = (
    root,
    *,
    cache = "auto",
    max_depth = None,
    one_filesystem = false,
    read_controls = fdu_core::query::Request::DEFAULTS.read_controls,
    control_budget = None,
    control_line_limit = None,
    analyze = ANALYZE_DEFAULT,
    analysis_workers = 0
))]
#[allow(
    clippy::needless_pass_by_value,
    clippy::fn_params_excessive_bools,
    clippy::too_many_arguments
)]
fn open(
    py: Python<'_>,
    root: PathBuf,
    cache: &str,
    max_depth: Option<usize>,
    one_filesystem: bool,
    read_controls: bool,
    control_budget: Option<&str>,
    control_line_limit: Option<&str>,
    analyze: &str,
    analysis_workers: usize,
) -> PyResult<PyIndex> {
    let basis = build_basis(
        &root,
        max_depth,
        one_filesystem,
        read_controls,
        control_budget,
        control_line_limit,
        analyze,
    )?;
    let delivery = Delivery {
        cache: parse_cache_policy(cache, AxisNames::FIELDS.cache)
            .map_err(|error| value_error(&error))?,
        cache_path: fdu_core::default_cache_path(&root),
        workers: fdu_core::query::Workers { analysis: analysis_workers, ..Default::default() },
        batch_size: fdu_core::ScanConfig::default().batch_size,
        order: fdu_core::ScanOrder::default(),
        accept_partial: false,
        // An index is opened here and may be watched later; `Index.watch` states the
        // watch delivery then, over this one.
        watch: None,
    };

    let opened = py.detach(|| fdu_core::open(&basis, &delivery));
    let (index, _report) = opened.map_err(to_py_err)?;
    Ok(PyIndex { inner: index, basis, delivery })
}

/// Walk a tree with no cache at all and return the index.
///
/// `read_controls` is on by default, as it is for [`open`].
#[pyfunction]
#[pyo3(signature = (
    root,
    *,
    max_depth = None,
    one_filesystem = false,
    read_controls = fdu_core::query::Request::DEFAULTS.read_controls,
    control_budget = None,
    control_line_limit = None,
    analyze = ANALYZE_DEFAULT,
    analysis_workers = 0
))]
#[allow(
    clippy::needless_pass_by_value,
    clippy::fn_params_excessive_bools,
    clippy::too_many_arguments
)]
fn scan(
    py: Python<'_>,
    root: PathBuf,
    max_depth: Option<usize>,
    one_filesystem: bool,
    read_controls: bool,
    control_budget: Option<&str>,
    control_line_limit: Option<&str>,
    analyze: &str,
    analysis_workers: usize,
) -> PyResult<PyIndex> {
    let basis = build_basis(
        &root,
        max_depth,
        one_filesystem,
        read_controls,
        control_budget,
        control_line_limit,
        analyze,
    )?;
    // A bare scan consults no cache at all, so its delivery names none.
    let delivery = Delivery {
        cache: CachePolicy::Off,
        cache_path: None,
        workers: fdu_core::query::Workers { analysis: analysis_workers, ..Default::default() },
        batch_size: fdu_core::ScanConfig::default().batch_size,
        order: fdu_core::ScanOrder::default(),
        accept_partial: false,
        watch: None,
    };
    let scanned = py.detach(|| fdu_core::open(&basis, &delivery));
    let (index, _report) = scanned.map_err(to_py_err)?;
    // A bare scan never consults the cache, so it is always cold.
    Ok(PyIndex { inner: index, basis, delivery })
}

/// Run the native CLI using Python's process arguments.
///
/// The generated console-script wrapper adds its own executable path to `sys.argv`, so
/// reading the process's native argument vector here would parse the wrapper twice.
#[pyfunction]
fn main(py: Python<'_>) -> PyResult<u8> {
    // PyO3's OsString conversion round-trips Python's surrogateescaped Unix argv and
    // native Windows wide strings. Narrowing here to String would make the wheel's
    // console script reject paths the native Rust binary accepts.
    let args: Vec<OsString> = py.import("sys")?.getattr("argv")?.extract()?;
    Ok(py.detach(move || fdu::run_process(args)))
}

/// Canonical cross-language vocabulary used by the public facade's parity test.
#[pyfunction]
fn contract(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
    let contract = PyDict::new(py);
    contract.set_item("cache_policies", ["auto", "refresh", "read-only", "only", "off"])?;
    contract.set_item("analysis", ["none", "lines", "code", "words", "all"])?;
    // Derived, never copied. A hand-written list here is a second definition of a
    // grammar the library owns, and a parity test comparing two copies of the same
    // mistake passes: this list had drifted out of ViewSpec::ALL order and the
    // assertion never noticed, because Python had been written from the same copy
    // (fdu-ggux). Deriving it means the next view needs no edit in this file.
    let mut views: Vec<&str> = ViewSpec::ALL.iter().map(|view| view.label()).collect();
    views.push("full");
    contract.set_item("views", views)?;
    contract.set_item("formats", ["text", "json", "jsonl", "yaml"])?;
    contract.set_item("entry_kinds", ["file", "dir", "symlink", "other"])?;
    contract.set_item("size_metrics", ["allocated", "apparent"])?;
    contract.set_item("sort_keys", ["size", "count", "mtime", "name"])?;
    contract.set_item("cache_scopes", fdu_core::CacheScope::LABELS)?;
    contract.set_item("cache_states", fdu_core::CacheState::LABELS)?;
    contract.set_item("content_states", fdu_core::ContentState::LABELS)?;
    contract.set_item("stale_reasons", fdu_core::StaleReason::LABELS)?;
    contract.set_item("leftover_kinds", fdu_core::LeftoverKind::LABELS)?;
    Ok(contract)
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    // The defaults table, so the Python models state one default rather than a second copy
    // of it: `Query.words_per_page` and `Selection.size` read these.
    m.add("DEFAULT_WORDS_PER_PAGE", Request::DEFAULTS.words_per_page)?;
    m.add("DEFAULT_SIZE", Request::DEFAULTS.size.label())?;
    m.add("DEFAULT_READ_CONTROLS", Request::DEFAULTS.read_controls)?;
    m.add("MAX_WATCH_INTERVAL_SECONDS", MAX_WATCH_INTERVAL_SECONDS)?;
    m.add("MIN_WATCH_INTERVAL_SECONDS", 1e-9_f64)?;
    m.add_class::<PyIndex>()?;
    m.add_class::<PyWatch>()?;
    m.add_function(wrap_pyfunction!(open, m)?)?;
    m.add_function(wrap_pyfunction!(cache_path, m)?)?;
    m.add_function(wrap_pyfunction!(cache_status, m)?)?;
    m.add_function(wrap_pyfunction!(render_cache_status, m)?)?;
    m.add_function(wrap_pyfunction!(report_once, m)?)?;
    m.add_function(wrap_pyfunction!(render_change, m)?)?;
    m.add_function(wrap_pyfunction!(watch_rule, m)?)?;
    m.add_class::<PyOneShot>()?;
    m.add_function(wrap_pyfunction!(list_caches, m)?)?;
    m.add_function(wrap_pyfunction!(clear_cache, m)?)?;
    m.add_function(wrap_pyfunction!(clear_all_caches, m)?)?;
    m.add_function(wrap_pyfunction!(scan, m)?)?;
    m.add_function(wrap_pyfunction!(main, m)?)?;
    m.add_function(wrap_pyfunction!(contract, m)?)?;
    opened_binding::register(m)?;

    Ok(())
}

// PyO3's extension-module mode deliberately omits libpython linkage. The mandatory
// `python-concurrency` gate disables that default feature and runs these embedding
// tests inside the project's locked uv environment.
#[cfg(all(test, not(feature = "extension-module")))]
mod tests {
    use super::*;
    use std::sync::mpsc::sync_channel;
    use std::time::Duration;

    #[test]
    fn cache_policy_accepts_only_the_canonical_read_only_spelling() {
        let axis = AxisNames::FIELDS.cache;
        assert_eq!(
            parse_cache_policy("read-only", axis).expect("the canonical policy name parses"),
            CachePolicy::ReadOnly
        );
        let refused = parse_cache_policy("readonly", axis)
            .expect_err("an unreleased alias must not become a contract");
        assert_eq!(
            refused.message(&AxisNames::FIELDS),
            "invalid cache policy \"readonly\": expected one of auto, refresh, read-only, only, off"
        );
    }

    #[test]
    fn native_watch_interval_conversion_refuses_values_that_would_panic() {
        for interval in
            [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, 0.0, 1e-300, 0.1e-9, f64::MAX]
        {
            assert!(watch_duration(interval).is_err(), "{interval:?}");
        }
        assert_eq!(watch_duration(0.25).expect("valid interval"), Duration::from_millis(250));
        assert!(watch_duration(MAX_WATCH_INTERVAL_SECONDS).is_ok());
        assert!(watch_duration(f64::from_bits(MAX_WATCH_INTERVAL_SECONDS.to_bits() + 1)).is_err());
    }

    #[test]
    fn same_python_index_uses_runtime_borrow_exclusion() {
        Python::initialize();
        Python::attach(|py| {
            let index = Py::new(
                py,
                PyIndex {
                    inner: fdu_core::Index::new("/unused"),
                    basis: Basis {
                        root: PathBuf::from("/unused"),
                        scope: fdu_core::query::Scope::default(),
                        content: AnalysisSet::NONE,
                    },
                    delivery: Delivery {
                        cache: CachePolicy::Off,
                        cache_path: None,
                        accept_partial: false,
                        watch: None,
                        workers: fdu_core::query::Workers::default(),
                        batch_size: fdu_core::ScanConfig::default().batch_size,
                        order: fdu_core::ScanOrder::default(),
                    },
                },
            )
            .expect("allocate Python index");

            let read = index.try_borrow(py).expect("initial immutable borrow");
            assert!(index.try_borrow_mut(py).is_err());
            drop(read);
            assert!(index.try_borrow_mut(py).is_ok());
        });
    }

    #[test]
    fn detached_native_work_allows_another_python_thread_to_progress() {
        Python::initialize();
        Python::attach(|py| {
            let (ready_tx, ready_rx) = sync_channel(1);
            let (progress_tx, progress_rx) = sync_channel(1);
            let worker = std::thread::spawn(move || {
                ready_tx.send(()).expect("signal worker ready");
                Python::attach(|_| progress_tx.send(()).expect("report Python progress"));
            });
            ready_rx.recv().expect("worker ready");

            py.detach(move || {
                progress_rx
                    .recv_timeout(Duration::from_secs(5))
                    .expect("another Python thread must run while native work is detached");
            });
            worker.join().expect("Python worker");
        });
    }
}
