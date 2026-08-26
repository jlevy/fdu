---
type: is
id: is-01m0y1sbmnpxmcyt3rvm7qt1rg
title: Introduce the exact prepared commit pipeline
kind: feature
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sc1h17y99grptjb9pzha
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:27.796Z
updated_at: 2026-08-26T09:24:27.043Z
closed_at: 2026-08-26T09:24:27.042Z
close_reason: Exact prepared commit kernel implemented and verified
resolution: null
duplicate_of: null
---
Add normalized prepared inputs and one atomic Commit containing exact effective changes, impact, state, and work. Replace boolean mutation reporting inside Index, make clocks advance only with a complete commit, and prove fault atomicity, no-op behavior, kind replacement, implicit ancestry, refusal, control, and state-only transitions.

## Notes

Implemented normalized prepared observations and one exact Commit journal. Exact changes record ancestry, replacements, cascaded removals, invalidation, and reconciliation state; impact and work are derived centrally; legacy AppliedDelta and ApplyOutcome fields remain derived compatibility projections. The independent reference model compares exact commits and the full journal. Resource refusal, control changes, and unknown-live-ancestry policy remain in their explicitly owning downstream beads. CARGO_INCREMENTAL=0 make check and make cross-lint pass.
