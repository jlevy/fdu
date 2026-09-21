---
type: is
id: is-01m32jvw75kh2v2b8kp6r3se4v
title: "PR #94 review S1: no gate catches a wrong verdict.change_pct once regenerated"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m32h6bvgxbks1d5t5q5p0k44
created_at: 2026-09-21T18:15:17.476Z
updated_at: 2026-09-21T18:15:17.476Z
---
Senior review of PR #94, suggestion S1 (non-blocking, separate PR).

The reviewer broke exp-138's verdict.change_pct to -99.9, left the paired results untouched, ran make perf-ledger && make perf-report, and both perf-ledger-check and perf-report-check exited 0 with the ledger printing '-99.9% accepted'. No gate in make check catches a wrong headline once it has been regenerated. This is how exp-116/119 shipped green.

Fix: add a model_validator on Experiment (explorations/benchmarks/realtree/experiment.py) asserting verdict.change_pct equals results[primary_job].metrics[primary_metric].paired.change_pct within 0.01 for accepted/rejected decisions. It will flag exp-116/119 until #104 merges, which is the right outcome.

Out of scope for #94 (no harness changes in that layer beyond R1).
