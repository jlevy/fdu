---
type: is
id: is-01m4ppd9j28xdn96t9pw9ngeve
title: Explain live Library peer discrepancies before tagging 0.5.0
kind: bug
status: in_progress
priority: 1
version: 5
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-11T23:57:44.386Z
updated_at: 2026-10-12T00:25:11.857Z
started_at: 2026-10-11T23:58:18.320Z
---
The required five-tree macOS peer pass at candidate d0862487 passed the repository, Rust toolchains, Applications, and Linux source tree, but failed four checks on the live home Library. GNU du -l was about 5.9 MiB higher than bracketed fdu allocated readings, localized to Application Support (6,205,440 unexplained bytes at the top level); BSD du had a smaller allocated discrepancy; one fdu reading reported Interrupted. The tree was about 68.9 GiB allocated and 144.9 GiB apparent with 511 denied or unstatted entries. Preserve raw private readings and investigate live churn versus a counting bug. Do not claim peer QA passed or tag while unexplained. All 15 narrower automated stability stages had passed beforehand.

## Notes

Both required five-tree runs failed on the live Library; four quieter trees had all 52 readings exact both times. First run GNU du allocated/top-level 6,205,440-byte discrepancy and small BSD difference cleared on the second; two focused Application Support brackets also did not reproduce a stable bias. The second run instead had pdu apparent 29,232 bytes above the following fdu reading after measured directory/symlink correction. Both runs had one Interrupted on an otherwise refused path. Four alternating reads each of published 0.4.0 and candidate 0.5.0 then reproduced Interrupted in published 0.4.0; that exact path was permission denied in all seven other reads. This demonstrates the transient diagnostic predates 0.5.0, but does not explain the peer size discrepancy or prove the original interrupted path identity (the earlier helper retained only path sets). All raw paths remain private; no thresholds changed. PR #197 merged at 33a3a890, identical tested tree, with this tag blocker explicit. Release evidence documentation and non-publishing rehearsal are underway.
