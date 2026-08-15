---
type: is
id: is-01m01cj5pdxg5a9vqdm3mtwhp9
title: "PR #26 review R7: release-rehearse unchecked python3>=3.11 floor"
kind: task
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m01chg3gqm5sjf58mt5ng9zw
created_at: 2026-08-15T00:18:48.140Z
updated_at: 2026-08-15T00:34:04.799Z
closed_at: 2026-08-15T00:34:04.799Z
close_reason: Fixed in 862190a on claude/fdu-pr-review-g8rsrm; full make check green; disposition map posted at https://github.com/jlevy/fdu/pull/26#issuecomment-5299527089
---
Makefile release-test/release-rehearse use system python3 (tomllib needs >=3.11). Route through uv (already-required tool) and extend UV_BACKED_TARGETS + coverage test.
