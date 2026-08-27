---
type: is
id: is-01m0y1sg8emg0sgyv1pj8sa6x7
title: Update the MetaBrowser Python reference provider
kind: feature
status: closed
priority: 1
version: 10
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sgqd1sd33stssgw25f2q
  - type: blocks
    target: is-01m0y1sjnptgqhgvqcx1cjkkhw
  - type: blocks
    target: is-01m10nrazfqj0ndxdpvv94kprg
  - type: blocks
    target: is-01m10nsf27ydw4sb116neghkg1
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
child_order_hints:
  - is-01m10nr9wf12chcxvgv2qjs4qr
  - is-01m10nra8p8h2pcb121nrqfe7c
  - is-01m10nramr8mgkw9trsfh345x0
created_at: 2026-08-26T03:28:32.526Z
updated_at: 2026-08-27T07:56:14.872Z
closed_at: 2026-08-27T07:56:14.872Z
close_reason: "Completed in MetaBrowser commit 45266a8 after its configuration checkpoint: the Python reference provider implements the revised contract and passes the complete provider and product gate."
resolution: null
duplicate_of: null
---
Make PythonInventoryBackend consume the injected registry and revised contract, enforce one budget policy across discovery and live operations, derive identities, return bounded opaque pages and honest totals, preserve provider readability, and pass the revised conformance registry before fdu is added.
