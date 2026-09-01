---
type: is
id: is-01m1dtqkbyrydtq60902w1sgkr
title: Profile detached, opened, and large-batch mutation with scoped counters
kind: task
status: in_progress
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - performance
  - instrumentation
dependencies:
  - type: blocks
    target: is-01m1dtqrgwd4fn6ekn7dq8a6tg
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:33:01.053Z
updated_at: 2026-09-01T08:06:51.586Z
---
Add the default-tree, cold-scan-index, opened-discovery, delta-apply-large, and delta-apply-batched jobs plus provenance and consequence counters. Record exact main, PR #51, and correctness-fixed baselines with semantic oracles before changing the mutation pipeline.

## Notes

Correctness prerequisites are complete. Initial uncontrolled interleaved screen on cargo-registry-src (11,142 entries; 4 trials, 1 warmup) compared b75bf85, PR #51 e8f1bed, and correctness head b5d9ba4. PR #51 versus pre-rewrite: cold-scan-index +8.21% median wall (53.1 ms vs 48.0 ms component) and default-tree +7.68% median wall (59.3 ms vs 54.8 ms component). A counters-on scan showed allocations/bytes rising from 141,392/21.2 MB to 289,457/42.8 MB while digest and entry totals stayed exact. Artifact: /tmp/fdu-streaming-parity/results/run-initial-three-way.json (exploratory only). Next: add exact opened/public-batch jobs and lifecycle/consequence counters, then profile caller trees.
