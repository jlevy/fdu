---
type: is
id: is-01m4xmy4txa7se60qzydepebgp
title: Clarify API names and separate lifecycle from materialization costs
kind: task
status: in_progress
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-14T16:46:40.476Z
updated_at: 2026-10-14T16:49:47.683Z
started_at: 2026-10-14T16:46:52.679Z
---

## Notes

User accepted three user-facing groups: Report API, Index API, Progressive API. Explain near top of public README, Python guide, architecture and review. Separate lifecycle/ownership from result detail/materialization and avoid universal speed rankings. Progressive supports offline/batch use; for a single final unfiltered metadata summary the one-shot Report planner can use a compact native reducer, while existing appropriate handles should be reused. Preserve current copying, GIL, partial-coverage and cursor constraints. Sol handles mechanics; parent verifies technical accuracy.
