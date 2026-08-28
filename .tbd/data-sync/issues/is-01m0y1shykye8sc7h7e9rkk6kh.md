---
type: is
id: is-01m0y1shykye8sc7h7e9rkk6kh
title: Add measured native indexes and handle-local continuations
kind: feature
status: in_progress
priority: 1
version: 11
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sjbfs5h264xhme2vqymg
  - type: blocks
    target: is-01m10nsdjx7z9h87m4nf8hzhyh
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
child_order_hints:
  - is-01m10nrc1rnh7e8zzwx0z8r76c
  - is-01m10nrce3rsh4hmydsf01rcbx
  - is-01m10nrcsqr74exp5xrxetmf83
  - is-01m10nrd50t3xfevcxx7j98x5h
  - is-01m12xch71jmwypv71hygaw5cj
  - is-01m12xe7z1vh1739cmpc4k2f7z
created_at: 2026-08-26T03:28:34.258Z
updated_at: 2026-08-28T00:48:41.440Z
---
Using 3A evidence, add only the path, recency, classification, partition, and navigation indexes required for bounded MetaBrowser projections. Add a bounded handle-local continuation table with version/query/traversal position, proportional work, stale/foreign/evicted results, and close cleanup.
