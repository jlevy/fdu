---
type: is
id: is-01m3td4t746a9zyvgps3nvwpn0
title: "PR #170 review R4: the harness's own skips pass"
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
created_at: 2026-10-01T00:17:05.245Z
updated_at: 2026-10-01T03:26:31.342Z
started_at: 2026-10-01T00:17:30.123Z
closed_at: 2026-10-01T03:26:31.341Z
close_reason: "Fixed in df829ec8 and 9e53c996: a harness that left a phase out or stopped is skipped; tables block the stopped phases. PR #170, disposition map https://github.com/jlevy/fdu/pull/170#issuecomment-5924110534"
resolution: null
duplicate_of: null
---
scripts/release/stability_pass.py:825-845 passes whenever run_installed_cli_qa.py exits 0, which it also does with FDU_QA_MEDIUM/FDU_QA_LARGE unset or after an RSS-limit stop (rows verdict skip). harness_note :1552-1567 marks such a phase Passed. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
