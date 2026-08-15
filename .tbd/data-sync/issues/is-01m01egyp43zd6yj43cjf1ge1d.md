---
type: is
id: is-01m01egyp43zd6yj43cjf1ge1d
title: Profile the heterogeneous macOS worker-policy failure before changing it
kind: task
status: closed
priority: 1
version: 14
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - profiling
  - research
dependencies:
  - type: blocks
    target: is-01m01eahm9z4y9w8a36423xrt1
  - type: blocks
    target: is-01m01eb1b1pkyywa9v6mzsar85
  - type: blocks
    target: is-01m01easxwr1504mqmnha3brzh
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:53:05.347Z
updated_at: 2026-08-15T11:06:42.069Z
closed_at: 2026-08-15T11:06:42.069Z
close_reason: Profiled the frozen 100,001-entry Apple Silicon/APFS reproduction before controller work. Shipped held at six with 73.71% kernel/syscall stacks; repeated scaling reached sixteen, 80.72% kernel/syscall stacks, and materially more lock wait. Bulk succeeded for all 60,314 directories with no fallback.
---
Before changing controller behavior, create one immutable, path-redacted reproduction of the observed Application Support scheduling shape and profile the shipped automatic policy plus fixed 6, available-parallelism, and 16 controls. Use policy/backend telemetry, per-layer counters, process metrics, and macOS stack samples to attribute directory open and bulk enumeration, portable fallback, ready/in-flight starvation or contention, path/batch work, index/report consumption, rendering, and scheduler delay. Compare perf_probe with the actual CLI so CLI-only cost is visible; dust may be used as an exploratory phase-attribution reference until fdu-b722 makes it claim-grade.

Acceptance: the profile explains the two observed automatic latency populations or explicitly falsifies the completion-order diagnosis; every sample names policy history and backend; the frozen subject is complete, immutable, semantically verified, and contains no private paths; partial live-tree evidence remains diagnostic only; conclusions identify which signals deserve broader corpora/controller research and which hypotheses are rejected before fdu-w3ra, fdu-7y4v, H86-H89, or H70 consume them.
