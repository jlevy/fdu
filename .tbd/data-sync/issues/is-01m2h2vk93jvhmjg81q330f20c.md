---
type: is
id: is-01m2h2vk93jvhmjg81q330f20c
title: "PR #55 review PR55-IMM-1: replaceable names break repeatable A-to-B comparisons"
kind: bug
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2h2v87crh401e90pfe52w8g
created_at: 2026-09-14T23:08:25.757Z
updated_at: 2026-09-14T23:54:21.820Z
closed_at: 2026-09-14T23:54:21.816Z
close_reason: "4727de0: comparisons bind to immutable checkpoint ids; labels are movable names; removed checkpoints refused; slice 2 acceptance updated"
resolution: null
duplicate_of: null
---
PR #55 at dc27c14: checkpoints plan :55-57, :65-67, :139, :173, :246. Names can be replaced but comparisons and the slice-2 acceptance rule refer to checkpoints by name. Bind comparisons to immutable checkpoint ids with the name as a movable label, or refuse replacement.
