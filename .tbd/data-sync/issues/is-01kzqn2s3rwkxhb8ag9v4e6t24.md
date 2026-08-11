---
type: is
id: is-01kzqn2s3rwkxhb8ag9v4e6t24
title: "P2: CachePolicy in open() with fail-closed 'only'"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn33mgtc6j3rk6tns1bawg
  - type: blocks
    target: is-01kzqn3c33pyf3vh7070ehnfss
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:35:16.599Z
updated_at: 2026-08-11T16:44:43.045Z
closed_at: 2026-08-11T16:44:43.044Z
close_reason: CachePolicy {Auto,Refresh,ReadOnly,Only,Off} in open(), replacing cache_path+save_on_open. OpenPath gains CacheOnly; cache-only marks the index unverified so freshness reports stale (a test caught it inheriting Fresh from the snapshot), and fails closed with no usable snapshot. Foreign-root/scope snapshots filtered at load. CLI --cache flag, Python parity, help and SKILL.md rewritten around the five axes. 6 library tests for the policy matrix plus 5 golden blocks covering cold->warm->cache-only.
---
Replace the --no-cache boolean with CachePolicy { Auto, Refresh, ReadOnly, Only, Off } consumed by open(). Semantics per the spec table: auto reads the snapshot, revalidates, writes on complete; refresh ignores and rewrites (the benchmark cold control); read-only reads and revalidates but never writes; only reads and never touches the tree, labeling freshness stale; off does a full scan and leaves no trace. Every Report carries source (cold_scan | warm_revalidate | cache_only), freshness, and complete in all formats so no policy can silently lie (Principle 5). --cache only with no usable snapshot is fatal (exit 1) - there is no data to answer with and guessing would violate Principle 5. Tests: one tryscript block per policy against a scratch XDG_CACHE_HOME asserting the source label and whether the snapshot file exists afterward.
