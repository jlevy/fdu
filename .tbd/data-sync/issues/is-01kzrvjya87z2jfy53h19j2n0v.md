---
type: is
id: is-01kzrvjya87z2jfy53h19j2n0v
title: Traversal order as a scan policy, breadth-first by default
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
created_at: 2026-08-11T16:48:12.103Z
updated_at: 2026-08-11T16:48:12.437Z
closed_at: 2026-08-11T16:48:12.436Z
close_reason: Landed on the fsevents-scoped-revalidation branch; 162 tests pass including the three new order tests. Remaining follow-ups (CLI --order flag, one-shot CLI defaulting to DepthFirst) tracked in the progressive-results plan Phase 1.
---
All four traversal loops (serial walk, parallel worker queue, revalidation sweep, subtree reconcile) were hardcoded to a LIFO stack, making partial results depth-first: one child of the root complete at its final size while siblings read zero, so any consumer ranking by size mid-scan ranked confidently wrong. ScanOrder is now a policy with BreadthFirst as the default, threaded through all four loops via one VecDeque taken from either end. Measured on the 59,654-entry tree over six interleaved trials each: breadth-first costs ~8% on a complete scan (51.0 vs 47.2 ms) and nothing measurable in memory (11 MB either way, since the queue holds directories not entries). Three tests pin the contract: identical engine digests across both orders and several worker counts, non-decreasing directory depth in emission order under breadth-first, and scope equality so order cannot invalidate a cache. DepthFirst stays available for consumers that only read the finished index.
