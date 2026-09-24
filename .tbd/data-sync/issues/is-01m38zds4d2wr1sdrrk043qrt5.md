---
type: is
id: is-01m38zds4d2wr1sdrrk043qrt5
title: "PR #120 review R-2: pty test skips when the binary is missing and leaks the child on timeout"
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
created_at: 2026-09-24T05:50:13.644Z
updated_at: 2026-09-24T05:56:04.844Z
started_at: 2026-09-24T05:50:16.008Z
closed_at: 2026-09-24T05:56:04.844Z
close_reason: "Fixed in f636e583 on claude/progress-indicator-review-fixes; coordinator merges into #120"
resolution: null
duplicate_of: null
---
tests/terminal/test_progress_pty.py:92 (SkipTest) and :80 (SIGKILL without waitpid/close). Fail loudly instead of skipping; reap and close on the timeout path. PR #120 review.
