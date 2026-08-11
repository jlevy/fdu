---
type: is
id: is-01kzq1vz569z3fbz6a82kat2rd
title: "Phase 2: Cache policy axis and lifecycle utilities"
kind: feature
status: open
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq1w4rnhr2z0eamhsy19h6m
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-10T23:59:30.469Z
updated_at: 2026-08-11T00:50:09.268Z
---
CachePolicy auto/refresh/only/off in open() (only fails closed with no snapshot); library cache_status/list_caches/clear_cache/clear_all_caches with bounded header reads and never-delete-unrecognized; --cache-status[=root|all] and --cache-clear[=root|all] lifecycle flags through the format axis; Python cache accessors; tryscript coverage per flowmark cache-behavior suite.

## Notes

Cache-write design decision (2026-08-10, from research-2026-08-10-performance-frontier.md): core snapshot is ALWAYS written on complete+Fresh under auto/refresh, on every platform and query tier — write is tens of ms, moved AFTER report flush (never user-visible latency), soft-fail to stderr warning. 'Skip write for stat-only runs' explicitly rejected: stat-tier snapshot is the only warm path on cloud and the FSEvents journal-resume anchor on macOS (reserve event ID + volume UUID + platform tag fields in format). Content-tier results go in a SEPARATE derived-data layer keyed (fingerprint, analyzer id, analyzer version): additive, lazy-loaded, per-analyzer invalidation, size-bounded, purgeable — where cache pays most. Under auto, verification cost follows the query's reducer tiers (name→D stats, stat→N stats, content→N stats + changed reads), exact and unlabeled; tiered verification lands with reducer registry (fdu-a6dz). Retention/GC is spec Open Question 5.
