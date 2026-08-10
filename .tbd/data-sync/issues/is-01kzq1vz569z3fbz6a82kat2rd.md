---
type: is
id: is-01kzq1vz569z3fbz6a82kat2rd
title: "Phase 2: Cache policy axis and lifecycle utilities"
kind: feature
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq1w4rnhr2z0eamhsy19h6m
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-10T23:59:30.469Z
updated_at: 2026-08-10T23:59:36.212Z
---
CachePolicy auto/refresh/only/off in open() (only fails closed with no snapshot); library cache_status/list_caches/clear_cache/clear_all_caches with bounded header reads and never-delete-unrecognized; --cache-status[=root|all] and --cache-clear[=root|all] lifecycle flags through the format axis; Python cache accessors; tryscript coverage per flowmark cache-behavior suite.
