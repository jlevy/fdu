---
type: is
id: is-01kzqwkhq65rv9y9ngw960b4xr
title: Track benchmark decisions across reproducible environment cells
kind: feature
status: in_progress
priority: 1
version: 4
labels:
  - performance
  - benchmark
  - ci
dependencies: []
created_at: 2026-08-11T07:46:46.117Z
updated_at: 2026-08-11T08:31:50.855Z
---
Make the real-tree benchmark harness rerunnable on cloud Linux and other hosts, identify each host/filesystem/runner as a separate evidence cell, recompute acceptance decisions per cell, compare only equivalent workloads and revisions, and publish the Linux-versus-Mac outcome in PR #4.

## Notes

Pre-commit review: R1 High compare full job contracts and variant args, not names alone; R2 High recompute portable workload identity from archived fields; R3 Medium bind runner_class to evidence_grade/provider; R4 Medium remove temporary pull_request trigger after first cloud artifact. R1-R4 fixed. Final evidence review found R5 High: GitHub Linux wait4 peak RSS was exactly 81,678,336 B in all 150 samples, so the apparent 0% RSS change was non-discriminating. Added run-wide degeneracy detection; the matrix now marks Linux RSS not-measured and rejects every Linux overall verdict while retaining wall/CPU findings. env-001 artifacts are path-free, deterministic, and zero-invalid; follow-up fdu-wfvx owns controlled fixed-concurrency repetition plus launcher-independent Linux RSS.
