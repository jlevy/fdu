---
type: is
id: is-01m36bkeecpc2ek6jcxdqnjqmx
title: Text output truncates walk errors silently beyond 64 retained issues
kind: bug
status: open
priority: 1
version: 2
labels:
  - stack-followup
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:18.923Z
updated_at: 2026-09-23T05:25:51.355Z
---
Stack review R113-3 (#113; present at #117). crates/fdu/src/cli.rs prints one warning per retained issue (MAX_RETAINED_ISSUES=64) and never states status.errors_omitted; render_text does not either (only machine formats emit errors_omitted). Violates Never Truncate Silently. Content operational failures also print without their path. Fix: after the loop print a warning with the omitted count; include issue.path when the message lacks it; add a golden or renderer test with more than 64 unreadable directories.
