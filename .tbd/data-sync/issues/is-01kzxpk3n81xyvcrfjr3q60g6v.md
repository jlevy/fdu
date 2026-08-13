---
type: is
id: is-01kzxpk3n81xyvcrfjr3q60g6v
title: Fix overflow retry reconciliation statistics
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - review
  - correctness
dependencies: []
parent_id: is-01kzpvshmzfp0804ywk18v4pzr
created_at: 2026-08-13T13:57:06.856Z
updated_at: 2026-08-13T14:11:40.705Z
closed_at: 2026-08-13T14:11:40.704Z
close_reason: Late overflow now resumes at the unapplied wave without repeating the completed prefix; deterministic serial-oracle regression and complete handoff gate pass.
---
Address PR #8 review: when a later parallel reconciliation wave exceeds MAX_DEFERRED_RECONCILE_OPS, the serial whole-tree retry must not double-count unchanged/apply statistics from already-applied prefix waves. Add a deterministic regression test that forces a late-wave overflow and compare final report statistics and index semantics with an ordinary serial reconcile.

## Notes

Fixed parallel reconciliation overflow fallback to retain completed-wave report data exactly once and resume serial traversal from the first unapplied wave plus remaining frontier. Added a deterministic 1,025-directory late-overflow regression that compares ApplyStats, scan counts, and final index fingerprint with a full serial oracle. Focused overflow tests, clippy/MSRV, docs formatting, and the complete make check handoff gate pass.
