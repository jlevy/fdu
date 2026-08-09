//! Python bindings for fdu.
//!
//! # API shape
//!
//! Every method here is **bulk**: it returns a whole structured result in one call
//! rather than exposing a cursor that Python iterates. A million small zero-copy calls
//! lose comfortably to one large call, because the per-call boundary cost dominates once
//! the native work per item is a field read.
//!
//! Native work runs with the GIL released, so a scan of a large tree does not stall the
//! host process's other threads.

use std::path::{Path, PathBuf};

use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use fdu::{OpenConfig, RollUp, ScanConfig};

fn to_py_err(err: fdu::Error) -> PyErr {
    match err {
        fdu::Error::Io { .. } => PyOSError::new_err(err.to_string()),
        other => PyValueError::new_err(other.to_string()),
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
        by_ext.set_item(ext, entry)?;
    }
    dict.set_item("by_extension", by_ext)?;
    Ok(dict)
}

/// A live index over one directory tree.
#[pyclass(name = "Index", module = "fdu")]
pub struct PyIndex {
    inner: fdu::Index,
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

    /// Number of entries held, including the root.
    fn __len__(&self) -> usize {
        usize::try_from(self.inner.len()).unwrap_or(usize::MAX)
    }

    /// Roll-up totals for the whole tree.
    fn total<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        rollup_dict(py, self.inner.total())
    }

    /// Roll-up totals for one directory, or `None` if it is absent or not a directory.
    #[pyo3(signature = (path))]
    fn rollup<'py>(&self, py: Python<'py>, path: &str) -> PyResult<Option<Bound<'py, PyDict>>> {
        match self.inner.rollup(Path::new(path)) {
            Some(roll) => Ok(Some(rollup_dict(py, roll)?)),
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
            entry.set_item("name", name)?;
            let is_dir = self.inner.kind_of(id).is_dir();
            entry.set_item("kind", if is_dir { "dir" } else { "file" })?;
            if let Some(roll) = self.inner.rollup_of(id) {
                entry.set_item("rollup", rollup_dict(py, roll)?)?;
            } else {
                let attrs = self.inner.attrs_of(id);
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
    /// because an upsert whose fingerprint already matches is discarded as a no-op.
    fn refresh<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let config = ScanConfig::default();
        let index = &mut self.inner;

        let (deltas, report) = py.detach(|| {
            let mut deltas = Vec::new();
            let report = fdu::scan::revalidate(index, &config, &mut |d| deltas.push(d));
            (deltas, report)
        });
        let report = report.map_err(to_py_err)?;

        let mut inserted = 0u64;
        let mut updated = 0u64;
        let mut removed = 0u64;
        let mut unchanged = 0u64;
        for delta in &deltas {
            let stats = index.apply(delta);
            inserted += stats.inserted;
            updated += stats.updated;
            removed += stats.removed;
            unchanged += stats.unchanged;
        }

        let out = PyDict::new(py);
        out.set_item("inserted", inserted)?;
        out.set_item("updated", updated)?;
        out.set_item("removed", removed)?;
        out.set_item("unchanged", unchanged)?;
        out.set_item("errors", report.errors.len())?;
        out.set_item("complete", report.is_complete())?;
        out.set_item("clock", index.clock().0)?;
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
                        item.set_item("kind", if kind.is_dir() { "dir" } else { "file" })?;
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
    let (index, _report) = opened.map_err(to_py_err)?;
    Ok(PyIndex { inner: index })
}

/// Walk a tree with no cache at all and return the index.
#[pyfunction]
#[pyo3(signature = (root, *, max_depth = None))]
fn scan(py: Python<'_>, root: &str, max_depth: Option<usize>) -> PyResult<PyIndex> {
    let root = PathBuf::from(root);
    let config = ScanConfig { max_depth, ..ScanConfig::default() };
    let scanned = py.detach(|| fdu::scan::scan_into_index(&root, &config));
    let (index, _report) = scanned.map_err(to_py_err)?;
    Ok(PyIndex { inner: index })
}

#[pymodule]
fn fdu_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyIndex>()?;
    m.add_function(wrap_pyfunction!(open, m)?)?;
    m.add_function(wrap_pyfunction!(scan, m)?)?;

    Ok(())
}
