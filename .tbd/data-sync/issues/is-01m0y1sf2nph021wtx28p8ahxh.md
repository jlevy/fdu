---
type: is
id: is-01m0y1sf2nph021wtx28p8ahxh
title: Expose the five synchronous OpenedIndex operations in Python
kind: feature
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sfedr9qf3sc7e4bf6fd7
  - type: blocks
    target: is-01m0y1sjbfs5h264xhme2vqymg
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:31.316Z
updated_at: 2026-08-26T03:28:58.855Z
---
Add PyOpenedIndex and immutable Python models for open, read, changes, refresh, prioritize, and close. Release the GIL around blocking or substantial native work, preserve shared close semantics, avoid a package-owned async executor, update stubs and typing, and prove overlap and shutdown.
