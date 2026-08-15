---
type: is
id: is-01m01ec396v5crqyg5sfasfehr
title: "H87: Measure hardware bounds and interactive-host pressure for worker selection"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - research
  - experiment
  - macos
dependencies:
  - type: blocks
    target: is-01m01cm1sb8xyw9ag3pabb5s3h
  - type: blocks
    target: is-01m01ebsw9cyhe8thve19grn1w
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:50:26.213Z
updated_at: 2026-08-15T00:52:00.834Z
---
Determine what hardware should constrain versus select. On Apple Silicon, compare logical CPUs, performance cores, all cores, and bounded oversubscription as fixed controls, then repeat automatic-policy cells on a quiet host and under a documented reproducible interactive/background-load regime. Record achieved CPU/wall parallelism, system CPU, scheduler pressure, P/E topology, thermal/power state where available, RSS, faults, and context switches. Coordinate with the broader controlled-host work in fdu-wfvx without duplicating its cross-platform environment matrix.

The expected outcome may be that hardware provides only lower/upper bounds. Do not introduce P/E-core sysctls or load-aware branches unless paired evidence shows a stable reversal worth the platform complexity. Feed the measured bounds and host-pressure behavior into H86 and the implementation bead; record negative results in the experiment ledger.
