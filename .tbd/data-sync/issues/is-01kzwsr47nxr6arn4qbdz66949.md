---
type: is
id: is-01kzwsr47nxr6arn4qbdz66949
title: "H61: Prototype dense immutable bootstrap plus sparse mutation overlay"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
dependencies: []
parent_id: is-01kzw3te81j66eehy48rx2djv5
created_at: 2026-08-13T05:33:02.579Z
updated_at: 2026-08-13T05:33:02.579Z
---
Treat the completed bootstrap as a compact dense immutable base and apply subsequent observations through a sparse mutable overlay/tombstone layer with bounded compaction. This is the larger design response to the million-scale dumac/pdu memory and wall gap after the H19-H22 layout ladder establishes the per-entry floor. Gate: preserve stable identities, exact snapshots, every query/view, progressive publication, errors, deltas, and watch semantics; target at least 40% lower million-scale RSS plus at least 3% cold indexed wall or a decisive warm/query win. Reject if lookup indirection or compaction makes steady mutation materially worse.
