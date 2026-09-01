---
type: is
id: is-01m1dtr903vj783j9ajaxfnczf
title: Prove one-shot parity and add deterministic regression guards
kind: task
status: in_progress
priority: 0
version: 12
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
updated_at: 2026-09-01T16:55:04.245Z
started_at: 2026-09-01T11:13:09.191Z
---
Re-profile after every accepted experiment, close only profile-named residual costs, meet the plan wall/component/allocation thresholds on control-free and control-rich real trees, add negative-tested per-entry allocation and detached zero-work guards, run the full and cross-platform gates, and record every experiment.

## Notes

2026-09-01 the controls-aware detached checkpoint passes its preregistered exploratory screen: controls-rich wall -33.55% (95% interval -36.41% to -33.14%), component -47.43%, allocations 6.02M to 0.99M, allocated bytes 491 MB to 133 MB, peak RSS -25.88%, exact digest stable. It also fixes specialized baseline handling of the documented ControlRemove for a non-file .gitignore. Full and no-default scanner suites pass. Against the preserved pre-rewrite binary, cold-scan-index construction is at component parity (331.2 ms candidate vs 332.0 ms historical), but default-tree remains about 8% slower by a noisy paired median and peak RSS about 19% higher. The next decision is profile-driven retained layout, snapshot/report handoff, and destruction; full H86 compact storage, deterministic guards, quiet-host evidence, and handoff gates remain open.
