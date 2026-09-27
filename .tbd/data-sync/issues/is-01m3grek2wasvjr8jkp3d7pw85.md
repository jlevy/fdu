---
type: is
id: is-01m3grek2wasvjr8jkp3d7pw85
title: "PR #133 review R1: Count selected directory contents in Code overview"
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
created_at: 2026-09-27T06:22:15.642Z
updated_at: 2026-09-27T07:10:17.520Z
started_at: 2026-09-27T06:22:53.973Z
closed_at: 2026-09-27T07:10:17.520Z
close_reason: "All six senior-review findings fixed in 6731aad9 with Linux parity record e8c189d7; full combined make check and Apple/Windows cross-lint passed, all 19 PR #133 CI jobs passed, and full review plus per-ID disposition are recorded on PR #133."
resolution: null
duplicate_of: null
---
Review https://github.com/jlevy/fdu/pull/133#issuecomment-5853335891 R1. query_report.rs:2211 aggregates flat matches instead of walked.members. Reuse selected file union; test directory/nested/excluded contents.

## Notes

R1 fixed using walked.members. TDD red/green; query_report tests 46/46. Shared golden now compares Code/Languages/Summary over selected directory plus overlapping file include and exclusion. Awaiting integrated gate.
