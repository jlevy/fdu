---
type: is
id: is-01m0ncyze04cc4bebm2ax1z9h3
title: Extract the contract models with an open subject and metric roles
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md
labels: []
dependencies: []
parent_id: is-01m0ncyyjd1r8yh51evp1v4vcn
created_at: 2026-08-22T18:50:36.351Z
updated_at: 2026-08-22T18:50:36.351Z
---
Move Experiment/MetricChange/JobResult/Complexity/Verdict/Method out of benchmarks/realtree/experiment.py with subject as a project-supplied validated payload. Add the declared metric vector: id, unit, direction, and role in {outcome, cost, guard, mechanism}. Method records which vector it was scored against so a later vector change cannot re-grade old evidence.
