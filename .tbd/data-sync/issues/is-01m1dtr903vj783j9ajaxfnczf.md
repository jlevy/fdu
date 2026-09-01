---
type: is
id: is-01m1dtr903vj783j9ajaxfnczf
title: Prove one-shot parity and add deterministic regression guards
kind: task
status: in_progress
priority: 0
version: 13
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - validation
dependencies: []
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
child_order_hints:
  - is-01m1edc4xady6k86e0hsbzfsk1
  - is-01m1eek06tcb89yygyc1xz2yz5
  - is-01m1egf3aa4wt4kc2z5qmhspqp
  - is-01m1egxbrdj757jr3bk8bhv1ce
  - is-01m1ejqfv4khft8mbkfw7f3q0f
  - is-01m1ekg6ewkj2mr9wf1xs9g01y
hold: null
hold_until: null
created_at: 2026-09-01T06:33:23.201Z
updated_at: 2026-09-01T18:30:39.681Z
started_at: 2026-09-01T11:13:09.191Z
---
Re-profile after every accepted experiment, close only profile-named residual costs, meet the plan wall/component/allocation thresholds on control-free and control-rich real trees, add negative-tested per-entry allocation and detached zero-work guards, run the full and cross-platform gates, and record every experiment.

## Notes

2026-09-01: correctness and deterministic regression guards are complete. The detached one-shot route matches the scanner oracle for worker counts 1-4, controls, control limits, non-file controls, ignored state, reports, and the first public mutation; it fixes baseline acceptance of the documented fixed-path ControlRemove. Controls-rich wall is -33.55% and component -47.43% versus c6380f7, with allocations 6.02M to 0.99M and exact digest stability. Historical cold construction is at practical median parity: wall +0.93% (95% interval -5.63% to +3.83%) and component -0.39% (interval -3.02% to +4.04%). Exp-098 rejects dynamic shared orchestration; exp-099 accepts monomorphized sharing at wall +0.16% (interval -1.46% to +0.81%). Full/capability-disabled suites, allocation/zero-work guards, clippy, docs, MSRV, and cross-lint pass. Quiet-host strict +3% proof, elevated historical RSS, Linux H86 evidence, clean PR CI, and the final handoff remain open.
