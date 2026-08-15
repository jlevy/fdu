---
type: is
id: is-01m01cj6zxj49egzx41wsk8ecd
title: "PR #26 review R11: macOS 11.0 floor not pinned in wheel builds"
kind: task
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m01chg3gqm5sjf58mt5ng9zw
created_at: 2026-08-15T00:18:49.469Z
updated_at: 2026-08-15T00:34:04.808Z
closed_at: 2026-08-15T00:34:04.808Z
close_reason: Fixed in 862190a on claude/fdu-pr-review-g8rsrm; full make check green; disposition map posted at https://github.com/jlevy/fdu/pull/26#issuecomment-5299527089
---
release.yml macOS legs lack MACOSX_DEPLOYMENT_TARGET; docs promise 11.0 floor. Pin 11.0 on both macOS matrix legs.
