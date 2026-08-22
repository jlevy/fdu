---
type: is
id: is-01m0ncyzvh3sz0yvnbgevysfkn
title: Extract the paired statistics and the accept rule
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md
labels: []
dependencies: []
parent_id: is-01m0ncyyjd1r8yh51evp1v4vcn
created_at: 2026-08-22T18:50:36.785Z
updated_at: 2026-08-22T18:50:36.785Z
---
Move the bootstrap paired-difference statistics and the accept rule out of benchmarks/realtree/ledger.py, parameterized by threshold, outcome metric, and per-guard limits. Keep the four separated evidence fields (passes_acceptance, ci_excludes_zero, direction, noninferiority) derived from the interval. Keep the bootstrap hand-rolled so the dependency list stays at softschema alone.
