---
type: is
id: is-01kzqn3c33pyf3vh7070ehnfss
title: "P2: cache introspection library functions"
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn3mqbn1wy0ms32gmm4nh6
  - type: blocks
    target: is-01kzqn44q2r4r04yjsweznvyxe
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:35:36.034Z
updated_at: 2026-08-11T16:47:23.546Z
---
New library functions so the CLI invents nothing (Principle 6): cache_status(root) -> CacheStatus { snapshot path, presence, size, entry count, scope, saved-at, engine-fingerprint match }; list_caches() -> Vec<CacheStatus> enumerating the cache directory and reading each snapshot's bounded header to recover its root path - this fixes today's opaque-path-hash problem where a cache file cannot be mapped back to its tree, and unrecognized files are listed as unrecognized rather than hidden; clear_cache(root) and clear_all_caches(), both idempotent, removing only files that parse as fdu snapshot headers and never unrecognized files. Header reads reuse the existing bounded-parse discipline (corrupt equals absent, never an error). Cache-root resolution follows flowmark's pattern: a pure testable function returning a path plus a named source tier (OS cache dir, home fallback, temp fallback). Tests: a corrupt file, a foreign file, and a wrong-fingerprint snapshot all behave correctly under status and clear.
