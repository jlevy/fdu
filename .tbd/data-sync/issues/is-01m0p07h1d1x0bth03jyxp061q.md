---
type: is
id: is-01m0p07h1d1x0bth03jyxp061q
title: "PR #42 R17: run-parity absoluteness test is POSIX-only"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:27:19.469Z
updated_at: 2026-08-23T00:57:54.159Z
closed_at: 2026-08-23T00:57:54.159Z
close_reason: "Fixed in 4e34ce3; addressed the review on PR #42, verified through make check and make release-rehearse."
---
scripts/run-parity.mjs:61 uses named.startsWith(/), false for C:\\..., so a Windows absolute path would be joined onto the repo root. Use path.isAbsolute.
