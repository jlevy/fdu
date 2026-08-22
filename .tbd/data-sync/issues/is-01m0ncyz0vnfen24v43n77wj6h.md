---
type: is
id: is-01m0ncyz0vnfen24v43n77wj6h
title: Define the run document schema and conformance fixtures
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md
labels: []
dependencies: []
parent_id: is-01m0ncyyjd1r8yh51evp1v4vcn
created_at: 2026-08-22T18:50:35.930Z
updated_at: 2026-08-22T18:50:35.930Z
---
The framework owns the run document; the project owns producing one. Specify variants (content-hashed, with flags), variant_order, jobs, conditions, an open subject block, and per-trial samples carrying a metric vector plus validity and reasons. Make fdu-realtree-run-v1 the first conformant profile. Ship two fixtures: one performance run, one non-performance run.
