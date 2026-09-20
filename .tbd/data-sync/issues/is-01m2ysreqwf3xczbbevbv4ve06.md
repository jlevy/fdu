---
type: is
id: is-01m2ysreqwf3xczbbevbv4ve06
title: Terminal multi-path reconciliation clears unvisited issues
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T06:58:47.666Z
updated_at: 2026-09-20T07:05:19.406Z
closed_at: 2026-09-20T07:05:19.403Z
close_reason: Fixed by distinguishing visited evidence from incomplete closure
resolution: null
duplicate_of: null
---
A multi-path reconciliation begins every subtree before walking. If an earlier subtree returns a terminal error, later subtrees are closed without a walk; unconditional partial-pass cleanup drops their older scoped issues despite no disproving observation. Carry visited evidence into closure and retain issues for skipped subtrees.

## Notes

Implemented an explicit disproves_old closure fact. Successful walks, including partial scan reports with precise errors and resource-refused application, may replace older issues. Terminal errors and later multi-path scopes skipped after an earlier terminal error close without clearing their prior issues. Added direct closure regression for an unvisited aborted scope.
