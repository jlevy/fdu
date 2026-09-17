---
type: is
id: is-01m2pmr9ytx0ye8d701mr5vp9s
title: "Request model: one typed request with defaults, grammars, and validation"
kind: task
status: open
priority: 0
version: 6
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - design
dependencies:
  - type: blocks
    target: is-01m2pmrbxvnerxyhnjhrswy1ye
  - type: blocks
    target: is-01m2pmrcrmjrm62bm1x3mxwgvm
  - type: blocks
    target: is-01m2phzn814exmf4ty5vw6zha0
  - type: blocks
    target: is-01m2pj0f459s8ad1efzyn2qmbq
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
created_at: 2026-09-17T02:57:24.441Z
updated_at: 2026-09-17T03:38:53.327Z
---
Root, scope, content (analyzer set and options), selection, views and view options; one defaults table; value
grammars; typed validation (analysis-requiring views, ignored-state selection without observation, watch
compatibility, limits); derived choices (default view, views `full` omits); scope and content identities.
The command line and Python construct it; the report reader receives the whole request. Removes separate
parsing in cli.rs and fdu-py, the seven validate_controls sites, surface-only validate_analysis, and the
size and watch-view default mismatches.

## Notes

2026-09-17 (PR #78 review): Phase 1 item 3. Compose Request {scope, content, selection, views, now} from existing typed parts; (scope, content) held by retained indexes and opened roots, (selection, views, now) per read; one constructor and validate; defaults: allocated sizes, tree view for watch on every surface; opened reads and Python Index validate through it; refuse --watch --cache only; Rust Session and Python Index.watch refuse an analyzed index (CLI already refuses --watch --analyze).
