---
type: is
id: is-01m3js06hdg5pgnqabfsfsnv6d
title: Enforce shared human size formatting across CLI outputs
kind: task
status: closed
priority: 2
version: 3
labels: []
dependencies: []
created_at: 2026-09-28T01:10:21.472Z
updated_at: 2026-09-28T01:50:05.372Z
closed_at: 2026-09-28T01:50:05.371Z
close_reason: Shared size formatter and color roles applied to cache/progress/CLI/Python; exact structured bytes preserved;111 installed checks passed.
resolution: null
duplicate_of: null
---

## Notes

Audit human byte amounts in cache current/stale/leftover/unrecognized rows and summary/cleanup totals; use shared binary size formatting and ANSI size roles (zero gray, >=1GiB bold). Add core renderer options shared by CLI and Python optional color. Preserve structured exact byte values and parser flag syntax. Document canonical design-system rule and validate plain/color/machine output.
