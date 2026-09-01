---
type: is
id: is-01m1dtqb9q9fnaqpwr5cw90j0m
title: Fix controls-on snapshot reuse for controls-off public opens
kind: bug
status: in_progress
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - correctness
  - cache
dependencies:
  - type: blocks
    target: is-01m1dtqkbyrydtq60902w1sgkr
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:32:52.790Z
updated_at: 2026-09-01T07:25:09.833Z
---
Add red tests for Auto and Only, require exact scope for every returned Index, and confine any directional controls-on to controls-off reuse to a consumed report projection. Public opens must neither fail late with ScanScopeMismatch nor return retained control state under a controls-off scope.

## Notes

Implemented on codex/streaming-performance-parity with red-green coverage. Initial exact admission fixed public Auto late ScanScopeMismatch and public Only incompatible-index escape. Full make check then exposed the required report-only exception: watch sessions persist controls-on snapshots consumed by ordinary cache-only CLI reports. Final design keeps every returned Index exact, treats scanning report mismatches as cold misses, and permits controls-on to controls-off projection only for no-scan reports; the Report is retagged to requested scope. A nine-view JSON parity test matches a cold controls-off report. Focused scope tests (4) and watch-persistence integration tests (2) pass. Full gate rerun pending.
