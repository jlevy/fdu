---
type: is
id: is-01m3n2834pcngmyn88t5a4bg46
title: Several roots in one report, under a combined total
kind: feature
status: closed
priority: 2
version: 2
labels:
  - parity
dependencies: []
parent_id: is-01m3n282ey52kzkf7ctv7fyeq3
created_at: 2026-09-28T22:30:26.197Z
updated_at: 2026-10-09T17:01:52.749Z
closed_at: 2026-10-09T17:01:52.748Z
close_reason: "Answered by the 0.5.0 several-roots work on claude/several-roots: fdu PATH... reports disjoint roots under a (total) row; overlap refused by path and (dev,ino) identity; one snapshot per root under one cache directory (fc407d6e, 625c7966, bb0dc086, b6b2d1b5, bf4fa8db)"
resolution: null
duplicate_of: null
---
pdu accepts several paths and renders them under a synthetic (total) root; fdu takes one PATH. Design the engine request (disjoint roots, overlap handling, cache scope per root) before the CLI flag.
