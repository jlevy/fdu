---
type: is
id: is-01m2y6d5mv0acps34bn0z9bga0
title: Add report-level semantic oracles before accepting report-path benchmark changes
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-20T01:20:34.970Z
updated_at: 2026-09-20T01:20:34.970Z
---
Follow-up from PR91/92 senior reviews: perf_probe content-query and opened-second-report validate retained index/content independently after discarding the timed report. Before using these jobs to accept further report-path changes, add an outside-timer semantic report oracle that catches incorrect rows, totals, order, or projection while preserving the measured workload and oracle-off profiling path. PR92 focused analyzed mixed-view expected-value tests cover the present H138 change; this is future harness capability, not a new timing claim.
