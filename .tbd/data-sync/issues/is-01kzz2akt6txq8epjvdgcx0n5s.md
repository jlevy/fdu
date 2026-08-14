---
type: is
id: is-01kzz2akt6txq8epjvdgcx0n5s
title: blocked_ns is reported as zero for exactly the parallel jobs it should measure
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:25.829Z
updated_at: 2026-08-14T02:41:25.829Z
---
benchmarks/realtree/measure.py computes metrics['blocked_ns'] = max(0, wall_ns - user_cpu_ns - system_cpu_ns) for every job.

getrusage CPU is summed across threads, so for any job that runs the parallel producer the subtraction is wall minus aggregate thread CPU, which is routinely negative and clamps to zero. The ledger prints that column as 'blocked (I/O+sched)' and experiment.py promotes it into every recorded artifact, so parallel jobs publish a confident zero off-CPU time where the real answer is unknown.

This matters more on main than it did when PR #4 wrote the fix, because main's scan is parallel by default with adaptive worker scale-up. PR #4 added a per-job process_cpu_can_exceed_wall flag and reports None instead of a fabricated zero. Port that.
