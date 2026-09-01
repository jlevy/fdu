---
type: is
id: is-01m1dtq2kd9dex87vs7mzajejc
title: "Spec: streaming performance parity without one-shot overhead"
kind: epic
status: in_progress
priority: 0
version: 11
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
updated_at: 2026-09-01T18:30:39.955Z
---
Restore detached one-shot performance to the pre-rewrite main control while preserving exact opened-root and public mutation semantics. Correctness fixes precede profiling and lifecycle specialization. The linked plan is the design and acceptance authority.

## Notes

Formal stacked draft PR: https://github.com/jlevy/fdu/pull/52 on codex/streaming-performance-parity, based on PR #51. Correctness fixes, scoped profiling, detached consequence suppression, ancestry-path removal, directory-shaped one-shot construction, controls support, and negative-tested deterministic allocation/zero-work guards are implemented. Controls-rich wall improved 33.55% and component 47.43% versus c6380f7; a valid historical comparison puts cold construction at practical median parity but narrowly misses the strict +3% interval and retains about 17-22% higher RSS. Experiments through exp-099 are recorded. Full/capability-disabled suites, clippy, docs, MSRV, and cross-lint pass; clean stacked-PR CI, quiet-host/RSS verdict, Linux H86 evidence, and final handoff remain open.
