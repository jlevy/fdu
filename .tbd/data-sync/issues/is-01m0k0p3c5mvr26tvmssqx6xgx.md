---
type: is
id: is-01m0k0p3c5mvr26tvmssqx6xgx
title: Pin --docs and the reshaped help in the golden suite
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0k0nb25qmg8fpvaqybdmpc2
created_at: 2026-08-21T20:37:33.700Z
updated_at: 2026-08-21T20:49:07.999Z
closed_at: 2026-08-21T20:49:07.999Z
close_reason: Landed on claude/fdu-content-axis; make check green (24 suites, 115 goldens).
---
`--docs` is a text contract like every other CLI surface, so it belongs in the golden
suite rather than being checked by eye.

- a golden session for `fdu --docs`
- re-record the help sessions, which lost `before_help` and gained the pointer line
- a unit test that `--docs` needs no PATH and exits 0

Regeneration hazard, now documented in the README: `make golden-update` records literal
output and overwrites the named [SCAN_PATH]/[RFC3339]/[ALLOCATED]/[MTIME_NS] patterns with
one run's values. Restore them before committing.
