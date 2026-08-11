---
type: is
id: is-01kzs6jne1jffaaxcyt67knzxt
title: "PR#6 R4: verification interval list is unbounded"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
parent_id: is-01kzs5141vz8jtgb4wh2j432vb
created_at: 2026-08-11T20:00:17.345Z
updated_at: 2026-08-11T20:02:22.195Z
closed_at: 2026-08-11T20:02:22.194Z
close_reason: "Fixed in 07e2073, with the partial rebuttal upheld. The stated mechanism was not the defect: siblings inheriting Revalidated from an earlier root-wide sweep is correct, because that sweep did verify them and observed_at carries the age for the consumer to judge. The real problem was unbounded growth, since finish_reconcile collapses only records under the swept path; the list is now capped at 256 dropping oldest first, which is fail-safe because a path losing its interval falls back to Cached. Regression test asserts the bound holds under repeated scoped sweeps."
---
Cursor Bugbot, Medium, on the R1 fix. Filed with a partial rebuttal. The stated mechanism - siblings inheriting Revalidated from an earlier root-wide interval - is actually correct behaviour: that sweep DID verify them, and observed_at carries the age so a consumer can judge staleness itself; that is what provenance is for. What is genuinely wrong is resource growth: finish_reconcile removes only intervals UNDER the reconciled path, so repeated scoped reconciles of sibling subtrees (exactly what a browser doing per-navigation revalidation would produce) accumulate entries without bound. Fix: collapse covered intervals and cap the list, dropping oldest first - losing an interval is fail-safe, since it reports Cached rather than Revalidated and so under-claims trust.
