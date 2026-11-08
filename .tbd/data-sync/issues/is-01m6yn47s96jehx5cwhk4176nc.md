---
type: is
id: is-01m6yn47s96jehx5cwhk4176nc
title: "Ages tests: partition and window identities, aging by instant, goldens, parity"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies:
  - type: blocks
    target: is-01m6yn48r2xapbnj24jydz2n62
parent_id: is-01m6ycahyd69v7gadhehz9e7px
created_at: 2026-11-08T22:40:38.185Z
updated_at: 2026-11-08T22:40:39.169Z
---
Rows sum to summary; each row equals its band-table window with --kind file --view summary at the same instant; cumulative equals --changed-since <edge>; engine aging by request instant across bands; goldens pin layout with patterns; Python parity.
