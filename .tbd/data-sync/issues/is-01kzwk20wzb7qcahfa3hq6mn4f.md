---
type: is
id: is-01kzwk20wzb7qcahfa3hq6mn4f
title: "H19-H22: Compact the reusable full-index entry layout"
kind: task
status: in_progress
priority: 1
version: 10
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - experiment
  - campaign-2
dependencies:
  - type: blocks
    target: is-01kzwsr47nxr6arn4qbdz66949
parent_id: is-01m01mqq3cqs8ae87qd2d3rydm
hold: null
hold_until: null
created_at: 2026-08-13T03:36:06.814Z
updated_at: 2026-09-01T21:02:59.423Z
started_at: 2026-09-01T15:20:04.375Z
---
Execute the existing H19-H22 memory-layout ladder against the new 1M evidence: unbox arena entries, store names once, move directory-only payloads off file entries, and compact identities/revisions. Profile and size the current Entry first; measure one structural arm at a time; preserve snapshots, deltas, stable identities, and exact query behavior. Preregister peak RSS (substantial reduction) plus wall/fault effects.

## Notes

2026-09-01 retained-layout attribution found one 280-byte boxed Entry per nonroot entry and two 4.11 MB extension-map node planes, explaining most of the 10 to 13.8 MB historical RSS gap. Exp-100 moved directory-only state out of line and inlined arena entries: RSS improved 24.63% on default-tree but wall changed only -0.83% [-3.33%, +0.74%], so it is rejected alone. Exp-101 adds one-name sorted child IDs on detached indexes with per-parent promotion on first arbitrary mutation: exploratory default-tree wall -7.70% [-10.16%, -3.77%], cold scan -5.87% [-15.86%, -3.16%], RSS -37.79%/-45.03%, and opened discovery -1.40% [-3.51%, +1.95%] with a 0.979 scoped-allocation ratio, exact engine/commit digests, and zero opened detached builds. Commit 5d7b86f passes the complete clean-checkout make check handoff gate, Apple/Windows cross-lint, and all 19 jobs in GitHub CI run 33558069518 on Ubuntu, macOS, and Windows. Deterministic allocation ceilings were tightened with a negative-test proof. The host remains uncontrolled because an unrelated test process holds a core; quiet-host final-binary confirmation and Linux H86 evidence remain open. Local continuation is additionally blocked at 127 MiB free: all disposable debug, docs, cross-target, and smoke-venv outputs have been staged in Trash along with the earlier incremental artifacts, but APFS space will not return until the user empties Trash. The bead stays in progress.
