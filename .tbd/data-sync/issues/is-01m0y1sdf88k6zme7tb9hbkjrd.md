---
type: is
id: is-01m0y1sdf88k6zme7tb9hbkjrd
title: Add progressive parent-first discovery, budget, and priority
kind: feature
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
delegate: codex@spud10.local
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1seqtkvcawjhawny979ry
  - type: blocks
    target: is-01m0yhq8268z0qrza1fnwrddfm
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
hold: null
hold_until: null
created_at: 2026-08-26T03:28:29.671Z
updated_at: 2026-08-26T12:05:27.834Z
started_at: 2026-08-26T12:05:27.833Z
---
Make scan feed bounded parent-first commits to the OpenedIndex owner, record per-directory completeness, expose honest resource-budget partial state, refuse expansion after a stop, and add best-effort scheduling-only prioritize paths. Preserve one-shot scan behavior.

## Notes

Extend the per-owner test control with deterministic worker count, discovery order, priority, budget, and named producer barriers while implementing the real discovery path; fdu-0kv7 composes the scenario later.
