---
type: is
id: is-01m3g3aqfqvh8d0bddvvrnbnyk
title: Apply shared presentation roles and consistent annotations across reports
kind: feature
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m3g53hk9nr8ff52d3xj77ttj
parent_id: is-01m3g53g8b7wg3qk2m70j8h0bp
created_at: 2026-09-27T00:13:08.982Z
updated_at: 2026-09-27T00:50:04.614Z
---
Implement the shared presentation contract: normal primary counts outside parentheses, cyan human paths, gray parenthetical ignored and metric details, shared integer grouping, explicit percentage denominators, and span-safe width/reset behavior. Preserve intentional path/machine format contracts. Cover every human report view with targeted ANSI span checks and reviewed goldens.
