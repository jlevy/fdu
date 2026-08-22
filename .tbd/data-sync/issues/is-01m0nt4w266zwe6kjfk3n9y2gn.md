---
type: is
id: is-01m0nt4w266zwe6kjfk3n9y2gn
title: "PR #40 review R7: render_cache_status signature promises value semantics it does not have"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nt3z2n7reqvg9sfy3j2ab1
created_at: 2026-08-22T22:41:00.998Z
updated_at: 2026-08-22T23:14:22.743Z
closed_at: 2026-08-22T23:14:22.743Z
close_reason: "Fixed in b8999e8; addressed the senior review on PR #40 with regression tests for each."
---
crates/fdu-py/python/fdu/_api.py:428. Takes Sequence[CacheStatus] but uses only .path and re-reads from disk.
