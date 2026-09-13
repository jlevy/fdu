---
type: is
id: is-01m2eb2sy2gjy29res0hj897ym
title: "PR #54 review H86-9: 'every candidate p95/median <= 1.109' holds for wall only"
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eb1cnhke030h15e79fyzvc
created_at: 2026-09-13T21:34:27.265Z
updated_at: 2026-09-13T22:07:43.034Z
closed_at: 2026-09-13T22:07:43.032Z
close_reason: "Fixed in 2d36eab: the bound is stated for wall p95/median (<= 1.109) everywhere, with the largest across all recorded metrics given as 1.137 (default-tree CPU)."
resolution: null
duplicate_of: null
---
Low. Artifact :423-424 (and plan insertion, PR body). Candidate cpu_ns p95/median reaches 1.137 (default-tree) and system_cpu_ns 1.124; nothing approaches the 1.5 limit. Fix: say wall.
