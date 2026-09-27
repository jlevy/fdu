---
type: is
id: is-01m2y6d5mv0acps34bn0z9bga0
title: Add report-level semantic oracles before accepting report-path benchmark changes
kind: task
status: in_progress
priority: 2
version: 2
delegate: codex
labels: []
dependencies: []
created_at: 2026-09-20T01:20:34.970Z
updated_at: 2026-09-27T07:31:37.647Z
---
Follow-up from PR91/92 senior reviews: perf_probe content-query and opened-second-report validate retained index/content independently after discarding the timed report. Before using these jobs to accept further report-path changes, add an outside-timer semantic report oracle that catches incorrect rows, totals, order, or projection while preserving the measured workload and oracle-off profiling path. PR92 focused analyzed mixed-view expected-value tests cover the present H138 change; this is future harness capability, not a new timing claim.

## Notes

2026-09-27: H152 delivered the content-query half in commit 1ba06b19. The probe now validates the combined four-view report outside the timer against independent single-view reports, including section order and exact debug-visible rows/totals/projection. Focused probe tests (21/21) and the full fdu-core no-default-features suite (774 passed, 1 ignored plus integration/doc tests) pass. Keep this bead open for the opened-second-report oracle.
