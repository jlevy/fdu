---
type: is
id: is-01m2eb39gbn8kqy11p8fh9jtan
title: Record per-arm max/min in the experiment contract and commit run JSON with each artifact
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - perf
  - campaign-2
dependencies: []
created_at: 2026-09-13T21:34:43.211Z
updated_at: 2026-09-13T21:34:55.601Z
---
Deferred from PR #54 review H86-3 (parent fdu-szu3). The campaign-2 H86 pre-registration requires 'all raw samples, p95/median, and max/min for every arm' and gates the candidate on max/min <= 2.0, describing tail spread as 'a recorded field on both arms'. MetricChange in explorations/benchmarks/realtree/experiment.py carries control_p95_over_median/candidate_p95_over_median but no max/min, so no artifact can satisfy that rule, and exp-103's candidate max/min is unverifiable because its run JSON lived only on a deleted VM. Add a max_over_min figure beside p95_over_median in measure.py's per-arm summary (measure.py:1841), carry it through experiment.py (:450) as optional control_max_over_min and candidate_max_over_min, recompile experiment.schema.yaml (make perf-schema), render it in the ledger beside the p95 tails (summary.py:315), and cover it in test_experiment.py. Also make the raw run JSON durable: follow #52's convention of committing it under docs/project/experiments/evidence/exp-NNN/run.json (or have perf-record copy it there) so run_artifact never points at a machine-local path again. Needed before the H86 Linux floor stage (fdu-xde5) is re-run.
