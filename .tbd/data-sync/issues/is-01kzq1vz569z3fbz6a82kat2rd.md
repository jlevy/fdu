---
type: is
id: is-01kzq1vz569z3fbz6a82kat2rd
title: "Phase 2: Cache policy axis and lifecycle utilities"
kind: feature
status: open
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzq1w4rnhr2z0eamhsy19h6m
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
child_order_hints:
  - is-01kzs53e2agcaeebb7nypyp77g
created_at: 2026-08-10T23:59:30.469Z
updated_at: 2026-08-11T19:37:38.159Z
---
CachePolicy auto/refresh/only/off in open() (only fails closed with no snapshot); library cache_status/list_caches/clear_cache/clear_all_caches with bounded header reads and never-delete-unrecognized; --cache-status[=root|all] and --cache-clear[=root|all] lifecycle flags through the format axis; Python cache accessors; tryscript coverage per flowmark cache-behavior suite.

## Notes

Cache-write design (updated 2026-08-10): core snapshot ALWAYS written on complete+Fresh under auto/refresh, on a BACKGROUND THREAD overlapped with rendering (index is read-only once producers finish; two concurrent readers), joined before exit, completing even on broken-pipe rendering; soft-fail stderr warning. Disable via policy value, not a new flag: --cache read-only (read+revalidate, never write) added as fifth CachePolicy value alongside auto/refresh/only/off. 'Skip write for stat-only runs' rejected per research-2026-08-10-performance-frontier.md (cloud: snapshot is only warm path; macOS: FSEvents resume anchor — reserve event ID + volume UUID + platform tag fields). Content tier goes in separate derived-data layer keyed (fingerprint, analyzer id, analyzer version). Verification cost follows the query's reducer tiers. Retention/GC is spec Open Question 5.
