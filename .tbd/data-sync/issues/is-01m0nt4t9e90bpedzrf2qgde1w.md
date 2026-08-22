---
type: is
id: is-01m0nt4t9e90bpedzrf2qgde1w
title: "PR #40 review R3: Python Report does not expose notes"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nt3z2n7reqvg9sfy3j2ab1
created_at: 2026-08-22T22:40:59.181Z
updated_at: 2026-08-22T23:14:22.733Z
closed_at: 2026-08-22T23:14:22.733Z
close_reason: "Fixed in b8999e8; addressed the senior review on PR #40 with regression tests for each."
---
crates/fdu-py/python/fdu/_models.py:497. Report.notes exists in the library so every surface can state an omission, but the Python Report has no notes field, so it is reachable only by rendering text.
