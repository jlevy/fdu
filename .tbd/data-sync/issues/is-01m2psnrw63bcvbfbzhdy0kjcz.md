---
type: is
id: is-01m2psnrw63bcvbfbzhdy0kjcz
title: "PR #78 review D5: Bounded errors need a stated order or cold and warm differ above 64 failures"
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2psnq245vvkwkfs8zt3nfk0
created_at: 2026-09-17T04:23:24.293Z
updated_at: 2026-09-17T04:33:04.615Z
closed_at: 2026-09-17T04:33:04.615Z
close_reason: "Fixed in PR #78 commits 3fb805b7 and e52383d4; disposition posted on the PR"
resolution: null
duplicate_of: null
---
PR #78 delta review (https://github.com/jlevy/fdu/pull/78#issuecomment-5708468480), finding D5.

State: errors are the 64 smallest by path, errors_omitted counts the rest; retain order-independent; add a >64 failures test.
