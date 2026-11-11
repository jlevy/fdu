---
type: is
id: is-01m75hvqx1x1pzwrgx67qfq796
title: "PR #209 H2: root_key guesses the root's recorded device; four ordinary cases guess wrong"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m75hvp13zk79cex02q85ffc1
hold: null
hold_until: null
created_at: 2026-11-11T14:58:15.073Z
updated_at: 2026-11-11T16:11:14.744Z
started_at: 2026-11-11T14:58:36.441Z
closed_at: 2026-11-11T16:11:14.744Z
close_reason: Fixed in 2886568e (root device recorded on every walk route, snapshot format 7, root_key and inferred_root_device deleted), tests 03c8dc3f.
resolution: null
duplicate_of: null
---
Medium. query/query_volumes.rs:377-411. Fix: record the root's device at the walk in the index (root entry attributes), persist in the snapshot (format bump), key the root by it on every route; delete root_key and inferred_root_device; add the four cases to the unit test. Review: https://github.com/jlevy/fdu/pull/209#issuecomment-6792308952 (PR #209).
