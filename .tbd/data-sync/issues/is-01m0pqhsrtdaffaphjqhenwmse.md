---
type: is
id: is-01m0pqhsrtdaffaphjqhenwmse
title: "PR #38 review R5: exp-065 explains the cold non-transfer by denominator alone"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m0pqh0yf7etx8dywann7tnx3
created_at: 2026-08-23T07:14:53.337Z
updated_at: 2026-08-23T07:34:38.204Z
closed_at: 2026-08-23T07:34:38.204Z
close_reason: "Fixed: exp-065 now decomposes the 4.36x wall gap into 1.31x mechanism (user CPU per file, depth-consistent) and 3.32x conversion to wall (0.95 sparse vs 0.29 dense). Rule carried into the loop guide and the evidence-scope plan. Also corrected the stated per-file wall saving from 4.39 to 4.31 us."
---
The per-file wall saving differs 4.4x (4.31 vs 0.99 us/file) while the per-file user-CPU saving differs only 1.31x (4.53 vs 3.45 us/file), consistent with depth 16 vs 10. The remaining 3.3x is conversion of CPU saving to wall: 95% on the sparse tree, 29% on the dense one. Name overlap as the second factor.
