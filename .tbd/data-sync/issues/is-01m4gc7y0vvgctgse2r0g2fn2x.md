---
type: is
id: is-01m4gc7y0vvgctgse2r0g2fn2x
title: "PR #191 A1: Linux folded tree drops directory and symlink times (H185 skip)"
kind: bug
status: in_progress
priority: 1
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4gc7na3mkr44xafhsfkk6sm
hold: null
hold_until: null
created_at: 2026-10-09T13:04:36.378Z
updated_at: 2026-10-09T13:04:46.803Z
started_at: 2026-10-09T13:04:46.801Z
---
Blocker. scan.rs:3513-3520 DetachedEmission::skip_dir_symlink_stat, scan.rs:1483-1513 listed_child_kind_and_attrs, scan.rs:3833 linux_dents StatPolicy, index.rs:6148/6156. Decision: turn the skip off on the folded tree route. Review: https://github.com/jlevy/fdu/pull/191#issuecomment-6081386402
