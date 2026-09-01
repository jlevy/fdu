---
type: is
id: is-01m1ekg6ewkj2mr9wf1xs9g01y
title: Differentially profile residual one-shot baseline mutation work
kind: task
status: in_progress
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - profiling
dependencies: []
parent_id: is-01m1dtr903vj783j9ajaxfnczf
hold: null
hold_until: null
created_at: 2026-09-01T13:45:52.859Z
updated_at: 2026-09-01T13:45:55.745Z
started_at: 2026-09-01T13:45:55.744Z
---
After H104 and H105 ruled out prepared-batch construction, control projection, causal publication frequency, and reducer-call count as primary wall costs, compare current and pre-rewrite profiles and instrument corpus-scale per-entry mutation mechanisms. In particular, measure revision-clock and parent children-revision bookkeeping before preregistering any specialization. Do not change semantics until evidence identifies a mechanism capable of explaining at least 3% default-tree wall time.
