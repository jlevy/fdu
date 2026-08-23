---
type: is
id: is-01m0pt934wtpzs87mtmg2hxhsg
title: "Python Index: shared reads during a write"
kind: bug
status: closed
priority: 0
version: 6
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
  - type: blocks
    target: is-01m0prhc835eec71rccdfe50zb
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:02:33.741Z
updated_at: 2026-08-23T18:03:50.839Z
closed_at: 2026-08-23T18:03:50.839Z
close_reason: "Implemented and gated. fdu-gav9: PyIndex now holds the IndexHandle the engine already provided, refresh takes &self through reconcile_handle (short write locks per wave), run state moved behind a short Mutex; IndexHandle gained provenance/analyze/with_index and ChildSnapshot carries provenance so a listing reads at one boundary. Measured: reader errors during a write 4 -> 0, with no regression (3,200 concurrent reads 0.31s -> 0.42s, 200 serial summaries 0.008s -> 0.011s; an intermediate version that snapshotted per report was 1,900x slower and every test still passed, which is why with_index exists). fdu-4vkz: --order and --threads join the Scope axis, ScanOrder lands in the Python models, the parity shim learned both flags, and four goldens pin that both orders and one worker answer identically. Parity holds with the four new sessions passing on BOTH surfaces and no new deviation class."
resolution: null
duplicate_of: null
---
MEASURED: with four reader threads calling rollup() while the main thread calls refresh(), readers raise 'FduError: Already mutably borrowed'. PyO3 treats refresh() as an exclusive borrow of the whole Index, rejecting what IndexHandle exists to allow — the engine already serves readers during short writes, so this is a binding-layer defect. A live server commits on every watch batch, so any request landing in that window fails; this is the one item that breaks a naive drop-in outright. Fix: Python Index reads take a shared borrow over the engine handle; mutation takes the handle's own short write. Tests pin that a concurrent read never raises and returns either the pre- or post-write value, never a torn one. Concurrent reads alone are already fine (3,200 calls across 16 threads, no errors).

## Notes

IMPLEMENTATION MAP (from the implementation spec). Defect site: PyIndex holds inner: fdu_core::Index (owned) at fdu-py/src/lib.rs:130, so refresh (lib.rs:440) must take &mut self and PyO3 holds an exclusive borrow of the whole pyclass across the detached reconcile. Every other method already takes &self. FIX: PyIndex.inner becomes IndexHandle (index.rs:384, Arc<RwLock<Index>>); refresh(&mut self) -> refresh(&self) calling scan::reconcile_handle (scan.rs:2861, already exists, short write locks per wave) instead of scan::reconcile (scan.rs:2839); read methods use the handle's read-locked equivalents (rollup/children/total/since/clock/freshness/len/root_path all exist); build_report (lib.rs:521) uses handle.snapshot() (index.rs:503) exactly as Session::report (watch_session.rs:106) already does; watch() (lib.rs:362) simplifies from IndexHandle::new(self.inner.clone()) — which clones an entire index — to an Arc clone. THREE GAPS TO CLOSE FIRST, all small, all in fdu-core: (1) IndexHandle has no provenance — add pub fn provenance(&self, path: &Path) -> Result<Option<Provenance>> beside the read-locked accessors at index.rs:450-480; (2) ChildSnapshot carries id/name/kind/attrs/rollup but not provenance, which PyIndex::children (lib.rs:389) reports per child — extend it so one read lock still serves the listing; (3) the analysis phase inside refresh calls content::analyze_index(&mut self.inner, ..) and needs a handle path or an explicit short write. TESTS: extend fdu-py/src/lib.rs:1599's same_python_index_uses_runtime_borrow_exclusion, which today asserts the CURRENT exclusion, to assert the new contract; driver is crates/fdu-py/tests/run_concurrency.py. Oracle for 'no torn read': every concurrent read equals the pre-write or post-write value and never a mixture, which a settled tree makes known. Add the counter relation that a read during a write triggers no rescan.
