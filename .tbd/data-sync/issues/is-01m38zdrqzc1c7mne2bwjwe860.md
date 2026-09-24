---
type: is
id: is-01m38zdrqzc1c7mne2bwjwe860
title: "PR #120 review R-1: no test for a drawing run whose stdout closes early (plan's Pipes item)"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m38zd0fgqycxzrgtyps04zsx
hold: null
hold_until: null
created_at: 2026-09-24T05:50:13.247Z
updated_at: 2026-09-24T05:56:04.835Z
started_at: 2026-09-24T05:50:16.002Z
closed_at: 2026-09-24T05:56:04.835Z
close_reason: "Fixed in f636e583 on claude/progress-indicator-review-fixes; coordinator merges into #120"
resolution: null
duplicate_of: null
---
crates/fdu/src/cli.rs tests near run_outcomes_and_broken_pipes_have_stable_exit_codes (:2989). Add a run_with_io test with interactive facts, drawing io, and a stdout writer returning BrokenPipe after a frame is drawn; assert status 0 and stderr ending with ERASE_LINE. PR #120 review, https://github.com/jlevy/fdu/pull/120#issuecomment-5808464400
