---
type: is
id: is-01m2eb2sj3fhbxk4g8n3wnrnpg
title: "PR #54 review H86-8: three pre-registration deviations go unstated"
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eb1cnhke030h15e79fyzvc
created_at: 2026-09-13T21:34:26.881Z
updated_at: 2026-09-13T22:07:41.060Z
closed_at: 2026-09-13T22:07:41.057Z
close_reason: "Fixed in 2d36eab: the research note states all three deviations (aggregate gate not evaluated; subject not the 450,463-entry primary; fdu worker count neither pinned nor recorded, matching the floor's four only below the six-worker cap) and says future floor cells should pin --threads to the floor count and record it; the artifact summarizes them. The perf-floor harness already pins it after PR #49 FLOOR-1 (fdu-i5v6)."
resolution: null
duplicate_of: null
---
Low. Campaign-2 plan :331-334; measure.py job argv passes no --threads. (1) The aggregate <=1.25x gate on the nominated real subjects was not evaluated. (2) The subject is a new 450,001-entry corpus tree, not the pre-registered 450,463-entry primary subject. (3) fdu's worker count was neither pinned nor recorded: the automatic pool started four workers on the 4-core host, matching the floor tools' four only because the host has at most six cores (the same defect found in #49's floor harness). Fix: state all three in the note; pin fdu --threads to the floor's worker count in future floor cells.
