---
type: is
id: is-01m31jnzma7mmnt1m13zrk94md
title: exp-119 still carries the cross-job change_pct that fdu-y050 fixes for exp-116
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T08:52:49.930Z
updated_at: 2026-09-21T16:50:48.220Z
closed_at: 2026-09-21T16:50:48.220Z
close_reason: "Fixed in e38afecc on cursor/review-leftovers-de1b (PR #104). change_pct is now the paired +1.5%; reason records the 1222x cross-job context. Ledger and chart regenerated in the same commit and now agree. Sweep found no other instance."
resolution: null
duplicate_of: null
---
`docs/project/experiments/exp-119-product-index-report-versus-one-shot-on-frameworks.md:304-309` has `primary_job: default-tree` with `change_pct: -99.919` — the 1,222x cross-job ratio, not the paired effect. Its own paired same-job figure is `+1.5%`, CI95 [-0.2%, +6.2%] (record lines 66-72).

The two generated views therefore disagree on the PR head: the ledger (`report-2026-08-10-fdu-performance-experiments.md:199`) prints `default-tree | -99.9% | accepted`, while the chart prints `exp-119 ... +1.5% on default-tree [-0.2%, +6.2%] - kept`.

That is precisely the exp-116 defect PR #104 already owns, in a sibling record, and it violates the performance loop's publishing rule that anything reporting a change takes it from the paired figure. `commit: ee014340` is real and in main, so only `change_pct` and `reason` need the exp-116 treatment, then `make perf-ledger` and `make perf-report` in the same commit.

A sweep for other instances found none: no other `-99.9` cell in docs, and no `396 ms` remains.
