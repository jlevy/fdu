---
type: is
id: is-01m36bm560f11v69fyb6xqewz1
title: One uninterrupted make check and cross-lint on the final composed candidate
kind: task
status: closed
priority: 0
version: 3
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:42.207Z
updated_at: 2026-09-23T08:58:58.446Z
closed_at: 2026-09-23T08:58:58.441Z
close_reason: "Uninterrupted make check (17m08s, exit 0) and make cross-lint (exit 0) passed on the composed tree 52146761 (correctness stack + #94 + #97 + #105 + #109); final main 7e06e5a4 has exactly that tree (verified by tree hash). Also make check (14m49s) and cross-lint on the correctness stack top fbc6534a before it merged."
resolution: null
duplicate_of: null
---
No uninterrupted make check exists on #117 or on the stack with the perf layers; evidence is scoped runs (fdu-xgjx notes) and the plan says scoped runs are not the gate. Run make check and make cross-lint in one invocation each at the exact commit that will merge (correctness stack + #94/#97/#105/#109 composition), on a host with enough disk (the 2026-09-22 host hit ENOSPC with ~60 alpha worktrees in /private/tmp). Blocks fdu-tyvq.
