---
type: is
id: is-01m0raccwe6ywyac61ezhxk2ws
title: Per-result work counters on every query result
kind: feature
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T22:03:13.678Z
updated_at: 2026-08-23T22:03:13.678Z
---
Entries visited, directories visited, rows returned, lock wait, query wall and CPU, and bytes copied across the binding, reported beside each result as execution telemetry rather than semantic payload. Converts 'no hidden O(index) pass' from a review principle into an assertable contract: a frequent read must be proportional to its output or to a maintained index, and a counter makes a regression visible. Feeds the client's own serving benchmark.
