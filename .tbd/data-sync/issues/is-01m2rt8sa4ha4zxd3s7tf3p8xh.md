---
type: is
id: is-01m2rt8sa4ha4zxd3s7tf3p8xh
title: Engine architecture Known Gaps still say report takes no analysis request
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m2rss53ech1v9fvh9hdd4cgg
created_at: 2026-09-17T23:12:16.196Z
updated_at: 2026-09-17T23:12:16.196Z
---
docs/project/architecture/fdu-engine-architecture.md Known Gaps still claims query::report receives an index, a query, and provenance but no analysis request. After core-models-4, report takes &Request and uses request.basis.content after validate_read. The Core Models table in the same file already says Yes. Rewrite the stale gap; the live-paths bullet is now current behavior, not a gap.
