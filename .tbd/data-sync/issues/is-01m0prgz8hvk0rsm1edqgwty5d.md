---
type: is
id: is-01m0prgz8hvk0rsm1edqgwty5d
title: "Watch: per-batch dirty roll-up set"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prhqd27m471dn47yt973k0
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:31:54.768Z
updated_at: 2026-08-23T07:33:04.319Z
---
Each watch batch names the set of paths whose roll-ups changed since the last batch, computed where merge_upward already knows it. This is the client's projection-invalidate signal. Asserted against independently computed ancestor sets.
