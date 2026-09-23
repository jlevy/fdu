---
type: is
id: is-01m36bm560f11v69fyb6xqewz1
title: One uninterrupted make check and cross-lint on the final composed candidate
kind: task
status: open
priority: 0
version: 2
labels:
  - release
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:42.207Z
updated_at: 2026-09-23T05:25:50.777Z
---
No uninterrupted make check exists on #117 or on the stack with the perf layers; evidence is scoped runs (fdu-xgjx notes) and the plan says scoped runs are not the gate. Run make check and make cross-lint in one invocation each at the exact commit that will merge (correctness stack + #94/#97/#105/#109 composition), on a host with enough disk (the 2026-09-22 host hit ENOSPC with ~60 alpha worktrees in /private/tmp). Blocks fdu-tyvq.
