---
type: is
id: is-01m0y1sdf88k6zme7tb9hbkjrd
title: Add progressive parent-first discovery, budget, and priority
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1seqtkvcawjhawny979ry
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:29.671Z
updated_at: 2026-08-26T03:28:56.760Z
---
Make scan feed bounded parent-first commits to the OpenedIndex owner, record per-directory completeness, expose honest resource-budget partial state, refuse expansion after a stop, and add best-effort scheduling-only prioritize paths. Preserve one-shot scan behavior.
