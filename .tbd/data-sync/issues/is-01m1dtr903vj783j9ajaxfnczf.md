---
type: is
id: is-01m1dtr903vj783j9ajaxfnczf
title: Prove one-shot parity and add deterministic regression guards
kind: task
status: in_progress
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - validation
dependencies: []
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
hold: null
hold_until: null
created_at: 2026-09-01T06:33:23.201Z
updated_at: 2026-09-01T11:38:41.731Z
started_at: 2026-09-01T11:13:09.191Z
---
Re-profile after every accepted experiment, close only profile-named residual costs, meet the plan wall/component/allocation thresholds on control-free and control-rich real trees, add negative-tested per-entry allocation and detached zero-work guards, run the full and cross-platform gates, and record every experiment.

## Notes

Second subject: current metabrowser source checkout, 113,794 entries, 15,221 directories, 98,525 files, 48 symlinks, max depth 21, digest 58bc2ea1deb0e212c7368177328184d412a1b2da24be8a77a7985a4bf6d4bc64. Controlled starts were discarded under host pressure. An uncontrolled 12-pair diagnostic found a real residual: default-tree +5.39% wall (95% CI +4.25% to +8.18%) and cold-scan-index +4.79% (+0.19% to +6.58%). Candidate repeat-40 counters versus pre-rewrite control: 62,537,499 versus 56,956,279 allocations (+1.226 per entry), 12,095,230,483 versus 10,217,437,728 requested bytes (+412 bytes per entry), and 53,795,216 versus 52,763,949 reallocations (+0.226 per entry), with equal entry/upsert/roll-up work. Profiles attribute the delta to duplicate fixed all/unignored roll-up map work; the control-free scope never uses ignore classification. Pre-registered next experiment: when ignore_rules_fingerprint is zero, maintain only all internally and project unignored=all at query boundaries; keep dual partitions unchanged for control-enabled/opened scopes. Accept only if semantic digests/tests remain exact, repeat-40 allocations fall within 1.05x of control, and the second-subject paired wall delta improves materially; otherwise revert. Deterministic slope guards pass locally but exposed platform baselines in CI: Ubuntu scan-index 10.172 allocs/entry and Windows 16.195 versus the initial macOS-derived 9.5 budget. Recalibrate per-platform only after the experiment, retaining less than one alloc/entry slack and negative-fixture proof.
