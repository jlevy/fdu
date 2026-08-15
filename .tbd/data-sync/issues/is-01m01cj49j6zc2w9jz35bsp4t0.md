---
type: is
id: is-01m01cj49j6zc2w9jz35bsp4t0
title: "PR #26 review R3: Selection/Query accept bare str for tuple fields"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m01chg3gqm5sjf58mt5ng9zw
created_at: 2026-08-15T00:18:46.706Z
updated_at: 2026-08-15T00:34:04.790Z
closed_at: 2026-08-15T00:34:04.790Z
close_reason: Fixed in 862190a on claude/fdu-pr-review-g8rsrm; full make check green; disposition map posted at https://github.com/jlevy/fdu/pull/26#issuecomment-5299527089
---
_models.py Selection/Query __post_init__: include='*.rs' iterates chars ('*' matches everything, silent wrong results); views=View.TREE fails with opaque AttributeError. Reject str with clear TypeError.
