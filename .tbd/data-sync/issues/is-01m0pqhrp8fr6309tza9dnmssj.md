---
type: is
id: is-01m0pqhrp8fr6309tza9dnmssj
title: "PR #38 review R2: make perf-test is not in make check"
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m0pqh0yf7etx8dywann7tnx3
created_at: 2026-08-23T07:14:52.232Z
updated_at: 2026-08-23T07:34:37.342Z
closed_at: 2026-08-23T07:34:37.341Z
close_reason: "Fixed: perf-test added to make check, and a new CI job 'Performance evidence' runs it. CI ran explorations/benchmarks/tests (67 tests) but never explorations/benchmarks/realtree/tests (184), so make check alone would not have bound CI."
---
The ~10k-line harness has 164 tests that CI never runs; the five rendering branches added in PR #38 could have shipped uncovered. Makefile:85 check target already runs perf-report-check in the same PERF_UV env, so adding perf-test costs ~4s.
