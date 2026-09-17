---
type: is
id: is-01m2pmr9ytx0ye8d701mr5vp9s
title: "Request model: one typed request with defaults, grammars, and validation"
kind: task
status: open
priority: 0
version: 5
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
updated_at: 2026-09-17T02:57:33.752Z
---
Root, scope, content (analyzer set and options), selection, views and view options; one defaults table; value
grammars; typed validation (analysis-requiring views, ignored-state selection without observation, watch
compatibility, limits); derived choices (default view, views `full` omits); scope and content identities.
The command line and Python construct it; the report reader receives the whole request. Removes separate
parsing in cli.rs and fdu-py, the seven validate_controls sites, surface-only validate_analysis, and the
size and watch-view default mismatches.
