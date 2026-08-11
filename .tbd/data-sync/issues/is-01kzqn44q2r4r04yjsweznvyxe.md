---
type: is
id: is-01kzqn44q2r4r04yjsweznvyxe
title: "P2: Python cache accessors"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:36:01.250Z
updated_at: 2026-08-11T05:36:01.250Z
---
fdu-py: expose cache_status(root), list_caches(), clear_cache(root), clear_all_caches() mirroring the library functions with the same names and return shapes, plus the cache= keyword on open() taking the same policy strings as --cache (auto|refresh|read-only|only|off). Parity requirement from Principle 6: a capability reachable from the CLI must be reachable as one typed call from Python. Update the installed-wheel smoke to open with a policy and read back a cache status.
