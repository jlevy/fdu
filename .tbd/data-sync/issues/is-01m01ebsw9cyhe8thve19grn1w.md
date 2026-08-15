---
type: is
id: is-01m01ebsw9cyhe8thve19grn1w
title: "H86: Compare continuous-window adaptive controller designs"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - research
  - experiment
dependencies:
  - type: blocks
    target: is-01m01cm1sb8xyw9ag3pabb5s3h
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:50:16.584Z
updated_at: 2026-08-15T00:50:46.264Z
---
Research the smallest controller that corrects the known one-shot failure without overfitting Application Support. Compare the shipped first-16K binary threshold against: repeated independent windows that keep observing after a fast decision; staged scale-up from six to available parallelism and only then to the bounded I/O reserve; queue/backlog-gated activation; throughput-gradient or latency-gradient feedback; and reversible parking with hysteresis where a trial increase does not improve useful throughput. Fixed 6, available CPUs, and 16 are diagnostic controls, not assumed solutions.

Pre-register signals, thresholds, overhead, and rejection rules before measurement. Acceptance for recommending a controller: exact output; no completion-order failure in the model; no unexplained bimodal outcome on quiet immutable fixtures; automatic wall within 3% of the best fixed arm across the required topology matrix; no existing regime regresses by 3%; additional CPU/RSS/context-switch cost must buy a qualifying wall improvement. Record and revert every rejected prototype. This bead selects a design but does not ship it.
