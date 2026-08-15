---
type: is
id: is-01m01eb1b1pkyywa9v6mzsar85
title: Model heterogeneous completion order in adaptive-scheduler tests
kind: bug
status: open
priority: 1
version: 2
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
updated_at: 2026-08-15T00:50:16.584Z
---
Current unit tests exercise only arithmetic fast/slow thresholds and deliberately assert that calibration decides once. Add a deterministic scheduler/controller model that presents identical total work in different chunk-completion orders, including slow in-flight censorship, fast-prefix/slow-suffix, slow-prefix/fast-suffix, narrow frontiers, end-of-scan activation, and consumer backpressure. Use the model to reproduce the current false-negative/false-positive classes before changing production behavior.

Acceptance: at least one test fails against the current one-shot policy for the same reason observed on Application Support; tests assert bounded workers, liveness, exact traversal independence, no late scale-up without useful queued work, and stable behavior under equivalent reordered completions; the eventual implementation tests are written against policy invariants rather than timing sleeps or platform-specific wall clocks.
