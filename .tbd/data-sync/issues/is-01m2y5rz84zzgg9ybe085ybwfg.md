---
type: is
id: is-01m2y5rz84zzgg9ybe085ybwfg
title: "PR #92 review R1: remove single-view full-tree copy regression"
kind: bug
status: closed
priority: 1
version: 4
delegate: unknown@spud10
labels: []
dependencies: []
parent_id: is-01m2y58e5s7yzpstkcqmheymy5
hold: null
hold_until: null
created_at: 2026-09-20T01:09:33.059Z
updated_at: 2026-09-20T01:26:29.683Z
started_at: 2026-09-20T01:20:42.954Z
closed_at: 2026-09-20T01:26:29.683Z
close_reason: "Addressed in f8a2ed94 on PR #92: R3 rejects noncanonical snapshot names and counts visited files; R1 owns a single-view walk; R2 covers analyzed mixed-view report semantics."
resolution: null
duplicate_of: null
---

## Notes

PR92 R1 review https://github.com/jlevy/fdu/pull/92#pullrequestreview-5258707833; query_report.rs1024/1345 eager full-tree clone even largest limit1. Reproduced201001 extra allocations on201000entries.
