---
type: is
id: is-01m1dtq2kd9dex87vs7mzajejc
title: "Spec: streaming performance parity without one-shot overhead"
kind: epic
status: in_progress
priority: 0
version: 10
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - performance
  - correctness
dependencies: []
child_order_hints:
  - is-01m1dtqb9q9fnaqpwr5cw90j0m
  - is-01m1dtqbkxhdqfhczrbdctcaxq
  - is-01m1dtqkbyrydtq60902w1sgkr
  - is-01m1dtqrgwd4fn6ekn7dq8a6tg
  - is-01m1dtqxh815zb3zz6m3g11cx6
  - is-01m1dtr3hap1kqbkfcap66paq8
  - is-01m1dtr903vj783j9ajaxfnczf
created_at: 2026-09-01T06:32:43.884Z
updated_at: 2026-09-01T07:04:43.621Z
---
Restore detached one-shot performance to the pre-rewrite main control while preserving exact opened-root and public mutation semantics. Correctness fixes precede profiling and lifecycle specialization. The linked plan is the design and acceptance authority.

## Notes

Formal stacked draft PR: https://github.com/jlevy/fdu/pull/52, branch codex/streaming-performance-parity, base claude/one-shot-commit-cost (PR #51), planning commit 62596eb. Local make check and make cross-lint passed; all 19 GitHub checks passed on 2026-09-01. Implementation remains open and starts with correctness beads fdu-vev7 and fdu-lksd before profiling or optimization.
