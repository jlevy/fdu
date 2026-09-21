---
type: is
id: is-01m32jvx0sev144k9re6crv69p
title: "PR #94 review S2: add a 'determination' decision value to the experiment contract"
kind: feature
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m32h6bvgxbks1d5t5q5p0k44
created_at: 2026-09-21T18:15:18.296Z
updated_at: 2026-09-21T18:15:18.296Z
---
Senior review of PR #94, suggestion S2 (non-blocking, separate PR).

Same-binary leftover profiles (exp-118, 122, 125-136, 139, 142, 143) render as 'accepted +0.6%', which reads as a shipped speedup. timeline.py:113 already calls a contract-level decision value 'the durable fix'. A 'determination' decision would also make the hand-maintained CLAIM_ONLY_EXPERIMENTS set unnecessary (see R1 on #94, where exp-141 had to be added by hand).

Out of scope for #94.
