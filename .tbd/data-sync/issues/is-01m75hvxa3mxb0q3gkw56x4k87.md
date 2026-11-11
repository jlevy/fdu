---
type: is
id: is-01m75hvxa3mxb0q3gkw56x4k87
title: "PR #209 H8 (suggestion): read a crossed directory where it is listed, not where its parent is"
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
created_at: 2026-11-11T14:58:20.611Z
updated_at: 2026-11-11T15:59:04.664Z
started_at: 2026-11-11T14:58:41.544Z
closed_at: 2026-11-11T15:59:04.663Z
close_reason: Deferred into fdu-vxji (one bead for the remainder of H4 and H8, under fdu-tjsi).
resolution: null
duplicate_of: null
---
Suggestion. entered_directory_attrs runs in the parent's listing loop (scan.rs:2145-2149, :4159-4163, five other sites). Review: https://github.com/jlevy/fdu/pull/209#issuecomment-6792308952 (PR #209).
