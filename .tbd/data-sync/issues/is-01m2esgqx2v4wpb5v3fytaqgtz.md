---
type: is
id: is-01m2esgqx2v4wpb5v3fytaqgtz
title: perf_probe accepts --no-controls for default-tree and summary, where it no longer has any effect
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-14T01:46:44.001Z
updated_at: 2026-09-14T01:46:44.001Z
---
Follow-up on PR #52 after #51 was propagated into it (merge 753e10f, codex/streaming-performance-parity). The same defect class as review PERF-3 (fdu-7uhv).

**What changed.** #51 moved the control-observation decision into the one-shot planner: `plan_report` sets `read_controls: false` whatever the caller passes. The probe's `default-tree` used to pass `ScanConfig { read_controls: false, .. }` to copy the command line. The merge now passes the scan configuration through, and the planner decides. The merge commit's own audit says: "--no-controls no longer changes default-tree or summary; no harness job passes it to either (explorations/benchmarks/realtree), and the only test using it is the opened-discovery refusal, which still holds."

**The harness-assumption audit is done.** No harness job relies on `--no-controls` for those modes, and `default_tree_snapshot_matches_the_non_watch_cli_scope` now tests the planner.

**What remains.** `perf_probe` (`crates/fdu-core/examples/perf_probe.rs`, `Arguments::parse`) still accepts `--no-controls` for `default-tree` and `summary`, where it has no effect. PERF-3 was exactly this: a flag accepted and silently ignored by a mode, so an operator believes a variable is pinned when it is not. #52 fixed PERF-3 by making `opened-discovery` refuse flags it cannot honour (888791b).

**Fix.** Make `default-tree` and `summary` refuse `--no-controls` with a message saying the one-shot planner decides control observation. Or, if an A/B of control observation on the report path is ever wanted, route it through an explicit planner capability rather than `ScanConfig`. Add a probe unit test beside the `opened-discovery` refusal test. Update `docs/project/guides/performance-loop.md` if it lists the flag for those modes.
