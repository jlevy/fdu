---
type: is
id: is-01kzg48zxv9jrjbrfswztx2q36
title: "Spike: metric-vector atomic-refcount roll-up"
kind: task
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels:
  - concurrency
dependencies:
  - type: blocks
    target: is-01kzg49s5s1gst3526wx73q9rf
  - type: blocks
    target: is-01kzg49sswr78gpjykxctbe6c7
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:26:53.371Z
updated_at: 2026-08-09T21:11:17.819Z
---
Prototype the described atomic-refcount bottom-up aggregation generalized from two u64 values to a reducer vector. Before code, specify node ownership, the exact state machine, every atomic memory ordering, and the happens-before edge that makes child metrics visible to the one thread that decrements a parent from 1 to 0. Prove exactly-once parent completion, no underflow, no use-after-free, and deterministic cancellation/error behavior when one worker fails. Compare scalar, fixed-vector, and per-extension-map contention rather than assuming the barrier-free property survives generalization. Use a small model-checking harness such as Loom for the concurrency protocol if it remains lock-free, plus deterministic stress and sequential-oracle tests; any new dependency must clear the supply-chain policy. Worker ownership is structured: every worker is joined and no panic is silently discarded. If a simpler scoped reduction is competitive or substantially easier to prove, prefer it. The design must be a clean implementation from the research description because dut is GPL; do not transliterate source.
