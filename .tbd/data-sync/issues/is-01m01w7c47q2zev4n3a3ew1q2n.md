---
type: is
id: is-01m01w7c47q2zev4n3a3ew1q2n
title: Model cumulative performance checkpoints explicitly
kind: feature
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-15T04:52:31.491Z
updated_at: 2026-08-15T04:52:31.491Z
---
Extend the experiment soft schema with explicit benchmark-series and cumulative-checkpoint metadata. Plot a connected progress line only for checkpoints that share a platform, workload, tree regime, baseline lineage, and primary metric. The current ledger has a sparse macOS cold-scan checkpoint sequence but no comparable Linux sequence, so infer neither from experiment order nor acceptance status.
