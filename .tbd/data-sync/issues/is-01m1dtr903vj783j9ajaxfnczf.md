---
type: is
id: is-01m1dtr903vj783j9ajaxfnczf
title: Prove one-shot parity and add deterministic regression guards
kind: task
status: in_progress
priority: 0
version: 18
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - validation
dependencies:
  - type: blocks
    target: is-01m1x444q4jz0680n8a057r5z8
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
updated_at: 2026-09-07T06:36:38.777Z
started_at: 2026-09-01T11:13:09.191Z
---
Re-profile after every accepted experiment, close only profile-named residual costs, meet the plan wall/component/allocation thresholds on control-free and control-rich real trees, add negative-tested per-entry allocation and detached zero-work guards, run the full and cross-platform gates, and record every experiment.

## Notes

Preregistered final cells remain unchanged: fixed 12 pairs, 3 warmups, quiet warm-steady, dense stable-Rust 72,026-entry control-free tree and dense 97,587-entry control-rich source tree; historical b75bf85 (also current main), structural c638 plus matched oracle115ff4f, allocation-only historical2010fa3. A premeasurement version check invalidated the initial candidate release artifact: Cargo reported a fresh build in the shared target, but the unqualified perf_probe output still identified the structural checkout115ff4f. No timing or allocation result has been accepted from these preliminary artifacts. Revise build protocol to a dedicated target per source checkout, reuse download caches only, rebuild every measured binary from a fresh target, and rerun the final local gate in its own validation target. The obsolete task-owned shared target may be staged in Trash after live-writer checks. Candidate code remains1a39be9; no production change is inferred from this build-cache finding.
