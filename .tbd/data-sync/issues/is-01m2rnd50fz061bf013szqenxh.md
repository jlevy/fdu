---
type: is
id: is-01m2rnd50fz061bf013szqenxh
title: "PR #83 review R4: Session::new fabricates a default delivery, so the engine cannot refuse a cache-only watch"
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2rnd0960gme4anbj6wq8m71
created_at: 2026-09-17T21:47:16.347Z
updated_at: 2026-09-17T23:46:52.963Z
closed_at: 2026-09-17T23:46:49.083Z
close_reason: "R2-R12 dispositions already landed in #83 (7390b62b) and are now on main via stack merge 98379c76."
duplicate_of: null
resolution: null
---
PR #83 review (https://github.com/jlevy/fdu/pull/83#issuecomment-5721669598), finding R4.

Take the delivery the caller opened with; only the two front doors enforce the rule today.
