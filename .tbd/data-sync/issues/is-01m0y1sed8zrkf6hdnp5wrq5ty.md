---
type: is
id: is-01m0y1sed8zrkf6hdnp5wrq5ty
title: Add bounded verified multi-path refresh
kind: feature
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1seqtkvcawjhawny979ry
  - type: blocks
    target: is-01m0y1sf2nph021wtx28p8ahxh
  - type: blocks
    target: is-01m0yhq8268z0qrza1fnwrddfm
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:30.631Z
updated_at: 2026-08-26T08:07:19.269Z
---
Implement refresh(paths) validation, canonicalization, deduplication, ancestor collapse, widening, I/O outside the write guard, conditional exact commits, typed accepted/rejected paths, journal range, state, and work. Reuse the sound algorithms and fixtures from PR #47 d19b0ce.

## Notes

Add the named after-verification/before-conditional-commit barrier through the per-owner test control while implementing real refresh; fdu-0kv7 owns final composition.
