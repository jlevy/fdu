---
type: is
id: is-01m32v067d5n4kfxdvfjpk35k0
title: "PR #96 review R5: paths cites path-escaping rules the project does not have (High)"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6cjrkz58ce2t05m3n2v7
created_at: 2026-09-21T20:37:27.405Z
updated_at: 2026-09-21T21:11:42.495Z
closed_at: 2026-09-21T21:11:42.495Z
close_reason: "Fixed in c3aeed8a on codex/directory-query-plan (R1 implementation in 5d6e56a2 on #103). Disposition: https://github.com/jlevy/fdu/pull/96"
resolution: null
duplicate_of: null
---
R5 High. plan :104-106, :553. Fix: define the byte contract of --format paths (lossy vs NUL/print0), acceptance cases for newline and undecodable bytes under fdu-ywh0.
