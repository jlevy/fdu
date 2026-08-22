---
type: is
id: is-01m0nt4wj7t3tg0pa3f6tf2xrp
title: "PR #40 review R8: Report.render unbound error names only Index.report"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nt3z2n7reqvg9sfy3j2ab1
created_at: 2026-08-22T22:41:01.511Z
updated_at: 2026-08-22T23:14:22.745Z
closed_at: 2026-08-22T23:14:22.745Z
close_reason: "Fixed in b8999e8; addressed the senior review on PR #40 with regression tests for each."
---
crates/fdu-py/python/fdu/_models.py:533. Also raises bare ValueError rather than InvalidArgumentError.
