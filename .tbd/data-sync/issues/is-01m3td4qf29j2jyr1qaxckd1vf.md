---
type: is
id: is-01m3td4qf29j2jyr1qaxckd1vf
title: "PR #170 review R2: make cross-lint can lint nothing and the gate still passes"
kind: bug
status: in_progress
priority: 1
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:02.428Z
updated_at: 2026-10-01T00:17:28.243Z
started_at: 2026-10-01T00:17:28.242Z
---
scripts/release/stability_pass.py:741-756 judges only the exit status; Makefile cross-lint prints '== skipping <target>' and exits 0 for each rustup target not installed. body() strips leading '== ' lines. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
