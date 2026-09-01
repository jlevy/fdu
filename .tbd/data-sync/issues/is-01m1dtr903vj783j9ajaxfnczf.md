---
type: is
id: is-01m1dtr903vj783j9ajaxfnczf
title: Prove one-shot parity and add deterministic regression guards
kind: task
status: in_progress
priority: 0
version: 7
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - validation
dependencies: []
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
child_order_hints:
  - is-01m1edc4xady6k86e0hsbzfsk1
  - is-01m1eek06tcb89yygyc1xz2yz5
hold: null
hold_until: null
created_at: 2026-09-01T06:33:23.201Z
updated_at: 2026-09-01T12:20:01.882Z
started_at: 2026-09-01T11:13:09.191Z
---
Re-profile after every accepted experiment, close only profile-named residual costs, meet the plan wall/component/allocation thresholds on control-free and control-rich real trees, add negative-tested per-entry allocation and detached zero-work guards, run the full and cross-platform gates, and record every experiment.

## Notes

The 113,794-entry second-subject run found a real residual. exp-083 tested skipping the redundant unignored reducer for control-free scopes: it removed 114,782 component allocations and 36.8 MB requested bytes per scan, brought repeat-40 allocation/reallocation/byte ratios within 1.02x of b75bf85, and cut RSS about 6%, but default-tree improved only 1.61% and cold-scan-index 0.47% with the latter CI crossing zero. Per the 3% rule, the spike was removed and recorded as rejected. Direct post-spike diagnostics still measured +4.92% default-tree and +4.01% cold-scan-index versus b75bf85. The post-spike profile leaves about 0.22 extra allocation/reallocation events and 90 requested bytes per entry, pointing to the always-inline second InternedRollUp retained on every entry. Child fdu-dg11 pre-registers H98: compact optional directory-only storage plus the control-free lane as one composite, gated on >=3% default-tree wall improvement with CI below zero, final <=1.05x resource ratios, exact semantics, and opened-discovery non-regression.
