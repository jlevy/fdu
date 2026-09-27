---
type: is
id: is-01m3j0bpkmvygrww2rb2gad55y
title: Keep default tree share pruning on partial scans and cover full output
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
created_at: 2026-09-27T17:59:43.969Z
updated_at: 2026-09-27T19:03:45.789Z
closed_at: 2026-09-27T19:03:45.788Z
close_reason: "Fixed per-subtree default pruning with shared core/CLI golden, bounded-discovery tryscript and selected-root checks. Full local make check passed; all 19 CI checks passed on #133 8de84067, #135 b742e299, #136 50a7581c. Reinstalled clean local wheel fdu 0.1.0-dev+g50a7581c1; real-tree output reduced from 1213467 to 93 lines while preserving partial status."
resolution: null
duplicate_of: null
---
User default fdu over external agent-scratch prints thousands of sub1% and zero-byte rows. expand globally bypasses share admission when index coverage is incomplete. Reproduce real scan, preserve uncertain branches without disabling pruning for known-complete rows, add portable golden for partial/default interaction plus exact completeness tests; test, push stack update and reinstall.
