---
type: is
id: is-01m2xawgy05gqny62s1tk7gd4q
title: "PR #91 review S2: share opened settle loop"
kind: task
status: closed
priority: 3
version: 3
delegate: unknown@spud10
labels: []
dependencies: []
parent_id: is-01m2xawb8wrb4e550za5ggdcw8
hold: null
hold_until: null
created_at: 2026-09-19T17:19:37.919Z
updated_at: 2026-09-19T17:26:55.202Z
started_at: 2026-09-19T17:19:41.260Z
closed_at: 2026-09-19T17:26:55.202Z
close_reason: "Addressed on PR #91 in e667b739: R1 apply timer isolated from decode; R2 completeness and close paths; S1–S3 applied cheaply."
resolution: null
duplicate_of: null
---
PR #91. Suggestion (cheap only).

Share the opened settle loop between opened_discovery and opened_second_report after R2, so the next probe mode cannot fork completeness again.
