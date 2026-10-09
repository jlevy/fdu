---
type: is
id: is-01m4gc8061k2s6rnzg9mtywpts
title: "PR #191 A3: performance evidence not in the ledger; goal sentence overclaims"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4gc7na3mkr44xafhsfkk6sm
hold: null
hold_until: null
created_at: 2026-10-09T13:04:38.591Z
updated_at: 2026-10-09T13:56:23.759Z
started_at: 2026-10-09T13:04:48.425Z
closed_at: 2026-10-09T13:56:23.757Z
close_reason: fixed in df3008f8 (exp-209 rejected, exp-210/211 accepted, H191-H192, ledger and page regenerated) and 8ff60613 (goal states the per-row bar and measured cost); Linux cell deferred to fdu-088k
resolution: null
duplicate_of: null
---
Medium. Record run-tree-age-column (rejected) and run-tree-age-rollup (accepted) with make perf-record; fix the spec Goals. Review: https://github.com/jlevy/fdu/pull/191#issuecomment-6081386402
