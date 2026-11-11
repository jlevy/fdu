---
type: is
id: is-01m76a5hn111z2jz5wvhx6jrxv
title: "ages: answer an unfiltered ages from the summary fold, not the full index"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-11T22:03:02.177Z
updated_at: 2026-11-11T22:03:02.177Z
---
Review K1 on #209 (https://github.com/jlevy/fdu/pull/209#issuecomment-6808107492). An unfiltered --view ages (and summary,ages / tree,ages) retains the full index: 60 MiB vs 14 MiB for summary on 179k entries, ~1 GiB projected for a 3M-entry home. Add an age counter to SummaryFold, widen summary_is_sufficient to views drawn from {Summary, Ages}, build the Ages section from the fold, and hold both routes equal with a test on both clocks plus the path-independence ages cases. Optionally let the folded tree count ages so tree,ages keeps the folded tree. Deferred from 0.6.0: a new planner route at the release step.
