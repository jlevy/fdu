---
type: is
id: is-01m0prgz8hvk0rsm1edqgwty5d
title: "Watch: per-batch dirty roll-up set"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:54.768Z
updated_at: 2026-08-23T17:01:30.551Z
---
Each watch batch names the set of paths whose roll-ups changed since the last batch, computed where merge_upward already knows it. This is the client's projection-invalidate signal. Asserted against independently computed ancestor sets.

## Notes

merge_upward (index.rs:1499) already walks exactly the ancestors whose rollups changed — the dirty set is knowledge the engine has and discards. Batch (watch_session.rs:61) carries it; PyWatch::__next__ (fdu-py/src/lib.rs:1023) surfaces it. Tested against an independently computed ancestor set, and goldened through a --watch driver as tests/golden/bin/watch-capture.mjs already does.
