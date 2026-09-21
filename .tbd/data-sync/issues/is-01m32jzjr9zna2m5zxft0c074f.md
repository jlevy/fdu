---
type: is
id: is-01m32jzjr9zna2m5zxft0c074f
title: "PR #97 review R1: claim-only for non-shipped accepted arms"
kind: bug
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels: []
dependencies: []
parent_id: is-01m32jzamqf58124kga00kdbd0
created_at: 2026-09-21T18:17:18.857Z
updated_at: 2026-09-21T18:32:07.565Z
closed_at: 2026-09-21T18:32:07.565Z
close_reason: "Addressed on #97: CLAIM_ONLY exp-146/148/149/150/152/154 plus recycle reuse test, publishing sentence, DT_UNKNOWN rustdoc, and finish unused-vec fix. Shipped on 16af624f / cfbd3533 after merge-down onto #94 c1ec3342."
---
High. Comment 5765288334. kept_variant treats accepted → candidate, so the evidence page draws --threads 8 screens (exp-146/148/149), the PGO profile-use build (exp-154), H85's rejected 20% arm (exp-150), and H72 rejected on v6.12 (exp-152) as current cost. Fix: add those ids to CLAIM_ONLY_EXPERIMENTS in explorations/benchmarks/realtree/timeline.py (exp-103 one-line reason style), then make perf-ledger && make perf-report. Do not invent a new decision-value contract. Do not add exp-141 (sibling on #94).
