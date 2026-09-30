---
type: is
id: is-01m3jba39zq8w3nqmtv0npvk5f
title: Stabilize second-pass watch report equality under valid setup races
kind: bug
status: closed
priority: 2
version: 4
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-27T21:11:05.789Z
updated_at: 2026-09-30T04:06:19.943Z
closed_at: 2026-09-30T04:06:19.943Z
close_reason: "Fixed in a4b07680 (test only): a_started_session_reports_its_second_pass_and_then_nothing now allows exactly a WatchSetupRace observation gap (IssueKind::ObservationGap whose message ends with the reason) in the natively watched session's status and nothing else, and compares the facts both sessions report as JSON with status and provenance set aside, so a real difference still fails and the gap is never suppressed. The race is FSEvents reporting a directory created just before registration, which the engine records truthfully; 200 of 200 local runs pass before and after (loop over the test binary, --exact)."
resolution: null
duplicate_of: null
---
CI run 36350320105 job 108707673904 failed a_started_session_reports_its_second_pass_and_then_nothing on macOS: watched report contained observation_gap at d3 with WatchSetupRace, while separately started plain session did not. All other 896 core tests passed. Same watch_session.rs passed prior run 36349895447; intervening commit changed only parity recording and validation docs. The test acknowledges startup replay but compares full reports including race-sensitive status. Assert stable report facts and progress while explicitly checking allowed setup-race diagnostics, or establish a deterministic settled comparison. Preserve truthful engine observation gaps; do not suppress them to satisfy the test.

## Notes

Recurrence: implementation head 0433bcc0, macOS CI run 36366851633 job 108754923371. watch_session::tests::a_started_session_reports_its_second_pass_and_then_nothing again compared a clean report with one containing observation_gap at d3 (WatchSetupRace); 911 other core tests passed. The deep renderer test passed. This confirms the tracked test can encounter a valid setup-race diagnostic; preserve the diagnostic and make the equality assertion race aware.
