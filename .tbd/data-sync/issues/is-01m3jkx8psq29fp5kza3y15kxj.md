---
type: is
id: is-01m3jkx8psq29fp5kza3y15kxj
title: Make deep-render stack test exercise the full tree and diagnose Windows failure
kind: bug
status: in_progress
priority: 2
version: 5
labels: []
dependencies: []
created_at: 2026-09-27T23:41:22.509Z
updated_at: 2026-09-28T00:40:19.922Z
closed_at: null
close_reason: null
resolution: null
duplicate_of: null
---

## Notes

Test-only fix ed65e05e separates report construction/verification on128KiB from renderer/stream/drop on64KiB. Pinned Rust1.97.1 Windows reserves20KiB for overflow handling. Independent1025nodes/no omissions assertion retained. Default and no-default focused macOS tests pass; formatting clean. Final Windows CI runs36362959010/36362958606/36362991775 pending. Prior failed log retained with b424a6bc0a01c2260f46c4ee654e3118ba5447926e76527d677aa57a06c80e6c.
