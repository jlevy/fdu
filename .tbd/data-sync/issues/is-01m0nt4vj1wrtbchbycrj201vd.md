---
type: is
id: is-01m0nt4vj1wrtbchbycrj201vd
title: "PR #40 review R6: watch_rule misreads naive datetimes and loses nanoseconds"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nt3z2n7reqvg9sfy3j2ab1
created_at: 2026-08-22T22:41:00.480Z
updated_at: 2026-08-22T23:14:22.741Z
closed_at: 2026-08-22T23:14:22.741Z
close_reason: "Fixed in b8999e8; addressed the senior review on PR #40 with regression tests for each."
---
crates/fdu-py/python/fdu/_api.py:415. Naive datetimes are read as local time (7h shift under TZ=America/Los_Angeles) and int(at.timestamp()*1e9) drifts 235ns.
