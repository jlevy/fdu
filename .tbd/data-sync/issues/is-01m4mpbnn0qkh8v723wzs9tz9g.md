---
type: is
id: is-01m4mpbnn0qkh8v723wzs9tz9g
title: "PR #194 A1: exp-220 calls H195 failed on the dense summary; the registered rule makes it inconclusive"
kind: bug
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: codex-pr194-finish
labels: []
dependencies: []
parent_id: is-01m4mpb135bvc7rjcgran42387
hold: null
hold_until: null
created_at: 2026-10-11T05:18:22.367Z
updated_at: 2026-10-11T21:53:45.529Z
started_at: 2026-10-11T05:18:34.945Z
---
Medium. exp-220-...md:397,444-449; performance-loop.md:940 (H195); plan-2026-10-09-...md:666-667; report-2026-08-20-...md:1189; performance-loop-runbook.md:1008; PR body. H195's registration makes an interval crossing +3% inconclusive; harness says noninferiority inconclusive. Also estimator-sensitive (mean +3.53%, HL +3.97%, marginal medians -0.68%). Fix: reword, cite exp-217 b1 outer-arm +5.45% [+1.99%, +7.20%], add b1/b2 aggregate-summary rows to exp-217, note estimator range in fdu-4arl. Review: https://github.com/jlevy/fdu/pull/194#issuecomment-6105735578
