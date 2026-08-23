---
type: is
id: is-01m0pt934wtpzs87mtmg2hxhsg
title: "Python Index: shared reads during a write"
kind: bug
status: closed
priority: 0
version: 8
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
  - type: blocks
    target: is-01m0prhc835eec71rccdfe50zb
  - type: blocks
    target: is-01m0racc8tf20x27jjhh35vh5q
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:02:33.741Z
updated_at: 2026-08-23T22:03:13.050Z
closed_at: 2026-08-23T18:03:50.839Z
close_reason: "Implemented and gated. fdu-gav9: PyIndex now holds the IndexHandle the engine already provided, refresh takes &self through reconcile_handle (short write locks per wave), run state moved behind a short Mutex; IndexHandle gained provenance/analyze/with_index and ChildSnapshot carries provenance so a listing reads at one boundary. Measured: reader errors during a write 4 -> 0, with no regression (3,200 concurrent reads 0.31s -> 0.42s, 200 serial summaries 0.008s -> 0.011s; an intermediate version that snapshotted per report was 1,900x slower and every test still passed, which is why with_index exists). fdu-4vkz: --order and --threads join the Scope axis, ScanOrder lands in the Python models, the parity shim learned both flags, and four goldens pin that both orders and one worker answer identically. Parity holds with the four new sessions passing on BOTH surfaces and no new deviation class."
resolution: null
duplicate_of: null
---
MEASURED: with four reader threads calling rollup() while the main thread calls refresh(), readers raise 'FduError: Already mutably borrowed'. PyO3 treats refresh() as an exclusive borrow of the whole Index, rejecting what IndexHandle exists to allow — the engine already serves readers during short writes, so this is a binding-layer defect. A live server commits on every watch batch, so any request landing in that window fails; this is the one item that breaks a naive drop-in outright. Fix: Python Index reads take a shared borrow over the engine handle; mutation takes the handle's own short write. Tests pin that a concurrent read never raises and returns either the pre- or post-write value, never a torn one. Concurrent reads alone are already fine (3,200 calls across 16 threads, no errors).

## Notes

LAND EARLY -- this is in the smallest coupled slice. Metabrowser's Phase 2 opens with a real PyO3 spike: open a shared fdu handle, do one bundled directory-plus-rollup read returning a single version/cursor/state/work record, and converge after one live mutation with no Python mirror index. Shared reads are a precondition for that slice, so this bead gates the first moment the contract meets an actual consumer, not just the eventual drop-in. Metabrowser Phase 1 (the Python provider refactor) has no fdu dependency, so nothing else here is blocked on the client -- but this one is worth doing first regardless.
