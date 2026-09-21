---
type: is
id: is-01m32jv65s9gkk6wsjbc6esaxr
title: "PR #94 review R6: exp-138/140 measured at a5c98d59, not the merged 937f9445 engine"
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-09-19-linux-parallel-validation.md
labels:
  - linux
dependencies: []
parent_id: is-01m32jv4eg76xt21gkexf9a2ns
created_at: 2026-09-21T18:14:54.905Z
updated_at: 2026-09-21T18:25:26.157Z
closed_at: 2026-09-21T18:25:26.157Z
close_reason: "Fixed on #94 in c1ec3342; disposition posted on https://github.com/jlevy/fdu/pull/94#issuecomment-5765440695"
---
https://github.com/jlevy/fdu/pull/94#issuecomment-5765270106 R6 Low. performance-loop-runbook.md:726-729; exp-138/exp-140 verdict.commit a5c98d59. Pick: state in Linux Standing and the spec that exp-138/140 are at a5c98d59 and not re-paired after R1–R3/c441edf6; expected below noise. Do not run a new pair on this layer.
