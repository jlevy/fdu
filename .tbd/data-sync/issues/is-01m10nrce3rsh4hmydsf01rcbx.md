---
type: is
id: is-01m10nrce3rsh4hmydsf01rcbx
title: Maintain the approved MetaBrowser query indexes inside exact commits
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nrcsqr74exp5xrxetmf83
parent_id: is-01m0y1shykye8sc7h7e9rkk6kh
created_at: 2026-08-27T03:55:56.225Z
updated_at: 2026-08-27T03:55:56.598Z
---
Add only the approved portable-path, recency, classification, partition, or navigation structures to fdu index mutation hooks. Update and remove each symmetrically in the exact commit; prove independent recomputation, conservation, rollback atomicity, memory bounds, and measured commit cost.
