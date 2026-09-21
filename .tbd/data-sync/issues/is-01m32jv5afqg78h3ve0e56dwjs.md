---
type: is
id: is-01m32jv5afqg78h3ve0e56dwjs
title: "PR #94 review R3: runbook still describes experiment-id reservation as pending"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-19-linux-parallel-validation.md
labels:
  - linux
dependencies: []
parent_id: is-01m32jv4eg76xt21gkexf9a2ns
created_at: 2026-09-21T18:14:54.031Z
updated_at: 2026-09-21T18:25:26.144Z
closed_at: 2026-09-21T18:25:26.144Z
close_reason: "Fixed on #94 in c1ec3342; disposition posted on https://github.com/jlevy/fdu/pull/94#issuecomment-5765440695"
---
https://github.com/jlevy/fdu/pull/94#issuecomment-5765270106 R3 Low. performance-loop-runbook.md:499-500. Reservation is assigned (exp-138–143 minted). Fix: Next free experiment id is exp-144 (numbering is shared with Linux); do not take exp-138–143 here.
