---
type: is
id: is-01m3td4nntegpk3pg4mf29qnvd
title: "PR #170 review R1: a break passes when it catches only some cases"
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
created_at: 2026-10-01T00:17:00.589Z
updated_at: 2026-10-01T00:17:27.513Z
started_at: 2026-10-01T00:17:27.512Z
---
scripts/release/stability_pass.py:1120-1121, :1138-1140 require only status == 1 and caught > 0; the runbook requires every case caught (every warm-cold row NO-SNAPSHOT or NOT-WARM, every same-analyzer cross-warm pair NOT-WARM, every refusal case PARTIAL-STORED). Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
