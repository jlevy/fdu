---
type: is
id: is-01m2et31c6a5zryh6mw57f7v0s
title: "PR #52 verification FIX52-1: opened-discovery still silently accepts --worker-policy and --diagnostics"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-14T01:56:43.525Z
updated_at: 2026-09-14T01:56:43.525Z
---
Verification of #52 fix 888791b (PERF-3). perf_probe Arguments::parse sets worker_policy (perf_probe.rs:241-250) and diagnostics (:233) for every mode, but walk_only_flag is set only for --no-controls/--order/--threads/--max-depth (:232-271); opened_discovery (:931-985) reads neither. A run is recorded under a worker policy or diagnostics setting it never applied -- the PERF-3 defect for two flags the review's list omitted. Fix: set walk_only_flag for both and add them to opened_discovery_refuses_walk_flags_it_cannot_apply's table.
