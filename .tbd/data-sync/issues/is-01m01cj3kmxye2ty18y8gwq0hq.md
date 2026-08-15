---
type: is
id: is-01m01cj3kmxye2ty18y8gwq0hq
title: "PR #26 review R1: release.yml macOS runner labels wrong on both legs"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m01chg3gqm5sjf58mt5ng9zw
created_at: 2026-08-15T00:18:46.004Z
updated_at: 2026-08-15T00:34:04.772Z
closed_at: 2026-08-15T00:34:04.771Z
close_reason: Fixed in 862190a on claude/fdu-pr-review-g8rsrm; full make check green; disposition map posted at https://github.com/jlevy/fdu/pull/26#issuecomment-5299527089
---
release.yml:132,137 — x86_64-apple-darwin runs on macos-15 (arm64); aarch64 leg uses invalid label macos-15-arm64. Swap: x86_64 -> macos-15-intel, aarch64 -> macos-15.
