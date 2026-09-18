---
type: is
id: is-01m2rt8sa4ha4zxd3s7tf3p8xh
title: Engine architecture Known Gaps still say report takes no analysis request
kind: task
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m2rss53ech1v9fvh9hdd4cgg
created_at: 2026-09-17T23:12:16.196Z
updated_at: 2026-09-18T00:55:01.964Z
closed_at: 2026-09-18T00:55:01.964Z
close_reason: "Rewrote engine Known Gaps: report/report_in take &Request and use request.basis.content after validate_read; live-path refusals are documented as current serving-lifecycle behavior, not a gap."
---
docs/project/architecture/fdu-engine-architecture.md Known Gaps still claims query::report receives an index, a query, and provenance but no analysis request. After core-models-4, report takes &Request and uses request.basis.content after validate_read. The Core Models table in the same file already says Yes. Rewrite the stale gap; the live-paths bullet is now current behavior, not a gap.
