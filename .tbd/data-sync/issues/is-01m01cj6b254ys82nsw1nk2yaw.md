---
type: is
id: is-01m01cj6b254ys82nsw1nk2yaw
title: "PR #26 review R9: python-check hardcodes one pytest file"
kind: task
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m01chg3gqm5sjf58mt5ng9zw
created_at: 2026-08-15T00:18:48.802Z
updated_at: 2026-08-15T00:34:04.804Z
closed_at: 2026-08-15T00:34:04.804Z
close_reason: Fixed in 862190a on claude/fdu-pr-review-g8rsrm; full make check green; disposition map posted at https://github.com/jlevy/fdu/pull/26#issuecomment-5299527089
---
Makefile:212 runs pytest tests/test_models.py; run bare pytest and let testpaths own discovery.
