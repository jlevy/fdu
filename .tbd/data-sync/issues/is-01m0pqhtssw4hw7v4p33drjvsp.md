---
type: is
id: is-01m0pqhtssw4hw7v4p33drjvsp
title: "PR #38 review R8: sparse-ratio threshold is a magic number"
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m0pqh0yf7etx8dywann7tnx3
created_at: 2026-08-23T07:14:54.392Z
updated_at: 2026-08-23T07:34:39.075Z
closed_at: 2026-08-23T07:34:39.074Z
close_reason: "Fixed: SPARSE_RATIO_THRESHOLD, documented with the exp-064/065 case and its relationship to the evidence-scope plan's margin."
---
summary.py uses a bare 2 for apparent/allocated; the evidence-scope plan proposes the same 2x as the materially-different margin on sparseness. Name it once and share it.
