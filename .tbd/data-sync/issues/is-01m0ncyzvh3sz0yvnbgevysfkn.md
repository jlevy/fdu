---
type: is
id: is-01m0ncyzvh3sz0yvnbgevysfkn
title: Extract the paired statistics and the accept rule
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md
labels: []
dependencies: []
parent_id: is-01m0ncyyjd1r8yh51evp1v4vcn
created_at: 2026-08-22T18:50:36.785Z
updated_at: 2026-08-22T19:00:59.015Z
---
Two statistics tests, no more: the paired bootstrap interval (from benchmarks/realtree/ledger.py) and the median-with-range overlap test (from metabrowser explorations/run.py _summarize/compare). Selected by declared evidence tier: exploratory accepts overlap at n>=3; confirmatory claims need the paired bootstrap or a pre-registered equivalent. Evidence flags (passes_acceptance, ci_excludes_zero, direction, noninferiority) derived from intervals/ranges on the way in, never stored opinions. Parameterized by threshold, outcome metric, guard limits.
