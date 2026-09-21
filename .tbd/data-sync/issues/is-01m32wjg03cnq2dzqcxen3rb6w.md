---
type: is
id: is-01m32wjg03cnq2dzqcxen3rb6w
title: Name benchmark jobs for the paths and long formats
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-09-20-directory-query-formats.md
labels: []
dependencies: []
parent_id: is-01m2y7b2f9f1cv1dssrtref7zd
created_at: 2026-09-21T21:04:55.811Z
updated_at: 2026-09-21T21:04:55.811Z
---
PR #103 review suggestion: 'Every Output Surface Is a Benchmark Job' has no named job for --format paths or --long. Add them to the performance harness beside the existing text/json jobs so a regression in flat rendering is measured rather than assumed.
