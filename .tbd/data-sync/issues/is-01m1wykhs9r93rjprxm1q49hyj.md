---
type: is
id: is-01m1wykhs9r93rjprxm1q49hyj
title: Reconcile parity criteria and final architecture documentation
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - review
  - documentation
dependencies: []
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T03:29:19.143Z
updated_at: 2026-09-07T03:29:19.143Z
---
Final PR 52 review R3: the acceptance section requires median at most 1.03 and CI upper at most +5%, while later checkpoints and PR body treat +3% CI as the final gate. Pick one documented contract before running the final-binary historical parity comparison on both real subjects. Update the PR body and epic checkpoint from exp099 to exp101, clearly separating exploratory improvement versus an intermediate control from historical parity. Update the durable engine architecture description of shared commit paths to explicitly describe private historyless detached initialization and promotion while preserving the public exact mutation boundary. Avoid prescribing a particular storage representation or introducing backend indirection without a concrete second implementation.
