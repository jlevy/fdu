---
type: is
id: is-01m38zdsggx9wntg8enq5q10yx
title: "PR #120 review R-3: an empty TERM is refused where an unset TERM is allowed (Windows)"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m38zd0fgqycxzrgtyps04zsx
hold: null
hold_until: null
created_at: 2026-09-24T05:50:14.031Z
updated_at: 2026-09-24T05:56:04.852Z
started_at: 2026-09-24T05:50:16.015Z
closed_at: 2026-09-24T05:56:04.852Z
close_reason: "Fixed in f636e583 on claude/progress-indicator-review-fixes; coordinator merges into #120"
resolution: null
duplicate_of: null
---
crates/fdu/src/progress_line.rs:97. Treat a set-and-empty TERM as unset; the 96-reading table then has 8 interactive readings. PR #120 review.
