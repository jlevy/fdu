---
type: is
id: is-01m0y1sce6j2y5ac1nzgtsdwsx
title: Add removal-aware control state and fixed all/unignored reducers
kind: feature
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
delegate: codex@spud10.local
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1scsff119ypyb93tbbnxh
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
hold: null
hold_until: null
created_at: 2026-08-26T03:28:28.613Z
updated_at: 2026-08-26T10:59:02.889Z
started_at: 2026-08-26T10:00:35.511Z
closed_at: 2026-08-26T10:59:02.887Z
close_reason: Implemented bounded exact .gitignore control state, atomic reclassification, fixed all/unignored reducers, producer integration, snapshot v3 persistence, feature-matrix gates, and MetaBrowser fixture coverage.
resolution: null
duplicate_of: null
---
Implement a bounded per-directory control table for exact .gitignore source and matcher state plus fixed PartitionRollUp all/unignored maintenance. Cover creation, edit, last-file deletion, negation, hidden control files, removal churn, atomic reclassification, and snapshot round-trip without importing generic tags or promoted planes.
