---
type: is
id: is-01m4ghh1tfx8b7fb8fb0xdyqng
title: "PR #191 B1: watch and refresh never re-read a changed directory own time"
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4ghgje0a1raa8fwqy6m085k
hold: null
hold_until: null
created_at: 2026-10-09T14:36:58.063Z
updated_at: 2026-10-09T15:39:09.086Z
started_at: 2026-10-09T14:37:11.145Z
closed_at: 2026-10-09T15:39:09.085Z
close_reason: fixed in 91a86abd (watch), e29157e4 (refresh), ae90aef4 (docs); red tests vs cold walk; reply https://github.com/jlevy/fdu/pull/191#issuecomment-6084147920
resolution: null
duplicate_of: null
---
High. crates/fdu-core/src/watch.rs:1007-1166 verify_intent stats only the event path; scan.rs:5734 reconcile_paths_target and :6066-6127 reconcile_target_inner never observe the parent. Removing or moving an old-stamped file in leaves the parent row age stale vs a cold walk. Review: https://github.com/jlevy/fdu/pull/191#issuecomment-6082917102
