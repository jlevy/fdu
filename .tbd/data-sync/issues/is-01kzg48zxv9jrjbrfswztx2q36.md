---
type: is
id: is-01kzg48zxv9jrjbrfswztx2q36
title: "Spike: metric-vector atomic-refcount roll-up"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg49s5s1gst3526wx73q9rf
  - type: blocks
    target: is-01kzg49sswr78gpjykxctbe6c7
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:26:53.371Z
updated_at: 2026-08-08T07:27:19.867Z
---
Prototype dut's lock-free bottom-up aggregation generalized from two u64s to a reducer vector: each node carries an atomic unsearched_children counter, and whichever thread decrements a parent from 1 to 0 continues upward merging the full metric vector. Roll-ups complete bottom-up with no barriers, joins, or locks.

The question is whether the barrier-free property survives generalization, or whether merging a vector (including per-extension maps) reintroduces contention that two scalar fetch_adds did not have.

Design must be written from the description in the research doc: dut is GPL, ideas only.
