---
type: is
id: is-01m0prhc835eec71rccdfe50zb
title: Asyncio adapters and thread-affinity docs for watch, with an SSE-resume example
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:32:08.066Z
updated_at: 2026-08-23T07:32:08.066Z
---
Watch stays a thread-affine blocking iterator; the package ships and documents the event-loop handoff (worker thread feeding an asyncio queue, same typed batches). Includes a tested example mapping since(clock)/ChangeSet.truncated to SSE Last-Event-ID resume and resync. Documents thread affinity and that live UIs set interval near their frame budget (51 ms end-to-end measured at interval=0.05).
