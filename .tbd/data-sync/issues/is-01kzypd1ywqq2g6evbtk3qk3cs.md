---
type: is
id: is-01kzypd1ywqq2g6evbtk3qk3cs
title: Use precomputed content rollups for unfiltered metric summaries
kind: feature
status: open
priority: 2
version: 5
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-13T23:13:02.939Z
updated_at: 2026-09-30T06:25:27.490Z
---
query::metric_summary traverses every selected file even when the query is unfiltered. Current ContentRollUp values do not retain all grouped, detection, and flag projections needed by the report. Extend rollups and make unfiltered metric summaries consume them.

## Notes

2026-09-30 stability triage (epic fdu-l4u1): not a defect in shipped behaviour but a design, feature or measurement task, so relabelled from bug. It is not a 0.3.0 release blocker.
