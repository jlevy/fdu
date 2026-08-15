---
type: is
id: is-01m01eb1b1pkyywa9v6mzsar85
title: Model heterogeneous completion order in adaptive-scheduler tests
kind: bug
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - testing
  - fix
dependencies:
  - type: blocks
    target: is-01m01ebsw9cyhe8thve19grn1w
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:49:51.456Z
updated_at: 2026-08-15T01:14:52.197Z
---
Build a deterministic pure controller/scheduler model that injects identical total work in different completion sequences, including slow in-flight censorship, fast-prefix/slow-suffix, slow-prefix/fast-suffix, alternating phases, narrow frontiers, late activation, and delayed handoff consumption. Use it to encode the current one-shot policy’s false-negative and false-positive signatures, then test candidate policies against behavioral invariants without wall-clock sleeps or assumptions about filesystem enumeration order.

Acceptance: the normal suite passes while a legacy-policy fixture explicitly demonstrates the observed violation rather than leaving a failing test; candidate tests assert bounded workers, liveness, traversal/exactness independence, no late scale-up without useful ready work, bounded reaction to reordered completions, and stable shutdown/disconnect behavior; the model’s cases trace back to the profile and remain portable across platforms.
