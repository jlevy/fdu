---
type: is
id: is-01m4mpbp6x7wsjs17sjemhsb0x
title: 'PR #194 A2: "order effect within +/-2%" is contradicted by the committed runs; order-split.txt lists a subset'
kind: bug
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: codex-pr194-finish
labels: []
dependencies: []
parent_id: is-01m4mpb135bvc7rjcgran42387
hold: null
hold_until: null
created_at: 2026-10-11T05:18:22.941Z
updated_at: 2026-10-11T22:47:45.707Z
started_at: 2026-10-11T05:18:35.292Z
closed_at: 2026-10-11T22:47:45.707Z
close_reason: "Fixed/disclosed in f98e52ddb813c5d4f557a715fd5295061e692330 and refreshed PR metadata; review A dispositions: https://github.com/jlevy/fdu/pull/194#issuecomment-6114742688. All local make-check targets passed across the documented interrupted prefix and guarded continuations, with explicit smoke-wheel path independence. CI completion is delegated to the coordinator."
resolution: null
duplicate_of: null
---
Medium. exp-220-...md:425-430; evidence/exp-220/order-split.txt; report-2026-08-20-...md:1202; PR body Method notes. Two job-cells exceed 2% by median split (exp-219 run-h3 default-tree, exp-220 run-g2 opened-second-report); exp-220 summary by means about +3.84%. order-split.txt lists 10 of 17 cells and names uncommitted files. Fix: regenerate over every job of every committed two-arm run; state the bound as measured. Review: https://github.com/jlevy/fdu/pull/194#issuecomment-6105735578
