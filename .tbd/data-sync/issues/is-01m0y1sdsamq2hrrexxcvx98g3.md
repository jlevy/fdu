---
type: is
id: is-01m0y1sdsamq2hrrexxcvx98g3
title: Add coherent bounded reads and maintained projections
kind: feature
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sf2nph021wtx28p8ahxh
  - type: blocks
    target: is-01m0yhq8268z0qrza1fnwrddfm
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:29.993Z
updated_at: 2026-08-26T08:07:18.733Z
---
Implement ReadRequest and ReadResponse under one version/state boundary with lookup, depth-one tree, flat rows, roll-up/report, diagnostics, exact-or-capped counts, work accounting, portable-path issues, and the minimal commit-maintained indexes justified by the current contract.

## Notes

Expose complete production read values to the shared scenario envelope and add no alternate renderer-side result model; fdu-0kv7 composes all projection scenarios after Phase 2.
