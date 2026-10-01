---
type: is
id: is-01m3td4ryxe3ffbdsw730tk1g1
title: "PR #170 review R3: a rerun exits 0 while the recorded pass has failed"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:03.956Z
updated_at: 2026-10-01T03:26:30.985Z
started_at: 2026-10-01T00:17:29.442Z
closed_at: 2026-10-01T03:26:30.984Z
close_reason: "Fixed in df829ec8 and 9e53c996: exit status is the whole record's; a step's verdict is dropped before it reruns. PR #170, disposition map https://github.com/jlevy/fdu/pull/170#issuecomment-5924110534"
resolution: null
duplicate_of: null
---
scripts/release/stability_pass.py:1728-1737 takes the exit status from this run's outcomes only; cleanup() also runs after a successful partial rerun and removes the worktree a failed gate kept. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
