---
type: is
id: is-01m01ec396v5crqyg5sfasfehr
title: "H87: Qualify Apple Silicon hardware bounds and interactive-host pressure"
kind: task
status: open
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - research
  - experiment
  - macos
dependencies:
  - type: blocks
    target: is-01m01ebsw9cyhe8thve19grn1w
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:50:26.213Z
updated_at: 2026-08-15T01:18:39.802Z
---
Own the Apple Silicon/local-APFS slice of the controlled-host question tracked more broadly by fdu-wfvx: reuse that bead’s portable environment contract, publish compatible Mac cells back to it, and do not wait for or duplicate its Linux launcher/RSS work. Compare only distinct fixed controls among logical CPUs, performance cores, all cores, available parallelism, and bounded oversubscription, then repeat automatic-policy cells on a defined quiet host and under one documented reproducible interactive/background-load regime. Do not duplicate equivalent logical/all/available arms on a host.

Acceptance: record achieved CPU/wall parallelism, system CPU, scheduler pressure, P/E topology, thermal/power state when available, RSS, faults, and context switches; predeclare host-idle and load acceptance windows and invalidate samples outside them; use discovery and held-out confirmation separately; decide whether hardware supplies bounds only or a stable selector signal. Do not add P/E sysctls or load-aware branches without a confirmed reversal that justifies the platform complexity. Feed measured bounds into fdu-9x4o and cross-reference overlapping fdu-wfvx artifacts rather than copying them.
