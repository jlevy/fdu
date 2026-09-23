---
type: is
id: is-01m35t5dddf8v2xh1vpkk59tm3
title: "PR #105 R3: label each restore-share timer definition"
kind: task
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m32h6g0hprgkrq1m6e8rx0f6
created_at: 2026-09-23T00:20:33.322Z
updated_at: 2026-09-23T08:07:04.813Z
closed_at: 2026-09-23T08:07:04.809Z
close_reason: "Fixed in 4d5d5e32 and 221cf85f on #105; delta-reviewed"
resolution: null
duplicate_of: null
---
Re-review at 245395c0: H121 correction arrived via main, but H83 at performance-loop.md:930 still juxtaposes exp-109 63%, exp-120 43%, exp-155 60-62% without qualifying each timer definition. Label whole-loop, apply-only from e667b739, and candidates.remove-expanded respectively. Correct exp-155:206 post-H112 wording. Prior R3 only partially resolved.
