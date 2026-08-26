---
type: is
id: is-01m0y1sd4tmt95d9tdsynn7v6g
title: Add the shared OpenedIndex owner and joined lifecycle
kind: feature
status: open
priority: 1
version: 10
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
  - type: blocks
    target: is-01m0yhq8268z0qrza1fnwrddfm
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:29.336Z
updated_at: 2026-08-26T08:07:18.129Z
---
Add the synchronous OpenedIndex as the direct public API over one shared internal owner. Use the associated OpenedIndex::open constructor so the existing free one-shot open remains source-compatible; do not introduce a facade, service mirror, or second source of truth. Implement shared clone identity, lifecycle state, idempotent concurrent close with one stored terminal outcome, cancellation, joined worker ownership, last-reference defensive shutdown, and typed closed errors without an async runtime. Workers hold Weak owner references or narrower state so they cannot form a final-reference cycle.

## Notes

When implementing the owner, add only the per-owner typed test-control container and lifecycle/barrier seams needed by fdu-0kv7; no fake operations, facts, commits, or results.
