---
type: is
id: is-01m1ejqfv4khft8mbkfw7f3q0f
title: Coalesce causal scanner fragments in the one-shot builder
kind: task
status: closed
priority: 0
version: 3
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
updated_at: 2026-09-01T13:45:41.455Z
started_at: 2026-09-01T13:32:30.395Z
closed_at: 2026-09-01T13:45:41.441Z
close_reason: H105 reduced baseline applies from about 2,670 to about 124 per scan, but default-tree changed +0.13% (95% CI -1.08% to +2.29%) and cold-scan-index was flat; preparation rose because earlier-parent lookup reverse-scans larger batches. Candidate removed and exp-088 recorded.
resolution: canceled
duplicate_of: null
---
Pre-registered H105. Preserve causal scanner publication for public scan and opened discovery, but concatenate adjacent fragments only inside scan_into_index and its diagnostic twin up to the configured 1,024-op target before invoking the unchanged atomic reducer. The metabrowser subject currently produces about 2,650 baseline applies for 113,793 ops versus a configured-batch minimum near 112. Accept only if default-tree is at least 3% faster with paired CI below zero, cold-scan-index moves in the same direction, exact digest/report parity holds, and baseline applies fall within 10% of the configured minimum; otherwise remove the coalescer.
