---
type: is
id: is-01m3td4t746a9zyvgps3nvwpn0
title: "PR #170 review R4: the harness's own skips pass"
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
created_at: 2026-10-01T00:17:05.245Z
updated_at: 2026-10-01T00:17:30.125Z
started_at: 2026-10-01T00:17:30.123Z
---
scripts/release/stability_pass.py:825-845 passes whenever run_installed_cli_qa.py exits 0, which it also does with FDU_QA_MEDIUM/FDU_QA_LARGE unset or after an RSS-limit stop (rows verdict skip). harness_note :1552-1567 marks such a phase Passed. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
