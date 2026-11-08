---
type: is
id: is-01m6yn411vxs4rbchbvbyc1zdz
title: "Unknown change time: count entries without an observed change time in a note"
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies:
  - type: blocks
    target: is-01m6yn41za441ay8y4vnmkd502
parent_id: is-01m6ycafymrea5sgq8r7n621re
created_at: 2026-11-08T22:40:31.291Z
updated_at: 2026-11-08T23:16:25.119Z
closed_at: 2026-11-08T23:16:25.119Z
close_reason: Done in 2757fdcb on claude/eloquent-galileo-dd771d
resolution: null
duplicate_of: null
---
Spec: 'An entry whose change time was not observed has an unknown change time ... a note counts such entries.' When a request uses change time (window, sort, or ages) and some counted entries lack it, the report carries a typed count and one note. Today only Windows entries served by observe_listed lack it.
