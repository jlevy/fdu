---
type: is
id: is-01m32v0hm3x4eh2ymxae4c5bz5
title: "PR #103 review R5: measure() seeds mtime_ns from structurally traversed directories selection would not admit (Low)"
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m32h6ewht09ryvnt0f5zmhz5
created_at: 2026-09-21T20:37:39.074Z
updated_at: 2026-09-21T20:37:39.074Z
---
R5 Low. query_subtrees.rs:79 takes own_time regardless of selection.ignored.admits. Unobservable today because descendants of an ignored directory are all ignored. Fix: add a guard or a comment recording why it is safe.
