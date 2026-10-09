---
type: is
id: is-01m4haj231ydz2wgf6hy7w2cy1
title: "PR #192 D8: roots-default-tree job claims to price snapshot naming it never does"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4hahez6scpwtc6yrg9qws5e
hold: null
hold_until: null
created_at: 2026-10-09T21:54:25.503Z
updated_at: 2026-10-09T23:36:06.086Z
started_at: 2026-10-09T21:54:43.159Z
closed_at: 2026-10-09T23:36:06.082Z
close_reason: "fixed in 9dc13460: probe doc and measure.py job description no longer claim the job names a snapshot; job unchanged"
resolution: null
duplicate_of: null
---
Nit. explorations/benchmarks/realtree/measure.py:404, crates/fdu-core/examples/perf_probe.rs:1109. Fix descriptions. PR #192, review D: https://github.com/jlevy/fdu/pull/192#issuecomment-6089866519
