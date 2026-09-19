---
type: is
id: is-01m2wa4dftz84m2n6gbga1peq9
title: "H116: cache-only restore without a full analysis_candidates HashMap"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2wa49kmg1xct59bpbdxvs77
created_at: 2026-09-19T07:47:13.528Z
updated_at: 2026-09-19T07:47:13.528Z
---
After H115, candidate install is 25% of restore (exp-109). Match sidecar records to the live index by path without building a Vec+HashMap of every AnalysisCandidate. Not H113 (completeness len walk). Metric: content-cache-hit wall on metabrowser-clone. Accept: median >=3% and 95% CI below zero; digest identical; incomplete sidecar refused. Uncontrolled OK if quiet fails.
