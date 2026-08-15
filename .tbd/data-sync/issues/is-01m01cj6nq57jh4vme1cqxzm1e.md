---
type: is
id: is-01m01cj6nq57jh4vme1cqxzm1e
title: "PR #26 review R10: dead ruff ignore COM812"
kind: task
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m01chg3gqm5sjf58mt5ng9zw
created_at: 2026-08-15T00:18:49.143Z
updated_at: 2026-08-15T00:34:04.806Z
closed_at: 2026-08-15T00:34:04.806Z
close_reason: Fixed in 862190a on claude/fdu-pr-review-g8rsrm; full make check green; disposition map posted at https://github.com/jlevy/fdu/pull/26#issuecomment-5299527089
---
crates/fdu-py/pyproject.toml:69 ignores COM812 but COM never selected; delete.
