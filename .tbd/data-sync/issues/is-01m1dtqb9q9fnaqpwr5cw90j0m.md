---
type: is
id: is-01m1dtqb9q9fnaqpwr5cw90j0m
title: Fix controls-on snapshot reuse for controls-off public opens
kind: bug
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - correctness
  - cache
dependencies:
  - type: blocks
    target: is-01m1dtqkbyrydtq60902w1sgkr
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:32:52.790Z
updated_at: 2026-09-01T06:33:01.053Z
---
Add red tests for Auto and Only, require exact scope for every returned Index, and confine any directional controls-on to controls-off reuse to a consumed report projection. Public opens must neither fail late with ScanScopeMismatch nor return retained control state under a controls-off scope.
