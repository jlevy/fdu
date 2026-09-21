---
type: is
id: is-01m32jqdbwhh12ekwymn8h6r1y
title: The evidence gates go green on a bogus verdict once it is regenerated
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T18:12:51.196Z
updated_at: 2026-09-21T21:01:24.120Z
closed_at: 2026-09-21T21:01:24.120Z
close_reason: "Fixed by https://github.com/jlevy/fdu/pull/109 (commit 36048330). Experiment carries a model validator: verdict.change_pct must be the paired primary_metric change for primary_job within 0.01 points; unmeasured job/metric and figures over empty results are refused; runs in summary._read (ledger, projection, page), perf-record before writing, and make perf-evidence-check in make check, which also fails on zero records or zero compared headlines. All 137 committed records pass; a scratch corpus with -99.919 reintroduced into exp-116 is refused by the gate and by both regeneration paths. fdu-k9h0 closed as duplicate; fdu-6xfq (kept arm as a record field) resolved in the same PR."
resolution: null
duplicate_of: null
---
Found during the PR #94 senior review, verified by execution.

A deliberately bogus `verdict.change_pct` (a -99.9 cross-job ratio where the schema wants the paired effect) is caught by `perf-ledger-check` only while the published files still disagree with it. Run `make perf-ledger` and `make perf-report` — which the publishing rule requires — and every gate goes green with the wrong figure in place.

So the drift checks verify that the generated files match the records; nothing verifies that a record's verdict matches its own measurements. That is the same shape as the defects found across this repository this week: the instrument moves with the thing it is measuring, so the signal cancels.

This is not hypothetical. Exactly this figure shipped twice: exp-116 carried -99.9% for a paired -2.2%, and exp-119 carried -99.919 for a paired +1.5%. Both were found by reading, not by a gate.

Fix: a model validator in `explorations/benchmarks/realtree/experiment.py` that rejects a record whose `verdict.change_pct` is not the `primary_metric`'s `change_pct` for the named `primary_job`, within a tolerance. It should run wherever records are parsed, so regeneration cannot launder a bad verdict.

Worth its own PR rather than riding a campaign branch, since it changes what the gate accepts.
