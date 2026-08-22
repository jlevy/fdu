---
type: is
id: is-01m0ncz1xe04mp79skh3r9vvzx
title: Re-host fdu on the framework and prove the views are byte-identical
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-22-experiment-loop-framework-extraction.md
labels: []
dependencies: []
parent_id: is-01m0ncyyjd1r8yh51evp1v4vcn
created_at: 2026-08-22T18:50:38.893Z
updated_at: 2026-08-22T18:50:38.893Z
---
Point the perf-* targets at the framework with fdu's subject model, job catalogue, metric vector, and chart config as the adapter. The load-bearing test: regenerate the ledger, projection, and HTML report and require an empty git diff. Validate all 64 committed artifacts against the generic contract unchanged; if they need edits, the contract is wrong. Then verify the drift check by mutation.
