---
type: is
id: is-01m0nt4sfe3b139bvcvdf939n6
title: "PR #40 review R1: Index.report render() re-projects the live index while as_dict() is a snapshot"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nt3z2n7reqvg9sfy3j2ab1
created_at: 2026-08-22T22:40:58.349Z
updated_at: 2026-08-22T23:14:22.724Z
closed_at: 2026-08-22T23:14:22.724Z
close_reason: "Fixed in b8999e8; addressed the senior review on PR #40 with regression tests for each."
---
crates/fdu-py/python/fdu/_api.py:231. After idx.refresh(), one frozen Report returns 4 files from as_dict() and 5 from render(). fdu.report() and Watch.report() bind to an owned OneShot and are stable; Index.report is the only producer that is not.
