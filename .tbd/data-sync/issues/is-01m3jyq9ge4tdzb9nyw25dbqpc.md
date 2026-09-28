---
type: is
id: is-01m3jyq9ge4tdzb9nyw25dbqpc
title: "Spike: resolve historical replay completion and shallow real-root refresh"
kind: task
status: closed
priority: 1
version: 4
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T02:50:21.069Z
updated_at: 2026-09-28T03:35:51.056Z
started_at: 2026-09-28T02:50:35.917Z
closed_at: 2026-09-28T03:35:51.055Z
close_reason: "Continuation spike completed in fe0174d0 / PR #131: controlled shallow refresh matched with 712 observations; two busy-root failures and independent aged open-writer reproductions establish a history-only coverage limitation; day-old completion is variable through 92.5s. Research, both active plans, reproducible probes and sanitized evidence updated. Twenty focused tests, strict Python checks, make docs-format, make check, make cross-lint and all PR CI passed. Production integration is not approved: fdu-vhrb remains open for writer coverage and fdu-uwhl remains open for remaining replay acceptance."
resolution: null
duplicate_of: null
---

## Notes

Continuation implemented and measured: shallow relisting/recursive invalidation with one parent index and cached hardlink-alias expansion. Controlled20204->20206-entry growth:712 observations,exact metadata+rollup match,expected66516 apparent/61440 allocated byte gain. Real455k root848/900 observations but1/2stable misses:failed. Investigated with targeted descriptor query then reproducible open-writer controls, now tracked fdu-vhrb. Final historical144 run completed at92.5s under120s diagnostic bound;60s repeats fail. Native default unchanged. Sources/tests/research/plans updated; full handoff gate in progress. No engine/dependency changes, all scratch external.
