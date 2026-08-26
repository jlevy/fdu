---
type: is
id: is-01m0y1shykye8sc7h7e9rkk6kh
title: Add measured native indexes and handle-local continuations
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sjbfs5h264xhme2vqymg
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:34.258Z
updated_at: 2026-08-26T03:28:58.843Z
---
Using 3A evidence, add only the path, recency, classification, partition, and navigation indexes required for bounded MetaBrowser projections. Add a bounded handle-local continuation table with version/query/traversal position, proportional work, stale/foreign/evicted results, and close cleanup.
