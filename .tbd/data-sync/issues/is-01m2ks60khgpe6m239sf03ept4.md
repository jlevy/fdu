---
type: is
id: is-01m2ks60khgpe6m239sf03ept4
title: "PR #65 review F2: query::report answers an ignored-state selection over a non-observing index with zero rows"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m2ks5fy0rg2stv9nf1vc7wzb
created_at: 2026-09-16T00:17:04.620Z
updated_at: 2026-09-16T00:17:04.620Z
---
PR #65. crates/fdu-core/src/query/query_report.rs:933-935,:1107-1112. The public report() is infallible and walk returns an empty Walked instead of refusing. Every other entry point validates. Return Result<Report> and refuse with ControlStateNotObserved; fix the test an_index_that_observed_no_control_state_has_no_ignored_share_to_select_by.
