---
type: is
id: is-01m3jkx8psq29fp5kza3y15kxj
title: Make deep-render stack test exercise the full tree and diagnose Windows failure
kind: bug
status: closed
priority: 2
version: 7
labels: []
dependencies: []
created_at: 2026-09-27T23:41:22.509Z
updated_at: 2026-09-28T00:48:12.824Z
closed_at: 2026-09-28T00:48:12.824Z
close_reason: Nonvacuous bounded deep-tree test passes macOS and Windows on all three stack layers; construction and rendering budgets are independently checked.
resolution: null
duplicate_of: null
---

## Notes

Fixed in ed65e05e. The test constructs and verifies all 1,025 nodes without omissions on a 128 KiB stack, then renders, streams, and explicitly drops the report on a separate 64 KiB stack. The pinned Rust 1.97.1 Windows runtime reserves 20 KiB for overflow handling, explaining the previous narrow construction margin; no depth-recursive report call was found. Default and no-default focused macOS tests and formatting pass. All 19 CI jobs pass on implementation run 36362959010 and documentation run 36362958606. Windows cargo tests also pass on top run 36362991775, whose remaining packaging checks are still running. Previous failure evidence is retained in the PR review.
