---
type: is
id: is-01m3grem5m0074665sfjkvp0e1
title: "PR #133 review R4: Respect shell heredoc tab-only stripping"
kind: bug
status: closed
priority: 2
version: 5
delegate: codex
labels: []
dependencies: []
parent_id: is-01m3gr3gmn8cwk3hdebm32m5w4
hold: null
hold_until: null
created_at: 2026-09-27T06:22:16.755Z
updated_at: 2026-09-28T16:20:53.100Z
started_at: 2026-09-27T06:22:49.755Z
closed_at: 2026-09-27T07:10:17.546Z
close_reason: "All six senior-review findings fixed in 6731aad9 with Linux parity record e8c189d7; full combined make check and Apple/Windows cross-lint passed, all 19 PR #133 CI jobs passed, and full review plus per-ID disposition are recorded on PR #133."
resolution: null
duplicate_of: null
---
Review https://github.com/jlevy/fdu/pull/133#issuecomment-5853335891 R4. content_code_metrics.rs:246 strips spaces from shell <<- terminators; preserve shell versus Ruby indentation policies.

## Notes

R4 fixed with tab-vs-space shell heredoc cases in existing syntax table. Lexer suite 11/11. Awaiting integrated gate.
