---
type: is
id: is-01m2pyeb3yz78d3d94dv78dv4g
title: "P2.2.6: Change records and cache status through walks (fdu.stream/2, fdu.cache/2)"
kind: task
status: open
priority: 0
version: 2
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pyebe93530evdeaw8tcxh6
parent_id: is-01m2pj0f459s8ad1efzyn2qmbq
created_at: 2026-09-17T05:46:43.709Z
updated_at: 2026-09-17T05:46:57.981Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 2: The Answer Model and Writers", commit 6. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `report_format.rs`: `emit_change` and `emit_cache_status`; `render_change` (`:1507-1549`) and `render_cache_status` (`:1610-1714`) go through the walks, with YAML change records as `---` documents; `report_schema` and schema constants (`:61`, `:63`, `:70`, `:1324-1332`, `:1481`) for `fdu.stream/2` and `fdu.cache/2`.

**Tests**

- Rewrite the stream-record (`:2708`) and cache-schema (`:2018`) tests.
- Goldens: `cli-watch`, `cli-lifecycle`, `cli-cache`.

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.
