---
type: is
id: is-01m32v0h01hfevps9e2zgtzcjb
title: "PR #103 review R4: unfiltered flat reads pay a measurement pass and duplicate every row (Low)"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6ewht09ryvnt0f5zmhz5
created_at: 2026-09-21T20:37:38.433Z
updated_at: 2026-09-21T21:11:49.226Z
closed_at: 2026-09-21T21:11:49.226Z
close_reason: "Deferred: needs a maintained-roll-up path or a perf-loop measurement, tracked as fdu-uea9 under the epic."
resolution: null
duplicate_of: null
---
R4 Low. Query::needs_selection_walk() ~query_report.rs:497 is true for List/Files in any flat format even unfiltered, so fdu PATH --format json runs measure() and walk() with row clones. Fix: skip the measurement pass when no selection predicate is present, or record a measurement showing the cost is acceptable.
