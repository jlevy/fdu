---
type: is
id: is-01m4fxapwnccxwmbfezf7gh0tk
title: "Age: human_age ladder adds mo and y, shared by tree and --long"
kind: task
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies:
  - type: blocks
    target: is-01m4fxar5gf3qrax2k0rwfczn0
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-09T08:43:58.740Z
updated_at: 2026-10-09T11:16:02.245Z
started_at: 2026-10-09T09:28:58.357Z
closed_at: 2026-10-09T11:16:02.244Z
close_reason: "Landed in 011ddfaa: human_age ladder s/m/h/d/mo (30.44d)/y (365.25d), units switch at their own length, floored, signed (-0s kept); shared by tree and --long; parse_age refusal names the day equivalent (3months -> use 91d); unit tests pin every edge; cli-axes golden expects years. make check green at 66652033."
resolution: null
duplicate_of: null
---
human_age: s <60s, m <60min, h <24h, d <30.44d, mo (30.44d) <365.25d, y (365.25d); units switch at their own length (no 0mo/0y); floored toward zero; signed, -0s under a second ahead as today. Shared by tree and --long. parse_age still refuses mo/y and its refusal names the day equivalent. Update unit tests at report_format.rs (30d, 1,000d) and the cli-axes golden (2000-01-01 fixture -> 26y).
