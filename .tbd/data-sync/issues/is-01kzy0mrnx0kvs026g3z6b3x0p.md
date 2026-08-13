---
type: is
id: is-01kzy0mrnx0kvs026g3z6b3x0p
title: "PR #11 review R1: normalize native separators in collect-files assertion"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01kzy0mjn886vmwkkca3962w78
created_at: 2026-08-13T16:52:46.907Z
updated_at: 2026-08-13T17:04:52.273Z
closed_at: 2026-08-13T17:04:52.273Z
close_reason: "Fixed PR #11 review R1 in 720a56a by normalizing native test paths and enforcing the policy test/audit on macOS, Linux, and Windows; full local make check and all CI passed, and the review thread was answered and resolved."
---
Cursor Bugbot review R1 at scripts/check-rust-module-names.test.mjs:39-49: path.relative() returns backslashes on Windows, while the assertion expects forward-slash literals. Normalize the test's native relative paths at the assertion boundary and verify the focused policy target plus full handoff gate.
