---
type: is
id: is-01kzqynv3k4rf6gb6cddsnz93e
title: Isolate env-001 concurrency and platform causes on controlled hosts
kind: task
status: open
priority: 2
version: 5
spec_path: docs/project/reports/report-2026-08-11-fdu-cache-environment-matrix.md
labels:
  - performance
  - benchmark
  - research
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-08-11T08:22:58.418Z
updated_at: 2026-08-15T01:18:39.554Z
---
Repeat the env-001 frozen corrected-control versus candidate comparison on controlled Mac and Linux hosts with the same portable 60k workload and a fixed two-thread producer count, then vary worker count independently. Replace or bypass the Linux process-launcher RSS floor so peak memory varies per measured child; the matrix must not accept a row with an unmeasured resource gate. Preserve the v3 environment matrix contract, record first-output and completion latency where supported, and decide whether the cold CPU divergence follows concurrency, host, OS, or filesystem. Do not promote an auto-selector platform row until equivalent controlled cells pass the latency and resource gates.

## Notes

env-001 found all 150 GitHub Linux samples pinned to exactly 81,678,336 B peak RSS across five jobs and two variants. The matrix now fails that degenerate signal closed. A controlled repeat needs launcher-independent RSS, plus fixed two-thread and worker-scaling cells.
