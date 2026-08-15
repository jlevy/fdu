---
type: is
id: is-01m01cj4zvdm9caemb4ez1x0k6
title: "PR #26 review R5: release tooling outside lint/format gates"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m01chg3gqm5sjf58mt5ng9zw
created_at: 2026-08-15T00:18:47.419Z
updated_at: 2026-08-15T00:34:04.795Z
closed_at: 2026-08-15T00:34:04.795Z
close_reason: Fixed in 862190a on claude/fdu-pr-review-g8rsrm; full make check green; disposition map posted at https://github.com/jlevy/fdu/pull/26#issuecomment-5299527089
---
scripts/release, tests/release not covered by ruff; examples not ruff-covered. Extend python-check ruff (check+format) with shared config; fix violations (inspect_artifacts.py:172, rollup_adapter.py:44 over 100 cols).
