---
type: is
id: is-01m0y1sbmnpxmcyt3rvm7qt1rg
title: Introduce the exact prepared commit pipeline
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m0y1sc1h17y99grptjb9pzha
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-08-26T03:28:27.796Z
updated_at: 2026-08-26T03:28:54.412Z
---
Add normalized prepared inputs and one atomic Commit containing exact effective changes, impact, state, and work. Replace boolean mutation reporting inside Index, make clocks advance only with a complete commit, and prove fault atomicity, no-op behavior, kind replacement, implicit ancestry, refusal, control, and state-only transitions.
