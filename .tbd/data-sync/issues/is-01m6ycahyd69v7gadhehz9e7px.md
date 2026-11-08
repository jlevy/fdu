---
type: is
id: is-01m6ycahyd69v7gadhehz9e7px
title: "Add the ages view: selected bytes by change-time band, every window from one walk"
kind: feature
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies:
  - type: blocks
    target: is-01m6ycajwxh4xpjv9f5qc0rx9m
parent_id: is-01m6yc9x14t8x0v0j2v2dc34jv
created_at: 2026-11-08T20:06:48.013Z
updated_at: 2026-11-08T20:06:51.994Z
---
Phase 2. ViewSpec::Ages: regular files partitioned by change-time band on the age ladder (future, under 1h, 1h-1d, 1d-7d, 7d-30d, 30d-90d, 90d-1y, 1y-3y, over 3y, unknown); grouped-row text with a cumulative 'within' continuation line; machine records with exact edges, tallies, ignored split, cumulative, and basis. Rows sum to summary; each row equals its --changed-since/--changed-before summary (tested); several roots; --view full; watch bands resolve from each repaint's instant. Metadata-only. This is the mtime-recency histogram reducer fdu-a6dz names.
