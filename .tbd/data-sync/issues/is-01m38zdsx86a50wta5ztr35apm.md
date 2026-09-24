---
type: is
id: is-01m38zdsx86a50wta5ztr35apm
title: "PR #120 review R-4: doc accuracy (CI empty, per-directory Option check, one walk cache line)"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m38zd0fgqycxzrgtyps04zsx
hold: null
hold_until: null
created_at: 2026-09-24T05:50:14.439Z
updated_at: 2026-09-24T05:56:04.859Z
started_at: 2026-09-24T05:50:16.020Z
closed_at: 2026-09-24T05:56:04.859Z
close_reason: "Fixed in f636e583 on claude/progress-indicator-review-fixes; coordinator merges into #120"
resolution: null
duplicate_of: null
---
docs/usage.md:306 (CI may be empty), crates/fdu-core/src/progress.rs:24 (per directory on revalidation/reconcile), plan line 157 on the #120 branch (walk counters share one line). PR #120 review.
