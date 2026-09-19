---
type: is
id: is-01m2wa4e6c57mactkm4pxs9c04
title: "H118: first-pass analyze uses insert-then-rebuild"
kind: task
status: in_progress
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2wa49kmg1xct59bpbdxvs77
created_at: 2026-09-19T07:47:14.251Z
updated_at: 2026-09-19T08:09:13.056Z
---
analyze_index still merge_ancestors per file. Apply H115 restore-only insert plus rebuild_rollups after the receive loop. Metric: content-basic component on metabrowser-clone. Accept: median >=3% and CI below zero; digest identical. Refute if I/O hides it.

## Notes

H118 / exp-115 pre-register (2026-09-19), before measure.

Claim: first-pass analyze_index still commits with merge_ancestors per file.
H115 deleted that shape on restore only. The receive loop can insert with
commit_without_rollup and rebuild_rollups once after the pool joins.

Job: content-basic component (analyze only) on deciding-scale metabrowser-clone.
Accept: median at least 3% faster and 95% paired interval entirely below zero;
content digest identical.
Control: HEAD 55261e6c (H115 in, H116 reverted). FDU_COUNTERS unset.
Candidate: apply_restored_analysis in the receive loop plus one rebuild.

Quiet first; if the start gate fails, uncontrolled (allowed). No RAM disk.
Revert engine on reject.
