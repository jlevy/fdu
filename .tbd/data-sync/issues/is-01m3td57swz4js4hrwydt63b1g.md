---
type: is
id: is-01m3td57swz4js4hrwydt63b1g
title: "PR #170 review suggestions (non-blocking): own rebinding, step timeout, H1 case, real gates run, permission-bits on root"
kind: task
status: in_progress
priority: 3
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:19.163Z
updated_at: 2026-10-01T00:17:38.723Z
started_at: 2026-10-01T00:17:38.719Z
---
Five non-blocking suggestions: scripts/qa_peer_agreement.py:722/742 rebinding of own; Host.execute has no timeout; render_tables H1 not in Title Case; the gates and wheel build have run only against the fake host; prerequisites() could name FDU_TEST_ALLOW_NO_PERMISSION_BITS on a root host. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
