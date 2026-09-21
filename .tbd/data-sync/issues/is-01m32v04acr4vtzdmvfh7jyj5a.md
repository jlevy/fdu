---
type: is
id: is-01m32v04acr4vtzdmvfh7jyj5a
title: "PR #96 review R2: one measured value, two mechanisms, no cost bound (High)"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6cjrkz58ce2t05m3n2v7
created_at: 2026-09-21T20:37:25.452Z
updated_at: 2026-09-21T21:11:41.128Z
closed_at: 2026-09-21T21:11:41.127Z
close_reason: "Fixed in c3aeed8a on codex/directory-query-plan (R1 implementation in 5d6e56a2 on #103). Disposition: https://github.com/jlevy/fdu/pull/96"
resolution: null
duplicate_of: null
---
R2 High. plan :514-516 and :523-529 name both query_subtrees::measure and query_report::walk as computing directory metrics. Fix: name one owner, state the other as consumer, and bound measurement cost to one pass over the retained index per report.
