---
type: is
id: is-01m10nsfqq5vawhed0nhy4wa43
title: Replay one verified observation script through both providers
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies: []
parent_id: is-01m0y1sjnptgqhgvqcx1cjkkhw
created_at: 2026-08-27T03:56:32.375Z
updated_at: 2026-08-27T03:56:32.375Z
---
Drive the same recorded verified operations through the Python and fdu providers and compare complete state, version movement, cursor, invalidations, issues, and settled reads after every checkpoint. Do not compare two simultaneous live walks; the replay must isolate semantic agreement from observation timing.
