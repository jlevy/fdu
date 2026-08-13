---
type: is
id: is-01kzwk20wzb7qcahfa3hq6mn4f
title: "H19-H22: Compact the reusable full-index entry layout"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
dependencies:
  - type: blocks
    target: is-01kzwsr47nxr6arn4qbdz66949
parent_id: is-01kzw3te81j66eehy48rx2djv5
created_at: 2026-08-13T03:36:06.814Z
updated_at: 2026-08-13T05:33:02.579Z
---
Execute the existing H19-H22 memory-layout ladder against the new 1M evidence: unbox arena entries, store names once, move directory-only payloads off file entries, and compact identities/revisions. Profile and size the current Entry first; measure one structural arm at a time; preserve snapshots, deltas, stable identities, and exact query behavior. Preregister peak RSS (substantial reduction) plus wall/fault effects.
