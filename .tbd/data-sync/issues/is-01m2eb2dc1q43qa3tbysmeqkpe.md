---
type: is
id: is-01m2eb2dc1q43qa3tbysmeqkpe
title: "PR #54 review H86-7: floor headline taken on cold-scan-index (4.86x) instead of the index tier's default-tree (2.60x)"
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eb1cnhke030h15e79fyzvc
created_at: 2026-09-13T21:34:14.400Z
updated_at: 2026-09-13T22:07:38.413Z
closed_at: 2026-09-13T22:07:38.412Z
close_reason: "Fixed in 2d36eab: verdict.primary_job is default-tree (change -31.704), the headline is 2.60x the syscall floor and 6.59x spike RSS with cold-scan-index 4.86x and 5.03x as supporting, in the artifact, research note, plan, ledger, and PR body."
resolution: null
duplicate_of: null
---
Low. Artifact :407 (verdict.primary_job) and :432-433; campaign-2 plan :73 defines the index tier as default tree, and the floor report's 2.68x is that job. The like-for-like figure is default-tree at 2.60x, still a fail, so the verdict stands but the distance from the gate was overstated. Fix: headline 2.60x (and 6.59x RSS) and give cold-scan-index's 4.86x/5.03x as supporting.
