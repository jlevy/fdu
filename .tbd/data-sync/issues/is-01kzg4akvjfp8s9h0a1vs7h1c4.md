---
type: is
id: is-01kzg4akvjfp8s9h0a1vs7h1c4
title: "Index concurrency: single-writer RwLock, escalate only on measured contention"
kind: task
status: open
priority: 2
version: 6
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels:
  - concurrency
  - performance
dependencies:
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:27:46.546Z
updated_at: 2026-08-09T21:11:18.165Z
---
Retain the phase-1 single-writer model behind the sealed IndexHandle boundary, initially using the current std::sync::RwLock. Writes are intended to be short O(depth) delta applications; reads consume precomputed roll-up state. Do not introduce parking_lot, arc-swap, epochs, sharding, or another synchronization dependency by assertion. First implement the guard-free API and common performance probe, then measure reader latency, writer wait/progress, throughput, and starvation under representative watch churn, wide-directory reconciliation, and snapshot capture. Report distribution and workload details, not only averages. Correctness remains owned by fdu-gd6n; this bead answers whether contention justifies a redesign. The cold-path walker builds before sharing and needs no index lock. Any change in primitive must preserve Delta-only mutation, typed poison/panic behavior, callback-after-unlock, and the same deterministic concurrency suite, and must pass supply-chain review before adding a dependency.

## Notes

The 2026-08-09 Rust guideline audit found that IndexHandle::read publicly returns a standard-library RwLockReadGuard. Complete fdu-s7wr first: callers must receive bounded operations and plain data, not a guard that exposes the implementation or can be held across I/O. Measure contention only after that supported boundary is in place.
