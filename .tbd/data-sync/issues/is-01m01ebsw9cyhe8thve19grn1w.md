---
type: is
id: is-01m01ebsw9cyhe8thve19grn1w
title: "H86: Compare continuous-window adaptive controller designs"
kind: task
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - research
  - experiment
dependencies:
  - type: blocks
    target: is-01m01cm1sb8xyw9ag3pabb5s3h
  - type: blocks
    target: is-01m01eg0efe53jc3smgaza7wk7
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:50:16.584Z
updated_at: 2026-08-15T01:17:11.770Z
---
Research the smallest controller that corrects the confirmed one-shot failure without overfitting the Application Support shape. Against the shipped first-16K control, screen repeated independent windows, staged scale-up, ready-work/backlog gating, useful-throughput gradients, and reversible parking with hysteresis; fixed 6, available parallelism, and 16 remain diagnostic controls rather than presumed solutions. Pre-register signals, constants, overhead budget, sample count, stopping rules, harmful policy histories, resource/Pareto thresholds, and rejection rules before running each screen.

Acceptance: exploratory samples may eliminate candidates but may not confirm the winner; any surviving controller is evaluated on independent paired/interleaved confirmation samples across the required topology and quiet/interactive Apple Silicon/APFS matrix. A recommendation requires exact output, completion-order-model invariants, stable trace histories, a passed +3% noninferiority/non-regression decision against the discovery-selected fixed control, and accepted CPU/RSS/context-switch tradeoffs. “No acceptable winner; retain the current policy” is a valid outcome and must be recorded without tuning until something passes. This bead selects or rejects a design and ships no production code.
