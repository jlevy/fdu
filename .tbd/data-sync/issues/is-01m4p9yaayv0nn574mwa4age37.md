---
type: is
id: is-01m4p9yaayv0nn574mwa4age37
title: Update the shared color golden for tree size and age styling
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4mybrzvh20q4nkmk5hjs3t1
created_at: 2026-10-11T20:19:50.748Z
updated_at: 2026-10-11T20:19:50.748Z
---
Review of claude/report-colors at 946e1b89 plus the current tree-only size-color edits: make check fails in query::query_report::tests::colored_tree_populations_share_one_golden (1077 core tests pass, 1 fails, 1 ignored). The fixture still expects plain size and age cells. Removing only the added size and age ANSI escapes makes the actual output byte-identical to the previous golden, so the remaining colors are preserved. Update the shared golden deliberately, inspect its diff, and rerun the handoff gate. Focused renderer tests (58), CLI color tests (4), formatting, and clippy passed. Source files were not modified by this review.
