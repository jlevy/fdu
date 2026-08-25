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
use std::sync::{Arc, Mutex};

use pyo3::exceptions::{PyOSError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use fdu_core::classify::TypeRegistry;
use fdu_core::content::{AnalysisRequest, AnalysisSet, CoverageReason};
use fdu_core::query::{
    AxisNames, Bound as Bound_, MetricRow, MetricSummary, Pattern, Query, Report, ReportSource,
    Section, Selection, SizeMetric, SortKey, SummaryRow, TreeNode, ViewSpec, document_words,
};
use fdu_core::watch::WatchConfig;
use fdu_core::watch_session::{ChangeKind, Session};
use fdu_core::{
    CachePolicy, EntryKind, Freshness, IndexHandle, OpenConfig, PerformanceSummary, RollUp,
    ScanConfig,
};
use std::time::{Duration, SystemTime};

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
        | fdu_core::Error::TypeRules(_)
        | fdu_core::Error::WatchRootMismatch { .. }) => PyValueError::new_err(error.to_string()),

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
    put_scalar(&dict, "files", roll.files)?;
    put_scalar(&dict, "dirs", roll.dirs)?;
    put_scalar(&dict, "others", roll.others)?;
    put_scalar(&dict, "bytes", roll.bytes)?;
    put_scalar(&dict, "allocated", roll.allocated)?;
    put_scalar(&dict, "newest_mtime_ns", roll.newest_mtime_ns)?;

    // These two are keyed by data rather than by schema, so their keys are payload: an
    // extension name is the answer, not the shape of it.
    let by_ext = PyDict::new(py);
    for (ext, tally) in &roll.by_ext {
        let entry = PyDict::new(py);
        put_scalar(&entry, "files", tally.files)?;
        put_scalar(&entry, "bytes", tally.bytes)?;
        put_scalar(&entry, "allocated", tally.allocated)?;
        ext.as_str().charge();
        by_ext.set_item(ext, entry)?;
    }
    dict.set_item("by_extension", by_ext)?;

    let by_group = PyDict::new(py);
    for (group, tally) in &roll.by_group {
        let entry = PyDict::new(py);
        put_scalar(&entry, "files", tally.files)?;
        put_scalar(&entry, "bytes", tally.bytes)?;
        put_scalar(&entry, "allocated", tally.allocated)?;
        group.as_str().charge();
        by_group.set_item(group, entry)?;
    }
    dict.set_item("by_group", by_group)?;
    put_nested(&dict, "extension_remainder", ext_remainder_dict(py, roll.ext_remainder)?)?;
    Ok(dict)
}

/// What an extension bound withheld, or `None` when it withheld nothing.
fn ext_remainder_dict(
    py: Python<'_>,
    remainder: Option<fdu_core::ExtRemainder>,
) -> PyResult<Option<Bound<'_, PyDict>>> {
    let Some(remainder) = remainder else {
        return Ok(None);
    };
    let value = PyDict::new(py);
    put_scalar(&value, "extensions", remainder.extensions)?;
    put_scalar(&value, "files", remainder.files)?;
    put_scalar(&value, "bytes", remainder.bytes)?;
    put_scalar(&value, "allocated", remainder.allocated)?;
    Ok(Some(value))
}

/// Read a caller's extension bound, where `None` means every row.
fn ext_bound(extensions: Option<usize>) -> fdu_core::Bound {
    extensions.map_or(fdu_core::Bound::All, fdu_core::Bound::Limit)
}

/// Read a caller's row bound, where `None` means every row.
fn row_bound(rows: Option<usize>) -> fdu_core::Bound {
    rows.map_or(fdu_core::Bound::All, fdu_core::Bound::Limit)
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

/// Run state that a refresh replaces, held apart from the index itself.
///
/// Behind a lock so `refresh` can take `&self`: the fix for readers raising during a
/// write is that no method takes an exclusive borrow of the whole object, and these
/// fields are the only other thing a refresh mutates. The lock is taken for a field
/// assignment and never across native work.
struct RunState {
    /// What the run that produced this state actually did.
    ///
    /// The only field left, and the only one that belongs here: it is about *this call*
    /// rather than about the index. Coverage, source, the start watermark and the typed
    /// failures moved onto the index itself, because a read has to return them with the
    /// rows they describe -- held beside the index in this lock, they were sampled at a
    /// different instant, and a refresh landing between the two acquisitions paired an
    /// old projection with new status.
    ///
    /// Describes the most recent operation only, never a running total. An embedder
    /// timing its own loop needs to attribute cost to the call it just made, and a
    /// counter that accumulated across refreshes would answer a different question
    /// while looking like this one.
    telemetry: PerformanceSummary,
}

/// A compiled set of file-type rules.
///
/// Built once and passed to as many scans as needed: parsing is the cost, and a registry
/// is read-only afterwards. Passing the same object to several opens shares one copy
/// rather than reparsing per call.
#[pyclass(name = "TypeRegistry", module = "fdu._native", frozen)]
pub struct PyTypeRegistry {
    inner: Arc<TypeRegistry>,
}

#[pymethods]
impl PyTypeRegistry {
    /// Parse rules in the `[[kind]]` manifest dialect.
    #[staticmethod]
    #[pyo3(signature = (source, expect_fingerprint = None))]
    fn from_manifest(source: &str, expect_fingerprint: Option<u64>) -> PyResult<Self> {
        let registry =
            TypeRegistry::from_manifest_expecting(source, expect_fingerprint).map_err(to_py_err)?;
        Ok(Self { inner: Arc::new(registry) })
    }

    /// The rules fdu ships, used when a scan names no others.
    #[staticmethod]
    fn compiled() -> Self {
        Self { inner: Arc::clone(TypeRegistry::compiled()) }
    }

    /// Identity of these rules.
    ///
    /// A snapshot and a content sidecar both record it, and both refuse a cached answer
    /// when it moves: a classification change can move a file between families, which
    /// invalidates the metrics rather than merely their labels.
    #[getter]
    fn fingerprint(&self) -> u64 {
        self.inner.fingerprint()
    }

    /// How many `[[kind]]` rules it holds.
    #[getter]
    fn rule_count(&self) -> usize {
        self.inner.rule_count()
    }

    /// Distinct extensions it claims.
    #[getter]
    fn extension_count(&self) -> usize {
        self.inner.extension_count()
    }

    /// Distinct exact basenames it claims.
    #[getter]
    fn filename_count(&self) -> usize {
        self.inner.filename_count()
    }

    /// Every stable type identifier it can produce, in manifest order.
    fn type_ids(&self) -> Vec<String> {
        self.inner.type_ids().map(str::to_owned).collect()
    }

    /// Classify one path against these rules, from its name alone.
    #[pyo3(signature = (path))]
    #[allow(clippy::needless_pass_by_value)]
    fn classify<'py>(&self, py: Python<'py>, path: PathBuf) -> PyResult<Bound<'py, PyDict>> {
        classification_dict(py, &fdu_core::classify::classify_with(&self.inner, &path, None))
    }

    fn __repr__(&self) -> String {
        format!(
            "TypeRegistry(rules={}, extensions={}, fingerprint={:#x})",
            self.inner.rule_count(),
            self.inner.extension_count(),
            self.inner.fingerprint()
        )
    }
}

/// One captured listing, shared by `children()` and the bundled read.
///
/// Shared so a bundle's rows cannot describe a child differently from the listing API's.
fn child_list<'py>(
    py: Python<'py>,
    children: &[fdu_core::ChildSnapshot],
) -> PyResult<Bound<'py, PyList>> {
    let out = PyList::empty(py);
    for child in children {
        let entry = PyDict::new(py);
        put_text(&entry, "name", child.name.as_os_str())?;
        put_text(&entry, "kind", entry_kind_label(child.kind))?;
        entry.set_item("provenance", provenance_dict(py, child.provenance)?)?;
        put_text(&entry, "extension", child.extension.as_deref())?;
        for tag in &child.tags {
            tag.as_str().charge();
        }
        entry.set_item("tags", child.tags.as_slice())?;
        put_nested(
            &entry,
            "classification",
            row_classification_dict(py, child.classification.as_ref(), child.group.as_deref())?,
        )?;
        if let Some(totals) = child.totals {
            // Scalars, not a roll-up: the breakdown belongs to the directory being
            // inspected, and one map per row is the cost this listing exists to avoid.
            put_scalar(&entry, "files", totals.files)?;
            put_scalar(&entry, "dirs", totals.dirs)?;
            put_scalar(&entry, "others", totals.others)?;
            put_scalar(&entry, "bytes", totals.bytes)?;
            put_scalar(&entry, "allocated", totals.allocated)?;
            put_scalar(&entry, "newest_mtime_ns", totals.newest_mtime_ns)?;
            // Decided here rather than in the consumer, because deciding it needs the
            // row's provenance as well as its counts: a partial subtree reporting zero
            // means "nothing found yet".
            put_scalar(&entry, "empty", child.is_empty_subtree())?;
            // Beside the whole-subtree numbers, never in place of them: the listing this
            // serves shows both -- "1.2 GB, 340 MB outside .gitignore" -- and a row that
            // carried only one would send the consumer back for the other per directory.
            // Absent unless a plane was asked for, which is the default.
            if let Some(plane) = child.plane_totals {
                let totals = PyDict::new(py);
                put_scalar(&totals, "files", plane.files)?;
                put_scalar(&totals, "dirs", plane.dirs)?;
                put_scalar(&totals, "others", plane.others)?;
                put_scalar(&totals, "bytes", plane.bytes)?;
                put_scalar(&totals, "allocated", plane.allocated)?;
                put_scalar(&totals, "newest_mtime_ns", plane.newest_mtime_ns)?;
                entry.set_item("plane", totals)?;
            }
        } else {
            put_scalar(&entry, "bytes", child.attrs.size)?;
            put_scalar(&entry, "allocated", child.attrs.allocated)?;
            put_scalar(&entry, "mtime_ns", child.attrs.mtime_ns)?;
        }
        out.append(entry)?;
    }
    Ok(out)
}

/// What one read actually did, beside its answer.
/// Nanoseconds since an instant, saturating rather than panicking on an absurd clock.
fn nanos_since(started: std::time::Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

/// What a result copies into Python objects, accumulated as each field crosses.
///
/// **The definition**, stated once because a number nobody can reproduce is not evidence:
/// every string and path *value* contributes its own bytes, and every fixed-width leaf --
/// an integer, a float, a boolean, or a null -- contributes [`SCALAR_BYTES`]. Dict keys
/// are excluded: they are a fixed schema rather than payload, identical on every result of
/// the same shape, and counting them would make the figure grow with the schema instead of
/// with the answer.
///
/// A thread-local rather than a threaded-through parameter, and the reason is the shape of
/// the alternative: the conversion runs through a dozen nested helpers that other paths
/// also call, so passing an accumulator would change every signature to serve one caller.
/// Conversion happens under the GIL and is single-threaded by construction, and `read`
/// clears before it starts, so the count belongs to exactly one call.
///
/// Counted here rather than by measuring the finished bundle. An after-the-fact pass would
/// be an extra O(output) traversal on the hot boundary, and -- as the first version of this
/// proved -- it drifts: it counted names and six assumed scalars per row while omitting
/// tags, classifications, provenance, extension and group tallies, and envelope errors.
mod payload {
    use std::cell::Cell;

    /// Bytes charged for one fixed-width leaf.
    ///
    /// Eight, because that is what an integer, a float, a boolean or a null costs to carry
    /// across as a logical field. Not the size of the resulting `PyObject`, which is an
    /// allocator question rather than a payload one.
    const SCALAR_BYTES: u64 = 8;

    thread_local! {
        static BYTES: Cell<u64> = const { Cell::new(0) };
    }

    /// Charge one text value by its own length.
    pub(super) fn text(len: usize) {
        add(len as u64);
    }

    /// Charge `count` fixed-width leaves.
    pub(super) fn scalars(count: u64) {
        add(count.saturating_mul(SCALAR_BYTES));
    }

    fn add(bytes: u64) {
        BYTES.with(|total| total.set(total.get().saturating_add(bytes)));
    }

    /// Take the running total and reset, so the next call starts from nothing.
    pub(super) fn take() -> u64 {
        BYTES.with(|total| total.replace(0))
    }
}

/// What one value contributes to the payload when it crosses.
///
/// Implemented rather than open-coded so the charge cannot be forgotten: every emission on
/// the read path goes through a setter below, which charges before it sets. The previous
/// arrangement -- `set_item` calls followed by a separate `payload::scalars(n)` -- drifted
/// exactly the way an unenforced convention does. A field added beside such a call is
/// emitted and not charged, and nothing anywhere fails; a count that had been six was
/// still six when the shape had gone to four, and to seven.
trait Charge {
    fn charge(&self);
}

impl Charge for &str {
    fn charge(&self) {
        payload::text(self.len());
    }
}

impl Charge for &std::ffi::OsStr {
    fn charge(&self) {
        payload::text(self.len());
    }
}

impl Charge for String {
    fn charge(&self) {
        payload::text(self.len());
    }
}

/// An absent value is a fixed-width leaf like any other.
///
/// The stated rule already said a null costs `SCALAR_BYTES`; the open-coded charges said
/// zero, so a result full of absent extensions and unclassified rows under-reported what
/// a consumer received.
impl<T: Charge> Charge for Option<T> {
    fn charge(&self) {
        match self {
            Some(value) => value.charge(),
            None => payload::scalars(1),
        }
    }
}

/// Set one fixed-width field -- an integer, a float, a boolean -- and charge it.
fn put_scalar<'py, V: IntoPyObject<'py>>(
    dict: &Bound<'py, PyDict>,
    key: &str,
    value: V,
) -> PyResult<()> {
    payload::scalars(1);
    dict.set_item(key, value)
}

/// Set one text field, charging its own bytes, or a null's fixed width when absent.
fn put_text<'py, V: Charge + IntoPyObject<'py>>(
    dict: &Bound<'py, PyDict>,
    key: &str,
    value: V,
) -> PyResult<()> {
    value.charge();
    dict.set_item(key, value)
}

/// Set a nested value that may be absent, charging a null when it is.
///
/// A present dict or list charged itself as it was built, so only the absent case is left
/// to account for here.
fn put_nested<'py, V: IntoPyObject<'py>>(
    dict: &Bound<'py, PyDict>,
    key: &str,
    value: Option<V>,
) -> PyResult<()> {
    if value.is_none() {
        payload::scalars(1);
    }
    dict.set_item(key, value)
}

fn work_dict(py: Python<'_>, work: fdu_core::Work) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("entries_visited", work.entries_visited)?;
    out.set_item("dirs_visited", work.dirs_visited)?;
    out.set_item("rows", work.rows)?;
    out.set_item("tally_rows", work.tally_rows)?;
    out.set_item("name_bytes", work.name_bytes)?;
    out.set_item("lock_wait_ns", work.lock_wait_ns)?;
    out.set_item("wall_ns", work.wall_ns)?;
    Ok(out)
}

/// One page of children, with the rest of the directory accounted for beside it.
fn child_page_dict<'py>(
    py: Python<'py>,
    page: &fdu_core::ChildPage,
) -> PyResult<Bound<'py, PyDict>> {
    let out = PyDict::new(py);
    out.set_item("rows", child_list(py, &page.rows)?)?;
    put_nested(&out, "remainder", child_remainder_dict(py, page.remainder)?)?;
    put_text(&out, "next", page.next.as_deref())?;
    Ok(out)
}

/// What a page does not carry, or `None` when it carries the whole directory.
fn child_remainder_dict(
    py: Python<'_>,
    remainder: Option<fdu_core::ChildRemainder>,
) -> PyResult<Option<Bound<'_, PyDict>>> {
    let Some(rest) = remainder else {
        return Ok(None);
    };
    let value = PyDict::new(py);
    put_scalar(&value, "rows", rest.rows)?;
    put_scalar(&value, "files", rest.files)?;
    put_scalar(&value, "dirs", rest.dirs)?;
    put_scalar(&value, "others", rest.others)?;
    put_scalar(&value, "bytes", rest.bytes)?;
    put_scalar(&value, "allocated", rest.allocated)?;
    Ok(Some(value))
}

/// The scan scope a read happened under, including the fingerprints a cache key needs.
fn scope_dict<'py>(py: Python<'py>, scope: &fdu_core::ScanScope) -> PyResult<Bound<'py, PyDict>> {
    let value = PyDict::new(py);
    put_scalar(&value, "max_depth", scope.max_depth)?;
    put_scalar(&value, "follow_symlinks", scope.follow_symlinks)?;
    put_scalar(&value, "one_filesystem", scope.one_filesystem)?;
    put_scalar(&value, "tag_rules_fingerprint", scope.tag_rules_fingerprint)?;
    put_scalar(&value, "type_rules_fingerprint", scope.type_rules_fingerprint)?;
    put_scalar(&value, "reducers_fingerprint", scope.reducers_fingerprint)?;
    put_scalar(&value, "hidden_fingerprint", scope.hidden_fingerprint)?;
    Ok(value)
}

/// A listing row's classification, with its group already resolved to a name.
///
/// `None` for an entry no rule set classifies, which is every directory and symlink: the
/// verdict is about a file's identity, and asking for one where there is none would make
/// a consumer branch on a sentinel instead of on presence.
fn row_classification_dict<'py>(
    py: Python<'py>,
    classification: Option<&fdu_core::classify::Classification>,
    group: Option<&str>,
) -> PyResult<Option<Bound<'py, PyDict>>> {
    let Some(classification) = classification else {
        return Ok(None);
    };
    let dict = classification_dict(py, classification)?;
    put_text(&dict, "group", group)?;
    Ok(Some(dict))
}

/// One classification verdict as a dict.
fn classification_dict<'py>(
    py: Python<'py>,
    classification: &fdu_core::classify::Classification,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    put_text(&dict, "file_type", classification.file_type.as_str())?;
    put_text(&dict, "family", classification.family.as_str())?;
    put_text(&dict, "source", classification.source.as_str())?;
    put_text(&dict, "confidence", classification.confidence.as_str())?;
    let flags = PyDict::new(py);
    put_scalar(&flags, "generated", classification.flags.generated)?;
    put_scalar(&flags, "vendored", classification.flags.vendored)?;
    put_scalar(&flags, "documentation", classification.flags.documentation)?;
    dict.set_item("flags", flags)?;
    Ok(dict)
}

/// A live index over one directory tree.
///
/// The index is held as an [`IndexHandle`] rather than owned outright, which is what
/// lets a reader run while a refresh applies. Holding it owned forced `refresh` to take
/// `&mut self`, and `PyO3` then kept an exclusive object borrow for the whole detached
/// reconciliation, so a concurrent `rollup` on the same object was rejected by the
/// runtime borrow check rather than served. A live consumer commits on every change, so
/// that rejected every request landing in the window.
#[pyclass(name = "Index", module = "fdu._native")]
pub struct PyIndex {
    inner: IndexHandle,
    config: ScanConfig,
    analysis: AnalysisRequest,
    state: Mutex<RunState>,
}

#[pymethods]
impl PyIndex {
    /// The absolute root this index covers.
    #[getter]
    fn root(&self) -> PyResult<OsString> {
        Ok(self.inner.root_path().map_err(to_py_err)?.as_os_str().to_os_string())
    }

    /// What the most recent scan or refresh actually did.
    ///
    /// Beside the report, never inside it: this is telemetry about a run, not a fact
    /// about the tree, and putting it in the versioned envelope would make every
    /// machine-readable answer depend on how it happened to be produced. The command
    /// line prints the same numbers as its footer.
    #[getter]
    fn telemetry<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        telemetry_dict(py, self.state().telemetry)
    }

    /// The current logical clock. Pass it to `since()` later to get what changed.
    #[getter]
    fn clock(&self) -> PyResult<u64> {
        Ok(self.inner.clock().map_err(to_py_err)?.0)
    }

    /// Whether every path in this index's configured scope is currently trustworthy.
    #[getter]
    fn complete(&self) -> bool {
        self.inner.with_index(|index| index.run_facts().complete).unwrap_or(false)
    }

    /// Current trust state: fresh, reconciling, stale, or partial.
    #[getter]
    fn freshness(&self) -> PyResult<&'static str> {
        Ok(freshness_label(self.inner.freshness().map_err(to_py_err)?))
    }

    /// Error details from the most recent scan or refresh.
    #[getter]
    fn errors(&self) -> Vec<String> {
        self.error_messages()
    }

    /// Coverage, currency, origin, and structured non-fatal errors.
    fn status<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        status_dict(py, self)
    }

    /// Number of entries held, including the root.
    fn __len__(&self) -> PyResult<usize> {
        Ok(usize::try_from(self.inner.len().map_err(to_py_err)?).unwrap_or(usize::MAX))
    }

    /// Build a report over this index.
    ///
    /// The same five axes the CLI exposes, as one typed call: a capability reachable by
    /// flag has to be reachable from the library, or the CLI has become a second
    /// implementation. String values accept exactly the CLI grammars.
    #[pyo3(signature = (
        *,
        views = None,
        include = None,
        exclude = None,
        min_size = None,
        max_size = None,
        modified_since = None,
        modified_before = None,
        kind = None,
        tags = None,
        not_tags = None,
        plane = None,
        depth = None,
        limit = None,
        sort = None,
        reverse = false,
        size = "allocated",
        words_per_page = 250,
        as_of_ns = None
    ))]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    fn report<'py>(
        &self,
        py: Python<'py>,
        views: Option<Vec<String>>,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        min_size: Option<&str>,
        max_size: Option<&str>,
        modified_since: Option<&str>,
        modified_before: Option<&str>,
        kind: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        not_tags: Option<Vec<String>>,
        plane: Option<&str>,
        depth: Option<&str>,
        limit: Option<&str>,
        sort: Option<&str>,
        reverse: bool,
        size: &str,
        words_per_page: u64,
        as_of_ns: Option<i64>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let report = self.build_report(
            views,
            include,
            exclude,
            min_size,
            max_size,
            modified_since,
            modified_before,
            kind,
            tags,
            not_tags,
            plane,
            depth,
            limit,
            sort,
            reverse,
            size,
            words_per_page,
            as_of_ns,
        )?;
        report_dict(py, &report)
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
        max_size = None,
        modified_since = None,
        modified_before = None,
        kind = None,
        tags = None,
        not_tags = None,
        plane = None,
        depth = None,
        limit = None,
        sort = None,
        reverse = false,
        size = "allocated",
        words_per_page = 250,
        as_of_ns = None
    ))]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    fn report_handle(
        &self,
        views: Option<Vec<String>>,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        min_size: Option<&str>,
        max_size: Option<&str>,
        modified_since: Option<&str>,
        modified_before: Option<&str>,
        kind: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        not_tags: Option<Vec<String>>,
        plane: Option<&str>,
        depth: Option<&str>,
        limit: Option<&str>,
        sort: Option<&str>,
        reverse: bool,
        size: &str,
        words_per_page: u64,
        as_of_ns: Option<i64>,
    ) -> PyResult<PyOneShot> {
        let report = self.build_report(
            views,
            include,
            exclude,
            min_size,
            max_size,
            modified_since,
            modified_before,
            kind,
            tags,
            not_tags,
            plane,
            depth,
            limit,
            sort,
            reverse,
            size,
            words_per_page,
            as_of_ns,
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
        poll_interval = None,
        views = None,
        include = None,
        exclude = None,
        min_size = None,
        max_size = None,
        modified_since = None,
        modified_before = None,
        kind = None,
        tags = None,
        not_tags = None,
        plane = None,
        depth = None,
        limit = None,
        sort = None,
        reverse = false,
        size = "allocated",
        words_per_page = 250,
        as_of_ns = None
    ))]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    fn watch(
        &self,
        interval: f64,
        poll_interval: Option<f64>,
        views: Option<Vec<String>>,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        min_size: Option<&str>,
        max_size: Option<&str>,
        modified_since: Option<&str>,
        modified_before: Option<&str>,
        kind: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        not_tags: Option<Vec<String>>,
        plane: Option<&str>,
        depth: Option<&str>,
        limit: Option<&str>,
        sort: Option<&str>,
        reverse: bool,
        size: &str,
        words_per_page: u64,
        as_of_ns: Option<i64>,
    ) -> PyResult<PyWatch> {
        let query = build_query(
            self.analysis.profile,
            Some(ViewSpec::Files),
            views,
            include,
            exclude,
            min_size,
            max_size,
            modified_since,
            modified_before,
            kind,
            tags,
            not_tags,
            plane,
            &self.config.tags(),
            depth,
            limit,
            sort,
            reverse,
            size,
            words_per_page,
            as_of_ns,
        )?;

        // The session watches *this* index, not a copy of it. An `IndexHandle` clone is an
        // `Arc` to the same authority, so a mutation the watcher applies is a mutation the
        // caller's `read`, `rollup` and `since` see -- which is the entire point of
        // watching. The previous deep clone gave the session a private index, so the
        // object the caller held went stale after the first event and never recovered
        // without a `refresh`; it also paid O(entries) in time and memory at watch start
        // and made "closing the feed disturbs the caller's index" a fear rather than a
        // risk. Dropping a handle drops a reference, and the index outlives it.
        let handle = self.inner.clone();
        let backend = match poll_interval {
            None => fdu_core::watch::WatchBackend::Native,
            Some(seconds) if seconds > 0.0 => {
                fdu_core::watch::WatchBackend::Poll { interval: Duration::from_secs_f64(seconds) }
            }
            Some(_) => {
                return Err(PyValueError::new_err(
                    "poll_interval must be positive; polling continuously restats the tree without ever finishing",
                ));
            }
        };
        let config = WatchConfig { backend, ..WatchConfig::default() };
        let session =
            Session::new(handle, self.config.clone(), query, config).map_err(to_py_err)?;

        Ok(PyWatch { session: Some(session), timeout: Duration::from_secs_f64(interval) })
    }

    /// Roll-up totals for the whole tree.
    #[pyo3(signature = (extensions = None, plane = None))]
    fn total<'py>(
        &self,
        py: Python<'py>,
        extensions: Option<usize>,
        plane: Option<&str>,
    ) -> PyResult<Bound<'py, PyDict>> {
        // The root is a directory in every index, so a plane read of it cannot be absent
        // for the reasons the per-path one can, and the name was resolved already.
        self.rollup_at(py, Path::new(""), extensions, plane)?
            .ok_or_else(|| PyValueError::new_err("the index root has no roll-up"))
    }

    /// Roll-up totals for one directory, or `None` if it is absent or not a directory.
    #[pyo3(signature = (path, extensions = None, plane = None))]
    #[allow(clippy::needless_pass_by_value)]
    fn rollup<'py>(
        &self,
        py: Python<'py>,
        path: PathBuf,
        extensions: Option<usize>,
        plane: Option<&str>,
    ) -> PyResult<Option<Bound<'py, PyDict>>> {
        self.rollup_at(py, &path, extensions, plane)
    }

    /// Several projections evaluated under one read guard.
    ///
    /// A composed page must not straddle a commit. Answering a listing and its parent's
    /// totals with two calls lets a write land between them, and the page is then
    /// internally inconsistent in a way nothing in it reports: the rows say one thing,
    /// the header another, and both are individually true.
    ///
    /// `clock` is the version every part of this bundle saw, so it is also the cursor to
    /// pass to `since()` next: a cache key derives from what was actually read rather
    /// than from a version sampled before dispatch.
    ///
    /// It is also one crossing and one lock acquisition instead of one of each per call.
    #[pyo3(signature = (
        children_of = None,
        rollups = None,
        total = false,
        extensions = None,
        after = None,
        limit = None,
        report = false,
        views = None,
        include = None,
        exclude = None,
        min_size = None,
        max_size = None,
        modified_since = None,
        modified_before = None,
        kind = None,
        tags = None,
        not_tags = None,
        plane = None,
        depth = None,
        limit_rows = None,
        sort = None,
        reverse = false,
        size = "allocated",
        words_per_page = 250,
        as_of_ns = None,
        expected = None,
    ))]
    #[allow(
        clippy::needless_pass_by_value,
        clippy::too_many_arguments,
        clippy::fn_params_excessive_bools
    )]
    fn read<'py>(
        &self,
        py: Python<'py>,
        children_of: Option<PathBuf>,
        rollups: Option<Vec<PathBuf>>,
        total: bool,
        extensions: Option<usize>,
        after: Option<std::ffi::OsString>,
        limit: Option<usize>,
        report: bool,
        views: Option<Vec<String>>,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        min_size: Option<&str>,
        max_size: Option<&str>,
        modified_since: Option<&str>,
        modified_before: Option<&str>,
        kind: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        not_tags: Option<Vec<String>>,
        plane: Option<&str>,
        depth: Option<&str>,
        limit_rows: Option<&str>,
        sort: Option<&str>,
        reverse: bool,
        size: &str,
        words_per_page: u64,
        as_of_ns: Option<i64>,
        expected: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyDict>> {
        // Run state is no longer sampled here. It used to be read from a second lock
        // before the bundle, which made the report and the bundle agree with each other
        // and with nothing else: a refresh landing between the two acquisitions paired an
        // old projection with new status, or the reverse, both halves individually true.
        // The engine holds these facts beside the tree now, so they arrive with the rows.
        // Resolved before the report is built, and used for both halves of the bundle: a
        // caller asking to see the tree without what git ignores means the child rows as
        // well as the report, and one plane name cannot resolve to two answers.
        let rows_plane = plane
            .map(|name| self.config.tags().plane_of(name))
            .transpose()
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let wanted = if report {
            let query = build_query(
                self.analysis.profile,
                None,
                views,
                include,
                exclude,
                min_size,
                max_size,
                modified_since,
                modified_before,
                kind,
                tags,
                not_tags,
                plane,
                &self.config.tags(),
                depth,
                limit_rows,
                sort,
                reverse,
                size,
                words_per_page,
                as_of_ns,
            )?;
            Some(fdu_core::ReportRequest { query, generated_at: SystemTime::now() })
        } else {
            None
        };
        let request = fdu_core::ReadRequest {
            children_of,
            children_page: fdu_core::ChildPageRequest {
                after,
                limit: row_bound(limit),
                plane: rows_plane,
            },
            rollups: rollups.unwrap_or_default(),
            total,
            extensions: ext_bound(extensions),
            report: wanted,
            expected: expected.map(parse_cursor).transpose()?,
        };
        // Detached for the whole native read. Every other native call here already
        // released the GIL; this one did not, and `fdu-samw` made it the heaviest of them
        // -- a filtered report re-aggregates the index, so holding the GIL across it
        // freezes every other Python thread for the length of a full traversal. An
        // asyncio consumer moving this to a worker thread would still have frozen its
        // event loop.
        let bundle = py.detach(|| self.inner.read(&request)).map_err(to_py_err)?;

        // Reacquired only for conversion, which is bounded by what the request asked for.
        // What it copies is counted: bytes crossing the binding are a real cost that no
        // engine-side counter can see, and an embedder comparing providers needs it.
        // Cleared, not merely read at the end: other calls on this thread also weigh what
        // they convert, and this count must belong to this call alone.
        let _ = payload::take();
        let converted = std::time::Instant::now();
        let out = PyDict::new(py);
        put_scalar(&out, "clock", bundle.clock.0)?;
        out.set_item("cursor", cursor_dict(py, bundle.cursor)?)?;
        put_text(&out, "root", bundle.root.as_os_str())?;
        put_scalar(&out, "entries", bundle.entries)?;
        put_text(&out, "freshness", freshness_label(bundle.freshness))?;
        put_text(&out, "phase", bundle.phase.as_str())?;
        put_text(&out, "coverage_reason", coverage_reason_label(bundle.coverage))?;
        out.set_item("scope", scope_dict(py, &bundle.scope)?)?;
        // From the bundle, so every field below describes the instant the rows were read.
        put_scalar(&out, "complete", bundle.run.complete)?;
        put_text(&out, "source", source_label(bundle.run.source))?;
        out.set_item("errors", error_list(py, &bundle.run.errors)?)?;
        put_nested(
            &out,
            "total",
            bundle.total.as_ref().map(|roll| rollup_dict(py, roll)).transpose()?,
        )?;
        let rollups = PyList::empty(py);
        for roll in &bundle.rollups {
            let converted = roll.as_ref().map(|roll| rollup_dict(py, roll)).transpose()?;
            if converted.is_none() {
                payload::scalars(1);
            }
            rollups.append(converted)?;
        }
        out.set_item("rollups", rollups)?;
        put_nested(
            &out,
            "children",
            bundle.children.as_ref().map(|page| child_page_dict(py, page)).transpose()?,
        )?;
        put_nested(
            &out,
            "report",
            bundle.report.as_ref().map(|rendered| report_dict(py, rendered)).transpose()?,
        )?;

        let work = work_dict(py, bundle.work)?;
        // The work record itself is telemetry about the call rather than payload, so it is
        // built last and not weighed: counting it would make the figure depend on how much
        // measurement the call did.
        work.set_item("binding_bytes", payload::take())?;
        // Stamped after every projection dict is built, including the ones below. The
        // first version stamped it before them and so excluded part of the same
        // conversion it claimed to measure.
        work.set_item("native_ns", bundle.work.wall_ns)?;
        // CPU is absent rather than inferred. Wall minus lock wait is not CPU time on a
        // preemptive system, and reporting it as such would put a number in the record
        // that nothing measured -- in the one field an embedder uses to compare engines.
        work.set_item("cpu_ns", py.None())?;
        out.set_item("work", work.clone())?;
        let projections = PyDict::new(py);
        projections.set_item("children", work_dict(py, bundle.projections.children)?)?;
        projections.set_item("total", work_dict(py, bundle.projections.total)?)?;
        projections.set_item("rollups", work_dict(py, bundle.projections.rollups)?)?;
        projections.set_item("report", work_dict(py, bundle.projections.report)?)?;
        out.set_item("projections", projections)?;
        work.set_item("conversion_ns", nanos_since(converted))?;
        Ok(out)
    }

    /// One page of a directory's children, in one call.
    ///
    /// Returns `None` when the path is absent or is not a directory — distinct from a
    /// page with no rows, which means a directory with no children.
    ///
    /// `after` resumes strictly past a name and `limit` bounds the rows, so a wide
    /// directory costs what is drawn rather than what it holds. Rows carry scalar subtree
    /// totals; ask `rollup()` for the extension breakdown of the one directory being
    /// inspected.
    #[pyo3(signature = (path = None, after = None, limit = None, plane = None))]
    fn children<'py>(
        &self,
        py: Python<'py>,
        path: Option<PathBuf>,
        after: Option<std::ffi::OsString>,
        limit: Option<usize>,
        plane: Option<&str>,
    ) -> PyResult<Option<Bound<'py, PyDict>>> {
        let path = path.unwrap_or_default();
        // One capture under one read lock, rather than a lookup per child per field:
        // `ChildSnapshot` already carries kind, attrs, totals and provenance together, so
        // a listing cannot see two different instants down its own rows.
        let request = fdu_core::ChildPageRequest {
            after,
            limit: row_bound(limit),
            plane: plane
                .map(|name| self.config.tags().plane_of(name))
                .transpose()
                .map_err(|error| PyValueError::new_err(error.to_string()))?,
        };
        let Some(page) = self.inner.children_page(&path, &request).map_err(to_py_err)? else {
            return Ok(None);
        };

        Ok(Some(child_page_dict(py, &page)?))
    }

    /// Provenance for one retained path, or `None` when it is absent.
    #[pyo3(signature = (path = None))]
    fn provenance<'py>(
        &self,
        py: Python<'py>,
        path: Option<PathBuf>,
    ) -> PyResult<Option<Bound<'py, PyDict>>> {
        let path = path.unwrap_or_default();
        let Some(provenance) = self.inner.provenance(&path).map_err(to_py_err)? else {
            return Ok(None);
        };
        Ok(Some(provenance_dict(py, provenance)?))
    }

    /// Reconcile the index against the filesystem and return what changed.
    ///
    /// This is the revalidation tier: unchanged entries cost a stat and nothing more,
    /// because an upsert whose complete observed state already matches is a no-op.
    ///
    /// `path` scopes the sweep to one subtree, which is what a caller ingesting hints
    /// from its own watcher wants: the cost is the subtree rather than the tree, and a
    /// missing or non-directory ancestor widens the scope rather than failing.
    #[pyo3(signature = (path = None))]
    fn refresh<'py>(&self, py: Python<'py>, path: Option<PathBuf>) -> PyResult<Bound<'py, PyDict>> {
        // The operation's start, which is the conservative watermark: a file modified
        // mid-sweep may have been observed before the modification.
        let started_at = Some(SystemTime::now());
        let config = self.config.clone();
        // A scoped refresh is the hint-ingestion primitive: a caller that keeps its own
        // watcher for a filesystem this build's backends cannot serve pushes each hint
        // through here, so every mutation still arrives through the one delta contract
        // rather than a second path into the index.
        let subtree = path.unwrap_or_default();
        // `reconcile_subtree_handle` takes the write lock per wave rather than for the
        // whole sweep, so a reader is served between waves instead of rejected for the
        // duration. This is the difference the bug was about.
        let report = py
            .detach(|| {
                fdu_core::scan::reconcile_subtree_handle(
                    &self.inner,
                    &subtree,
                    &config,
                    &mut |_| {},
                )
            })
            .map_err(to_py_err)?;
        let mut complete = report.scan.is_complete();
        let mut errors: Vec<fdu_core::Issue> =
            report.scan.errors.iter().map(fdu_core::Issue::from_error).collect();
        let mut analyzed = None;
        if self.analysis.profile.is_enabled() {
            let analysis = py.detach(|| self.inner.analyze(self.analysis)).map_err(to_py_err)?;
            let analysis_complete = analysis.is_complete();
            append_analysis_error(&mut errors, analysis);
            complete &= analysis_complete;
            analyzed = Some(analysis);
        }
        // Onto the index, under its own write guard, so the next read returns these facts
        // with the rows they describe rather than from a second lock a refresh can land
        // between. The binding keeps only telemetry, which is about this call rather than
        // about the index.
        self.inner
            .set_run_facts(fdu_core::query::RunFacts {
                scan_started_at: started_at,
                source: ReportSource::WarmRevalidate,
                complete,
                errors: errors.clone(),
            })
            .map_err(to_py_err)?;
        {
            let mut state = self.state();
            state.telemetry = PerformanceSummary::from_reconcile(
                &report.scan,
                analyzed,
                ReportSource::WarmRevalidate,
            );
        }
        let stats = report.apply;

        let out = PyDict::new(py);
        out.set_item("inserted", stats.inserted)?;
        out.set_item("updated", stats.updated)?;
        out.set_item("removed", stats.removed)?;
        out.set_item("unchanged", stats.unchanged)?;
        out.set_item("stale", stats.stale)?;
        out.set_item("error_count", errors.len())?;
        out.set_item("errors", error_list(py, &errors)?)?;
        out.set_item("source", source_label(ReportSource::WarmRevalidate))?;
        out.set_item("complete", complete)?;
        out.set_item("freshness", self.freshness()?)?;
        out.set_item("clock", self.inner.clock().map_err(to_py_err)?.0)?;
        Ok(out)
    }

    /// Changes applied after a cursor, and the cursor to resume from next.
    ///
    /// The cursor is `{session, clock}`, and both halves are load-bearing. `clock` says
    /// where to resume; `session` says whether resuming means anything -- a token held
    /// across a restart, or taken from a different opened root, is refused instead of
    /// answered with an empty list that reads as "nothing changed". Pass the whole dict
    /// back, not just the number.
    ///
    /// `truncated` is the other field that matters: when it is true the caller has fallen
    /// further behind than the retained journal and must re-read state instead of
    /// trusting the returned ops. Ignoring it is how an index silently diverges.
    ///
    /// `session` and `clock` come back together, from the same read that produced the
    /// ops. Sampling the clock separately let a commit land in between, and the resume
    /// position would then sit one past the last op returned -- skipping that commit
    /// permanently, with nothing in the result reporting the loss.
    #[pyo3(signature = (cursor))]
    fn since<'py>(
        &self,
        py: Python<'py>,
        cursor: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let since = self.inner.since(parse_cursor(cursor)?).map_err(to_py_err)?;
        let ops = PyList::empty(py);
        for delta in &since.deltas {
            for op in &delta.ops {
                let item = PyDict::new(py);
                item.set_item("clock", delta.clock.0)?;
                item.set_item("path", op.path().as_os_str())?;
                match op {
                    fdu_core::Op::Upsert { kind, attrs, .. } => {
                        item.set_item("op", "upsert")?;
                        item.set_item("kind", entry_kind_label(*kind))?;
                        item.set_item("bytes", attrs.size)?;
                        item.set_item("mtime_ns", attrs.mtime_ns)?;
                    }
                    fdu_core::Op::Remove { .. } => {
                        item.set_item("op", "remove")?;
                    }
                    fdu_core::Op::InvalidateSubtree { reason, .. } => {
                        item.set_item("op", "invalidate_subtree")?;
                        item.set_item("reason", format!("{reason:?}"))?;
                    }
                }
                ops.append(item)?;
            }
        }
        let state = PyList::empty(py);
        for delta in &since.deltas {
            for change in &delta.state {
                state.append(state_change_dict(py, delta.clock.0, change)?)?;
            }
        }

        let out = PyDict::new(py);
        out.set_item("truncated", since.truncated)?;
        out.set_item("cursor", cursor_dict(py, since.cursor)?)?;
        out.set_item("ops", ops)?;
        out.set_item("state", state)?;
        Ok(out)
    }

    /// The current resume position, as a cursor to pass to `since`.
    fn cursor<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        cursor_dict(py, self.inner.cursor().map_err(to_py_err)?)
    }
}

/// One resume position, as the dict `since` returns and accepts.
fn cursor_dict(py: Python<'_>, cursor: fdu_core::Cursor) -> PyResult<Bound<'_, PyDict>> {
    let out = PyDict::new(py);
    put_scalar(&out, "session", cursor.session.0)?;
    put_scalar(&out, "clock", cursor.clock.0)?;
    Ok(out)
}

/// Read a cursor a caller stored and handed back.
///
/// A bare integer is accepted as the start of *this* session, because that is the only
/// reading of it that cannot be wrong: an integer carries no session, and inventing one
/// would put the caller back where the session field was added to rescue them.
fn parse_cursor(value: &Bound<'_, PyAny>) -> PyResult<fdu_core::Cursor> {
    if let Ok(mapping) = value.cast::<PyDict>() {
        let session = mapping
            .get_item("session")?
            .ok_or_else(|| PyValueError::new_err("a cursor needs a 'session'"))?
            .extract::<u64>()?;
        let clock = mapping
            .get_item("clock")?
            .ok_or_else(|| PyValueError::new_err("a cursor needs a 'clock'"))?
            .extract::<u64>()?;
        return Ok(fdu_core::Cursor {
            session: fdu_core::SessionId(session),
            clock: fdu_core::Clock(clock),
        });
    }
    Err(PyValueError::new_err(
        "expected a cursor dict with 'session' and 'clock', as returned by Index.cursor() \
         or Index.since(); a bare integer cannot say which opened index it came from",
    ))
}

impl PyIndex {
    /// One directory's roll-up, in the whole subtree or in the plane the caller named.
    ///
    /// The one place either question is answered, so `total()` and `rollup()` cannot come
    /// to disagree about what naming a plane means -- including the part worth stating: an
    /// unpromoted or misspelled name is refused here, listing what this index does
    /// maintain, rather than quietly answering with the unrestricted totals.
    fn rollup_at<'py>(
        &self,
        py: Python<'py>,
        path: &Path,
        extensions: Option<usize>,
        plane: Option<&str>,
    ) -> PyResult<Option<Bound<'py, PyDict>>> {
        let roll = match plane {
            None => self.inner.rollup_bounded(path, ext_bound(extensions)).map_err(to_py_err)?,
            Some(name) => {
                let plane = self
                    .config
                    .tags()
                    .plane_of(name)
                    .map_err(|error| PyValueError::new_err(error.to_string()))?;
                self.inner
                    .with_index(|index| {
                        index.plane_rollup_bounded(path, plane, ext_bound(extensions))
                    })
                    .map_err(to_py_err)?
            }
        };
        match roll {
            Some(roll) => Ok(Some(rollup_dict(py, &roll)?)),
            None => Ok(None),
        }
    }

    /// The run-state lock.
    ///
    /// Poisoning is not recoverable state here, so it is unwrapped rather than
    /// surfaced: it would mean a panic inside a field assignment.
    fn state(&self) -> std::sync::MutexGuard<'_, RunState> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn error_messages(&self) -> Vec<String> {
        self.inner
            .with_index(|index| {
                index.run_facts().errors.iter().map(|issue| issue.message.clone()).collect()
            })
            .unwrap_or_default()
    }

    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    fn build_report(
        &self,
        views: Option<Vec<String>>,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        min_size: Option<&str>,
        max_size: Option<&str>,
        modified_since: Option<&str>,
        modified_before: Option<&str>,
        kind: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        not_tags: Option<Vec<String>>,
        plane: Option<&str>,
        depth: Option<&str>,
        limit: Option<&str>,
        sort: Option<&str>,
        reverse: bool,
        size: &str,
        words_per_page: u64,
        as_of_ns: Option<i64>,
    ) -> PyResult<Report> {
        // `now` is the report's own generated_at as well as the clock the time bounds are
        // resolved against, so both come from one reading rather than two.
        let now = SystemTime::now();
        let query = build_query(
            self.analysis.profile,
            None,
            views,
            include,
            exclude,
            min_size,
            max_size,
            modified_since,
            modified_before,
            kind,
            tags,
            not_tags,
            plane,
            &self.config.tags(),
            depth,
            limit,
            sort,
            reverse,
            size,
            words_per_page,
            as_of_ns,
        )?;
        // Read in place rather than cloning: `report` is a pure reader, so it runs under
        // the shared lock. Snapshotting here instead would copy every entry per call,
        // which is O(entries) on a path a consumer calls per navigation.
        //
        // The envelope is built *inside* the guard, off the same index the rows come
        // from. Sampling it outside let a refresh land between the two, so a report could
        // pair one instant's numbers with another instant's claim about how far to trust
        // them -- both individually true, and the disagreement invisible.
        self.inner
            .with_index(|index| {
                let provenance = index.run_facts().provenance(now);
                fdu_core::query::report(index, &query, &provenance)
            })
            .map_err(to_py_err)
    }
}

/// Translate the traversal-order grammar into its library value.
///
/// The same spellings the command line accepts, because a capability reachable by flag
/// has to be reachable as one typed call in the same vocabulary.
fn parse_scan_order(value: Option<&str>) -> PyResult<fdu_core::ScanOrder> {
    let Some(value) = value else {
        return Ok(fdu_core::ScanOrder::default());
    };
    match value.trim().to_ascii_lowercase().replace('_', "-").as_str() {
        "breadth-first" | "breadth" | "bfs" => Ok(fdu_core::ScanOrder::BreadthFirst),
        "depth-first" | "depth" | "dfs" => Ok(fdu_core::ScanOrder::DepthFirst),
        other => Err(PyValueError::new_err(format!(
            "unknown order {other}; expected breadth-first or depth-first"
        ))),
    }
}

/// Translate a cache-policy string into its library value.
///
/// The same spellings the CLI accepts, so a capability reachable by flag is reachable by
/// one typed call rather than by shelling out.
fn parse_cache_policy(value: &str) -> PyResult<CachePolicy> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Ok(CachePolicy::Auto),
        "refresh" => Ok(CachePolicy::Refresh),
        "read-only" | "readonly" => Ok(CachePolicy::ReadOnly),
        "only" => Ok(CachePolicy::Only),
        "off" => Ok(CachePolicy::Off),
        other => Err(PyValueError::new_err(format!(
            "invalid cache policy {other:?}: expected one of auto, refresh, read-only, only, off"
        ))),
    }
}

fn parse_analysis_request(profile: &str, workers: usize) -> PyResult<AnalysisRequest> {
    let profile = AnalysisSet::parse(profile).map_err(PyValueError::new_err)?;
    Ok(AnalysisRequest { profile, workers })
}

fn append_analysis_error(
    errors: &mut Vec<fdu_core::Issue>,
    analysis: fdu_core::content::AnalysisReport,
) {
    if let Some(message) = analysis.failure_message() {
        // Analysis runs above the engine, which therefore has no view of its failures --
        // but they make a result partial in exactly the same way, so they arrive in the
        // same vocabulary rather than a second one beside it.
        errors.push(fdu_core::Issue::reported(fdu_core::IssueKind::ProviderFailure, message));
    }
}

/// The typed issues a result carries, rendered for Python.
fn error_list<'py>(py: Python<'py>, errors: &[fdu_core::Issue]) -> PyResult<Bound<'py, PyList>> {
    let list = PyList::empty(py);
    for error in errors {
        let item = PyDict::new(py);
        put_text(&item, "path", error.path.as_deref().map(std::path::Path::as_os_str))?;
        put_text(&item, "kind", error.kind.as_str())?;
        put_text(&item, "message", error.message.as_str())?;
        put_scalar(&item, "os_error", error.os_error)?;
        list.append(item)?;
    }
    Ok(list)
}

fn status_dict<'py>(py: Python<'py>, index: &PyIndex) -> PyResult<Bound<'py, PyDict>> {
    let status = PyDict::new(py);
    // One guard for all four, including freshness: a status envelope whose fields came
    // from different instants is exactly the thing this describes itself as not being.
    let (complete, source, errors, freshness) = index
        .inner
        .with_index(|inner| {
            let run = inner.run_facts();
            (run.complete, run.source, run.errors.clone(), inner.freshness())
        })
        .map_err(to_py_err)?;
    status.set_item("complete", complete)?;
    status.set_item("freshness", freshness_label(freshness))?;
    status.set_item("source", source_label(source))?;
    status.set_item("errors", error_list(py, &errors)?)?;
    Ok(status)
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

/// Why coverage is partial, or `None` when it is complete.
///
/// Separate from the status label rather than folded into it, because a consumer that
/// only branches on complete-or-not should not have to learn six new strings to keep
/// working.
fn coverage_reason_label(status: fdu_core::Status) -> Option<&'static str> {
    let fdu_core::Status::Partial(reason) = status else {
        return None;
    };
    Some(match reason {
        fdu_core::CoverageReason::Building => "building",
        fdu_core::CoverageReason::Budget => "budget",
        fdu_core::CoverageReason::Cancelled => "cancelled",
        fdu_core::CoverageReason::Inaccessible => "inaccessible",
        _ => "failed",
    })
}

/// Why a coverage reason is partial, as a stable label.
fn reason_label(reason: fdu_core::CoverageReason) -> &'static str {
    coverage_reason_label(fdu_core::Status::Partial(reason)).unwrap_or("failed")
}

/// One committed state transition, rendered for a consumer.
///
/// The same shape wherever it appears -- resumed from the journal or delivered by a live
/// batch -- because it is the same fact. `paths` is a list rather than a scalar so a
/// re-tag naming many governed directories and a sweep naming one subtree read the same
/// way, and an index-wide transition is the empty list rather than a special case.
fn state_change_dict<'py>(
    py: Python<'py>,
    clock: u64,
    change: &fdu_core::StateChange,
) -> PyResult<Bound<'py, PyDict>> {
    let item = PyDict::new(py);
    item.set_item("clock", clock)?;
    let (transition, freshness, reason) = match change {
        fdu_core::StateChange::Verified { .. } => ("verified", None, None),
        fdu_core::StateChange::Freshness { freshness, reason, .. } => {
            ("freshness", Some(freshness_label(*freshness)), reason.map(reason_label))
        }
        fdu_core::StateChange::RunFacts => ("run_facts", None, None),
        fdu_core::StateChange::Phase { .. } => ("phase", None, None),
        fdu_core::StateChange::Retagged { .. } => ("retagged", None, None),
    };
    put_text(&item, "transition", transition)?;
    put_text(&item, "freshness", freshness)?;
    put_text(&item, "reason", reason)?;
    let paths: Vec<OsString> =
        change.paths().iter().map(|path| path.as_os_str().to_os_string()).collect();
    for path in &paths {
        path.as_os_str().charge();
    }
    item.set_item("paths", paths)?;
    Ok(item)
}

fn provenance_dict(
    py: Python<'_>,
    provenance: fdu_core::Provenance,
) -> PyResult<Bound<'_, PyDict>> {
    let value = PyDict::new(py);
    put_text(&value, "source", value_source_label(provenance.source))?;
    put_scalar(&value, "observed_at_ns", provenance.observed_at_ns)?;
    put_text(&value, "status", coverage_label_value(provenance.status))?;
    put_text(&value, "reason", coverage_reason_label(provenance.status))?;
    Ok(value)
}

/// Render one run's telemetry, field for field with the command line's footer.
///
/// Nanoseconds rather than seconds because the caller is measuring: a float of seconds
/// is the shape that loses the short intervals an interactive loop is made of.
fn telemetry_dict(py: Python<'_>, telemetry: PerformanceSummary) -> PyResult<Bound<'_, PyDict>> {
    let value = PyDict::new(py);
    value.set_item("walked_files", telemetry.walked_files)?;
    value.set_item("walked_bytes", telemetry.walked_bytes)?;
    value.set_item("fresh_files", telemetry.fresh_files)?;
    value.set_item("bytes_read", telemetry.bytes_read)?;
    value.set_item("analysis_ns", telemetry.analysis_ns)?;
    value.set_item("cached_files", telemetry.cached_files)?;
    value.set_item("cached_bytes", telemetry.cached_bytes)?;
    value.set_item("source", source_label(telemetry.source))?;
    Ok(value)
}

/// Name a cache tier for Python callers, matching the CLI's machine output.
fn source_label(source: fdu_core::query::ReportSource) -> &'static str {
    match source {
        fdu_core::query::ReportSource::ColdScan => "cold_scan",
        fdu_core::query::ReportSource::WarmRevalidate => "warm_revalidate",
        fdu_core::query::ReportSource::CacheOnly => "cache_only",
    }
}

/// Convert a parsed time bound to index nanoseconds, or raise.
///
/// Mirrors the CLI: an instant the index cannot represent must be rejected, never stored
/// as an absent bound, or the query silently runs with no time filter at all.
fn bound_nanos(input: &str, when: std::time::SystemTime, field: &str) -> PyResult<i64> {
    fdu_core::query::system_time_to_nanos(when).ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err(format!(
            "invalid {field} {input:?}: that time is outside the range fdu can represent \
             (about 1677 to 2262)"
        ))
    })
}

/// Convert a report into the dict shape Python callers get.
fn report_dict<'py>(py: Python<'py>, report: &Report) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    // The envelope identity, from the same two functions the serializers use, so a dict
    // and a rendered document cannot claim different schemas or different producers.
    put_text(&dict, "schema", fdu_core::report_format::report_schema(report))?;
    put_text(&dict, "generator", fdu_core::report_format::generator())?;
    put_text(&dict, "root", report.root.as_os_str())?;
    put_scalar(&dict, "complete", report.complete)?;
    for error in &report.errors {
        error.as_str().charge();
    }
    dict.set_item("errors", report.errors.clone())?;
    put_text(&dict, "source", source_label(report.source))?;
    put_text(&dict, "freshness", freshness_label(report.freshness))?;
    put_text(&dict, "generated_at", fdu_core::query::format_rfc3339(report.generated_at).as_str())?;
    let scan_started_at = report.scan_started_at.map(fdu_core::query::format_rfc3339);
    put_text(&dict, "scan_started_at", scan_started_at.as_deref())?;
    match report.analysis.as_ref() {
        None => put_scalar(&dict, "analysis", py.None())?,
        Some(analysis) => {
            let metadata = PyDict::new(py);
            let profile = analysis_set_labels(analysis.profile);
            for label in &profile {
                (*label).charge();
            }
            metadata.set_item("analyze", profile)?;
            put_scalar(
                &metadata,
                "type_rules_fingerprint",
                analysis.provenance.type_rules_fingerprint,
            )?;
            put_scalar(
                &metadata,
                "options_fingerprint",
                analysis.provenance.options_fingerprint.0,
            )?;
            let analyzers = PyList::empty(py);
            for (id, version) in &analysis.provenance.analyzers {
                let analyzer = PyDict::new(py);
                put_text(&analyzer, "id", id.0)?;
                put_scalar(&analyzer, "version", version.0)?;
                analyzers.append(analyzer)?;
            }
            metadata.set_item("analyzers", analyzers)?;
            dict.set_item("analysis", metadata)?;
        }
    }

    let sections = PyList::empty(py);
    for section in &report.sections {
        let entry = PyDict::new(py);
        match section {
            Section::Summary(row) => {
                put_text(&entry, "view", "summary")?;
                entry.set_item("summary", summary_dict(py, row)?)?;
            }
            Section::Extensions { rows, total } => {
                put_text(&entry, "view", "extensions")?;
                put_nested(&entry, "bound", bound_dict(py, rows.len(), *total)?)?;
                let list = PyList::empty(py);
                for row in rows {
                    let item = PyDict::new(py);
                    put_text(&item, "extension", row.extension.as_str())?;
                    put_scalar(&item, "files", row.files)?;
                    put_scalar(&item, "bytes", row.bytes)?;
                    put_scalar(&item, "allocated", row.allocated)?;
                    list.append(item)?;
                }
                entry.set_item("extensions", list)?;
            }
            Section::Groups { rows, total } => {
                put_text(&entry, "view", "groups")?;
                put_nested(&entry, "bound", bound_dict(py, rows.len(), *total)?)?;
                let list = PyList::empty(py);
                for row in rows {
                    let item = PyDict::new(py);
                    put_text(&item, "id", row.id.as_str())?;
                    put_text(&item, "label", row.label.as_str())?;
                    put_scalar(&item, "files", row.files)?;
                    put_scalar(&item, "bytes", row.bytes)?;
                    put_scalar(&item, "allocated", row.allocated)?;
                    list.append(item)?;
                }
                entry.set_item("groups", list)?;
            }
            Section::Metrics { view, summary } => {
                put_text(&entry, "view", view.label())?;
                entry.set_item("metrics", metric_summary_dict(py, summary)?)?;
            }
            Section::Files { view, rows, total } => {
                put_text(&entry, "view", view.label())?;
                put_nested(&entry, "bound", bound_dict(py, rows.len(), *total)?)?;
                let list = PyList::empty(py);
                for row in rows {
                    let item = PyDict::new(py);
                    put_text(&item, "path", row.path.as_os_str())?;
                    put_text(&item, "kind", entry_kind_label(row.kind))?;
                    put_scalar(&item, "bytes", row.bytes)?;
                    put_scalar(&item, "allocated", row.allocated)?;
                    put_scalar(&item, "mtime_ns", row.mtime_ns)?;
                    for tag in &row.tags {
                        tag.as_str().charge();
                    }
                    item.set_item("tags", row.tags.as_slice())?;
                    put_text(&item, "extension", row.extension.as_deref())?;
                    put_nested(
                        &item,
                        "classification",
                        row_classification_dict(
                            py,
                            row.classification.as_ref(),
                            row.group.as_deref(),
                        )?,
                    )?;
                    list.append(item)?;
                }
                entry.set_item("files", list)?;
            }
            Section::Tree(root) => {
                put_text(&entry, "view", "tree")?;
                entry.set_item("tree", tree_dict(py, root)?)?;
            }
        }
        sections.append(entry)?;
    }
    dict.set_item("reports", sections)?;
    Ok(dict)
}

fn metric_summary_dict<'py>(
    py: Python<'py>,
    summary: &MetricSummary,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    put_text(
        &dict,
        "group",
        match summary.group {
            fdu_core::query::MetricGroup::Type => "type",
            fdu_core::query::MetricGroup::Family => "family",
        },
    )?;
    put_text(&dict, "share_metric", summary.share_metric.as_str())?;
    put_scalar(&dict, "words_per_page", summary.words_per_page)?;
    dict.set_item("total", metric_row_dict(py, &summary.total, summary.words_per_page)?)?;
    let rows = PyList::empty(py);
    for row in &summary.rows {
        rows.append(metric_row_dict(py, row, summary.words_per_page)?)?;
    }
    dict.set_item("rows", rows)?;
    Ok(dict)
}

fn metric_row_dict<'py>(
    py: Python<'py>,
    row: &MetricRow,
    words_per_page: u64,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    put_text(&dict, "id", row.id.as_str())?;
    put_text(&dict, "family", row.family.as_str())?;
    put_scalar(&dict, "files", row.files)?;
    put_scalar(&dict, "bytes", row.bytes)?;
    put_scalar(&dict, "allocated", row.allocated)?;
    put_scalar(&dict, "analyzed_files", row.analyzed_files)?;
    put_scalar(&dict, "share_numerator", row.share.numerator)?;
    put_scalar(&dict, "share_denominator", row.share.denominator)?;
    put_scalar(&dict, "physical_lines", row.metrics.physical_lines)?;
    put_scalar(&dict, "blank_lines", row.metrics.blank_lines)?;
    put_scalar(&dict, "nonblank_lines", row.metrics.nonblank_lines)?;
    put_scalar(&dict, "code_lines", row.metrics.code_lines)?;
    put_scalar(&dict, "comment_lines", row.metrics.comment_lines)?;
    put_scalar(&dict, "code_blank_lines", row.metrics.code_blank_lines)?;
    put_scalar(&dict, "raw_words", row.metrics.raw_words)?;
    put_scalar(&dict, "logical_words", row.metrics.logical_word_stats.logical_words())?;
    put_scalar(&dict, "paragraphs", row.metrics.paragraphs)?;
    put_scalar(&dict, "visible_words", row.metrics.visible_words)?;
    put_scalar(
        &dict,
        "visible_logical_words",
        row.metrics.visible_logical_word_stats.logical_words(),
    )?;
    put_scalar(&dict, "document_words", document_words(row))?;
    put_scalar(&dict, "page_words", document_words(row))?;
    put_scalar(&dict, "words_per_page", words_per_page)?;
    // Three maps keyed by which verdicts this row actually saw, so their keys vary with
    // the answer and are payload like any other value.
    let coverage = PyDict::new(py);
    for (reason, count) in &row.coverage {
        coverage_label(*reason).charge();
        put_scalar(&coverage, coverage_label(*reason), count)?;
    }
    dict.set_item("coverage", coverage)?;
    let detection = PyDict::new(py);
    let sources = PyDict::new(py);
    for (source, count) in &row.detection_sources {
        source.as_str().charge();
        put_scalar(&sources, source.as_str(), count)?;
    }
    detection.set_item("sources", sources)?;
    let confidence = PyDict::new(py);
    for (level, count) in &row.detection_confidence {
        level.as_str().charge();
        put_scalar(&confidence, level.as_str(), count)?;
    }
    detection.set_item("confidence", confidence)?;
    let flags = PyDict::new(py);
    put_scalar(&flags, "generated", row.generated_files)?;
    put_scalar(&flags, "vendored", row.vendored_files)?;
    put_scalar(&flags, "documentation", row.documentation_files)?;
    detection.set_item("flags", flags)?;
    dict.set_item("detection", detection)?;
    Ok(dict)
}

/// What a section dropped, or `None` when it dropped nothing.
///
/// `None` rather than an absent key, so a consumer branches on the value instead of on
/// presence — the same shape the JSON and YAML forms use.
fn bound_dict(py: Python<'_>, shown: usize, total: usize) -> PyResult<Option<Bound<'_, PyDict>>> {
    if shown >= total {
        return Ok(None);
    }
    let bound = PyDict::new(py);
    put_scalar(&bound, "shown", shown)?;
    put_scalar(&bound, "total", total)?;
    Ok(Some(bound))
}

fn coverage_label(reason: CoverageReason) -> &'static str {
    match reason {
        CoverageReason::Analyzed => "analyzed",
        CoverageReason::Binary => "binary",
        CoverageReason::InvalidUtf8 => "invalid_utf8",
        CoverageReason::Unsupported => "unsupported",
        CoverageReason::IoError => "io_error",
        CoverageReason::ChangedDuringRead => "changed_during_read",
    }
}

fn analysis_set_labels(profile: AnalysisSet) -> Vec<&'static str> {
    profile.labels()
}

/// One summary row as a dict.
fn summary_dict<'py>(py: Python<'py>, row: &SummaryRow) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    put_scalar(&dict, "files", row.files)?;
    put_scalar(&dict, "dirs", row.dirs)?;
    put_scalar(&dict, "others", row.others)?;
    put_scalar(&dict, "bytes", row.bytes)?;
    put_scalar(&dict, "allocated", row.allocated)?;
    put_scalar(&dict, "newest_mtime_ns", row.newest_mtime_ns)?;
    Ok(dict)
}

/// What a tree node's bound withheld, or `None` when it withheld nothing.
///
/// Presence is the signal, matching the section-level bound: a consumer branches on it
/// rather than comparing counts it would have to know to compare.
fn remainder_dict(
    py: Python<'_>,
    remainder: Option<fdu_core::query::Remainder>,
) -> PyResult<Option<Bound<'_, PyDict>>> {
    let Some(remainder) = remainder else {
        return Ok(None);
    };
    let value = PyDict::new(py);
    put_scalar(&value, "rows", remainder.rows)?;
    put_scalar(&value, "files", remainder.files)?;
    put_scalar(&value, "dirs", remainder.dirs)?;
    put_scalar(&value, "others", remainder.others)?;
    put_scalar(&value, "bytes", remainder.bytes)?;
    put_scalar(&value, "allocated", remainder.allocated)?;
    Ok(Some(value))
}

/// One tree node as a nested dict.
///
/// Built with an explicit stack: the tree can be deeper than the interpreter's recursion
/// budget, and a report that aborts on a deep tree fails where it is most useful.
fn tree_dict<'py>(py: Python<'py>, root: &TreeNode) -> PyResult<Bound<'py, PyDict>> {
    fn node_dict<'py>(py: Python<'py>, node: &TreeNode) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        put_text(&dict, "name", node.name.as_str())?;
        put_text(&dict, "path", node.path.as_os_str())?;
        put_scalar(&dict, "bytes", node.bytes)?;
        put_scalar(&dict, "allocated", node.allocated)?;
        put_scalar(&dict, "files", node.files)?;
        put_scalar(&dict, "dirs", node.dirs)?;
        put_scalar(&dict, "others", node.others)?;
        put_scalar(&dict, "newest_mtime_ns", node.newest_mtime_ns)?;
        put_scalar(&dict, "truncated", node.truncated())?;
        put_nested(&dict, "remainder", remainder_dict(py, node.remainder)?)?;
        // Filled by the caller's traversal; each child charges itself as it is built.
        dict.set_item("children", PyList::empty(py))?;
        Ok(dict)
    }

    let built = node_dict(py, root)?;
    let mut stack: Vec<(&TreeNode, Bound<'py, PyDict>)> = vec![(root, built.clone())];
    while let Some((node, dict)) = stack.pop() {
        let children = dict.get_item("children")?.expect("children present");
        let children: Bound<'py, PyList> =
            children.cast_into().map_err(|_| PyValueError::new_err("children is not a list"))?;
        for child in &node.children {
            let child_dict = node_dict(py, child)?;
            children.append(child_dict.clone())?;
            stack.push((child, child_dict));
        }
    }
    Ok(built)
}

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

/// Resolve a caller's view list, expanding the `full` total.
///
/// `full` names the whole report rather than one projection, so it cannot be combined.
/// The Python `View` enum offers it, so the binding must honour it — it previously listed
/// `full` as valid in its error message while rejecting it.
fn resolve_views(
    values: &[String],
    analysis: AnalysisSet,
) -> PyResult<(Vec<ViewSpec>, Vec<ViewSpec>)> {
    // The binding used to carry its own copy of this axis: the `full` expansion, the
    // exclusivity rule, and the default. Each drifted -- the exclusivity message lost a
    // clause (fdu-gw5b), the view order fell out of step (fdu-ggux), and the list grammar
    // was never here at all, so ["tree", "tree"] was a silent no-op where the CLI calls it
    // a typo (fdu-jozr). One resolver now, named as the Python API spells the axis.
    let spec = values.join(",");
    ViewSpec::resolve(Some(&spec), analysis, "view").map_err(PyValueError::new_err)
}

/// Parse an entry-kind name.
fn parse_kind(value: &str) -> PyResult<EntryKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "file" => Ok(EntryKind::File),
        "dir" => Ok(EntryKind::Dir),
        "symlink" => Ok(EntryKind::Symlink),
        "other" => Ok(EntryKind::Other),
        other => Err(PyValueError::new_err(format!(
            "invalid kind {other:?}: expected one of file, dir, symlink, other"
        ))),
    }
}

/// Parse a sort key.
fn parse_sort(value: &str) -> PyResult<SortKey> {
    match value.trim().to_ascii_lowercase().as_str() {
        "size" => Ok(SortKey::Size),
        "count" => Ok(SortKey::Count),
        "mtime" => Ok(SortKey::Mtime),
        "name" => Ok(SortKey::Name),
        other => Err(PyValueError::new_err(format!(
            "invalid sort {other:?}: expected one of size, count, mtime, name"
        ))),
    }
}

/// Parse a size metric.
fn parse_size_metric(value: &str) -> PyResult<SizeMetric> {
    match value.trim().to_ascii_lowercase().as_str() {
        "allocated" => Ok(SizeMetric::Allocated),
        "apparent" => Ok(SizeMetric::Apparent),
        other => Err(PyValueError::new_err(format!(
            "invalid size {other:?}: expected allocated or apparent"
        ))),
    }
}

/// Parse a bound that accepts `all`.
fn parse_bound(value: &str, name: &str) -> PyResult<Bound_> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("all") {
        return Ok(Bound_::All);
    }
    value.parse::<usize>().map(Bound_::Limit).map_err(|_| {
        PyValueError::new_err(format!("invalid {name} {value:?}: expected a whole number or `all`"))
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
    /// The live answer, as of now.
    ///
    /// A watch run has no final answer: the aggregates are only true until the next
    /// change, so a caller redrawing them needs what the session has applied. That used to
    /// require reaching for the session's *own* index, because a watch held a private copy
    /// and the opened index went stale at the first event -- a display that looked live and
    /// was not (`fdu-m66a`). A session now watches the index it was opened from, so the
    /// distinction is gone: this and `Index.read` are two views of one authority, and both
    /// are current.
    ///
    /// Returns a snapshot, so rendering it twice gives the same answer both times.
    fn report(&self) -> PyResult<PyOneShot> {
        let session =
            self.session.as_ref().ok_or_else(|| PyRuntimeError::new_err("this watch is closed"))?;
        let provenance = session.live_provenance(SystemTime::now());
        let report = session.report(&provenance).map_err(to_py_err)?;
        Ok(PyOneShot { report })
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Wait for the next batch, yielding one immutable record of what happened.
    ///
    /// A batch, not a list of changes. The changes alone are lossy: a mutation the
    /// selection filters out still moves the totals a tree view reports, so a consumer
    /// that re-rendered on a non-empty list would miss it entirely -- and the dirty set
    /// used to sit beside the feed as mutable state, readable only if you knew to look
    /// between iteration steps. Everything a consumer must not miss now travels in the
    /// value it already receives.
    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<Py<PyDict>>> {
        let Some(session) = self.session.as_mut() else {
            // A closed feed is exhausted rather than an error, so `for ... in` ends
            // cleanly after close().
            return Ok(None);
        };

        // The GIL is released for the whole wait: this blocks for up to `timeout`, and
        // holding the GIL across it would freeze every other Python thread.
        let batch = py.detach(|| session.next_batch(self.timeout)).map_err(to_py_err)?;

        let out = PyDict::new(py);
        let list = PyList::empty(py);
        let state = PyList::empty(py);
        let mut issues = PyList::empty(py);
        let mut dirty_queries: Vec<&'static str> = Vec::new();
        let mut batch_work = work_dict(py, fdu_core::Work::default())?;
        let mut dirty = false;
        let mut all_dirty = false;
        let mut reset = false;
        let mut dirty_rollups: Vec<OsString> = Vec::new();
        let mut cursor = None;
        if let Some(batch) = batch {
            dirty = batch.dirty;
            all_dirty = batch.all_dirty;
            reset = batch.reset;
            dirty_rollups =
                batch.dirty_rollups.iter().map(|path| path.as_os_str().to_os_string()).collect();
            cursor = batch.cursor;
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
                list.append(dict)?;
            }
            issues = error_list(py, &batch.issues)?;
            dirty_queries =
                batch.dirty_queries.iter().map(|kind| fdu_core::QueryKind::as_str(*kind)).collect();
            batch_work = work_dict(py, batch.work)?;
            for committed in &batch.state {
                // The clock the transition committed at, not the batch's terminal one.
                // Stamping them all with the end said every transition happened last, which
                // is both false and unorderable against the changes beside them.
                state.append(state_change_dict(py, committed.clock.0, &committed.change)?)?;
            }
        }
        out.set_item("changes", list)?;
        out.set_item("state", state)?;
        out.set_item("issues", issues)?;
        out.set_item("dirty_queries", dirty_queries)?;
        out.set_item("work", batch_work)?;
        out.set_item("dirty", dirty)?;
        out.set_item("dirty_rollups", dirty_rollups)?;
        out.set_item("all_dirty", all_dirty)?;
        out.set_item("reset", reset)?;
        out.set_item(
            "cursor",
            match cursor {
                Some(cursor) => Some(cursor_dict(py, cursor)?),
                None => None,
            },
        )?;
        Ok(Some(out.unbind()))
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

/// Resolve tag-rule names into the set an index will evaluate.
///
/// `None` and an empty list are the same request -- no rules -- and both fingerprint to
/// zero, so a caller who does not ask for tags keeps every snapshot they already have.
/// Resolve the hidden-path admission rule a caller asked for.
///
/// `None` and `"keep"` are the same request -- admit everything -- and both fingerprint to
/// zero, so a caller who does not ask keeps every snapshot they already have. An allowlist
/// without pruning is refused rather than ignored: it can only have been written by someone
/// who believed pruning was on.
fn admission_policy(
    hidden: Option<&str>,
    hidden_allow: Option<Vec<String>>,
) -> PyResult<Option<Arc<fdu_core::HiddenPolicy>>> {
    let allow = hidden_allow.unwrap_or_default();
    let policy = fdu_core::admission::parse_policy(hidden.unwrap_or("keep"), allow)
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    Ok(policy.map(Arc::new))
}

fn enabled_tag_rules(
    names: Option<Vec<String>>,
    promote: Option<Vec<String>>,
) -> PyResult<Arc<fdu_core::tags::TagRules>> {
    let rules = match names {
        None => fdu_core::tags::TagRules::none().clone(),
        Some(names) => fdu_core::tags::TagRules::from_names(names)
            .map_err(|error| PyValueError::new_err(error.to_string()))?,
    };
    // Promotion belongs to the same set for the same reason enabling does: it moves the
    // fingerprint the snapshot records, and it decides which names a plane read accepts.
    // Applying it anywhere else would let an index be written under one answer to those
    // two questions and read under another. `promote` without `tag_rules` is refused by
    // the library against the empty set, which says so exactly.
    let Some(promote) = promote else {
        return Ok(Arc::new(rules));
    };
    let rules =
        rules.with_promoted(promote).map_err(|error| PyValueError::new_err(error.to_string()))?;
    Ok(Arc::new(rules))
}

/// Build a query from the keyword arguments both report paths accept.
///
/// Extracted so the session path and the one-shot cannot disagree about what a request
/// means. Every value grammar here belongs to the library and is called, not restated.
#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
/// A caller-supplied epoch instant as a `SystemTime`.
///
/// Signed nanoseconds, because that is what every other time this binding exchanges is,
/// and pre-epoch values are representable rather than an error nobody expected.
fn epoch_nanos_to_time(nanos: i64) -> PyResult<SystemTime> {
    let magnitude = std::time::Duration::from_nanos(nanos.unsigned_abs());
    let at = if nanos < 0 {
        SystemTime::UNIX_EPOCH.checked_sub(magnitude)
    } else {
        SystemTime::UNIX_EPOCH.checked_add(magnitude)
    };
    at.ok_or_else(|| {
        PyValueError::new_err(format!("as_of is not a representable instant: {nanos}"))
    })
}

// One function assembling one query from the whole grammar. Splitting it would move
// arguments into a struct that exists only to be destructured here, which is the same
// count spelled differently.
#[allow(clippy::too_many_arguments)]
fn build_query(
    profile: AnalysisSet,
    default_view: Option<ViewSpec>,
    views: Option<Vec<String>>,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    min_size: Option<&str>,
    max_size: Option<&str>,
    modified_since: Option<&str>,
    modified_before: Option<&str>,
    kind: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    not_tags: Option<Vec<String>>,
    plane: Option<&str>,
    rules: &fdu_core::tags::TagRules,
    depth: Option<&str>,
    limit: Option<&str>,
    sort: Option<&str>,
    reverse: bool,
    size: &str,
    words_per_page: u64,
    as_of_ns: Option<i64>,
) -> PyResult<Query> {
    // The caller's reference instant, or this one. A pinned read fixes the tree and not
    // the clock, so a relative bound resolved afresh on every page let a version-pinned
    // assembly change membership while its version stood still -- and nothing reported it,
    // because the version genuinely had not moved.
    let now = match as_of_ns {
        Some(nanos) => epoch_nanos_to_time(nanos)?,
        None => SystemTime::now(),
    };
    let mut selection =
        Selection { reverse, size: parse_size_metric(size)?, ..Selection::default() };
    if let Some(value) = depth {
        selection.depth = Some(parse_bound(value, "depth")?);
    }
    if let Some(value) = limit {
        selection.limit = Some(parse_bound(value, "limit")?);
    }
    for pattern in include.unwrap_or_default() {
        selection.include.push(Pattern::parse(&pattern).map_err(to_py_err)?);
    }
    for pattern in exclude.unwrap_or_default() {
        selection.exclude.push(Pattern::parse(&pattern).map_err(to_py_err)?);
    }
    if let Some(value) = max_size {
        selection.max_size = Some(fdu_core::query::parse_size(value).map_err(to_py_err)?);
    }
    if let Some(value) = min_size {
        selection.min_size = Some(fdu_core::query::parse_size(value).map_err(to_py_err)?);
    }
    if let Some(value) = modified_since {
        let at = fdu_core::query::parse_when(value, now).map_err(to_py_err)?;
        selection.modified.since = Some(bound_nanos(value, at, "modified_since")?);
    }
    if let Some(value) = modified_before {
        let at = fdu_core::query::parse_when(value, now).map_err(to_py_err)?;
        selection.modified.before = Some(bound_nanos(value, at, "modified_before")?);
    }
    for value in kind.unwrap_or_default() {
        selection.kinds.push(parse_kind(&value)?);
    }
    // Resolved against the set this index evaluates, not the catalogue: naming a rule that
    // is off is refused, because a mask of zero reads as "no constraint" and the caller
    // would get every entry back believing they had narrowed.
    selection.tags.any = rules
        .mask_of(tags.unwrap_or_default())
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    selection.tags.none = rules
        .mask_of(not_tags.unwrap_or_default())
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    // Against the same set, for the stricter of the two reasons: a plane is a bit position
    // in *this* rule set's order, so one resolved elsewhere would name a real plane here
    // and mean a different tag.
    selection.plane = plane
        .map(|name| rules.plane_of(name))
        .transpose()
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    if let Some(value) = sort {
        selection.sort = Some(parse_sort(value)?);
    }
    let (views, omitted_views) = match views {
        Some(values) => resolve_views(&values, profile)?,
        // Derived, not fixed: a request that paid to read files must display what it read,
        // which is what the CLI has done since the content axis landed. A fixed `tree` here
        // reproduced the defect that axis removed.
        //
        // A watch stream is the one caller with a fixed answer: it emits per-entry records,
        // so `files` is what it means regardless of analyzers. Passed in rather than
        // derived, because sharing this builder silently turned that into a tree.
        None => (vec![default_view.unwrap_or_else(|| ViewSpec::default_for(profile))], Vec::new()),
    };
    if words_per_page == 0 {
        return Err(PyValueError::new_err("words_per_page must be positive"));
    }
    // The Python API names these axes with fields, so its diagnostics do too: there is no
    // `--analyze` for a caller here to add (fdu-4apt).
    let query = Query { selection, views, omitted_views, axes: AxisNames::FIELDS, words_per_page };
    query.validate_analysis(profile).map_err(PyValueError::new_err)?;
    Ok(query)
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
/// An unfiltered summary is answered by a transient tier that retains nothing, so writing
/// a snapshot for it caches state the walk never saved -- and a Python caller therefore
/// left cache state on a tree that the same command would not have, which a later
/// cache-only read could see (fdu-4msv).
#[pyfunction]
#[pyo3(signature = (
    root,
    *,
    cache = "auto",
    max_depth = None,
    one_filesystem = false,
    order = None,
    threads = None,
    type_rules = None,
    tag_rules = None,
    promote = None,
    hidden = None,
    hidden_allow = None,
    analyze = "none",
    analysis_workers = 0,
    views = None,
    include = None,
    exclude = None,
    min_size = None,
    max_size = None,
    modified_since = None,
    modified_before = None,
    kind = None,
    tags = None,
    not_tags = None,
    plane = None,
    depth = None,
    limit = None,
    sort = None,
    reverse = false,
    size = "allocated",
    words_per_page = 250,
    as_of_ns = None,
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
    order: Option<&str>,
    threads: Option<usize>,
    type_rules: Option<&PyTypeRegistry>,
    tag_rules: Option<Vec<String>>,
    promote: Option<Vec<String>>,
    hidden: Option<&str>,
    hidden_allow: Option<Vec<String>>,
    analyze: &str,
    analysis_workers: usize,
    views: Option<Vec<String>>,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    min_size: Option<&str>,
    max_size: Option<&str>,
    modified_since: Option<&str>,
    modified_before: Option<&str>,
    kind: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    not_tags: Option<Vec<String>>,
    plane: Option<&str>,
    depth: Option<&str>,
    limit: Option<&str>,
    sort: Option<&str>,
    reverse: bool,
    size: &str,
    words_per_page: u64,
    as_of_ns: Option<i64>,
) -> PyResult<PyOneShot> {
    let rules = enabled_tag_rules(tag_rules, promote)?;
    let hidden = admission_policy(hidden, hidden_allow)?;
    let analysis = parse_analysis_request(analyze, analysis_workers)?;
    let config = OpenConfig {
        scan: ScanConfig {
            max_depth,
            one_filesystem,
            order: parse_scan_order(order)?,
            threads,
            types: type_rules.map(|registry| Arc::clone(&registry.inner)),
            tags: Some(Arc::clone(&rules)),
            hidden: hidden.clone(),
            ..ScanConfig::default()
        },
        cache_path: fdu_core::default_cache_path(&root),
        policy: parse_cache_policy(cache)?,
        analysis,
    };
    let query = build_query(
        analysis.profile,
        None,
        views,
        include,
        exclude,
        min_size,
        max_size,
        modified_since,
        modified_before,
        kind,
        tags,
        not_tags,
        plane,
        &rules,
        depth,
        limit,
        sort,
        reverse,
        size,
        words_per_page,
        as_of_ns,
    )?;

    let prepared = py.detach(|| fdu_core::prepare_report(&root, &config, &query));
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
fn watch_rule(at_nanos: i64) -> PyResult<String> {
    // Nanoseconds because that is what a Change already carries, so a caller repainting
    // after a batch has the instant to hand without converting through anything.
    let at = if at_nanos >= 0 {
        std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_nanos(at_nanos.unsigned_abs()))
    } else {
        std::time::UNIX_EPOCH.checked_sub(std::time::Duration::from_nanos(at_nanos.unsigned_abs()))
    };
    let at = at.ok_or_else(|| {
        PyValueError::new_err("timestamp is outside the range this platform can represent")
    })?;
    Ok(fdu_core::report_format::watch_rule(at))
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
#[pyo3(signature = (path, op, clock, kind = None, bytes = None, allocated = None, mtime_ns = None, format = "jsonl"))]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn render_change(
    path: PathBuf,
    op: &str,
    clock: u64,
    kind: Option<&str>,
    bytes: Option<u64>,
    allocated: Option<u64>,
    mtime_ns: Option<i64>,
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
        entry_kind: kind.map(parse_kind).transpose()?,
        bytes,
        allocated,
        mtime_ns,
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
/// Takes the statuses as paths rather than as reconstructed values: re-reading the files
/// is cheap, keeps one definition of what a status *is*, and means a caller cannot hand
/// the renderer a status the engine never produced.
#[pyfunction]
#[pyo3(signature = (paths, format = "text"))]
#[allow(clippy::needless_pass_by_value)]
fn render_cache_status(paths: Vec<PathBuf>, format: &str) -> PyResult<String> {
    let format = parse_format(format)?;
    let statuses = paths
        .iter()
        .map(|path| fdu_core::cache_status(path).map_err(to_py_err))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(fdu_core::report_format::render_cache_status(&statuses, format))
}

/// One cache file's status as a dict.
fn cache_status_dict<'py>(
    py: Python<'py>,
    status: &fdu_core::CacheStatus,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("path", status.path.as_os_str())?;
    dict.set_item("bytes", status.bytes)?;
    dict.set_item("content_bytes", status.content_bytes)?;
    dict.set_item("recognized", status.is_recognized())?;
    if let Some(info) = &status.snapshot {
        dict.set_item("root", info.root.as_os_str())?;
        dict.set_item("entries", info.entries)?;
        dict.set_item("max_depth", info.scope.max_depth)?;
        dict.set_item("one_filesystem", info.scope.one_filesystem)?;
    } else {
        dict.set_item("root", py.None())?;
        dict.set_item("entries", py.None())?;
        dict.set_item("max_depth", py.None())?;
        dict.set_item("one_filesystem", py.None())?;
    }
    Ok(dict)
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

/// Remove the snapshot for one root. Returns whether a file was removed.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn clear_cache(root: PathBuf) -> PyResult<bool> {
    match fdu_core::default_cache_path(&root) {
        Some(path) => fdu_core::clear_cache(&path).map_err(to_py_err),
        None => Ok(false),
    }
}

/// Remove every recognized snapshot, leaving unrecognized files alone.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn clear_all_caches(root: PathBuf) -> PyResult<usize> {
    match fdu_core::default_cache_path(&root).and_then(|p| p.parent().map(Path::to_path_buf)) {
        Some(dir) => fdu_core::clear_all_caches(&dir).map_err(to_py_err),
        None => Ok(0),
    }
}

/// Open a directory tree, using the snapshot cache according to `cache`.
#[pyfunction]
#[pyo3(signature = (
    root,
    *,
    cache = "auto",
    max_depth = None,
    one_filesystem = false,
    order = None,
    threads = None,
    type_rules = None,
    tag_rules = None,
    promote = None,
    hidden = None,
    hidden_allow = None,
    analyze = "none",
    analysis_workers = 0
))]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn open(
    py: Python<'_>,
    root: PathBuf,
    cache: &str,
    max_depth: Option<usize>,
    one_filesystem: bool,
    order: Option<&str>,
    threads: Option<usize>,
    type_rules: Option<&PyTypeRegistry>,
    tag_rules: Option<Vec<String>>,
    promote: Option<Vec<String>>,
    hidden: Option<&str>,
    hidden_allow: Option<Vec<String>>,
    analyze: &str,
    analysis_workers: usize,
) -> PyResult<PyIndex> {
    let operation_started_at = SystemTime::now();
    let policy = parse_cache_policy(cache)?;
    let tags = enabled_tag_rules(tag_rules, promote)?;
    let hidden = admission_policy(hidden, hidden_allow)?;
    let analysis = parse_analysis_request(analyze, analysis_workers)?;
    let config = OpenConfig {
        scan: ScanConfig {
            max_depth,
            one_filesystem,
            order: parse_scan_order(order)?,
            threads,
            types: type_rules.map(|rules| Arc::clone(&rules.inner)),
            tags: Some(Arc::clone(&tags)),
            hidden: hidden.clone(),
            ..ScanConfig::default()
        },
        cache_path: fdu_core::default_cache_path(&root),
        policy,
        analysis,
    };

    let opened = py.detach(|| fdu_core::open(&root, &config));
    let (index, report) = opened.map_err(to_py_err)?;
    let operation_complete = report.is_complete();
    let mut errors = report.errors().iter().map(fdu_core::Issue::from_error).collect::<Vec<_>>();
    if let Some(message) =
        report.analysis.as_ref().and_then(fdu_core::content::AnalysisReport::failure_message)
    {
        errors.push(fdu_core::Issue::reported(fdu_core::IssueKind::ProviderFailure, message));
    }
    let source = match report.path_taken {
        fdu_core::OpenPath::ColdScan => ReportSource::ColdScan,
        fdu_core::OpenPath::WarmRevalidate => ReportSource::WarmRevalidate,
        fdu_core::OpenPath::CacheOnly => ReportSource::CacheOnly,
    };
    let scan_started_at =
        (report.path_taken != fdu_core::OpenPath::CacheOnly).then_some(operation_started_at);
    // Onto the index, before anything can read it. The engine recorded what the *scan*
    // did; this adds what analysis did, which the engine has no view of, and commits the
    // combined answer where a read finds it under the same guard as the rows.
    let index = index.with_run_facts(fdu_core::query::RunFacts {
        scan_started_at,
        source,
        complete: operation_complete,
        errors: errors.clone(),
    });
    Ok(PyIndex {
        inner: IndexHandle::new(index),
        config: config.scan,
        analysis,
        state: Mutex::new(RunState { telemetry: PerformanceSummary::from_open_report(&report) }),
    })
}

/// Walk a tree with no cache at all and return the index.
#[pyfunction]
#[pyo3(signature = (
    root,
    *,
    max_depth = None,
    one_filesystem = false,
    order = None,
    threads = None,
    type_rules = None,
    tag_rules = None,
    promote = None,
    hidden = None,
    hidden_allow = None,
    analyze = "none",
    analysis_workers = 0
))]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn scan(
    py: Python<'_>,
    root: PathBuf,
    max_depth: Option<usize>,
    one_filesystem: bool,
    order: Option<&str>,
    threads: Option<usize>,
    type_rules: Option<&PyTypeRegistry>,
    tag_rules: Option<Vec<String>>,
    promote: Option<Vec<String>>,
    hidden: Option<&str>,
    hidden_allow: Option<Vec<String>>,
    analyze: &str,
    analysis_workers: usize,
) -> PyResult<PyIndex> {
    let scan_started_at = Some(SystemTime::now());
    let tags = enabled_tag_rules(tag_rules, promote)?;
    let hidden = admission_policy(hidden, hidden_allow)?;
    let analysis = parse_analysis_request(analyze, analysis_workers)?;
    let config = OpenConfig {
        scan: ScanConfig {
            max_depth,
            one_filesystem,
            order: parse_scan_order(order)?,
            threads,
            types: type_rules.map(|rules| Arc::clone(&rules.inner)),
            tags: Some(Arc::clone(&tags)),
            hidden: hidden.clone(),
            ..ScanConfig::default()
        },
        cache_path: None,
        policy: CachePolicy::Off,
        analysis,
    };
    let scanned = py.detach(|| fdu_core::open(&root, &config));
    let (index, report) = scanned.map_err(to_py_err)?;
    let operation_complete = report.is_complete();
    let mut errors = report.errors().iter().map(fdu_core::Issue::from_error).collect::<Vec<_>>();
    if let Some(message) =
        report.analysis.as_ref().and_then(fdu_core::content::AnalysisReport::failure_message)
    {
        errors.push(fdu_core::Issue::reported(fdu_core::IssueKind::ProviderFailure, message));
    }
    // A bare scan never consults the cache, so it is always cold.
    let telemetry = PerformanceSummary::from_open_report(&report);
    let index = index.with_run_facts(fdu_core::query::RunFacts {
        scan_started_at,
        source: ReportSource::ColdScan,
        complete: operation_complete,
        errors: errors.clone(),
    });
    Ok(PyIndex {
        inner: IndexHandle::new(index),
        config: config.scan,
        analysis,
        state: Mutex::new(RunState { telemetry }),
    })
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
    contract.set_item("cache_scopes", ["root", "all"])?;
    Ok(contract)
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyIndex>()?;
    m.add_class::<PyWatch>()?;
    m.add_class::<PyTypeRegistry>()?;
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

    fn test_index() -> PyIndex {
        PyIndex {
            inner: IndexHandle::new(fdu_core::Index::new("/unused")),
            config: ScanConfig::default(),
            analysis: AnalysisRequest::default(),
            state: Mutex::new(RunState { telemetry: PerformanceSummary::default() }),
        }
    }

    /// No method takes an exclusive object borrow, so a reader is never rejected.
    ///
    /// This is the contract `fdu-gav9` established, and it replaced the opposite one: the
    /// index used to be owned rather than held, `refresh` took `&mut self`, and `PyO3`
    /// kept that exclusive borrow across the whole detached reconciliation. Any
    /// overlapping call on the same object raised `Already mutably borrowed`, which for a
    /// live consumer meant a failed request every time the tree changed under a reader.
    /// The whole read path is detachable: it holds nothing Python and blocks nobody.
    ///
    /// This is the precondition for `PyIndex.read` releasing the GIL, and it is the part a
    /// test can pin. `read` was the one native call here that did not detach, and
    /// `fdu-samw` made it the heaviest -- a filtered report re-aggregates the entire index,
    /// so holding the GIL across it freezes every other Python thread for a full
    /// traversal, and an asyncio consumer moving it to a worker thread would still have
    /// frozen its event loop.
    ///
    /// The oracle blocks *inside* the detach rather than counting after it: a second
    /// thread has to reach `Python::attach` and send while the read is running, so a path
    /// that needed the interpreter would hang here rather than merely finish late. That
    /// the request and bundle are `Send` at all is the other half, and the compiler checks
    /// that one every build.
    #[test]
    fn the_native_read_path_runs_detached_and_blocks_no_python_thread() {
        Python::initialize();
        Python::attach(|py| {
            let index = Py::new(py, test_index()).expect("allocate Python index");
            let handle = index.borrow(py).inner.clone();

            let (ready_tx, ready_rx) = sync_channel(1);
            let (progress_tx, progress_rx) = sync_channel(1);
            let worker = std::thread::spawn(move || {
                ready_tx.send(()).expect("signal worker ready");
                Python::attach(|_| progress_tx.send(()).expect("report Python progress"));
            });
            ready_rx.recv().expect("worker ready");

            // The read's own detach is the only thing that can let the worker through.
            let bundle = py
                .detach(move || {
                    progress_rx
                        .recv_timeout(Duration::from_secs(5))
                        .expect("another Python thread must run while the native read works");
                    handle.read(&fdu_core::ReadRequest { total: true, ..Default::default() })
                })
                .expect("read");
            assert!(bundle.total.is_some(), "and the read still answers");
            worker.join().expect("Python worker");
        });
    }

    /// Reads during a write are served rather than rejected, and never tear.
    ///
    /// The oracle is that the handle publishes whole applied states: a reader sees the
    /// clock before or after an apply, never a value assembled from both.
    #[test]
    fn reads_during_a_write_are_served_and_never_torn() {
        let handle = IndexHandle::new(fdu_core::Index::new("/unused"));
        let before = handle.clock().expect("clock before").0;

        let readers: Vec<_> = (0..4)
            .map(|_| {
                let handle = handle.clone();
                std::thread::spawn(move || {
                    let mut seen = Vec::new();
                    for _ in 0..200 {
                        seen.push(handle.clock().expect("a read during a write is served").0);
                    }
                    seen
                })
            })
            .collect();

        let observation = fdu_core::Observation::new(vec![fdu_core::Op::InvalidateSubtree {
            path: PathBuf::new(),
            reason: fdu_core::InvalidateReason::Requested,
        }]);
        handle.apply(&observation).expect("apply during reads");
        let after = handle.clock().expect("clock after").0;

        for reader in readers {
            for clock in reader.join().expect("reader thread") {
                assert!(
                    clock == before || clock == after,
                    "a read observed {clock}, which is neither the pre-write {before} nor \
                     the post-write {after}",
                );
            }
        }
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
