---
type: is
id: is-01kzz2bbj7wbktyst23qwx2c8t
title: "Retire PR #4 as superseded"
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01kzz29dspd7bsy6jk98mpb9z3
created_at: 2026-08-14T02:41:50.150Z
updated_at: 2026-08-14T02:41:50.150Z
---
Close https://github.com/jlevy/fdu/pull/4 once the salvaged fixes land, with a comment recording what was ported, what was dropped, and why, so the branch's review history stays findable.

Superseded and deliberately not ported:
- 110k lines of archived exp-000..exp-012 evidence JSON. main renumbered the ledger and regenerates it from artifacts; PR #4's exp-012 title even collides with main's exp-012.
- The env-001 cross-environment matrix: environment.py, decision.py, scale.py, evidence.py, compat.py, environment-matrix.schema.yaml, and .github/workflows/performance-environment.yml. main answers the same question with docs/project/guides/platform-tuning.md plus benchmarks/spikes.
- Plan edits for the composable CLI surface, which shipped in PR #5.
- The R9 statistics work, which reached main independently: measure.py already reports two-sided intervals with an explicit direction.
