---
type: is
id: is-01m1dtqbkxhdqfhczrbdctcaxq
title: Canonicalize encoded public observation paths before mutation
kind: bug
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - correctness
  - paths
dependencies:
  - type: blocks
    target: is-01m1dtqkbyrydtq60902w1sgkr
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:32:53.116Z
updated_at: 2026-09-01T07:56:44.946Z
---
Add encoded-byte or native-unit regressions for interior current-directory components and repeated separators, restore full normalization for public inputs, and reserve any canonical fast lane for a private scanner-owned type with tested invariants.

## Notes

Implemented red-green on codex/streaming-performance-parity. The encoded-output regression failed against the inherited fast lane with dotted/./file.txt retained verbatim. Public normalization now validates and rebuilds in one component pass into a pre-sized PathBuf, canonicalizing current-directory components and repeated native separators without a temporary component vector. Exact commit changes, compatibility operations, and dirty impact paths are checked by encoded bytes; existing root, escape, non-Unicode, and platform cases remain green. Minimal and all-feature fdu-core suites, isolated make check, and macOS/Windows make cross-lint pass. Commit and GitHub CI pending.
