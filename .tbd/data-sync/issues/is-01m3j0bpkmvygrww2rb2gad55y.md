---
type: is
id: is-01m3j0bpkmvygrww2rb2gad55y
title: Keep default tree share pruning on partial scans and cover full output
kind: bug
status: in_progress
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-09-27T17:59:43.969Z
updated_at: 2026-09-27T18:00:06.811Z
---
User default fdu over external agent-scratch prints thousands of sub1% and zero-byte rows. expand globally bypasses share admission when index coverage is incomplete. Reproduce real scan, preserve uncertain branches without disabling pruning for known-complete rows, add portable golden for partial/default interaction plus exact completeness tests; test, push stack update and reinstall.
