---
type: is
id: is-01m2ew97f3ay7c2rp9ztb7b8et
title: "perf_probe: --worker-policy and --diagnostics are still accepted by modes that ignore them"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-14T02:35:03.518Z
updated_at: 2026-09-14T02:35:03.518Z
---
Found while fixing fdu-8yv0 and fdu-tilu on PR #52 (codex/streaming-performance-parity at ba83690). It is the same defect class as review PERF-3: a flag is accepted and silently ignored, so a run can be recorded under a setting it never applied.

`crates/fdu-core/examples/perf_probe.rs` `Arguments::parse` accepts `--diagnostics` and `--worker-policy` for every mode. Since ba83690 only opened-discovery refuses them. At ba83690 they are read by:
- `scan_producer` (:552-557) and `scan_index` (:610-614), which read both.
- `default_tree` (:743-744), which reads `--diagnostics` only. `prepare_report_with_scan_diagnostics` takes no worker policy, so `default-tree --worker-policy repeated` runs the shipped policy while the invocation names another.
- No other mode. `summary` (`summary_tier`, :654) and every content, detect, delta, revalidate, snapshot, cold-open-save, and query mode accept both and read neither.

No realtree job passes either flag to those modes today: only `scan-index` takes `--diagnostics` (explorations/benchmarks/realtree/measure.py:453). This is latent, not live.

Fix: make the check per mode rather than one opened-discovery special case. An allowlist of which modes apply each flag is the simplest form: refuse `--worker-policy` outside scan-producer and scan-index, and `--diagnostics` outside those two and default-tree. Add refusal tests beside `opened_discovery_refuses_walk_flags_it_cannot_apply` and `one_shot_report_modes_refuse_no_controls_the_planner_overrides`.
