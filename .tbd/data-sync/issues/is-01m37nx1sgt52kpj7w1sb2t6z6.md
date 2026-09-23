---
type: is
id: is-01m37nx1sgt52kpj7w1sb2t6z6
title: Progress engine handle and walk counters
kind: task
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-23-fdu-progress-indicator.md
labels:
  - cli
  - ux
dependencies:
  - type: blocks
    target: is-01m37nx2ra4v4f584gv1x3pn9z
  - type: blocks
    target: is-01m37nxa8vshzngyvzw9ax67a0
parent_id: is-01m2nsj4zgw7r9j3d1rphnmwf2
created_at: 2026-09-23T17:44:33.838Z
updated_at: 2026-09-23T17:44:49.771Z
---
Add public Progress, ProgressSnapshot and ProgressPhase to fdu-core per the plan's Engine section. Walker workers add ScanReport deltas to cache-line-padded shared counters once per batch they already hand to the sink (cold walk, summary fold, warm reconciliation); no handle costs one Option check per batch. Tests: counters monotonic across snapshots; at completion files and bytes equal the report's walked totals on cold, warm and summary routes; a run without a handle produces identical report bytes.
