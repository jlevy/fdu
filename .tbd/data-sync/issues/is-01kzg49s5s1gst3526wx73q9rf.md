---
type: is
id: is-01kzg49s5s1gst3526wx73q9rf
title: "Walk layer: work-stealing parallelism and batched distribution"
kind: feature
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels:
  - concurrency
dependencies:
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:27:19.224Z
updated_at: 2026-08-09T21:12:27.275Z
---
Implement measured parallel walking only after the syscall walker and atomic-rollup spike settle the worker and aggregation contracts. Start with scoped threads and a safe bounded queue; the existing unsafe_code deny remains in force. A custom intrusive or unsafe lock-free stack is not the default design and may be proposed only if the safe implementation misses a measured target, in a separate reviewed unsafe boundary with written invariants, Miri where applicable, and model-checking evidence. The scheduler must bound global and per-worker queued paths/batches, apply backpressure without blocking while holding filesystem or index resources, and turn worker panic, I/O failure, cancellation, or consumer abandonment into an explicit joined outcome. No detached worker may outlive the scan; cancellation must drain or safely discard owned work with no partial result reported as fresh. Batch discovered children with bounded allocation, wake only useful workers, cap the default I/O pool around the measured range, and keep traversal order tunable only if evidence supports it. Cache-line alignment, CAS batching, backoff, DFS/BFS, and thread count are hypotheses to benchmark, not fixed claims. Add a small model-checking proof for any custom atomic queue/counter protocol, deterministic injected failure/shutdown tests with bounded deadlines, sequential-oracle equivalence, and thread-scaling measurements. Ideas from dut are description-only because it is GPL; do not transliterate its source.
