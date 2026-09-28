---
type: is
id: is-01m3j3pcc6dkj4bwqm605qv8vt
title: Confirm H153 major-fault non-regression on a quiet host
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-27-macos-performance-rerun.md
delegate: codex@spud10.local
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjgq9x471eftadpm5z1v9
hold: paused
hold_until: null
created_at: 2026-09-27T18:57:59.685Z
updated_at: 2026-09-28T07:44:57.792Z
started_at: 2026-09-28T03:34:15.963Z
---
Run a predeclared quiet, paired content-query confirmation of the one-pass H153 candidate on the deciding-scale subject. Resolve the exp-159 major-fault zero-delta non-regression gate and update its provisional verdict, README, and generated evidence only after the gate is decidable. Preserve raw run JSON in repository evidence. This is the explicit post-PR #137 confirmation required by the senior review; do not infer acceptance from the exploratory uncontrolled run.

## Notes

User explicitly deferred all macOS timing runs while selecting further integrations. No measurement launched. Resume only after final stack freeze and explicit user authorization; keep exact H153 oracle,100-query job,fixed12pairs,quiet/resource gates.
