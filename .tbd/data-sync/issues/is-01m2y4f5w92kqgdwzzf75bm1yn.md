---
type: is
id: is-01m2y4f5w92kqgdwzzf75bm1yn
title: "H141: Linux content-query H138 same or different"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-linux-parallel-validation.md
labels:
  - linux
  - H141
dependencies: []
parent_id: is-01m2y4f4g34vdbgxf0jcvt8dw3
created_at: 2026-09-20T00:46:43.592Z
updated_at: 2026-09-20T01:25:29.252Z
closed_at: 2026-09-20T01:25:29.251Z
close_reason: "Same on Linux (exp-140): content-query wall -17.60% [-18.07%, -17.17%] uncontrolled on linux-v6.12; digest identical; no engine patch"
---
Pair #91 control vs this branch on Linux content-query. Same means share-one-walk still clears 3%.

## Notes

PREDICT (2026-09-20, Linux KVM)

Hypothesis: H141 / exp-140
Job: content-query (100 Types/Families/Languages/Documents reports)
Subject: linux-v6.12 (same reconstructible clone as exp-138/139)
Control: e667b739 #91 probe
Candidate: HEAD probe with H138 share-every-entry
Prediction: same -- share-one-walk still clears 3% wall
Smoke: one candidate run 10.6s wall / 8.3s component, digest 06260f6c matches exp-138
Regime: quiet first. Do not lower the 25% bar.
