---
type: is
id: is-01m37nxa8vshzngyvzw9ax67a0
title: Measure the progress handle's cost
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
labels:
  - cli
  - ux
dependencies: []
parent_id: is-01m2nsj4zgw7r9j3d1rphnmwf2
created_at: 2026-09-23T17:44:42.521Z
updated_at: 2026-09-24T08:20:20.619Z
---
Add a progress-handle option to the performance probe; interleaved paired make perf-compare on a real tree: no handle must be indistinguishable from main, handle attached within noise. Record with make perf-record (regime stated), then make perf-ledger and make perf-report.

## Notes

2026-09-24: exp-156 (indicator without a handle vs main) -1.78% [-6.33%, +2.90%], no regression; exp-157 (handle attached vs none) +5.75% [-5.34%, +10.91%], inconclusive under host load 45-90. Open only for a quiet-host rerun of exp-157.
