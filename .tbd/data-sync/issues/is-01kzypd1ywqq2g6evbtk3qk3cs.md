---
type: is
id: is-01kzypd1ywqq2g6evbtk3qk3cs
title: Use precomputed content rollups for unfiltered metric summaries
kind: bug
status: open
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-13T23:13:02.939Z
updated_at: 2026-08-14T00:04:19.866Z
---
query::metric_summary traverses every selected file even when the query is unfiltered. Current ContentRollUp values do not retain all grouped, detection, and flag projections needed by the report. Extend rollups and make unfiltered metric summaries consume them.
