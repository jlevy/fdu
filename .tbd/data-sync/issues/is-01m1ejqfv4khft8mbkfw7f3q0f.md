---
type: is
id: is-01m1ejqfv4khft8mbkfw7f3q0f
title: Coalesce causal scanner fragments in the one-shot builder
kind: task
status: in_progress
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01m1dtr903vj783j9ajaxfnczf
hold: null
hold_until: null
created_at: 2026-09-01T13:32:23.267Z
updated_at: 2026-09-01T13:32:30.396Z
started_at: 2026-09-01T13:32:30.395Z
---
Pre-registered H105. Preserve causal scanner publication for public scan and opened discovery, but concatenate adjacent fragments only inside scan_into_index and its diagnostic twin up to the configured 1,024-op target before invoking the unchanged atomic reducer. The metabrowser subject currently produces about 2,650 baseline applies for 113,793 ops versus a configured-batch minimum near 112. Accept only if default-tree is at least 3% faster with paired CI below zero, cold-scan-index moves in the same direction, exact digest/report parity holds, and baseline applies fall within 10% of the configured minimum; otherwise remove the coalescer.
