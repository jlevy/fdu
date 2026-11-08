---
type: is
id: is-01m6ycafymrea5sgq8r7n621re
title: "Change time: --changed-since, --changed-before, --sort changed, changed_ns fields"
kind: feature
status: open
priority: 2
version: 9
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies:
  - type: blocks
    target: is-01m6ycahyd69v7gadhehz9e7px
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
child_order_hints:
  - is-01m6yn3x9wb56tr7fj9k0b9ded
  - is-01m6yn3y6jr59byzba5mqm9zjn
  - is-01m6yn3z4tykn7bw45j1j1eba1
  - is-01m6yn405v7pxcye1mfah3yxhe
  - is-01m6yn411vxs4rbchbvbyc1zdz
  - is-01m6yn41za441ay8y4vnmkd502
  - is-01m6yn432rea1hjv0v3r2frebb
created_at: 2026-11-08T20:06:45.972Z
updated_at: 2026-11-08T22:40:33.368Z
---
Phase 1. TimeBasis in the request model; Selection gains a change-time window (max(mtime, ctime); directories by newest change time in the eligible subtree, as for modification); SortKey::Changed; rows and tree nodes that carry mtime_ns gain changed_ns/changed_at. Unknown change time (Windows entries served by observe_listed, ctime 0) is typed: never admitted by a bound, counted in notes. Python changed_since/changed_before and SortKey.CHANGED. macOS: ATTR_CMN_CHGTIME must stay requested whenever change time is (see fdu-f6n7). Goldens prove a cp -p copy, tar -x extract, and touch -d into the past are admitted by change time and not by modification time. Switch the skill and docs recipe to --changed-since.
