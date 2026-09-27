---
type: is
id: is-01m3g3atcmtntbefj5cmysvg00
title: Design adaptive tree display with composable depth share breadth and row limits
kind: feature
status: in_progress
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-09-26-code-analysis-presentation.md
delegate: sol-presentation
labels: []
dependencies:
  - type: blocks
    target: is-01m3g53hk9nr8ff52d3xj77ttj
parent_id: is-01m3g53g8b7wg3qk2m70j8h0bp
hold: null
hold_until: null
created_at: 2026-09-27T00:13:11.955Z
updated_at: 2026-09-27T01:27:24.073Z
started_at: 2026-09-27T01:27:24.072Z
---
Implement engine-owned display bounds. Proposed tree defaults: depth 5, minimum 1% of fixed selected root size, unlimited breadth and total rows, with all controls overridable. Use breadth for per-directory caps and limit for per-section data rows across views. Include significant regular-file leaves, preserve aggregate totals and scan scope, and expose typed omission reasons. Validate eleven 2% siblings, depth boundaries, exact threshold equality, zero/unknown measures, root-versus-parent denominators, and cross-surface parity. Owner waives alpha compatibility.
