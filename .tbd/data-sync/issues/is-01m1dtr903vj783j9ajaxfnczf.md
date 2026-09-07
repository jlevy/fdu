---
type: is
id: is-01m1dtr903vj783j9ajaxfnczf
title: Prove one-shot parity and add deterministic regression guards
kind: task
status: in_progress
priority: 0
version: 19
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
updated_at: 2026-09-07T06:51:57.084Z
started_at: 2026-09-01T11:13:09.191Z
---
Re-profile after every accepted experiment, close only profile-named residual costs, meet the plan wall/component/allocation thresholds on control-free and control-rich real trees, add negative-tested per-entry allocation and detached zero-work guards, run the full and cross-platform gates, and record every experiment.

## Notes

Fresh-target final gate and cross-lint passed; 1a39be9 CI all 19 passed. All four release artifacts now verify against their own clean revisions with claim-grade provenance (manifest e191bbe062bf90a3c9f4819f8f7cdf8a0af85dd3d2737988a3010f121976ed61). Five profiles completed with the actual controls-enabled candidate. Detached cold/default profiles show zero effects, impacts, journal clones and ancestry overlays, with bulk metadata reads/open dominating raw symbols. Opened records no detached-builder work. Public large/batched mutation still has material path-comparison self time; summarize_commits diagnostic work also appears outside the component interval, so attribution must distinguish harness work. First historical-Rust timing attempt was refused before sampling by the unchanged quiet-host preflight (CPU busy34.8% >25%). No trial exists from that attempt; wait for host quiet and retry the preregistered cell without changing N or thresholds. Profile and refusal artifacts are preserved in the final run directory.
