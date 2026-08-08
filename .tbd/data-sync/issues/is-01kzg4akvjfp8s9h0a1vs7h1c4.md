---
type: is
id: is-01kzg4akvjfp8s9h0a1vs7h1c4
title: "Index concurrency: single-writer RwLock, escalate only on measured contention"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:27:46.546Z
updated_at: 2026-08-08T07:27:46.546Z
---
Settled design decision, carried over from the research (Goal Coverage and Deviations): the index uses a single-writer model with parking_lot::RwLock for phase 1. Writes are short (O(depth) delta applies); reads are pre-computed roll-up field lookups, not queries that walk.

Phase-1 task: implement it, then MEASURE read contention under watch churn before considering anything fancier. The delta contract being the only mutation path is what keeps a later escalation to epoch or arc-swap snapshots contained rather than a rewrite.

The cold-path walk needs no locks at all — the atomic-refcount roll-up builds the tree before it is ever shared.

Retrofitting concurrency later would be a rewrite, which is why this is settled now rather than left open.
