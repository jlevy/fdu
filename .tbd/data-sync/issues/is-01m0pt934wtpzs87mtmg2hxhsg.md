---
type: is
id: is-01m0pt934wtpzs87mtmg2hxhsg
title: "Python Index: shared reads during a write"
kind: bug
status: open
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
  - type: blocks
    target: is-01m0prhc835eec71rccdfe50zb
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:02:33.741Z
updated_at: 2026-08-23T08:19:18.915Z
---
MEASURED: with four reader threads calling rollup() while the main thread calls refresh(), readers raise 'FduError: Already mutably borrowed'. PyO3 treats refresh() as an exclusive borrow of the whole Index, rejecting what IndexHandle exists to allow — the engine already serves readers during short writes, so this is a binding-layer defect. A live server commits on every watch batch, so any request landing in that window fails; this is the one item that breaks a naive drop-in outright. Fix: Python Index reads take a shared borrow over the engine handle; mutation takes the handle's own short write. Tests pin that a concurrent read never raises and returns either the pre- or post-write value, never a torn one. Concurrent reads alone are already fine (3,200 calls across 16 threads, no errors).
