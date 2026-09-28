---
type: is
id: is-01m3kect0s63pmr6z6tkvev6kv
title: Name the replacement when Python's CachePolicy is given a retired value
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T07:24:14.744Z
updated_at: 2026-09-28T07:24:14.744Z
---
fdu.CachePolicy("only") raises the enum's own ValueError before the engine can name --stale-ok/stale_ok, so the parity sessions for retired values test the shim's restated messages, not the package. Route retired spellings through the engine's parse_cache_policy (or a _missing_ hook) so Python names the replacement. REG-4 in the stack 141 regression review.
