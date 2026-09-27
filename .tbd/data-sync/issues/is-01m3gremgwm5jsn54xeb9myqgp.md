---
type: is
id: is-01m3gremgwm5jsn54xeb9myqgp
title: "PR #133 review R5: Recognize regex statements after JavaScript control conditions"
kind: bug
status: closed
priority: 2
version: 4
delegate: codex@spud10.local
labels: []
dependencies: []
parent_id: is-01m3gr3gmn8cwk3hdebm32m5w4
hold: null
hold_until: null
created_at: 2026-09-27T06:22:17.115Z
updated_at: 2026-09-27T07:10:17.554Z
started_at: 2026-09-27T06:22:49.768Z
closed_at: 2026-09-27T07:10:17.554Z
close_reason: "All six senior-review findings fixed in 6731aad9 with Linux parity record e8c189d7; full combined make check and Apple/Windows cross-lint passed, all 19 PR #133 CI jobs passed, and full review plus per-ID disposition are recorded on PR #133."
resolution: null
duplicate_of: null
---
Review https://github.com/jlevy/fdu/pull/133#issuecomment-5853335891 R5. content_code_metrics.rs:403 mistakes valid regex after control closing parenthesis for division and a comment; protect division and nested/chunk cases.

## Notes

R5 fixed with nested/multiline/consecutive control conditions and call/property-call division, every byte chunk split. Lexer suite 11/11. Awaiting integrated gate.
