---
type: is
id: is-01m1dtqbkxhdqfhczrbdctcaxq
title: Canonicalize encoded public observation paths before mutation
kind: bug
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - correctness
  - paths
dependencies:
  - type: blocks
    target: is-01m1dtqkbyrydtq60902w1sgkr
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:32:53.116Z
updated_at: 2026-09-01T07:37:08.085Z
---
Add encoded-byte or native-unit regressions for interior current-directory components and repeated separators, restore full normalization for public inputs, and reserve any canonical fast lane for a private scanner-owned type with tested invariants.

## Notes

Implementation started after snapshot-scope commit 5d4a6e2. Red-green target: byte-level canonicalization for public observation paths containing current-directory components or repeated separators, while preserving rejection of absolute, empty, parent-traversal, non-Unicode, and platform-specific invalid paths.
