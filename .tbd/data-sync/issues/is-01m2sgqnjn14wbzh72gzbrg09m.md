---
type: is
id: is-01m2sgqnjn14wbzh72gzbrg09m
title: "PR #88 review R2: release-body tests never run flowmark unwrap"
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
created_at: 2026-09-18T05:44:52.564Z
updated_at: 2026-09-18T05:50:22.082Z
started_at: 2026-09-18T05:45:13.624Z
closed_at: 2026-09-18T05:50:22.082Z
close_reason: "Fixed on PR #88 in 4828d950"
resolution: null
duplicate_of: null
---
tests/release/test_release_body.py
Add one non-identity unwrap test through unwrap_with_flowmark / default main.
