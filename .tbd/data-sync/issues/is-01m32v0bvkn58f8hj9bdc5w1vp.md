---
type: is
id: is-01m32v0bvkn58f8hj9bdc5w1vp
title: "PR #96 review R14: tie-breaking specified for one sort key; --sort count undefined for flat rows (Low)"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6cjrkz58ce2t05m3n2v7
created_at: 2026-09-21T20:37:33.171Z
updated_at: 2026-09-21T21:11:46.841Z
closed_at: 2026-09-21T21:11:46.840Z
close_reason: "Fixed in c3aeed8a on codex/directory-query-plan (R1 implementation in 5d6e56a2 on #103). Disposition: https://github.com/jlevy/fdu/pull/96"
resolution: null
duplicate_of: null
---
R14 Low. plan :177, :179. Fix: every sort key breaks ties on relative path in one direction; define --sort count for flat rows.
