---
type: is
id: is-01m2xawgfnhk5cp3ys55zxr3q4
title: "PR #91 review S1: sidecar-load nested roll-up test"
kind: task
status: closed
priority: 3
version: 3
delegate: unknown@spud10
labels: []
dependencies: []
parent_id: is-01m2xawb8wrb4e550za5ggdcw8
hold: null
hold_until: null
created_at: 2026-09-19T17:19:37.460Z
updated_at: 2026-09-19T17:26:55.188Z
started_at: 2026-09-19T17:19:41.250Z
closed_at: 2026-09-19T17:26:55.188Z
close_reason: "Addressed on PR #91 in e667b739: R1 apply timer isolated from decode; R2 completeness and close paths; S1–S3 applied cheaply."
resolution: null
duplicate_of: null
---
PR #91. Suggestion (cheap only).

One sidecar-load test that compares rollup(\"\") and one nested directory to a freshly analyzed index. ContentIndex unit test is the real H115 evidence; content_digest only hashes the root roll-up (perf_probe.rs:1513-1546).
