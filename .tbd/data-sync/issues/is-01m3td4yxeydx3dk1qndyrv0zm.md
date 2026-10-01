---
type: is
id: is-01m3td4yxeydx3dk1qndyrv0zm
title: "PR #170 review R7: a usage error in prerequisites() escapes as a traceback"
kind: bug
status: in_progress
priority: 3
version: 2
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:10.060Z
updated_at: 2026-10-01T00:17:33.044Z
started_at: 2026-10-01T00:17:33.042Z
---
scripts/release/stability_pass.py:1708 calls prerequisites() outside the try at :1691-1701, and uv_floor() raises UsageError (:475). Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
