---
type: is
id: is-01m3grekt2gd25tc58c2hb5esw
title: "PR #133 review R3: Reject ineffective extension metric sorts"
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
created_at: 2026-09-27T06:22:16.385Z
updated_at: 2026-09-27T07:10:17.537Z
started_at: 2026-09-27T06:24:05.754Z
closed_at: 2026-09-27T07:10:17.537Z
close_reason: "All six senior-review findings fixed in 6731aad9 with Linux parity record e8c189d7; full combined make check and Apple/Windows cross-lint passed, all 19 PR #133 CI jobs passed, and full review plus per-ID disposition are recorded on PR #133."
resolution: null
duplicate_of: null
---
Review https://github.com/jlevy/fdu/pull/133#issuecomment-5853335891 R3. query_request.rs:646 and query_report.rs:1924 accept metric sort with no extension values. Shared validation should refuse and name supported alternatives before filesystem I/O.

## Notes

R3 fixed by shared validation refusal for Extensions metric sorts. Query request tests 24/24; missing-root shared golden confirms validation precedes I/O. Usage and plan updated. Awaiting integrated gate.
