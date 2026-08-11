---
type: is
id: is-01kzs52ksq9znnf5p0pcan8rpc
title: "Loop experiment: time to a useful top-level ranking"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T19:34:02.806Z
updated_at: 2026-08-11T19:34:02.806Z
---
The metric this whole plan exists to move, and one a completion-time benchmark cannot see. New harness job measuring how long until the top-level ranking by size is stable to within a tolerance of the final answer, breadth-first against depth-first, on a home-folder-scale tree. exp-012 established that breadth-first is free on COMPLETE scans; this establishes what it buys on PARTIAL ones, which is the actual justification.
