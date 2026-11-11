---
type: is
id: is-01m74c902xda2h06ng9a6gn9zh
title: "PR #209 F1: volumes statfs on an unlisted directory mounts an autofs trigger"
kind: bug
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m74c8yycvgvjfyne7k65v8gv
created_at: 2026-11-11T04:01:23.549Z
updated_at: 2026-11-11T06:09:28.605Z
closed_at: 2026-11-11T06:09:28.605Z
close_reason: "Fixed in 0248fe77: capacity asked only of roots and directories the walk listed; seam tests and the root-only autofs test. Dispositions: https://github.com/jlevy/fdu/pull/209#issuecomment-6788495799"
resolution: null
duplicate_of: null
---
High. query_volumes.rs:135-145,228-230; capacity_statfs.rs:58. Capacity must be observed only for scan roots and volume entries the walk listed. Review F: https://github.com/jlevy/fdu/pull/209#issuecomment-6786513339
