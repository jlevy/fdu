---
type: is
id: is-01m3gafb3hsvs7vejhkc483wc2
title: Consolidate tests and close meaningful coverage gaps from the audit
kind: task
status: in_progress
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
delegate: sol-code-metrics
labels: []
dependencies:
  - type: blocks
    target: is-01m3gafrkpczr2mra09xfvpzge
parent_id: is-01m3g53g8b7wg3qk2m70j8h0bp
hold: null
hold_until: null
created_at: 2026-09-27T02:18:00.172Z
updated_at: 2026-09-27T02:29:33.934Z
started_at: 2026-09-27T02:29:33.933Z
---
Implement evidence-backed testing-review findings. Consolidate redundant setup/assertions and move public CLI contracts into concise readable tryscript sessions when coverage remains equivalent. Keep fast focused lexer/chunk, arithmetic, allocation, concurrency and failure-injection invariants where goldens cannot replace them. Close concrete coverage gaps with minimal reusable fixtures. No blanket conversions, weakened assertions, catch-all elisions, machine-specific recordings, or test-count/coverage-percentage targets.
