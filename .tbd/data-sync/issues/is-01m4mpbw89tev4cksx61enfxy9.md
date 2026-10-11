---
type: is
id: is-01m4mpbw89tev4cksx61enfxy9
title: "PR #194 A10: balance slots in multi-arm cells (cyclic rotation / Latin square schedule)"
kind: task
status: in_progress
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: codex-pr194-finish
labels: []
dependencies: []
parent_id: is-01m4mpb135bvc7rjcgran42387
hold: null
hold_until: null
created_at: 2026-10-11T05:18:29.128Z
updated_at: 2026-10-11T21:59:14.256Z
started_at: 2026-10-11T21:53:45.622Z
---
Suggestion. round-robin-by-ordinal-v1 only reverses order, so middle arms never run first or last. Review: https://github.com/jlevy/fdu/pull/194#issuecomment-6105735578

## Notes

Deferred outside this record-correction PR. Waits on a separately qualified harness protocol revision with an explicit schedule identity and tests proving every arm occupies every slot equally. Retain the original recorded schedule of the committed runs; continue using two-arm cells for deciding comparisons until the new schedule is qualified.
