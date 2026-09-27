---
type: is
id: is-01m3grekem5vmh41p9cqvywshs
title: "PR #133 review R2: Show population contributions consistently in Code text"
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
created_at: 2026-09-27T06:22:16.019Z
updated_at: 2026-09-27T06:29:45.512Z
started_at: 2026-09-27T06:23:32.532Z
---
Review https://github.com/jlevy/fdu/pull/133#issuecomment-5853335891 R2. report_format.rs:1484 and language loop omit per-language populations and leave ignored inline data unstyled. Show concise population details with gray parentheses and unknown classification.

## Notes

R2 fixed: per-language population details in gray parentheses; combined totals primary, unknown separate. Color/plain/machine renderer regression passed red/green. Shared corpus 185/185. Awaiting integrated gate.
