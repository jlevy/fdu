---
type: is
id: is-01m1dtqkbyrydtq60902w1sgkr
title: Profile detached, opened, and large-batch mutation with scoped counters
kind: task
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - performance
  - instrumentation
dependencies:
  - type: blocks
    target: is-01m1dtqrgwd4fn6ekn7dq8a6tg
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:33:01.053Z
updated_at: 2026-09-01T06:33:06.331Z
---
Add the default-tree, cold-scan-index, opened-discovery, delta-apply-large, and delta-apply-batched jobs plus provenance and consequence counters. Record exact main, PR #51, and correctness-fixed baselines with semantic oracles before changing the mutation pipeline.
