---
type: is
id: is-01kzg48zktc7ager8tcy3cst7r
title: "Spike: snapshot format candidates, open latency vs first-listing latency"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels:
  - phase1-foundation
dependencies:
  - type: blocks
    target: is-01kzg4ajxc0pvgcmj834gahcgt
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:26:53.050Z
updated_at: 2026-08-09T20:36:45.687Z
---
Time flat-read-everything (fsearch model) against block-compressed-with-tail-index (ncdu 2 model) on a 500k-entry snapshot.

Measure two things separately, because they rank the candidates differently: time to open (before any directory can be listed) and time to first directory listing. Open latency is the one that matters for the product — a monolithic decode can win on raw throughput and still lose, because a UI opens now and expands later. Also measure steady-state listing latency with an 8-slot LRU block cache.
