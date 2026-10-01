---
type: is
id: is-01m3td57swz4js4hrwydt63b1g
title: "PR #170 review suggestions (non-blocking): own rebinding, step timeout, H1 case, real gates run, permission-bits on root"
kind: task
status: closed
priority: 3
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:19.163Z
updated_at: 2026-10-01T03:26:34.541Z
started_at: 2026-10-01T00:17:38.719Z
closed_at: 2026-10-01T03:26:34.540Z
close_reason: "Four suggestions applied (a2e5d541, df829ec8, 9e53c996); the real run was done in part on macOS and found the prerequisite bug fixed in fdf46d71. make check, cross-lint and semver-check have still not run through the driver. PR #170, disposition map https://github.com/jlevy/fdu/pull/170#issuecomment-5924110534"
resolution: null
duplicate_of: null
---
Five non-blocking suggestions: scripts/qa_peer_agreement.py:722/742 rebinding of own; Host.execute has no timeout; render_tables H1 not in Title Case; the gates and wheel build have run only against the fake host; prerequisites() could name FDU_TEST_ALLOW_NO_PERMISSION_BITS on a root host. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
