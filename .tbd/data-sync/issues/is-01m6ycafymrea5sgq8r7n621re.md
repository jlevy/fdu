---
type: is
id: is-01m6ycafymrea5sgq8r7n621re
title: "Change time: --changed-since, --changed-before, --sort changed, changed_ns fields"
kind: feature
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies:
  - type: blocks
    target: is-01m6ycahyd69v7gadhehz9e7px
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-08T20:06:45.972Z
updated_at: 2026-11-08T20:06:51.091Z
---
Phase 1. TimeBasis in the request model; Selection gains a change-time window (max(mtime, ctime); directories by newest change time in the eligible subtree, as for modification); SortKey::Changed; rows and tree nodes that carry mtime_ns gain changed_ns/changed_at. Unknown change time (Windows entries served by observe_listed, ctime 0) is typed: never admitted by a bound, counted in notes. Python changed_since/changed_before and SortKey.CHANGED. macOS: ATTR_CMN_CHGTIME must stay requested whenever change time is (see fdu-f6n7). Goldens prove a cp -p copy, tar -x extract, and touch -d into the past are admitted by change time and not by modification time. Switch the skill and docs recipe to --changed-since.
