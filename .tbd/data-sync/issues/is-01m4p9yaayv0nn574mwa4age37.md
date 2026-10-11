---
type: is
id: is-01m4p9yaayv0nn574mwa4age37
title: Update the shared color golden for tree size and age styling
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m4mybrzvh20q4nkmk5hjs3t1
hold: null
hold_until: null
created_at: 2026-10-11T20:19:50.748Z
updated_at: 2026-10-11T21:09:53.676Z
started_at: 2026-10-11T20:56:47.934Z
closed_at: 2026-10-11T21:09:53.674Z
close_reason: "Updated the shared color golden with only size and age ANSI changes. All 1078 core tests and CLI color tests pass; PR #196 contains the fix. Full handoff and CI validation continue."
resolution: null
duplicate_of: null
---
Review of claude/report-colors at 946e1b89 plus the current tree-only size-color edits: make check fails in query::query_report::tests::colored_tree_populations_share_one_golden (1077 core tests pass, 1 fails, 1 ignored). The fixture still expects plain size and age cells. Removing only the added size and age ANSI escapes makes the actual output byte-identical to the previous golden, so the remaining colors are preserved. Update the shared golden deliberately, inspect its diff, and rerun the handoff gate. Focused renderer tests (58), CLI color tests (4), formatting, and clippy passed. Source files were not modified by this review.
