---
type: is
id: is-01m37nx1sgt52kpj7w1sb2t6z6
title: Progress engine handle and walk counters
kind: task
status: closed
priority: 2
version: 6
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
updated_at: 2026-09-24T08:20:16.649Z
closed_at: 2026-09-24T08:20:16.647Z
close_reason: "Shipped in PR #120 (merged to main as 1d2da61a, 2026-09-24): make check, cross-lint, and CI green on tree 121b0cba."
resolution: null
duplicate_of: null
---
Add public Progress, ProgressSnapshot and ProgressPhase to fdu-core per the plan's Engine section. Walker workers add ScanReport deltas to cache-line-padded shared counters once per batch they already hand to the sink (cold walk, summary fold, warm reconciliation); no handle costs one Option check per batch. Tests: counters monotonic across snapshots; at completion files and bytes equal the report's walked totals on cold, warm and summary routes; a run without a handle produces identical report bytes.

## Notes

2026-09-23: implemented on claude/progress-engine (6414ad27, 302d46e4), merged into claude/progress-indicator (407d5b8a). Public API: Progress/ProgressSnapshot/ProgressPhase (with Starting), prepare_report_with_progress, Session::start_with_progress, ScanConfig::progress (observer, excluded from scope identity). Exact equality invariant on every route except the documented overflowed-reconcile-wave rewalk (pinned). Awaiting independent review.
