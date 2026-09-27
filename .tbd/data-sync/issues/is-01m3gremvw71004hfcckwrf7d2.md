---
type: is
id: is-01m3gremvw71004hfcckwrf7d2
title: "PR #133 review R6: Enforce ignored population at reconciliation roots"
kind: bug
status: closed
priority: 1
version: 4
delegate: codex@spud10.local
labels: []
dependencies: []
parent_id: is-01m3gr3gmn8cwk3hdebm32m5w4
hold: null
hold_until: null
created_at: 2026-09-27T06:22:17.467Z
updated_at: 2026-09-27T07:10:17.561Z
started_at: 2026-09-27T06:22:58.650Z
closed_at: 2026-09-27T07:10:17.561Z
close_reason: "All six senior-review findings fixed in 6731aad9 with Linux parity record e8c189d7; full combined make check and Apple/Windows cross-lint passed, all 19 PR #133 CI jobs passed, and full review plus per-ID disposition are recorded on PR #133."
resolution: null
duplicate_of: null
---
Review https://github.com/jlevy/fdu/pull/133#issuecomment-5853335891 R6. scan.rs:5155 upserts targeted ignored boundary before population admission. Direct and handle subtree reconciliation must match cold scope, including controls and unknown classification.

## Notes

R6 fixed by nearest retained parent reconciliation, climbing unknown governing-control boundaries. Direct/handle/pruned-ancestry/rule-recovery/unreadable controls TDD regressions; scan tests 125/125. Awaiting full gate and correctness runbook.
