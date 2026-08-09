---
type: is
id: is-01kzg4bey8nn4k8y1daxc9exhd
title: CLI human polish is product work, not cosmetics
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:28:14.279Z
updated_at: 2026-08-09T18:59:17.685Z
---
Goal 2 deviation guard. Earlier drafts described the CLI as 'for testing, scripting, and agent use', which contradicted Goal 2. It is a first-class product surface and must be scheduled in phase 1 alongside the engine, not deferred as cosmetics.

Present: colored tree with percentage bars, --by-type, NO_COLOR, pipe detection, broken-pipe handled as success.
Still needed: terminal-width awareness (bars and columns should adapt, currently fixed width), gitignore-aware display once tagging lands, sensible depth/top-N defaults validated against real trees, --sort by the metrics the reducer registry exposes, and a review of the human output against dust/dut for readability.

## Notes

The 2026-08-09 Rust guideline audit found recursive human tree rendering. Complete fdu-zsdy first so terminal-width and product-polish work builds on an iterative traversal that cannot exhaust the process stack on a deep retained tree.
