---
type: is
id: is-01m6yn3x9wb56tr7fj9k0b9ded
title: "Change time in the selection model: time window, Selection.changed, Candidate change time, --sort changed key"
kind: task
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies:
  - type: blocks
    target: is-01m6yn3y6jr59byzba5mqm9zjn
  - type: blocks
    target: is-01m6yn3z4tykn7bw45j1j1eba1
parent_id: is-01m6ycafymrea5sgq8r7n621re
hold: null
hold_until: null
created_at: 2026-11-08T22:40:27.452Z
updated_at: 2026-11-08T23:16:21.900Z
started_at: 2026-11-08T22:41:43.114Z
closed_at: 2026-11-08T23:16:21.900Z
close_reason: Done in 2757fdcb on claude/eloquent-galileo-dd771d
resolution: null
duplicate_of: null
---
Spec: Change time; Change-time windows and sort. Generalize ModifiedWindow into a window type reused for change time; Selection gains a change-time window (admits and is_unfiltered honor it); Candidate carries change time = max(mtime, ctime) or unknown when ctime was not observed (ctime_ns 0); SortKey::Changed parses 'changed'. Public-struct additions are a 0.x breaking change: note in CHANGELOG. Unit tests: half-open bounds, max(mtime, ctime), unknown never admitted.
