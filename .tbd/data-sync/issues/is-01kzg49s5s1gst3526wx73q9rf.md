---
type: is
id: is-01kzg49s5s1gst3526wx73q9rf
title: "Walk layer: work-stealing parallelism and batched distribution"
kind: feature
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:27:19.224Z
updated_at: 2026-08-08T07:27:19.224Z
---
Parallel walk that saturates the syscall path without a thundering herd.

- Push a whole batch of discovered children with a SINGLE CAS onto an intrusive lock-free stack, then wake min(children, blocked_threads) - 1 workers by semaphore (dut).
- Cache-line-align hot atomics; prefer fetch_add over CAS loops; exponential backoff before parking (dut, bfs).
- Batch stat calls in small chunks per work item (dua-core uses 4 per job).
- Cap I/O worker threads around 8 — bfs measured little speedup past that.
- Make traversal order a tunable: DFS for warm-cache locality, BFS fan-out for cold-cache queue depth. dut loses on cold cache precisely because it is depth-first.

Ideas only from dut (GPL): write from the description, do not transliterate.
