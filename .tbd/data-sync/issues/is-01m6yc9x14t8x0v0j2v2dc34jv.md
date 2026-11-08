---
type: is
id: is-01m6yc9x14t8x0v0j2v2dc34jv
title: "Recent growth without a baseline: change time, age profiles, and volume capacity"
kind: epic
status: open
priority: 2
version: 9
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies: []
child_order_hints:
  - is-01m6ycae17xp8ypt4jhhgx3am2
  - is-01m6ycaezvjs33xsh7fp1whpka
  - is-01m6ycafymrea5sgq8r7n621re
  - is-01m6ycagweqhppqznjzwagp365
  - is-01m6ycahyd69v7gadhehz9e7px
  - is-01m6ycajwxh4xpjv9f5qc0rx9m
  - is-01m2xbag4cjyqqvq776rnfxcyt
  - is-01m6yn4ejvcq7bxw43fd0em0cq
created_at: 2026-11-08T20:06:26.596Z
updated_at: 2026-11-08T22:40:45.146Z
---
Answer 'where did disk space go over a window?' from one metadata walk with no saved state, as the spec describes: phase 0 documents the recipe that works today; phase 1 adds change time (max of mtime and ctime) as a selection window and sort key plus a selection-coverage note; phase 2 adds the ages view (selected bytes partitioned by change-time band, every window from one walk); phase 3 adds the volumes view (capacity of each filesystem the report touches beside the bytes it counted; promotes fdu-eas0). Net change with deletions stays with checkpoints (fdu-8ybz).
