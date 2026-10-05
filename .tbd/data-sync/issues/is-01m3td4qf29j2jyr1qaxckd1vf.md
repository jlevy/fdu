---
type: is
id: is-01m3td4qf29j2jyr1qaxckd1vf
title: "PR #170 review R2: make cross-lint can lint nothing and the gate still passes"
kind: bug
status: closed
priority: 1
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:02.428Z
updated_at: 2026-10-01T03:26:30.643Z
started_at: 2026-10-01T00:17:28.242Z
closed_at: 2026-10-01T03:26:30.642Z
close_reason: "Fixed in df829ec8: cross-lint targets are a prerequisite and a skipped target records the gate skipped. PR #170, disposition map https://github.com/jlevy/fdu/pull/170#issuecomment-5924110534"
resolution: null
duplicate_of: null
---
scripts/release/stability_pass.py:741-756 judges only the exit status; Makefile cross-lint prints '== skipping <target>' and exits 0 for each rustup target not installed. body() strips leading '== ' lines. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
