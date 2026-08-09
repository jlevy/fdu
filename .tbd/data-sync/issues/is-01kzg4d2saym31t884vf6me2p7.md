---
type: is
id: is-01kzg4d2saym31t884vf6me2p7
title: io_uring accelerator for openat, close, and statx
kind: feature
status: open
priority: 3
version: 2
spec_path: docs/project/specs/future/plan-2026-08-09-fdu-post-phase-1-roadmap.md
labels:
  - future
dependencies: []
parent_id: is-01kzm3v6nndedpwk414enwysv3
created_at: 2026-08-08T07:29:07.370Z
updated_at: 2026-08-09T20:36:52.385Z
---
Open question from the research: phase-1 complexity or a later accelerator behind a feature flag? It is a large amount of machinery for a Linux-only win, so it is parked here rather than in phase 1.

bfs's pattern is the proven one: IORING_OP_OPENAT, IORING_OP_CLOSE, IORING_OP_STATX, with per-opcode availability probing and a per-thread synchronous fallback, opting into SUBMIT_ALL, SINGLE_ISSUER, DEFER_TASKRUN, and ATTACH_WQ where the kernel supports them.

Not for getdents: bfs leaves it synchronous with a TODO, because kernel support is still landing.

Gate on the benchmark harness showing the syscall layer is the remaining bottleneck. If it is not, this is machinery for nothing.
