---
type: is
id: is-01m00v04rrc8hwykzq4cry2fsx
title: "PR #22 review R3: preserve counters from every worker"
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m00tzk6myk9ba0110gv86kdz
created_at: 2026-08-14T19:11:51.575Z
updated_at: 2026-08-14T19:24:47.673Z
closed_at: 2026-08-14T19:24:47.672Z
close_reason: Fixed with a guarded TLS drop fallback and deterministic worker flush guards across all production fdu thread entry points. Automatic-exit, content-worker, and parallel-reconciliation regression tests pass; native macOS content probe now reports 1 open, 2 reads, and 26,484 bytes instead of zero.
---
High. PR #22 review R3. crates/perfkit/src/macros.rs:35-126, crates/fdu/src/content/content_analysis.rs:94-107, crates/fdu/src/scan.rs:1834-1849. Worker-local counts are lost without a manual flush. Make folding lifecycle-safe and test content and reconciliation workers.

## Notes

Implemented TLS drop fallback plus deterministic ThreadFlushGuard in scan, reconcile, content, snapshot-save, content-cache, and watch workers. Content and parallel reconcile regression tests pass on macOS.
