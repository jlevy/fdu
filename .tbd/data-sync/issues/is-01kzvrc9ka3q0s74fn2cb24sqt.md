---
type: is
id: is-01kzvrc9ka3q0s74fn2cb24sqt
title: Expand serial/parallel reconciliation equivalence coverage
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
parent_id: is-01kzvqcp0wf2y0fwh6cgq16dxp
created_at: 2026-08-12T19:49:51.849Z
updated_at: 2026-08-13T05:51:07.419Z
closed_at: 2026-08-13T05:51:07.418Z
close_reason: Added compact serial/parallel equivalence coverage across nested structural transitions, order, workers, scope, digest, totals, extensions, completeness, and apply statistics.
---
PR #8 changes exclusive full reconciliation from serial conditional batches to bounded parallel immutable-baseline waves. Add compact high-value equivalence coverage for nested additions/removals, directory-to-file and file-to-directory transitions, BFS/DFS, several worker counts, and bounded scan depth; compare final digest, totals, named extension roll-ups, completeness, and apply statistics with the serial reference/fresh oracle.
