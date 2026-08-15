---
type: is
id: is-01kzy1w2vbam0mr1z5we4y6fy0
title: "H70: Tune a shared macOS directory-opener pool"
kind: task
status: open
priority: 2
version: 10
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
  - macos
  - research
dependencies: []
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-13T17:14:15.274Z
updated_at: 2026-08-15T01:17:11.246Z
---
Revisit the shared macOS directory-opener pool only after the adaptive-controller outcome is resolved, because opener threads and scan/parser workers consume one total concurrency budget. Treat the prior H69/H70 results as exploratory: screen exactly two, three, and four opener threads under the settled controller or fixed diagnostic control, then confirm any selected count on independent held-out pairs and an independent large topology. Pre-register wall and CPU/system-CPU/RSS/context-switch rejection thresholds before screening.

Acceptance: exact parsing, fallback, path, scope, error, and partial-result semantics remain unchanged; selection and confirmation samples are disjoint; no count is accepted when its resource cost is disproportionate or its confirmation interval misses the declared improvement rule; all prototypes and negative results are recorded and reverted unless confirmed. This P2 optimization does not block fdu-9x4o, fdu-8evu, or release qualification.

## Notes

The earlier two-opener five-pair screen suggested a 3.98% wall improvement but also materially higher involuntary context switches, and later interactive sweeps were too noisy for selection. Resume only after the adaptive-controller decision, with one explicit total concurrency budget, quiet held-out confirmation, and an independent topology; retain no production change from exploratory PR #8.
