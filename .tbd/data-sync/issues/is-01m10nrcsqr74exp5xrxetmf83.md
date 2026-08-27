---
type: is
id: is-01m10nrcsqr74exp5xrxetmf83
title: Implement bounded handle-local continuation authority
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nrd50t3xfevcxx7j98x5h
parent_id: is-01m0y1shykye8sc7h7e9rkk6kh
created_at: 2026-08-27T03:55:56.598Z
updated_at: 2026-08-27T03:55:56.959Z
---
Implement opened/continuation.rs create/resume/evict/clear over version, normalized fdu-native query identity, and structural traversal position. Public IDs remain opaque; enforce table/record bounds, stale/query-mismatch/foreign/evicted/closed outcomes, proportional resume work, and close cleanup without historical images or signing.
