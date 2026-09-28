---
type: is
id: is-01m3j3pcc6dkj4bwqm605qv8vt
title: Confirm H153 major-fault non-regression on a quiet host
kind: task
status: in_progress
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-09-27-macos-performance-rerun.md
delegate: codex
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjgq9x471eftadpm5z1v9
hold: paused
hold_until: null
created_at: 2026-09-27T18:57:59.685Z
updated_at: 2026-09-28T16:20:58.012Z
started_at: 2026-09-28T03:34:15.963Z
---
Run a predeclared quiet, paired content-query confirmation of the one-pass H153 candidate on the deciding-scale subject. Resolve the exp-159 major-fault zero-delta non-regression gate and update its provisional verdict, README, and generated evidence only after the gate is decidable. Preserve raw run JSON in repository evidence. This is the explicit post-PR #137 confirmation required by the senior review; do not infer acceptance from the exploratory uncontrolled run.

## Notes

User explicitly deferred all macOS timing runs while selecting further integrations. No measurement launched. Resume only after final stack freeze and explicit user authorization; keep exact H153 oracle,100-query job,fixed12pairs,quiet/resource gates.

2026-09-28 (Claude, lead's macOS performance agent, fdu-nr2y): with the user's authorization for tonight's stack 141 re-measurement, the lead's macOS run took this cell. Stack frozen at a5c0ab46 (#139, equal to the post-merge main). Candidate: perf_probe release example from a5c0ab46 (sha256 02e8f380...). Control: the same source with the shared one-pass metric resolution disabled (one-line gate in report_in; every metric view resolves its own classification and content row per file, the pre-H153 per-view path; patch sha256 7df4709e..., perf_probe sha256 b5d219e5...). A full revert of d0902cfd and 93e417ca conflicts with the later section-builder merges, so the control removes only the shared pass, as the plan requires. Job content-query (100 reports per process, exact report oracle on every sample), 3 warmups, 12 interleaved pairs, quiet regime with the unchanged 25% gate, exploratory stage. Subject: metabrowser-clone, re-observed immediately before the cell. No other benchmark was running at launch (pgrep check in the cell log).

2026-09-28 04:34 OUTCOME: failed to qualify. Subject matched the exp-159 fingerprint exactly and was unchanged across the run. The quiet start gate admitted the run at 25.0% CPU busy; unrelated host work then kept the one-second boundary observations at a median 25.9% (10.6-56.8%), and the unchanged 25% gate invalidated 20 of 24 timed samples (1 valid control, 3 valid candidate, no valid pair). No verdict, no figure used; not relabeled or rerun uncontrolled. perf-record cannot write an artifact without a paired metric, so no experiment id was used; the raw run is docs/project/experiments/evidence/exp-159/quiet-confirmation-attempt-2026-09-28-run.json and exp-159's body records the attempt. Diagnostic (gate, not timing): major faults tracked run position, not arm: 117 on the second process of 10 of 12 pairs, 0 on the first, both 0 in the other two. The zero-delta major-fault rule compares arms, so explain this position effect before the next attempt, and run it only with the host's other workloads paused. Control recipe for the next attempt: a5c0ab46 + one-line gate disabling shared_metric_summaries in report_in (patch sha256 7df4709e...).
