---
type: is
id: is-01m0y1sd4tmt95d9tdsynn7v6g
title: Add the shared OpenedIndex owner and joined lifecycle
kind: feature
status: open
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sdf88k6zme7tb9hbkjrd
  - type: blocks
    target: is-01m0y1sdsamq2hrrexxcvx98g3
  - type: blocks
    target: is-01m0y1se38tcc11akkz34mjrme
  - type: blocks
    target: is-01m0y1sed8zrkf6hdnp5wrq5ty
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:29.336Z
updated_at: 2026-08-26T03:42:35.609Z
---
Add the synchronous OpenedIndex facade and one shared internal owner. Use the associated OpenedIndex::open constructor so the existing free one-shot open remains source-compatible. Implement shared clone identity, lifecycle state, idempotent concurrent close with one stored terminal outcome, cancellation, joined worker ownership, last-reference defensive shutdown, and typed closed errors without an async runtime. Workers hold Weak owner references or narrower state so they cannot form a final-reference cycle.
