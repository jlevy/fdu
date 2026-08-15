---
type: is
id: is-01m01cj3ybmebfv0j5a31ksv2k
title: "PR #26 review R2: naive datetime rejected by modified_since/before"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m01chg3gqm5sjf58mt5ng9zw
created_at: 2026-08-15T00:18:46.347Z
updated_at: 2026-08-15T00:34:04.787Z
closed_at: 2026-08-15T00:34:04.787Z
close_reason: Fixed in 862190a on claude/fdu-pr-review-g8rsrm; full make check green; disposition map posted at https://github.com/jlevy/fdu/pull/26#issuecomment-5299527089
---
crates/fdu-py/python/fdu/_api.py:57 _when() isoformats naive datetimes without offset; engine parse_when rejects offset-less timestamps. Attach local offset via astimezone(); add tests.
