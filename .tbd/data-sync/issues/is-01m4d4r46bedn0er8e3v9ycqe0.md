---
type: is
id: is-01m4d4r46bedn0er8e3v9ycqe0
title: "0.4.0 release standing: paired non-regression of the report path against v0.3.0"
kind: task
status: open
priority: 2
version: 1
labels:
  - release
  - performance
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
created_at: 2026-10-08T06:55:55.082Z
updated_at: 2026-10-08T06:55:55.082Z
---
0.4.0 changes report construction (query_report.rs +588, query_request.rs +933 lines since v0.3.0) and makes content views imply analysis (--view documents now runs words), and none of #174/#177/#179 timed it. Walker/scan code is unchanged. 0.3.0 had a release standing cell (exp-202). Before tagging 0.4.0, run a paired non-regression screen of the release candidate against v0.3.0 on default-tree, aggregate-summary, and the code/documents views (cold and warm content cache) on a nominated real tree, and record it. Relates to fdu-mvnp (content-query re-baseline).
