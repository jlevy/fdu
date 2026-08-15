---
type: is
id: is-01m01easxwr1504mqmnha3brzh
title: Fail closed on unstable or unobservable automatic-worker policy
kind: bug
status: open
priority: 1
version: 9
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - benchmark
  - fix
dependencies:
  - type: blocks
    target: is-01m01eb8bdvte030yrhmng830e
  - type: blocks
    target: is-01m01ebsw9cyhe8thve19grn1w
  - type: blocks
    target: is-01m01ec396v5crqyg5sfasfehr
  - type: blocks
    target: is-01m01ecbhsetn1rmvfn8m26w7e
  - type: blocks
    target: is-01m01ed61j7yty2bqp0zw8v0xc
  - type: blocks
    target: is-01m01edfz3bd7x2w91bh4qft2m
  - type: blocks
    target: is-01kzy1w2vbam0mr1z5we4y6fy0
  - type: blocks
    target: is-01m01eg0efe53jc3smgaza7wk7
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:49:43.867Z
updated_at: 2026-08-15T00:52:34.382Z
---
The current accept rule can approve a fast median while hiding that automatic mode made different one-shot decisions and produced two latency populations. Extend the run/result/experiment contracts and report generator with pre-registered worker-policy gates. A policy run must identify its decision history, group outcomes by decision, detect unexplained bimodality on a quiet immutable subject, and compare automatic wall/resource cost against fixed 6, available-parallelism, and bounded-reserve controls.

Acceptance: raw samples remain the source of truth; missing policy fields invalidate policy claims; equivalent variants are paired/interleaved with at least the standard 12 trials and additional repetitions for threshold-adjacent regimes; auto must remain within 3% of the best fixed arm unless a documented resource trade changes the pre-registered verdict; no existing regime may regress by 3%; CPU, system CPU, RSS, faults, and context switches are always reported; tests prove the gate rejects a constructed bimodal run and accepts a stable one.
