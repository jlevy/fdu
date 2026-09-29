---
type: is
id: is-01m3p2sn2afjnt0wq6bjbsef9c
title: "Harness: refuse ACCEPT when a cell has invalid samples"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-09-29T07:59:16.042Z
updated_at: 2026-09-29T09:24:56.743Z
closed_at: 2026-09-29T09:24:56.743Z
close_reason: "Fixed in the harness: INCONCLUSIVE verdict for any job with invalid samples; record.py and the Experiment model refuse an accepted verdict from such a cell. 366 harness tests pass; schema/evidence/ledger checks pass."
resolution: null
duplicate_of: null
---
Found by the 2026-09-29 overnight-plan review (R9). ledger.verdict prints ACCEPT without checking invalid samples; only the exit code 3 and the runbook enforce it (explorations/benchmarks/realtree/ledger.py, __main__.py). Make the printed verdict and the record path refuse (or mark inconclusive) a comparison whose job has invalid samples, with a unit test in realtree/tests.
