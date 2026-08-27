---
type: is
id: is-01m10nrcsqr74exp5xrxetmf83
title: Implement bounded handle-local continuation authority
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nrd50t3xfevcxx7j98x5h
parent_id: is-01m0y1shykye8sc7h7e9rkk6kh
created_at: 2026-08-27T03:55:56.598Z
updated_at: 2026-08-27T09:38:03.564Z
---
Implement opened/continuation.rs create/resume/evict/clear over version, normalized fdu-native query identity, and structural traversal position. Public IDs remain opaque; enforce table/record bounds, stale/query-mismatch/foreign/evicted/closed outcomes, proportional resume work, and close cleanup without historical images or signing.

## Notes

Continuation authority from cc60adb is being audited as retained Phase 2 work rather than rewritten: it already supplies handle-local opaque IDs, a 128-record FIFO cap, version pinning, structural tree/flat resume positions, single-use semantics, underfunded retry, foreign/evicted unavailable results, proportional resumed work, and close cleanup. Remaining hardening is limited to proving/bounding retained query-record size and completing exact acceptance evidence; no new facade or stateless token format.
