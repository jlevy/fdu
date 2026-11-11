---
type: is
id: is-01m75v5s1wh91jay2a7ekrvww1
title: "PR #209 I4: Python Index.since() gives no upsert its changed_ns, and after a remount lists the root"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m75v586ca3vv6vremva5h5er
hold: null
hold_until: null
created_at: 2026-11-11T17:41:01.116Z
updated_at: 2026-11-11T18:08:52.129Z
started_at: 2026-11-11T17:41:26.281Z
closed_at: 2026-11-11T18:08:52.129Z
close_reason: "Fixed in c9601842: since() upserts carry changed_ns = attrs.change_time_ns(); the root's own upsert/remove records are left out (as a watch's are), root invalidations kept (coordinator agreed). Python tests test_since_gives_each_upsert_its_change_time and test_since_never_lists_the_root_after_its_filesystem_is_renumbered (root-only Linux) fail before, pass after; pytest 88 passed."
resolution: null
duplicate_of: null
---
Low. crates/fdu-py/src/lib.rs:455-465; field meaning at crates/fdu-py/python/fdu/_models.py:1356-1360. Fix: set changed_ns from attrs.change_time_ns() and leave the empty (root) path out of change records, with a Python test for both. Review: https://github.com/jlevy/fdu/pull/209#issuecomment-6802796047 (PR #209).
