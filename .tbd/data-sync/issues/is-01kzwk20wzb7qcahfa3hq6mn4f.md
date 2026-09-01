---
type: is
id: is-01kzwk20wzb7qcahfa3hq6mn4f
title: "H19-H22: Compact the reusable full-index entry layout"
kind: task
status: in_progress
priority: 1
version: 7
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
updated_at: 2026-09-01T16:55:02.395Z
started_at: 2026-09-01T15:20:04.375Z
---
Execute the existing H19-H22 memory-layout ladder against the new 1M evidence: unbox arena entries, store names once, move directory-only payloads off file entries, and compact identities/revisions. Profile and size the current Entry first; measure one structural arm at a time; preserve snapshots, deltas, stable identities, and exact query behavior. Preregister peak RSS (substantial reduction) plus wall/fault effects.

## Notes

2026-09-01 size attribution after the detached-builder win still shows Entry at 280 bytes and candidate peak RSS about 19% above the pre-rewrite historical control. The prior H98 partial optional-roll-up experiment removed 56 bytes per entry and roughly 6.37 MB requested, but improved default-tree only 2.63%, so it must not be repeated alone. A fresh historical run shows cold construction parity while the cost appears after the cold-scan timer. Profile retained layout, snapshot/report handoff, and destruction before implementing the full H86 packed representation.
