---
type: is
id: is-01m01cj4mn4jc7rhcy4ezh4b33
title: "PR #26 review R4: report_from_dict validates wire via assert"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m01chg3gqm5sjf58mt5ng9zw
created_at: 2026-08-15T00:18:47.061Z
updated_at: 2026-08-15T00:34:04.792Z
closed_at: 2026-08-15T00:34:04.792Z
close_reason: Fixed in 862190a on claude/fdu-pr-review-g8rsrm; full make check green; disposition map posted at https://github.com/jlevy/fdu/pull/26#issuecomment-5299527089
---
_models.py:585+ asserts vanish under python -O; replace load-bearing asserts with explicit isinstance raises consistent with existing TypeError paths.
