---
type: is
id: is-01kzm5eqahbmtm5gwhf6fmejwh
title: Prove fdu concurrency contracts with deterministic state-machine tests
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - concurrency
  - testing
  - merge-blocker
dependencies:
  - type: blocks
    target: is-01kzm3t12dcq5h7n92xztnhcyd
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T21:04:27.472Z
updated_at: 2026-08-09T21:04:42.552Z
---
Build the final merge-gate evidence for every current shared-state boundary after the atomic apply, guard-free API, lock-free watch arbitration, and bounded watcher transport changes land. Use deterministic barriers, handoff channels, injectable seams, and bounded subprocess or receive deadlines; do not use timing sleeps as synchronization. Required evidence: simultaneous writers linearize into unique contiguous clocks and journal order; readers see complete before-or-after batches and consistent roll-ups, never partial mutation; reconciliation freshness epochs cannot clear a newer invalidation; callbacks and serialization/file I/O execute after locks are released; poisoning/panic and worker-stop paths return typed outcomes; concurrent snapshot readers observe a complete old or new image during competing saves; Index and IndexHandle satisfy the intended Send/Sync contract while Watcher documents and proves its single-consumer ownership; Python native work releases the GIL so unrelated threads progress, while same-PyIndex access has an explicit serialized-or-rejected contract. Run the suite under all-features and watch-only configurations on supported CI platforms. No model-checker dependency is required for the current safe std RwLock design, but any future unsafe or lock-free protocol in fdu-gdrv or fdu-aky1 must add a model-checking proof before adoption.
