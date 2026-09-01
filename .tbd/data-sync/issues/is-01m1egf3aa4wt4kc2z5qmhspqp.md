---
type: is
id: is-01m1egf3aa4wt4kc2z5qmhspqp
title: Attribute scanner preparation and baseline reduction time
kind: task
status: in_progress
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - instrumentation
dependencies: []
parent_id: is-01m1dtr903vj783j9ajaxfnczf
hold: null
hold_until: null
created_at: 2026-09-01T12:52:51.146Z
updated_at: 2026-09-01T12:52:56.379Z
started_at: 2026-09-01T12:52:56.378Z
---
The fresh H100 sampling profile places only about 1% of samples in fdu::index, while producer component time is at parity with b75bf85 and end-to-end default-tree remains slower. Add off-by-default, per-batch elapsed counters around trusted scanner preparation and detached reduction so FDU_COUNTERS=1 can separate preparation, control projection, reduction, and baseline reset without timing the ordinary path. Use the result to preregister or reject a true one-shot lane before implementation.
