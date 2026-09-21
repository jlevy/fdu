---
type: is
id: is-01m32v0hm3x4eh2ymxae4c5bz5
title: "PR #103 review R5: measure() seeds mtime_ns from structurally traversed directories selection would not admit (Low)"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6ewht09ryvnt0f5zmhz5
created_at: 2026-09-21T20:37:39.074Z
updated_at: 2026-09-21T21:11:49.704Z
closed_at: 2026-09-21T21:11:49.704Z
close_reason: "Fixed in 5d6e56a2: own_time seeded only when the selection admits the directory, with the reason recorded."
resolution: null
duplicate_of: null
---
R5 Low. query_subtrees.rs:79 takes own_time regardless of selection.ignored.admits. Unobservable today because descendants of an ignored directory are all ignored. Fix: add a guard or a comment recording why it is safe.
