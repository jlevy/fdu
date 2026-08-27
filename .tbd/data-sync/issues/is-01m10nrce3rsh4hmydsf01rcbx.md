---
type: is
id: is-01m10nrce3rsh4hmydsf01rcbx
title: Maintain the approved MetaBrowser query indexes inside exact commits
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nrcsqr74exp5xrxetmf83
parent_id: is-01m0y1shykye8sc7h7e9rkk6kh
created_at: 2026-08-27T03:55:56.225Z
updated_at: 2026-08-27T09:38:03.324Z
closed_at: 2026-08-27T09:38:03.323Z
close_reason: Approved minimal opened-only serving index set is implemented, independently proven, measured, and green locally and in the complete 19-job PR CI matrix at a286145.
resolution: null
duplicate_of: null
---
Add only the approved portable-path, recency, classification, partition, or navigation structures to fdu index mutation hooks. Update and remove each symmetrically in the exact commit; prove independent recomputation, conservation, rollback atomicity, memory bounds, and measured commit cost.

## Notes

Completed at fdu commit a286145. The optional ServingIndexes allocation now maintains portable path/child order, semantic all/unignored tallies, a declared-vocabulary exact-basename tally, and global portable-file recency only for OpenedIndex. Independent recomputation covers insert, metadata update, kind change, ignore reclassification, removal, subtree removal, invalid native paths, snapshots, and failed commits. Structural tests bound exact-name maps by the eleven-name registry vocabulary and directory arena. The alternating seven-sample release probe over 10,101 entries measured detached median 51,246 us and opened median 62,781 us (1.225x), with 202 exact-name rows. Detached Index, one-shot Python, and the standalone CLI return before any serving classification/allocation. Full local make check passed with 125 unchanged CLI goldens; PR #48 CI run 33058939287 passed all 19 jobs on Linux, macOS, Windows, MSRV 1.85, feature boundaries, wheels/parity, docs, audit, and performance evidence.
