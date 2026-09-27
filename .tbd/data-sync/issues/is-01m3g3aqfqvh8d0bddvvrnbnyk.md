---
type: is
id: is-01m3g3aqfqvh8d0bddvvrnbnyk
title: Apply shared presentation roles and consistent annotations across reports
kind: feature
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
delegate: sol-presentation
labels: []
dependencies:
  - type: blocks
    target: is-01m3g53hk9nr8ff52d3xj77ttj
parent_id: is-01m3g53g8b7wg3qk2m70j8h0bp
hold: null
hold_until: null
created_at: 2026-09-27T00:13:08.982Z
updated_at: 2026-09-27T03:12:09.140Z
started_at: 2026-09-27T01:19:37.942Z
closed_at: 2026-09-27T03:12:09.140Z
close_reason: Implemented across engine, CLI, Python, schemas and docs in PR133.183 shared CLI goldens,886 core tests,68 cross-format cases, full16,787-case path matrix, cache fault proofs, and Apple/Windows cross-lint pass. Final packaging/CI handoff remains tracked by fdu-7jtp.
resolution: null
duplicate_of: null
---
Implement the shared presentation contract: normal primary counts outside parentheses, cyan human paths, gray parenthetical ignored and metric details, shared integer grouping, explicit percentage denominators, and span-safe width/reset behavior. Preserve intentional path/machine format contracts. Cover every human report view with targeted ANSI span checks and reviewed goldens.
