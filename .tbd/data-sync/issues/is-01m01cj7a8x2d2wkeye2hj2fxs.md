---
type: is
id: is-01m01cj7a8x2d2wkeye2hj2fxs
title: "PR #26 review S1: drop construction-time deepcopy in report_from_dict"
kind: task
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m01chg3gqm5sjf58mt5ng9zw
created_at: 2026-08-15T00:18:49.800Z
updated_at: 2026-08-15T00:34:04.811Z
closed_at: 2026-08-15T00:34:04.811Z
close_reason: Fixed in 862190a on claude/fdu-pr-review-g8rsrm; full make check green; disposition map posted at https://github.com/jlevy/fdu/pull/26#issuecomment-5299527089
---
_models.py:662 double deepcopy; take ownership of freshly parsed wire, keep as_dict() copy, document ownership.
