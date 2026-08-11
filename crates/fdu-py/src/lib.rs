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

use fdu::{EntryKind, Freshness, OpenConfig, RollUp, ScanConfig};

fn to_py_err(err: fdu::Error) -> PyErr {
    match err {
        fdu::Error::Io { .. } => PyOSError::new_err(err.to_string()),
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
    errors: Vec<String>,
}

#[pymethods]
impl PyIndex {
    /// The absolute root this index covers.
    #[getter]
    fn root(&self) -> String {
        self.inner.root_path().display().to_string()
    }

    /// The current logical clock. Pass it to `since()` later to get what changed.
    #[getter]
    fn clock(&self) -> u64 {
        self.inner.clock().0
    }

    /// Whether every path in this index's configured scope is currently trustworthy.
    #[getter]
    fn complete(&self) -> bool {
        self.inner.freshness() == Freshness::Fresh
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

    /// Roll-up totals for the whole tree.
    fn total<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        rollup_dict(py, &self.inner, self.inner.total())
    }

    /// Roll-up totals for one directory, or `None` if it is absent or not a directory.
    #[pyo3(signature = (path))]
    fn rollup<'py>(&self, py: Python<'py>, path: &str) -> PyResult<Option<Bound<'py, PyDict>>> {
        match self.inner.rollup(Path::new(path)) {
            Some(roll) => Ok(Some(rollup_dict(py, &self.inner, roll)?)),
            None => Ok(None),
        }
    }

    /// Every direct child of a directory, with its roll-up, in one call.
    ///
    /// Returns `None` when the path is absent or is not a directory — distinct from an
    /// empty list, which means a directory with no children.
    #[pyo3(signature = (path = ""))]
    fn children<'py>(&self, py: Python<'py>, path: &str) -> PyResult<Option<Bound<'py, PyList>>> {
        let Some(children) = self.inner.children(Path::new(path)) else {
            return Ok(None);
        };

        let out = PyList::empty(py);
        for (name, id) in children {
            let entry = PyDict::new(py);
            entry.set_item("name", name.to_string_lossy().as_ref())?;
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
                item.set_item("path", op.path().display().to_string())?;
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

/// Open a directory tree, using the snapshot cache when one is usable.
#[pyfunction]
#[pyo3(signature = (root, *, cache = true, max_depth = None))]
fn open(py: Python<'_>, root: &str, cache: bool, max_depth: Option<usize>) -> PyResult<PyIndex> {
    let root = PathBuf::from(root);
    let config = OpenConfig {
        scan: ScanConfig { max_depth, ..ScanConfig::default() },
        cache_path: if cache { fdu::default_cache_path(&root) } else { None },
        save_on_open: cache,
    };

    let opened = py.detach(|| fdu::open(&root, &config));
    let (index, report) = opened.map_err(to_py_err)?;
    let errors = report.errors().iter().map(ToString::to_string).collect();
    Ok(PyIndex { inner: index, config: config.scan, errors })
}

/// Walk a tree with no cache at all and return the index.
#[pyfunction]
#[pyo3(signature = (root, *, max_depth = None))]
fn scan(py: Python<'_>, root: &str, max_depth: Option<usize>) -> PyResult<PyIndex> {
    let root = PathBuf::from(root);
    let config = ScanConfig { max_depth, ..ScanConfig::default() };
    let scanned = py.detach(|| fdu::scan::scan_into_index(&root, &config));
    let (index, report) = scanned.map_err(to_py_err)?;
    let errors = report.errors.iter().map(ToString::to_string).collect();
    Ok(PyIndex { inner: index, config, errors })
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
    m.add_function(wrap_pyfunction!(open, m)?)?;
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
                    errors: Vec::new(),
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
