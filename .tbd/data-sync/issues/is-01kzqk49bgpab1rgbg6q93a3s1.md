---
type: is
id: is-01kzqk49bgpab1rgbg6q93a3s1
title: "PR #3 review R5: specify one race-safe FSEvents cursor transaction"
kind: bug
status: closed
priority: 1
version: 2
labels:
  - pr-review
dependencies: []
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T05:01:08.847Z
updated_at: 2026-08-11T06:28:13.655Z
closed_at: 2026-08-11T06:28:13.655Z
close_reason: Specified one device-relative FullHistory transaction, applied-event cursor semantics, race fallbacks, and periodic exactness sweeps.
---
FDU-PR3-R5. docs/project/specs/active/plan-2026-08-10-fdu-fsevents-scoped-revalidation.md and performance-frontier research. Reconcile fence timing, device-relative replay, FullHistory, FileEvents path normalization, applied-boundary persistence, retention/drop fallbacks, and periodic full sweeps; make required API/state changes explicit and enumerate race tests. Review: https://github.com/jlevy/fdu/pull/3#issuecomment-5249058288.
