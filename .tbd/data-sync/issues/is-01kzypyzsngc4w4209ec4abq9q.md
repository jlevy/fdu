---
type: is
id: is-01kzypyzsngc4w4209ec4abq9q
title: Preserve requested content provenance for empty analyzed trees
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-13T23:22:50.543Z
updated_at: 2026-08-14T00:03:56.024Z
closed_at: 2026-08-14T00:03:56.024Z
close_reason: Implemented in c2b646c; full make check and all 16 required PR checks pass.
---
An enabled analysis request on a tree with zero regular files currently emits fdu.report/1 and no analysis metadata. Preserve profile/provenance and require a usable empty sidecar for cache-only analysis.
