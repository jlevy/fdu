---
type: is
id: is-01m2eb2tnqsvj39phr0xe31swp
title: "PR #54 review H86-11: 'not in the syscall layer' stated more strongly than the cell supports"
kind: bug
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eb1cnhke030h15e79fyzvc
created_at: 2026-09-13T21:34:28.022Z
updated_at: 2026-09-13T22:07:47.784Z
closed_at: 2026-09-13T22:07:47.783Z
close_reason: "Fixed in 2d36eab: softened in the artifact, research note, plan, PR body and fdu-xde5 notes to 2.60x total and 1.60x above the floor, pointing at consumer-side work (consistent with the earlier strace census) while stating the cell does not locate the residual: no floor-tool CPU and about 64% of candidate default-tree CPU in the kernel. Per-sample floor CPU is named as what would settle it."
resolution: null
duplicate_of: null
---
Low. Research note :71-82; artifact :442-447; plan insertion. The claim infers from wall time alone (parfloor 316 ms, arena_spike 363 ms, default-tree 822 ms) that none of the residual is in the syscall layer, but the cells record no CPU for either floor tool and the candidate's default-tree spends 64% of its CPU in the kernel. Also 'residual 2.6x' is the total ratio; the residual above the floor is 1.6x. Fix: soften the claim, or record per-sample CPU and RSS for both floor tools.
