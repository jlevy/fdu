---
type: is
id: is-01m01eaac9f07exaqb7erjzf0y
title: Expose adaptive-worker and macOS backend decisions in performance artifacts
kind: bug
status: open
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - instrumentation
  - fix
dependencies:
  - type: blocks
    target: is-01m01easxwr1504mqmnha3brzh
  - type: blocks
    target: is-01m01ebsw9cyhe8thve19grn1w
  - type: blocks
    target: is-01m01ec396v5crqyg5sfasfehr
  - type: blocks
    target: is-01m01ecbhsetn1rmvfn8m26w7e
  - type: blocks
    target: is-01kzy1w2vbam0mr1z5we4y6fy0
  - type: blocks
    target: is-01m01eg0efe53jc3smgaza7wk7
  - type: blocks
    target: is-01m01egyp43zd6yj43cjf1ge1d
parent_id: is-01m01ea0psdcnb2sdwdj6vh171
created_at: 2026-08-15T00:49:27.944Z
updated_at: 2026-08-15T00:53:05.347Z
---
The benchmark artifacts currently cannot prove whether automatic scheduling stayed at six or scaled, even when reports claim that it did. Add runtime-gated, low-overhead observability for available/initial/maximum workers, calibration windows and measured signal, decision and entry ordinal, active/peak workers, pending/outstanding directory work, consumer backpressure, and macOS bulk versus portable-fallback directory counts. Carry the required fields into perf-probe raw artifacts without changing stable user output. Fail closed when a policy comparison requests fields the measured binary cannot provide.

Acceptance: unit tests cover all policy outcomes and unavailable-field behavior; a deliberate missing counter/field makes the artifact invalid; FDU_COUNTERS and probe values cross-check; instrumentation overhead is measured using the established paired protocol and recorded before controller experiments rely on it; make check and cross-lint pass for touched platform-gated code.
