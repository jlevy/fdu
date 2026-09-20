---
type: is
id: is-01m2y4f4z059xvgn1hhc65fb78
title: "H139: Linux cache-hit stack same or different"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-19-linux-parallel-validation.md
labels:
  - linux
  - H139
dependencies:
  - type: blocks
    target: is-01m2y4f5w92kqgdwzzf75bm1yn
  - type: blocks
    target: is-01m2y4f6c3ea0q3ngn99nz7eek
parent_id: is-01m2y4f4g34vdbgxf0jcvt8dw3
created_at: 2026-09-20T00:46:42.654Z
updated_at: 2026-09-20T01:06:36.275Z
closed_at: 2026-09-20T01:06:36.275Z
close_reason: "exp-138: same on Linux, content-cache-hit wall -22.48% [-23.46%, -21.39%] quiet on linux-v6.12; RSS -10.24%; no engine patch"
---
Pair #91 (H115+H120) control vs this branch on Linux content-cache-hit. Replication, not a new cut. Record same vs different. exp-138+. Do not revert landed engine on a miss.

## Notes

PREDICT (2026-09-20, Linux KVM 4-core Xeon, ext4, virtualized)

Hypothesis: H139 / exp-138
Job: content-cache-hit (wall_ns primary; peak RSS reported)
Subject: linux-v6.12 — reconstructible shallow clone of github.com/torvalds/linux tag v6.12 (adc21867), 92,474 entries / 86,643 files / 5,769 dirs. Deciding-scale source-checkout. Clean metabrowser clone on this host is only 916 entries (Darwin 146k was workspace state); do not treat that as deciding.
Control: e667b739 release probe (H115+H120 only), --no-default-features
Candidate: HEAD of perf/campaign-linux-2026-09-19 (H125+H129+H131+H133 stacked on that control)
Prediction: same — stacked cache-hit commits still clear 3% wall with interval below zero. Replication, not a new cut. Do not revert landed engine on a miss.
Regime: PERF_HOST_REGIME=quiet first (Linux gate is load/core <= 0.25). Do not lower the bar. If start gate fails or samples invalidate, label uncontrolled and re-run.
Host: 4-core KVM, 15 GiB, ext4, virtualized. Same class as exp-103.
