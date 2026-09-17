---
type: is
id: is-01m2pmrbxvnerxyhnjhrswy1ye
title: "Execution plan model: one planner and one write rule for every route"
kind: task
status: open
priority: 0
version: 5
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - design
  - cache
dependencies:
  - type: blocks
    target: is-01m2pmrcb8he4a8a54zt957vcs
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
created_at: 2026-09-17T02:57:26.458Z
updated_at: 2026-09-17T03:38:57.681Z
---
Given request, delivery, and available stored state: tiers needed, stored entries that serve (through the
stored-state model), what is verified, computed, written, and the resulting provenance. One-shot reports, open,
Python Index refresh and watch, CLI watch, and opened roots all use it. Alternative plans differ in cost, never
answer. Write rule: write only tiers verified completely, keyed by identity, never replacing another identity's
entry with narrower data.

## Notes

2026-09-17 (PR #78 review): Phase 2. Typed Delivery {cache, workers, accept_partial, watch} is this model's second input; format and colour belong to the answer model. The harness iterates deliveries through the type.
