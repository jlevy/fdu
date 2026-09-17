---
type: is
id: is-01m2rnd1chvhg0tpdx81mdmrkw
title: "PR #83 review R1: A cfg-gated fdu-py test still builds PyIndex with the replaced fields"
kind: task
status: open
priority: 0
version: 1
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2rnd0960gme4anbj6wq8m71
created_at: 2026-09-17T21:47:12.655Z
updated_at: 2026-09-17T21:47:12.655Z
---
PR #83 review (https://github.com/jlevy/fdu/pull/83#issuecomment-5721669598), finding R1.

Fixed in cf297661: the concurrency test builds only under --no-default-features, which cargo check -p fdu-py does not cover.
