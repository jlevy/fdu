---
type: is
id: is-01m10nrce3rsh4hmydsf01rcbx
title: Maintain the approved MetaBrowser query indexes inside exact commits
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nrcsqr74exp5xrxetmf83
parent_id: is-01m0y1shykye8sc7h7e9rkk6kh
created_at: 2026-08-27T03:55:56.225Z
updated_at: 2026-08-27T08:06:30.972Z
---
Add only the approved portable-path, recency, classification, partition, or navigation structures to fdu index mutation hooks. Update and remove each symmetrically in the exact commit; prove independent recomputation, conservation, rollback atomicity, memory bounds, and measured commit cost.

## Notes

Approved design recorded in the plan after Phase 3A review: move existing portable path/child structures into optional ServingIndexes enabled only by OpenedIndex; add one semantic type/classifier tally and one global portable-file recency order; derive canonical/family/group/preset/navigation rows; no duplicate catalog/dimension cache. Gate independent recomputation and exact mutations plus proof detached Index and standalone CLI allocate no serving state and preserve scan memory/time.
