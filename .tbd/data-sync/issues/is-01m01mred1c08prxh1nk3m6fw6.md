---
type: is
id: is-01m01mred1c08prxh1nk3m6fw6
title: Probe --no-oracle mode and engine-scoped counters
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
dependencies: []
created_at: 2026-08-15T02:42:02.273Z
updated_at: 2026-08-15T02:42:02.273Z
---
The probe oracle (path_of + digest per entry) is ~39% of scan-index instructions and 46% of allocation events (dhat, 40k subtree), and FDU_COUNTERS tallies include it, so per-entry ratios overstate engine work (resolved fdu-zgxd). Add a --no-oracle probe mode for attribution runs (timing runs keep the oracle) and scope counter guards to engine phases. Gates honest attribution for every later experiment.
