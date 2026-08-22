---
type: is
id: is-01m0nt4v3a1fer10a9yax4s8zb
title: "PR #40 review R5: Change.render bypasses _call and escapes the error hierarchy"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nt3z2n7reqvg9sfy3j2ab1
created_at: 2026-08-22T22:41:00.009Z
updated_at: 2026-08-22T23:14:22.739Z
closed_at: 2026-08-22T23:14:22.738Z
close_reason: "Fixed in b8999e8; addressed the senior review on PR #40 with regression tests for each."
---
crates/fdu-py/python/fdu/_models.py:568. Raises bare ValueError where every other path raises InvalidArgumentError.
