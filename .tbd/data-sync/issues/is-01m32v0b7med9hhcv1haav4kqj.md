---
type: is
id: is-01m32v0b7med9hhcv1haav4kqj
title: "PR #96 review R13: subtree counts and unknown age must be absent, not zero (Low)"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6cjrkz58ce2t05m3n2v7
created_at: 2026-09-21T20:37:32.532Z
updated_at: 2026-09-21T21:11:46.363Z
closed_at: 2026-09-21T21:11:46.363Z
close_reason: "Fixed in c3aeed8a on codex/directory-query-plan (R1 implementation in 5d6e56a2 on #103). Disposition: https://github.com/jlevy/fdu/pull/96"
resolution: null
duplicate_of: null
---
R13 Low. plan :107-108, :221, :558. Fix: regular-file rows omit subtree counts (null in machine formats, blank in long); unknown age is null.
