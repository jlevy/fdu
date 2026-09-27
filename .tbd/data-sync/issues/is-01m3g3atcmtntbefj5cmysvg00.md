---
type: is
id: is-01m3g3atcmtntbefj5cmysvg00
title: Design adaptive tree display with composable depth share breadth and row limits
kind: feature
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-09-26-presentation-design.md
labels: []
dependencies: []
created_at: 2026-09-27T00:13:11.955Z
updated_at: 2026-09-27T00:13:11.955Z
---
Implement engine-owned display bounds. Proposed tree defaults: depth 5, minimum 1% of fixed selected root size, unlimited breadth and total rows, with all controls overridable. Use breadth for per-directory caps and limit for per-section data rows across views. Include significant regular-file leaves, preserve aggregate totals and scan scope, and expose typed omission reasons. Validate eleven 2% siblings, depth boundaries, exact threshold equality, zero/unknown measures, root-versus-parent denominators, and cross-surface parity. Owner waives alpha compatibility.
