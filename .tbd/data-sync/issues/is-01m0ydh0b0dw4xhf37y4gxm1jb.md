---
type: is
id: is-01m0ydh0b0dw4xhf37y4gxm1jb
title: "Reconcile PR #44 research with the opened-root rewrite"
kind: task
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T06:53:36.982Z
updated_at: 2026-08-26T07:20:59.819Z
closed_at: 2026-08-26T07:20:59.818Z
close_reason: "Reconciled PR #44 into the current review and active plan at c8acfee; preserved measured research and source findings, rejected wholesale merge/cherry-pick, repaired fdu-u7vo's spec authority, updated draft PR #48, and passed local make check plus all 19 CI checks."
resolution: null
duplicate_of: null
---
Inventory PR #44 as the independent design base beneath abandoned PR #47. Map every unique design decision, measurement, research artifact, review correction, and open bead against PR #48's current durable architecture and implementation plan. Recommend wholesale merge, selective transplant, archival retention, or supersession per artifact, avoiding two active authorities.

## Notes

PR #44 exact head 7f18f20 is the 12-commit docs-only ancestor of PR #47; PR #48 is independent from their shared main base. Decision: do not merge or cherry-pick #44. Preserve its measured 120,001-entry comparison, source checks, two-level extension correction, no-callback rationale, and requirement inventory in the existing PR #47 review; add an artifact disposition map to the active plan; repoint fdu-u7vo from the unmerged old spec to the active opened-root plan. PR #44 may close as superseded after the reconciliation commit is visible, but closure is not performed without explicit user direction.
