---
type: is
id: is-01m1dtqkbyrydtq60902w1sgkr
title: Profile detached, opened, and large-batch mutation with scoped counters
kind: task
status: closed
priority: 0
version: 6
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - performance
  - instrumentation
dependencies:
  - type: blocks
    target: is-01m1dtqrgwd4fn6ekn7dq8a6tg
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-01T06:33:01.053Z
updated_at: 2026-09-01T08:44:43.641Z
closed_at: 2026-09-01T08:44:43.640Z
close_reason: Five exact jobs, scoped lifecycle counters, fresh comparative baselines, experiment records, and sampled caller-tree attribution are complete.
resolution: null
duplicate_of: null
---
Add the default-tree, cold-scan-index, opened-discovery, delta-apply-large, and delta-apply-batched jobs plus provenance and consequence counters. Record exact main, PR #51, and correctness-fixed baselines with semantic oracles before changing the mutation pipeline.

## Notes

Correctness prerequisites and Phase 1 attribution are complete. Exact jobs now cover default-tree, cold-scan-index, opened-discovery, delta-apply-large (100,001 ops), and delta-apply-batched (4,096-op batches), with independent index and commit-shape oracles. Records exp-074 through exp-076 capture the exploratory main/PR #51/correctness and instrumentation baselines on cargo-registry-src (11,142 entries). Detached cold counters: 11,141 effective paths, 66,903 impact ancestor visits, 15,914 retained dirty paths, 338 AppliedDelta projections, and zero journal entries. A single 100,001-op public commit is cloned then rejected as oversized; the batched control retains 25 commits and drops 9. Sampling artifacts: /tmp/fdu-streaming-parity/results/profile-cold-baseline.json, profile-delta-large-baseline.json, and profile-opened-baseline.json. Cold path iteration/comparison is 7.67% of all samples with allocator at 6.21%; delta-large path layer is 45.47%; opened path layer is 16.09%. Phase 2 should first make baseline application stats-only through a private consequence sink, then measure before replacing ancestry proof.
