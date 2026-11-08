---
type: is
id: is-01m6yn41za441ay8y4vnmkd502
title: "Change-time tests: aging by request instant, synthetic edges, soundness goldens, Python"
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies:
  - type: blocks
    target: is-01m6yn432rea1hjv0v3r2frebb
  - type: blocks
    target: is-01m6yn47s96jehx5cwhk4176nc
parent_id: is-01m6ycafymrea5sgq8r7n621re
created_at: 2026-11-08T22:40:32.234Z
updated_at: 2026-11-08T23:16:26.081Z
closed_at: 2026-11-08T23:16:26.080Z
close_reason: Done in 2757fdcb on claude/eloquent-galileo-dd771d
resolution: null
duplicate_of: null
---
Spec: Testing Strategy. Engine tests build requests at later instants (Request::build(spec, later, axes)) to age fixtures by both clocks; synthetic entry facts for exact nanosecond edges and unknown change time; a CLI golden where cp -p, tar -x, and touch -d arrivals are admitted by --changed-since and not by --modified-since (chmod admitted too); Python tests relative to now.
