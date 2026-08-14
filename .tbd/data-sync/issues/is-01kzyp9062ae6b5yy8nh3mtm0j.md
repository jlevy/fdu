---
type: is
id: is-01kzyp9062ae6b5yy8nh3mtm0j
title: Reconcile completed content spec with missing generic metrics projection CLI
kind: bug
status: open
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-13T23:10:50.049Z
updated_at: 2026-08-14T00:04:19.591Z
---
The completed content spec describes a generic metrics projection and flags such as --metric, --group-by, --content-family, --percent-of, and metric-qualified sort, but ViewSpec exposes only fixed types, families, languages, and documents reports. Reconcile the implementation and spec. A field projection is also needed if a literal bytes-and-files-only text summary is desired without breaking the existing directory count.
