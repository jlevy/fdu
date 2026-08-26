---
type: is
id: is-01m0y1seqtkvcawjhawny979ry
title: Add the no-gap discovery-to-observation handoff
kind: feature
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sf2nph021wtx28p8ahxh
  - type: blocks
    target: is-01m0yhq8268z0qrza1fnwrddfm
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:30.969Z
updated_at: 2026-08-26T19:32:55.748Z
---
Start the supported observer before baseline capture, buffer bounded hints, reconcile overflow and registration gaps, perform required final verification, and enter watching only after freshness is proven. Reuse the scripted backend and deterministic interleaving fixtures; keep watch fully removable.

## Notes

Retain the scripted hint source and named observation boundaries as per-owner test controls over the real verifier; fdu-0kv7 composes the five session goldens after this capability lands.
