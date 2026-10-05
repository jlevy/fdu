---
type: is
id: is-01m3td5058tna29x8fy26jsky4
title: "PR #170 review R8: the regime and tooling describe only the last run"
kind: bug
status: closed
priority: 3
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:11.332Z
updated_at: 2026-10-01T03:26:32.755Z
started_at: 2026-10-01T00:17:34.120Z
closed_at: 2026-10-01T03:26:32.754Z
close_reason: "Fixed in df829ec8: tooling revision and user recorded per step. PR #170, disposition map https://github.com/jlevy/fdu/pull/170#issuecomment-5924110534"
resolution: null
duplicate_of: null
---
scripts/release/stability_pass.py:1658-1686 remember() rewrites state['regime'] on every run, --only report included; tooling HEAD and user follow whoever ran last, and the tooling checkout's dirty state is never recorded. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
