---
type: is
id: is-01kzwk20wzb7qcahfa3hq6mn4f
title: "H19-H22: Compact the reusable full-index entry layout"
kind: task
status: in_progress
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
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
updated_at: 2026-09-01T15:20:04.375Z
started_at: 2026-09-01T15:20:04.375Z
---
Execute the existing H19-H22 memory-layout ladder against the new 1M evidence: unbox arena entries, store names once, move directory-only payloads off file entries, and compact identities/revisions. Profile and size the current Entry first; measure one structural arm at a time; preserve snapshots, deltas, stable identities, and exact query behavior. Preregister peak RSS (substantial reduction) plus wall/fault effects.
