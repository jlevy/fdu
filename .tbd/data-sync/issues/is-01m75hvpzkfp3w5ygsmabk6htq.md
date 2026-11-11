---
type: is
id: is-01m75hvpzkfp3w5ygsmabk6htq
title: "PR #209 H1: volumes mounts an expired indirect-map key or kernel automount on a route that did not just walk"
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
created_at: 2026-11-11T14:58:14.130Z
updated_at: 2026-11-11T16:11:13.912Z
started_at: 2026-11-11T14:58:35.609Z
closed_at: 2026-11-11T16:11:13.912Z
close_reason: Fixed in 2886568e (mount-table gate) with tests c99ddaca; docs 9d74ace1.
resolution: null
duplicate_of: null
---
High. query/query_volumes.rs:281-286 (directory_device), :333-345 (the check), :406 (root_key stats). AT_NO_AUTOMOUNT covers only the last component, where it exists; a missing indirect key, a path through a trigger the index has no entry for, and an entry skipped because its filesystem was asked earlier all mount. Fix: gate every non-root capacity call on the OS mount table (Linux mountinfo, macOS getfsstat MNT_NOWAIT), check every listed entry, never read through one that failed; root-only tests in tests/volumes_automount.rs. Review: https://github.com/jlevy/fdu/pull/209#issuecomment-6792308952 (PR #209).
