---
type: is
id: is-01m32v04ycdtma0gmpaj9hyja5
title: "PR #96 review R3: existing requests change their answers, unchanged claim unscoped (High)"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m32h6cjrkz58ce2t05m3n2v7
created_at: 2026-09-21T20:37:26.091Z
updated_at: 2026-09-21T20:37:26.091Z
---
R3 High. plan :90-96, :142, :149-150. --min-size/--modified-* without --kind now test subtrees; newest-mtime redefined. Fix: list every pre-existing request whose answer changes, scope line 90 to default text output, decide summary.newest_mtime_ns schema consequence.
