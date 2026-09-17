---
type: is
id: is-01m2rnd50fz061bf013szqenxh
title: "PR #83 review R4: Session::new fabricates a default delivery, so the engine cannot refuse a cache-only watch"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2rnd0960gme4anbj6wq8m71
created_at: 2026-09-17T21:47:16.347Z
updated_at: 2026-09-17T21:47:16.347Z
---
PR #83 review (https://github.com/jlevy/fdu/pull/83#issuecomment-5721669598), finding R4.

Take the delivery the caller opened with; only the two front doors enforce the rule today.
