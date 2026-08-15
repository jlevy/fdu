---
type: is
id: is-01m01easxwr1504mqmnha3brzh
title: Fail closed on unstable or unobservable automatic-worker policy
kind: bug
status: open
priority: 1
version: 16
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - benchmark
  - fix
dependencies:
  - type: blocks
    target: is-01m01ec396v5crqyg5sfasfehr
  - type: blocks
    target: is-01m01ecbhsetn1rmvfn8m26w7e
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:49:43.867Z
updated_at: 2026-08-15T01:17:13.528Z
---
Extend run, result, experiment, and report contracts with explicit adaptive-policy observability and statistical verdicts. Raw samples remain authoritative; policy claims require a complete versioned trace. Encode superiority separately from a +3% noninferiority/non-regression decision: for paired percent difference Delta where positive is slower, pass only when the confidence-interval upper bound is at most +3%, establish inferiority when the lower bound exceeds +3%, and otherwise report inconclusive. Never reuse the existing superiority boolean for this result.

Acceptance: candidate screening and held-out confirmation are distinguished in artifacts; the best fixed arm is selected on discovery samples and confirmed on independent, paired/interleaved samples with sample count, interval convention, and stopping rule fixed before measurement; instability is judged from trace-defined harmful policy histories and a pre-registered frequency/impact rule rather than an unspecified bimodality test at n=12; resource acceptance uses pre-registered CPU, system CPU, RSS, fault, and context-switch thresholds or Pareto rules; missing measurements are null plus a reason and invalidate dependent claims; tests cover pass, inferior, inconclusive, unstable, missing-field, and resource-rejection cases.
