---
type: is
id: is-01m32jv4r3vbxkkj1dgsc1zz5r
title: "PR #94 review R1: add exp-141 to CLAIM_ONLY_EXPERIMENTS"
kind: bug
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-19-linux-parallel-validation.md
labels:
  - linux
dependencies: []
parent_id: is-01m32jv4eg76xt21gkexf9a2ns
created_at: 2026-09-21T18:14:53.443Z
updated_at: 2026-09-21T18:25:26.133Z
closed_at: 2026-09-21T18:25:26.133Z
close_reason: "Fixed on #94 in c1ec3342; disposition posted on https://github.com/jlevy/fdu/pull/94#issuecomment-5765440695"
---
https://github.com/jlevy/fdu/pull/94#issuecomment-5765270106 R1 Medium. explorations/benchmarks/realtree/timeline.py:114-116; docs/project/reports/performance-evidence/timeline.json (exp-141 kept: control); docs/project/experiments/exp-141-h111-linux-floor-and-rss-gates-fail-on-current-engine.md:179. exp-141 is claim-only (same binary both arms, floor scoreboard verdict, copies exp-103 new_failure_modes). Add exp-141 to CLAIM_ONLY_EXPERIMENTS, then make perf-ledger + make perf-report so projection has kept: null.
