---
type: is
id: is-01m4mpbwn8k09cvbrgd7xn7td4
title: "PR #194 A11: report a second estimator beside the paired median"
kind: task
status: open
priority: 3
version: 4
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: null
labels: []
dependencies: []
parent_id: is-01m4mpb135bvc7rjcgran42387
hold: null
hold_until: null
created_at: 2026-10-11T05:18:29.543Z
updated_at: 2026-10-11T22:00:09.034Z
started_at: 2026-10-11T21:53:45.648Z
---
Suggestion. Hodges-Lehmann or mean with bootstrap, plus marginal-median difference; flag cells where they disagree by more than the margin. Review: https://github.com/jlevy/fdu/pull/194#issuecomment-6105735578

## Notes

The committed order.py.txt now reports paired median, bootstrap mean, Hodges-Lehmann, and marginal medians for all archived two-arm cells. General harness/report integration remains deferred, waiting on a separate statistics/schema change that defines the disagreement flag and its margin and tests estimator identity; it must not silently change the registered estimator of old runs.
