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
//! the GIL. One `PyIndex` still owns one ordinary Rust [`fdu::Index`]: `refresh()` keeps
//! `PyO3`'s exclusive object borrow for the whole detached reconciliation, so an
//! overlapping call on that same Python object is rejected by `PyO3`'s runtime borrow
//! check rather than becoming an unsynchronized shared-index read. Calls on independent
//! indexes may run concurrently. Python dictionary/list conversion happens after native
//! work returns and therefore runs with the GIL held.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use fdu::content::{AnalysisProfile, AnalysisRequest, CoverageReason};
use fdu::query::{
    Bound as Bound_, MetricRow, MetricSummary, Pattern, Provenance, Query, Report, ReportSource,
    Section, Selection, SizeMetric, SortKey, SummaryRow, TreeNode, ViewSpec,
};
use fdu::session::{ChangeKind, Session};
use fdu::watch::WatchConfig;
use fdu::{CachePolicy, EntryKind, Freshness, IndexHandle, OpenConfig, RollUp, ScanConfig};
use std::time::{Duration, SystemTime};

fn to_py_err(err: fdu::Error) -> PyErr {
    match err {
        fdu::Error::Io { path, source } => PyOSError::new_err((
            source.raw_os_error(),
            source.to_string(),
            path.as_os_str().to_os_string(),
        )),
        other => PyValueError::new_err(other.to_string()),
    }
}

/// Extension tallies carry interned ids internally, so the owning index resolves
/// them back to names at this boundary — Python callers always see extension strings.
fn rollup_dict<'py>(
    py: Python<'py>,
    index: &fdu::Index,
    roll: &RollUp,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("files", roll.files)?;
    dict.set_item("dirs", roll.dirs)?;
    dict.set_item("bytes", roll.bytes)?;
    dict.set_item("allocated", roll.allocated)?;
    dict.set_item("newest_mtime_ns", roll.newest_mtime_ns)?;

    let by_ext = PyDict::new(py);
    for (ext, tally) in index.by_ext_named(roll) {
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
#[pyclass(name = "Index", module = "fdu")]
pub struct PyIndex {
    inner: fdu::Index,
    config: ScanConfig,
    analysis: AnalysisRequest,
    errors: Vec<String>,
    /// Which cache tier produced this index.
    ///
    /// Carried rather than assumed: reporting `warm_revalidate` for an index built by a
    /// cold scan would be a small lie in exactly the field a caller consults to decide
    /// whether to trust the answer.
    source: ReportSource,
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
        self.inner.freshness() == Freshness::Fresh && self.errors.is_empty()
    }

    /// Current trust state: fresh, reconciling, stale, or partial.
    #[getter]
    fn freshness(&self) -> &'static str {
        freshness_label(self.inner.freshness())
    }

    /// Error details from the most recent scan or refresh.
    #[getter]
    fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }

    /// Number of entries held, including the root.
    fn __len__(&self) -> usize {
        usize::try_from(self.inner.len()).unwrap_or(usize::MAX)
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
        modified_since = None,
        modified_before = None,
        kind = None,
        depth = None,
        limit = None,
        sort = None,
        reverse = false,
        size = "allocated",
        words_per_page = 300
    ))]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    fn report<'py>(
        &self,
        py: Python<'py>,
        views: Option<Vec<String>>,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        min_size: Option<&str>,
        modified_since: Option<&str>,
        modified_before: Option<&str>,
        kind: Option<Vec<String>>,
        depth: Option<&str>,
        limit: Option<&str>,
        sort: Option<&str>,
        reverse: bool,
        size: &str,
        words_per_page: u64,
    ) -> PyResult<Bound<'py, PyDict>> {
        let now = SystemTime::now();
        let mut selection =
            Selection { reverse, size: parse_size_metric(size)?, ..Selection::default() };
        if let Some(value) = depth {
            selection.depth = parse_bound(value, "depth")?;
        }
        if let Some(value) = limit {
            selection.limit = parse_bound(value, "limit")?;
        }
        for pattern in include.unwrap_or_default() {
            selection.include.push(Pattern::parse(&pattern).map_err(to_py_err)?);
        }
        for pattern in exclude.unwrap_or_default() {
            selection.exclude.push(Pattern::parse(&pattern).map_err(to_py_err)?);
        }
        if let Some(value) = min_size {
            selection.min_size = Some(fdu::query::parse_size(value).map_err(to_py_err)?);
        }
        if let Some(value) = modified_since {
            let at = fdu::query::parse_when(value, now).map_err(to_py_err)?;
            selection.modified.since = Some(bound_nanos(value, at, "modified_since")?);
        }
        if let Some(value) = modified_before {
            let at = fdu::query::parse_when(value, now).map_err(to_py_err)?;
            selection.modified.before = Some(bound_nanos(value, at, "modified_before")?);
        }
        for value in kind.unwrap_or_default() {
            selection.kinds.push(parse_kind(&value)?);
        }
        if let Some(value) = sort {
            selection.sort = Some(parse_sort(value)?);
        }

        let views = match views {
            Some(values) => {
                values.iter().map(|value| parse_view(value)).collect::<PyResult<Vec<_>>>()?
            }
            None => vec![ViewSpec::Tree],
        };

        let provenance = Provenance {
            scan_started_at: None,
            generated_at: now,
            source: self.source,
            complete: self.errors.is_empty(),
            errors: self.errors.clone(),
        };
        if words_per_page == 0 {
            return Err(PyValueError::new_err("words_per_page must be positive"));
        }
        let report = fdu::query::report(
            &self.inner,
            &Query { selection, views, words_per_page },
            &provenance,
        );
        report_dict(py, &report)
    }

    /// Watch this tree, yielding batches of changes as they arrive.
    ///
    /// Detection is event-driven, so an idle tree costs nothing; `interval` bounds how
    /// long a single wait blocks before yielding an empty batch.
    #[pyo3(signature = (*, interval = 2.0, views = None, include = None, exclude = None, kind = None))]
    fn watch(
        &self,
        interval: f64,
        views: Option<Vec<String>>,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
        kind: Option<Vec<String>>,
    ) -> PyResult<PyWatch> {
        let mut selection = Selection::default();
        for pattern in include.unwrap_or_default() {
            selection.include.push(Pattern::parse(&pattern).map_err(to_py_err)?);
        }
        for pattern in exclude.unwrap_or_default() {
            selection.exclude.push(Pattern::parse(&pattern).map_err(to_py_err)?);
        }
        for value in kind.unwrap_or_default() {
            selection.kinds.push(parse_kind(&value)?);
        }
        let views = match views {
            Some(values) => {
                values.iter().map(|value| parse_view(value)).collect::<PyResult<Vec<_>>>()?
            }
            None => vec![ViewSpec::Files],
        };

        if !interval.is_finite() || interval <= 0.0 {
            return Err(PyValueError::new_err("interval must be a positive number of seconds"));
        }

        // The index is cloned into the session: a watcher owns its own handle, so closing
        // the feed cannot disturb the caller's index.
        let handle = IndexHandle::new(self.inner.clone());
        let session = Session::new(
            handle,
            self.config.clone(),
            Query { selection, views, ..Query::default() },
            WatchConfig::default(),
        )
        .map_err(to_py_err)?;

        Ok(PyWatch { session: Some(session), timeout: Duration::from_secs_f64(interval) })
    }

    /// Roll-up totals for the whole tree.
    fn total<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        rollup_dict(py, &self.inner, self.inner.total())
    }

    /// Roll-up totals for one directory, or `None` if it is absent or not a directory.
    #[pyo3(signature = (path))]
    #[allow(clippy::needless_pass_by_value)]
    fn rollup<'py>(&self, py: Python<'py>, path: PathBuf) -> PyResult<Option<Bound<'py, PyDict>>> {
        match self.inner.rollup(&path) {
            Some(roll) => Ok(Some(rollup_dict(py, &self.inner, roll)?)),
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
            if let Some(roll) = self.inner.rollup_of(id) {
                entry.set_item("rollup", rollup_dict(py, &self.inner, roll)?)?;
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

    /// Reconcile the index against the filesystem and return what changed.
    ///
    /// This is the revalidation tier: unchanged entries cost a stat and nothing more,
    /// because an upsert whose complete observed state already matches is a no-op.
    fn refresh<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let config = self.config.clone();
        let report = py
            .detach(|| fdu::scan::reconcile(&mut self.inner, &config, &mut |_| {}))
            .map_err(to_py_err)?;
        self.errors = report.scan.errors.iter().map(ToString::to_string).collect();
        if self.analysis.profile.is_enabled() {
            let analysis =
                py.detach(|| fdu::content::analyze_index(&mut self.inner, self.analysis));
            append_analysis_error(&mut self.errors, analysis);
        }
        let stats = report.apply;

        let out = PyDict::new(py);
        out.set_item("inserted", stats.inserted)?;
        out.set_item("updated", stats.updated)?;
        out.set_item("removed", stats.removed)?;
        out.set_item("unchanged", stats.unchanged)?;
        out.set_item("stale", stats.stale)?;
        out.set_item("error_count", self.errors.len())?;
        out.set_item("errors", self.errors.clone())?;
        out.set_item("complete", self.complete())?;
        out.set_item("freshness", self.freshness())?;
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
        let since = self.inner.since(fdu::Clock(clock));
        let ops = PyList::empty(py);
        for delta in &since.deltas {
            for op in &delta.ops {
                let item = PyDict::new(py);
                item.set_item("clock", delta.clock.0)?;
                item.set_item("path", op.path().as_os_str())?;
                match op {
                    fdu::Op::Upsert { kind, attrs, .. } => {
                        item.set_item("op", "upsert")?;
                        item.set_item("kind", entry_kind_label(*kind))?;
                        item.set_item("bytes", attrs.size)?;
                        item.set_item("mtime_ns", attrs.mtime_ns)?;
                    }
                    fdu::Op::Remove { .. } => {
                        item.set_item("op", "remove")?;
                    }
                    fdu::Op::InvalidateSubtree { reason, .. } => {
                        item.set_item("op", "invalidate_subtree")?;
                        item.set_item("reason", format!("{reason:?}"))?;
                    }
                }
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

fn parse_analysis_request(
    profile: &str,
    max_file_size: &str,
    workers: usize,
) -> PyResult<AnalysisRequest> {
    let profile = match profile.trim().to_ascii_lowercase().as_str() {
        "none" | "off" | "disabled" => AnalysisProfile::Disabled,
        "basic" => AnalysisProfile::Basic,
        "code" => AnalysisProfile::Code,
        "documents" | "docs" => AnalysisProfile::Documents,
        "full" => AnalysisProfile::Full,
        other => {
            return Err(PyValueError::new_err(format!(
                "invalid analysis profile {other:?}: expected one of none, basic, code, documents, full"
            )));
        }
    };
    let max_file_bytes = fdu::query::parse_size(max_file_size).map_err(to_py_err)?;
    Ok(AnalysisRequest { profile, max_file_bytes, workers })
}

fn append_analysis_error(errors: &mut Vec<String>, analysis: fdu::content::AnalysisReport) {
    if !analysis.is_complete() {
        errors.push(format!(
            "content analysis incomplete: {} invalid UTF-8, {} too large, {} I/O errors, {} changed during read, {} unsupported, {} stale",
            analysis.invalid_utf8,
            analysis.too_large,
            analysis.io_errors,
            analysis.changed_during_read,
            analysis.unsupported,
            analysis.stale
        ));
    }
}

/// Name a cache tier for Python callers, matching the CLI's machine output.
fn source_label(source: fdu::query::ReportSource) -> &'static str {
    match source {
        fdu::query::ReportSource::ColdScan => "cold_scan",
        fdu::query::ReportSource::WarmRevalidate => "warm_revalidate",
        fdu::query::ReportSource::CacheOnly => "cache_only",
    }
}

/// Convert a parsed time bound to index nanoseconds, or raise.
///
/// Mirrors the CLI: an instant the index cannot represent must be rejected, never stored
/// as an absent bound, or the query silently runs with no time filter at all.
fn bound_nanos(input: &str, when: std::time::SystemTime, field: &str) -> PyResult<i64> {
    fdu::query::system_time_to_nanos(when).ok_or_else(|| {
        pyo3::exceptions::PyValueError::new_err(format!(
            "invalid {field} {input:?}: that time is outside the range fdu can represent \
             (about 1677 to 2262)"
        ))
    })
}

/// Convert a report into the dict shape Python callers get.
fn report_dict<'py>(py: Python<'py>, report: &Report) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("root", report.root.as_os_str())?;
    dict.set_item("complete", report.complete)?;
    dict.set_item("errors", report.errors.clone())?;
    dict.set_item("source", source_label(report.source))?;
    dict.set_item("freshness", freshness_label(report.freshness))?;
    dict.set_item("generated_at", fdu::query::format_rfc3339(report.generated_at))?;
    dict.set_item("scan_started_at", report.scan_started_at.map(fdu::query::format_rfc3339))?;
    match report.analysis.as_ref() {
        None => dict.set_item("analysis", py.None())?,
        Some(analysis) => {
            let metadata = PyDict::new(py);
            metadata.set_item("profile", analysis_profile_label(analysis.profile))?;
            metadata
                .set_item("type_rules_fingerprint", analysis.provenance.type_rules_fingerprint)?;
            metadata.set_item("options_fingerprint", analysis.provenance.options_fingerprint.0)?;
            let analyzers = PyList::empty(py);
            for (id, version) in &analysis.provenance.analyzers {
                let analyzer = PyDict::new(py);
                analyzer.set_item("id", id.0)?;
                analyzer.set_item("version", version.0)?;
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
                entry.set_item("view", "summary")?;
                entry.set_item("summary", summary_dict(py, row)?)?;
            }
            Section::Extensions(rows) => {
                entry.set_item("view", "extensions")?;
                let list = PyList::empty(py);
                for row in rows {
                    let item = PyDict::new(py);
                    item.set_item("extension", &row.extension)?;
                    item.set_item("files", row.files)?;
                    item.set_item("bytes", row.bytes)?;
                    item.set_item("allocated", row.allocated)?;
                    list.append(item)?;
                }
                entry.set_item("extensions", list)?;
            }
            Section::Metrics { view, summary } => {
                entry.set_item("view", view_label(*view))?;
                entry.set_item("metrics", metric_summary_dict(py, summary)?)?;
            }
            Section::Files(rows) => {
                entry.set_item("view", "files")?;
                let list = PyList::empty(py);
                for row in rows {
                    let item = PyDict::new(py);
                    item.set_item("path", row.path.as_os_str())?;
                    item.set_item("kind", entry_kind_label(row.kind))?;
                    item.set_item("bytes", row.bytes)?;
                    item.set_item("allocated", row.allocated)?;
                    item.set_item("mtime_ns", row.mtime_ns)?;
                    list.append(item)?;
                }
                entry.set_item("files", list)?;
            }
            Section::Tree(root) => {
                entry.set_item("view", "tree")?;
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
    dict.set_item(
        "group",
        match summary.group {
            fdu::query::MetricGroup::Type => "type",
            fdu::query::MetricGroup::Family => "family",
        },
    )?;
    dict.set_item("words_per_page", summary.words_per_page)?;
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
    dict.set_item("id", &row.id)?;
    dict.set_item("family", row.family.as_str())?;
    dict.set_item("files", row.files)?;
    dict.set_item("bytes", row.bytes)?;
    dict.set_item("allocated", row.allocated)?;
    dict.set_item("analyzed_files", row.analyzed_files)?;
    dict.set_item("share_numerator", row.share.numerator)?;
    dict.set_item("share_denominator", row.share.denominator)?;
    dict.set_item("physical_lines", row.metrics.physical_lines)?;
    dict.set_item("blank_lines", row.metrics.blank_lines)?;
    dict.set_item("nonblank_lines", row.metrics.nonblank_lines)?;
    dict.set_item("code_lines", row.metrics.code_lines)?;
    dict.set_item("comment_lines", row.metrics.comment_lines)?;
    dict.set_item("code_blank_lines", row.metrics.code_blank_lines)?;
    dict.set_item("raw_words", row.metrics.raw_words)?;
    dict.set_item("logical_words", row.metrics.logical_word_stats.logical_words())?;
    dict.set_item("paragraphs", row.metrics.paragraphs)?;
    dict.set_item("visible_words", row.metrics.visible_words)?;
    dict.set_item("words_per_page", words_per_page)?;
    let coverage = PyDict::new(py);
    for (reason, count) in &row.coverage {
        coverage.set_item(coverage_label(*reason), count)?;
    }
    dict.set_item("coverage", coverage)?;
    Ok(dict)
}

fn view_label(view: ViewSpec) -> &'static str {
    match view {
        ViewSpec::Tree => "tree",
        ViewSpec::Extensions => "extensions",
        ViewSpec::Types => "types",
        ViewSpec::Families => "families",
        ViewSpec::Languages => "languages",
        ViewSpec::Documents => "documents",
        ViewSpec::Files => "files",
        ViewSpec::Summary => "summary",
    }
}

fn coverage_label(reason: CoverageReason) -> &'static str {
    match reason {
        CoverageReason::Analyzed => "analyzed",
        CoverageReason::Binary => "binary",
        CoverageReason::InvalidUtf8 => "invalid_utf8",
        CoverageReason::TooLarge => "too_large",
        CoverageReason::Unsupported => "unsupported",
        CoverageReason::IoError => "io_error",
        CoverageReason::ChangedDuringRead => "changed_during_read",
    }
}

fn analysis_profile_label(profile: AnalysisProfile) -> &'static str {
    match profile {
        AnalysisProfile::Disabled => "none",
        AnalysisProfile::Basic => "basic",
        AnalysisProfile::Code => "code",
        AnalysisProfile::Documents => "documents",
        AnalysisProfile::Full => "full",
    }
}

/// One summary row as a dict.
fn summary_dict<'py>(py: Python<'py>, row: &SummaryRow) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("files", row.files)?;
    dict.set_item("dirs", row.dirs)?;
    dict.set_item("bytes", row.bytes)?;
    dict.set_item("allocated", row.allocated)?;
    dict.set_item("newest_mtime_ns", row.newest_mtime_ns)?;
    Ok(dict)
}

/// One tree node as a nested dict.
///
/// Built with an explicit stack: the tree can be deeper than the interpreter's recursion
/// budget, and a report that aborts on a deep tree fails where it is most useful.
fn tree_dict<'py>(py: Python<'py>, root: &TreeNode) -> PyResult<Bound<'py, PyDict>> {
    fn node_dict<'py>(py: Python<'py>, node: &TreeNode) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("name", &node.name)?;
        dict.set_item("path", node.path.as_os_str())?;
        dict.set_item("bytes", node.bytes)?;
        dict.set_item("allocated", node.allocated)?;
        dict.set_item("files", node.files)?;
        dict.set_item("dirs", node.dirs)?;
        dict.set_item("newest_mtime_ns", node.newest_mtime_ns)?;
        dict.set_item("truncated", node.truncated)?;
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

/// Parse a view name.
fn parse_view(value: &str) -> PyResult<ViewSpec> {
    match value.trim().to_ascii_lowercase().as_str() {
        "tree" => Ok(ViewSpec::Tree),
        "types" => Ok(ViewSpec::Types),
        "extensions" => Ok(ViewSpec::Extensions),
        "families" => Ok(ViewSpec::Families),
        "languages" => Ok(ViewSpec::Languages),
        "documents" | "docs" => Ok(ViewSpec::Documents),
        "files" => Ok(ViewSpec::Files),
        "summary" => Ok(ViewSpec::Summary),
        other => Err(PyValueError::new_err(format!(
            "invalid view {other:?}: expected one of tree, extensions, types, families, languages, documents, files, summary"
        ))),
    }
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
#[pyclass(name = "Watch", module = "fdu_py", unsendable)]
struct PyWatch {
    session: Option<Session>,
    timeout: Duration,
}

#[pymethods]
impl PyWatch {
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

/// One cache file's status as a dict.
fn cache_status_dict<'py>(
    py: Python<'py>,
    status: &fdu::CacheStatus,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("path", status.path.as_os_str())?;
    dict.set_item("bytes", status.bytes)?;
    dict.set_item("content_bytes", status.content_bytes)?;
    dict.set_item("recognized", status.is_recognized())?;
    if let Some(info) = &status.snapshot {
        dict.set_item("root", info.root.as_os_str())?;
        dict.set_item("entries", info.entries)?;
    } else {
        dict.set_item("root", py.None())?;
        dict.set_item("entries", py.None())?;
    }
    Ok(dict)
}

/// The cache directory this build would use for a root.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn cache_path(root: PathBuf) -> Option<PathBuf> {
    fdu::default_cache_path(&root)
}

/// Status of the snapshot for one root.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn cache_status(py: Python<'_>, root: PathBuf) -> PyResult<Option<Bound<'_, PyDict>>> {
    let Some(path) = fdu::default_cache_path(&root) else {
        return Ok(None);
    };
    let status = fdu::cache_status(&path).map_err(to_py_err)?;
    Ok(Some(cache_status_dict(py, &status)?))
}

/// Every cache file this build can see, recognized or not.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn list_caches(py: Python<'_>, root: PathBuf) -> PyResult<Bound<'_, PyList>> {
    let list = PyList::empty(py);
    let Some(dir) = fdu::default_cache_path(&root).and_then(|p| p.parent().map(Path::to_path_buf))
    else {
        return Ok(list);
    };
    for status in fdu::list_caches(&dir).map_err(to_py_err)? {
        list.append(cache_status_dict(py, &status)?)?;
    }
    Ok(list)
}

/// Remove the snapshot for one root. Returns whether a file was removed.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn clear_cache(root: PathBuf) -> PyResult<bool> {
    match fdu::default_cache_path(&root) {
        Some(path) => fdu::clear_cache(&path).map_err(to_py_err),
        None => Ok(false),
    }
}

/// Remove every recognized snapshot, leaving unrecognized files alone.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
fn clear_all_caches(root: PathBuf) -> PyResult<usize> {
    match fdu::default_cache_path(&root).and_then(|p| p.parent().map(Path::to_path_buf)) {
        Some(dir) => fdu::clear_all_caches(&dir).map_err(to_py_err),
        None => Ok(0),
    }
}

/// Open a directory tree, using the snapshot cache according to `cache`.
#[pyfunction]
#[pyo3(signature = (root, *, cache = "auto", max_depth = None, analyze = "none", max_file_size = "16MiB", analysis_workers = 0))]
#[allow(clippy::needless_pass_by_value)]
fn open(
    py: Python<'_>,
    root: PathBuf,
    cache: &str,
    max_depth: Option<usize>,
    analyze: &str,
    max_file_size: &str,
    analysis_workers: usize,
) -> PyResult<PyIndex> {
    let policy = parse_cache_policy(cache)?;
    let analysis = parse_analysis_request(analyze, max_file_size, analysis_workers)?;
    let config = OpenConfig {
        scan: ScanConfig { max_depth, ..ScanConfig::default() },
        cache_path: fdu::default_cache_path(&root),
        policy,
        analysis,
    };

    let opened = py.detach(|| fdu::open(&root, &config));
    let (index, report) = opened.map_err(to_py_err)?;
    let mut errors = report.errors().iter().map(ToString::to_string).collect::<Vec<_>>();
    if let Some(analysis_report) = report.analysis {
        append_analysis_error(&mut errors, analysis_report);
    }
    let source = match report.path_taken {
        fdu::OpenPath::ColdScan => ReportSource::ColdScan,
        fdu::OpenPath::WarmRevalidate => ReportSource::WarmRevalidate,
        fdu::OpenPath::CacheOnly => ReportSource::CacheOnly,
    };
    Ok(PyIndex { inner: index, config: config.scan, analysis, errors, source })
}

/// Walk a tree with no cache at all and return the index.
#[pyfunction]
#[pyo3(signature = (root, *, max_depth = None, analyze = "none", max_file_size = "16MiB", analysis_workers = 0))]
#[allow(clippy::needless_pass_by_value)]
fn scan(
    py: Python<'_>,
    root: PathBuf,
    max_depth: Option<usize>,
    analyze: &str,
    max_file_size: &str,
    analysis_workers: usize,
) -> PyResult<PyIndex> {
    let analysis = parse_analysis_request(analyze, max_file_size, analysis_workers)?;
    let config = OpenConfig {
        scan: ScanConfig { max_depth, ..ScanConfig::default() },
        cache_path: None,
        policy: CachePolicy::Off,
        analysis,
    };
    let scanned = py.detach(|| fdu::open(&root, &config));
    let (index, report) = scanned.map_err(to_py_err)?;
    let mut errors = report.errors().iter().map(ToString::to_string).collect::<Vec<_>>();
    if let Some(analysis_report) = report.analysis {
        append_analysis_error(&mut errors, analysis_report);
    }
    // A bare scan never consults the cache, so it is always cold.
    Ok(PyIndex {
        inner: index,
        config: config.scan,
        analysis,
        errors,
        source: ReportSource::ColdScan,
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
    Ok(py.detach(move || fdu::cli::run_process(args)))
}

#[pymodule]
fn fdu_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyIndex>()?;
    m.add_class::<PyWatch>()?;
    m.add_function(wrap_pyfunction!(open, m)?)?;
    m.add_function(wrap_pyfunction!(cache_path, m)?)?;
    m.add_function(wrap_pyfunction!(cache_status, m)?)?;
    m.add_function(wrap_pyfunction!(list_caches, m)?)?;
    m.add_function(wrap_pyfunction!(clear_cache, m)?)?;
    m.add_function(wrap_pyfunction!(clear_all_caches, m)?)?;
    m.add_function(wrap_pyfunction!(scan, m)?)?;
    m.add_function(wrap_pyfunction!(main, m)?)?;

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
    fn same_python_index_uses_runtime_borrow_exclusion() {
        Python::initialize();
        Python::attach(|py| {
            let index = Py::new(
                py,
                PyIndex {
                    inner: fdu::Index::new("/unused"),
                    config: ScanConfig::default(),
                    analysis: AnalysisRequest::default(),
                    errors: Vec::new(),
                    source: ReportSource::ColdScan,
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
