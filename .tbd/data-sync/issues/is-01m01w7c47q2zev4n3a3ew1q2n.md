---
type: is
id: is-01m01w7c47q2zev4n3a3ew1q2n
title: Model cumulative performance checkpoints explicitly
kind: feature
status: closed
priority: 2
version: 4
labels: []
dependencies: []
created_at: 2026-08-15T04:52:31.491Z
updated_at: 2026-08-15T06:05:23.919Z
closed_at: 2026-08-15T06:05:23.918Z
close_reason: Added explicit kept-arm checkpoints, complete index-core-v1 measurements, truthful absolute-history rendering, exact source provenance, and an interleaved Git revision replay runner; validated with make check.
---
Separate local A/B evidence from the kept-build checkpoint history. Project every artifact's chosen post-decision arm across a fixed multi-dimensional profile, expose missing historical cells instead of inventing them, and connect only comparable platform/workload/tree regimes. Add a reproducible Git-revision replay path so one frozen benchmark matrix can be rerun at each kept source revision on macOS and Linux; make missing revision provenance visible.
