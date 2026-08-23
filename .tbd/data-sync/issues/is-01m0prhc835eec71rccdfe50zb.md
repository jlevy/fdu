---
type: is
id: is-01m0prhc835eec71rccdfe50zb
title: Asyncio adapters and thread-affinity docs for watch, with an SSE-resume example
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:32:08.066Z
updated_at: 2026-08-23T17:01:32.711Z
---
Watch stays a thread-affine blocking iterator; the package ships and documents the event-loop handoff (worker thread feeding an asyncio queue, same typed batches). Includes a tested example mapping since(clock)/ChangeSet.truncated to SSE Last-Event-ID resume and resync. Documents thread affinity and that live UIs set interval near their frame budget (51 ms end-to-end measured at interval=0.05).

## Notes

A worker thread draining Watch.__next__ into an asyncio.Queue, shipped in the package rather than left to each consumer to reinvent around the GIL. Watch.__next__ (fdu-py/src/lib.rs:1023) already does py.detach(|| session.next_batch(timeout)) — pull, GIL released, taken once per batch. The SSE-resume example maps since/truncated (Since at index.rs:251; DEFAULT_JOURNAL_OP_CAPACITY at index.rs:55, 64*1024 ops, journal_floor advancing on eviction) to Last-Event-ID and resync. Blocked by fdu-gav9: an event-loop adapter over a surface that raises under concurrent access relocates the defect rather than fixing it.
