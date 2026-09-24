---
type: is
id: is-01m38zdq6nd24sqfedrsq1s2b6
title: "PR #119 review A-1: Ctrl-C contract is silent on the wheel console entry"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m38zd02aamagph2hxgq8548d
hold: null
hold_until: null
created_at: 2026-09-24T05:50:11.668Z
updated_at: 2026-09-24T05:56:04.502Z
started_at: 2026-09-24T05:50:15.971Z
closed_at: 2026-09-24T05:56:04.501Z
close_reason: "Fixed: plan requirement in f636e583 on claude/progress-indicator-review-fixes (the paragraph #120 rewrote); test half (FDU_BIN, runbook) in bb543676 on claude/progress-indicator-plan"
resolution: null
duplicate_of: null
---
Plan lines 186-200 and 330 (docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md). The handler must reset SIGINT to SIG_DFL rather than restore the previous handler, so the die-by-SIGINT guarantee is independent of what the embedding process (the wheel's console script) installed; the pty test should be runnable against the wheel via FDU_BIN. PR #119 review, https://github.com/jlevy/fdu/pull/119#issuecomment-5808464209
