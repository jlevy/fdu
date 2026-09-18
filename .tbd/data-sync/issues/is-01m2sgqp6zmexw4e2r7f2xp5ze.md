---
type: is
id: is-01m2sgqp6zmexw4e2r7f2xp5ze
title: "PR #88 review R3: write release body before check_release_body"
kind: bug
status: closed
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m2sgape5ss3nrhm5em7a2fqq
hold: null
hold_until: null
created_at: 2026-09-18T05:44:53.214Z
updated_at: 2026-09-18T05:50:22.090Z
started_at: 2026-09-18T05:45:13.632Z
closed_at: 2026-09-18T05:50:22.090Z
close_reason: "Fixed on PR #88 in 4828d950"
resolution: null
duplicate_of: null
---
scripts/release/release_body.py:221-226
Check first, then write. Exit with the ValueError message, no traceback.
