---
type: is
id: is-01m3j3pcc6dkj4bwqm605qv8vt
title: Confirm H153 major-fault non-regression on a quiet host
kind: task
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-27-macos-performance-rerun.md
delegate: codex@spud10.local
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjgq9x471eftadpm5z1v9
hold: paused
hold_until: null
created_at: 2026-09-27T18:57:59.685Z
updated_at: 2026-09-28T11:15:19.670Z
started_at: 2026-09-28T03:34:15.963Z
---
Run a predeclared quiet, paired content-query confirmation of the one-pass H153 candidate on the deciding-scale subject. Resolve the exp-159 major-fault zero-delta non-regression gate and update its provisional verdict, README, and generated evidence only after the gate is decidable. Preserve raw run JSON in repository evidence. This is the explicit post-PR #137 confirmation required by the senior review; do not infer acceptance from the exploratory uncontrolled run.

## Notes

User explicitly deferred all macOS timing runs while selecting further integrations. No measurement launched. Resume only after final stack freeze and explicit user authorization; keep exact H153 oracle,100-query job,fixed12pairs,quiet/resource gates.

2026-09-28 (Claude, lead's macOS performance agent, fdu-nr2y): with the user's authorization for tonight's stack 141 re-measurement, the lead's macOS run took this cell. Stack frozen at a5c0ab46 (#139, equal to the post-merge main). Candidate: perf_probe release example from a5c0ab46 (sha256 02e8f380...). Control: the same source with the shared one-pass metric resolution disabled (one-line gate in report_in; every metric view resolves its own classification and content row per file, the pre-H153 per-view path; patch sha256 7df4709e..., perf_probe sha256 b5d219e5...). A full revert of d0902cfd and 93e417ca conflicts with the later section-builder merges, so the control removes only the shared pass, as the plan requires. Job content-query (100 reports per process, exact report oracle on every sample), 3 warmups, 12 interleaved pairs, quiet regime with the unchanged 25% gate, exploratory stage. Subject: metabrowser-clone, re-observed immediately before the cell. No other benchmark was running at launch (pgrep check in the cell log).
