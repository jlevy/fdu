---
type: is
id: is-01m2wa4dftz84m2n6gbga1peq9
title: "H116: cache-only restore without a full analysis_candidates HashMap"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2wa49kmg1xct59bpbdxvs77
created_at: 2026-09-19T07:47:13.528Z
updated_at: 2026-09-19T08:08:34.230Z
closed_at: 2026-09-19T08:08:34.229Z
close_reason: "exp-114 rejected: wall +8.70% [-19.33%, +63.90%] uncontrolled; user CPU -15.65%; engine reverted; do not retry uncontrolled"
resolution: null
duplicate_of: null
---
After H115, candidate install is 25% of restore (exp-109). Match sidecar records to the live index by path without building a Vec+HashMap of every AnalysisCandidate. Not H113 (completeness len walk). Metric: content-cache-hit wall on metabrowser-clone. Accept: median >=3% and 95% CI below zero; digest identical; incomplete sidecar refused. Uncontrolled OK if quiet fails.

## Notes

H116 / exp-114 pre-register (2026-09-19), before measure.

Claim: after H115, candidate install is the largest remaining named restore stage
(exp-109: 25.4% of restore). Cache-only restore can match sidecar records to the live
index by path (Index::lookup + restored_analysis_candidate) without allocating a
Vec+HashMap of every AnalysisCandidate, and without re-classifying every file.
Not H113 (that only skipped the second completeness len() walk).

Job: content-cache-hit wall on deciding-scale metabrowser-clone.
Accept: median at least 3% faster and 95% paired interval entirely below zero;
content digest identical; incomplete sidecar still refused.
Control: HEAD 7f289d5f (H115 in, H112 timers off). FDU_COUNTERS unset.
Candidate: point lookup + skip classify on apply_restored_analysis only.

Quiet first; if the start gate fails, uncontrolled (allowed). No RAM disk.
Revert engine on reject. exp-113 remains reserved for H113 quiet confirmatory.
