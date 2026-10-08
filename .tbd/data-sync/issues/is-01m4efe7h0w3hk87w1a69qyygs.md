---
type: is
id: is-01m4efe7h0w3hk87w1a69qyygs
title: "PR #186 A5: make stage-before-expect structural in expected_assets"
kind: bug
status: closed
priority: 3
version: 3
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m4efe57c19n7q72g0edd20v4
hold: null
hold_until: null
created_at: 2026-10-08T19:21:59.583Z
updated_at: 2026-10-08T19:28:19.865Z
started_at: 2026-10-08T19:22:25.980Z
closed_at: 2026-10-08T19:28:19.865Z
close_reason: "Declined: staging inside expected_assets would thread host through asset_check, announce_command, and missing_assets (and re-stage between upload and verification); residual risk is narrow and each call site is test-pinned. Disposition https://github.com/jlevy/fdu/pull/186#issuecomment-6067458642"
resolution: null
duplicate_of: null
---
Suggestion. scripts/release/maintainer.py expected_assets (~1220-1238). Review: https://github.com/jlevy/fdu/pull/186#issuecomment-6067330629
