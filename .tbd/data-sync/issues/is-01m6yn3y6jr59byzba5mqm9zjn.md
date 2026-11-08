---
type: is
id: is-01m6yn3y6jr59byzba5mqm9zjn
title: "Directory change time: newest change time per subtree, with completeness"
kind: task
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies:
  - type: blocks
    target: is-01m6yn3z4tykn7bw45j1j1eba1
  - type: blocks
    target: is-01m6yn405v7pxcye1mfah3yxhe
  - type: blocks
    target: is-01m6yn45sy1gvyfdtpgwvvnhvw
parent_id: is-01m6ycafymrea5sgq8r7n621re
created_at: 2026-11-08T22:40:28.370Z
updated_at: 2026-11-08T23:16:22.719Z
closed_at: 2026-11-08T23:16:22.719Z
close_reason: Done in 2757fdcb on claude/eloquent-galileo-dd771d
resolution: null
duplicate_of: null
---
Spec: Change time ('A directory's change time is the newest change time in its eligible subtree'). query_subtrees::measure and activity gain newest change time and whether every counted entry's change time was observed; a directory with an unobserved one matches no change bound, as an incomplete subtree matches no modification bound. Tests mirror the modified-window directory tests.
