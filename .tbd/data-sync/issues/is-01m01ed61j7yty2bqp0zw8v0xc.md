---
type: is
id: is-01m01ed61j7yty2bqp0zw8v0xc
title: Run the Apple Silicon/APFS release-CLI noninferiority matrix against dust
kind: task
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - benchmark
  - validation
dependencies:
  - type: blocks
    target: is-01m01edfz3bd7x2w91bh4qft2m
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:51:01.809Z
updated_at: 2026-08-15T01:17:13.773Z
---
Qualify each supported installed fdu command from fdu-o3s4 against the validated, pinned dust adapter from fdu-b722 on a predeclared Apple Silicon/local-APFS workload matrix. Use semantically equivalent complete tree work, immutable exact-oracle subjects, adjacent paired/interleaved release processes, quiet-host warm-steady cells, separately labeled interactive-load cells, and diagnostic-only partial/error cells. In this bead, fdu --cache off means the reusable fdu snapshot cache is disabled; it does not imply an OS cold-cache state.

Pre-registered decision: define paired percent difference Delta so positive means fdu is slower; pass the +3% noninferiority margin only when the chosen confidence interval’s upper bound is at most +3%, establish inferiority when its lower bound is above +3%, and otherwise report inconclusive. Fix the interval convention, sample count, stopping rule, matrix, and invalidation rules before measurement; do not add repetitions or select fixtures after seeing a threshold result. Every valid sample records CPU, system CPU, RSS, faults, context switches, policy history, exact totals, versions, hashes, tree fingerprint, and host regime; missing fields are null plus a reason. Any supported cell that is inferior or inconclusive blocks a positive release-performance conclusion until resolved or explicitly removed from supported scope with product justification.
