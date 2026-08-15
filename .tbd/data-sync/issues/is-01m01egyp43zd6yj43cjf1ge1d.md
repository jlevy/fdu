---
type: is
id: is-01m01egyp43zd6yj43cjf1ge1d
title: Profile the heterogeneous macOS worker-policy failure before changing it
kind: task
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - profiling
  - research
dependencies:
  - type: blocks
    target: is-01m01ec396v5crqyg5sfasfehr
  - type: blocks
    target: is-01m01ecbhsetn1rmvfn8m26w7e
  - type: blocks
    target: is-01m01eg0efe53jc3smgaza7wk7
  - type: blocks
    target: is-01kzy1w2vbam0mr1z5we4y6fy0
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:53:05.347Z
updated_at: 2026-08-15T00:53:13.406Z
---
Create an immutable, path-redacted reproduction of the Application Support scheduling shape and profile the shipped automatic policy, fixed 6/available/16 controls, and pinned dust before proposing a production change. Combine policy telemetry, per-layer counters, process metrics, and macOS stack samples to separate directory open/bulk enumeration, portable fallback, queue starvation/contention, path/batch work, the index/report consumer, rendering, and scheduler delay. Compare perf_probe with the actual CLI so CLI-only cost is not inferred away.

Acceptance: the profile explains the two automatic latency populations or explicitly falsifies the current completion-order diagnosis; samples name the policy decision and backend; fdu and dust perform semantically comparable complete work for claim-grade profiling, with the partial live tree retained only as a diagnostic; no private paths enter artifacts; the resulting bottleneck and candidate signals are written into the research loop before H86-H89 or H70 are decided.
