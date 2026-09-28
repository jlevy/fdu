---
type: is
id: is-01m3jkx8psq29fp5kza3y15kxj
title: Make deep-render stack test exercise the full tree and diagnose Windows failure
kind: bug
status: closed
priority: 2
version: 8
labels: []
dependencies: []
created_at: 2026-09-27T23:41:22.509Z
updated_at: 2026-09-28T00:50:10.372Z
closed_at: 2026-09-28T00:48:12.824Z
close_reason: Nonvacuous bounded deep-tree test passes macOS and Windows on all three stack layers; construction and rendering budgets are independently checked.
resolution: null
duplicate_of: null
---

## Notes

Fixed in ed65e05e. Construction and independent 1,025-node/no-omission verification use a 128 KiB thread; rendering, streaming, and explicit drop use a separate 64 KiB thread. The pinned Rust 1.97.1 Windows runtime reserves 20 KiB for overflow handling; no depth-recursive construction helper was found. Default and no-default focused macOS tests and formatting pass. Final CI: all 19 jobs pass on each of implementation run 36362959010 (ed65e05e), documentation run 36362958606 (0dce3bc0), and top run 36362991775 (270d8be2), without reruns. Prior failure evidence and final review are recorded at PR 136 comment 5860865321. Installed runtime remains 22e69f53a with 102 independent acceptance checks; subsequent commits change tests and review docs only.
