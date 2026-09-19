---
type: is
id: is-01m2wa4e6c57mactkm4pxs9c04
title: "H118: first-pass analyze uses insert-then-rebuild"
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
created_at: 2026-09-19T07:47:14.251Z
updated_at: 2026-09-19T07:47:14.251Z
---
analyze_index still merge_ancestors per file. Apply H115 restore-only insert plus rebuild_rollups after the receive loop. Metric: content-basic component on metabrowser-clone. Accept: median >=3% and CI below zero; digest identical. Refute if I/O hides it.
