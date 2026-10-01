---
type: is
id: is-01m3td4ryxe3ffbdsw730tk1g1
title: "PR #170 review R3: a rerun exits 0 while the recorded pass has failed"
kind: bug
status: in_progress
priority: 2
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:03.956Z
updated_at: 2026-10-01T00:17:29.443Z
started_at: 2026-10-01T00:17:29.442Z
---
scripts/release/stability_pass.py:1728-1737 takes the exit status from this run's outcomes only; cleanup() also runs after a successful partial rerun and removes the worktree a failed gate kept. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
