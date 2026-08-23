---
type: is
id: is-01kzy1w2vbam0mr1z5we4y6fy0
title: "H70: Tune a shared macOS directory-opener pool"
kind: task
status: deferred
priority: 2
version: 12
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - experiment
  - macos
  - research
  - campaign-2
dependencies: []
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-13T17:14:15.274Z
updated_at: 2026-08-23T09:09:03.925Z
---
Revisit the shared macOS directory-opener pool only after the adaptive-controller outcome is resolved, because opener threads and scan/parser workers consume one total concurrency budget. Treat the prior H69/H70 results as exploratory: screen exactly two, three, and four opener threads under the settled controller or fixed diagnostic control, then confirm any selected count on independent held-out pairs and an independent large topology. Pre-register wall and CPU/system-CPU/RSS/context-switch rejection thresholds before screening.

Acceptance: exact parsing, fallback, path, scope, error, and partial-result semantics remain unchanged; selection and confirmation samples are disjoint; no count is accepted when its resource cost is disproportionate or its confirmation interval misses the declared improvement rule; all prototypes and negative results are recorded and reverted unless confirmed. This P2 optimization does not block fdu-9x4o, fdu-8evu, or release qualification.

## Notes

Reparented from the completed adaptive-worker epic. Revisit the shared opener pool only as a separate P2 optimization under the unchanged shipped controller and one total concurrency budget; prior H69/H70 screens remain exploratory.
