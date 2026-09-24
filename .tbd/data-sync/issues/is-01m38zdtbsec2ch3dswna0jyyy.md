---
type: is
id: is-01m38zdtbsec2ch3dswna0jyyy
title: "PR #120 review R-5: Saving is at most one frame on the one-shot route; undocumented"
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
created_at: 2026-09-24T05:50:14.903Z
updated_at: 2026-09-24T05:56:04.867Z
started_at: 2026-09-24T05:50:16.026Z
closed_at: 2026-09-24T05:56:04.867Z
close_reason: "Fixed in f636e583 on claude/progress-indicator-review-fixes; coordinator merges into #120"
resolution: null
duplicate_of: null
---
crates/fdu/src/cli.rs:729-777: the ticker stops before pending_save.join(). Document in the plan's Clearing bullet and prepare_report_with_progress's rustdoc. PR #120 review.
