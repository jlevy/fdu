---
type: is
id: is-01m3grekem5vmh41p9cqvywshs
title: "PR #133 review R2: Show population contributions consistently in Code text"
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
created_at: 2026-09-27T06:22:16.019Z
updated_at: 2026-09-27T07:10:17.529Z
started_at: 2026-09-27T06:23:32.532Z
closed_at: 2026-09-27T07:10:17.529Z
close_reason: "All six senior-review findings fixed in 6731aad9 with Linux parity record e8c189d7; full combined make check and Apple/Windows cross-lint passed, all 19 PR #133 CI jobs passed, and full review plus per-ID disposition are recorded on PR #133."
resolution: null
duplicate_of: null
---
Review https://github.com/jlevy/fdu/pull/133#issuecomment-5853335891 R2. report_format.rs:1484 and language loop omit per-language populations and leave ignored inline data unstyled. Show concise population details with gray parentheses and unknown classification.

## Notes

R2 fixed: per-language population details in gray parentheses; combined totals primary, unknown separate. Color/plain/machine renderer regression passed red/green. Shared corpus 185/185. Awaiting integrated gate.
