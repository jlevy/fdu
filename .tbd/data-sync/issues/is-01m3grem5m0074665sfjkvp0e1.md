---
type: is
id: is-01m3grem5m0074665sfjkvp0e1
title: "PR #133 review R4: Respect shell heredoc tab-only stripping"
kind: bug
status: in_progress
priority: 2
version: 3
delegate: codex@spud10.local
labels: []
dependencies: []
parent_id: is-01m3gr3gmn8cwk3hdebm32m5w4
hold: null
hold_until: null
created_at: 2026-09-27T06:22:16.755Z
updated_at: 2026-09-27T06:30:01.328Z
started_at: 2026-09-27T06:22:49.755Z
---
Review https://github.com/jlevy/fdu/pull/133#issuecomment-5853335891 R4. content_code_metrics.rs:246 strips spaces from shell <<- terminators; preserve shell versus Ruby indentation policies.

## Notes

R4 fixed with tab-vs-space shell heredoc cases in existing syntax table. Lexer suite 11/11. Awaiting integrated gate.
