---
type: is
id: is-01m3n1xw8pn6ns13f6sv7chv36
title: "H170: per-worker transient summary fold without per-entry owned paths crossing threads"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-09-28-pdu-and-the-linux-peer-gap.md
labels:
  - performance
  - linux
  - experiment
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
created_at: 2026-09-28T22:24:51.478Z
updated_at: 2026-09-28T22:24:51.478Z
---
jemalloc screen: --no-controls summary 1.21 -> 1.01 s on balanced-1m, so ~0.2 s is allocator traffic even with no index. Not H85's 20% bar. Registry row in docs/project/guides/performance-loop.md; mechanism and pre-registration in the pdu brief.
