---
type: is
id: is-01m4ppd9j28xdn96t9pw9ngeve
title: Explain live Library peer discrepancies on macOS (post-0.5.0)
kind: bug
status: in_progress
priority: 2
version: 8
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-11T23:57:44.386Z
updated_at: 2026-10-13T18:01:19.168Z
started_at: 2026-10-11T23:58:18.320Z
---
The required five-tree macOS peer pass at candidate d0862487 passed the repository, Rust toolchains, Applications, and Linux source tree, but failed four checks on the live home Library. GNU du -l was about 5.9 MiB higher than bracketed fdu allocated readings, localized to Application Support (6,205,440 unexplained bytes at the top level); BSD du had a smaller allocated discrepancy; one fdu reading reported Interrupted. The tree was about 68.9 GiB allocated and 144.9 GiB apparent with 511 denied or unstatted entries. Preserve raw private readings and investigate live churn versus a counting bug. Do not claim peer QA passed or tag while unexplained. All 15 narrower automated stability stages had passed beforehand.

## Notes

Both required five-tree runs failed on the live Library; four quieter trees had all 52 readings exact both times. First run GNU du allocated/top-level 6,205,440-byte discrepancy and small BSD difference cleared on the second; two focused Application Support brackets also did not reproduce a stable bias. The second run instead had pdu apparent 29,232 bytes above the following fdu reading after measured directory/symlink correction. Both runs had one Interrupted on an otherwise refused path. Four alternating reads each of published 0.4.0 and candidate 0.5.0 then reproduced Interrupted in published 0.4.0; that exact path was permission denied in all seven other reads. This demonstrates the transient diagnostic predates 0.5.0, but does not explain the peer size discrepancy or prove the original interrupted path identity (the earlier helper retained only path sets). All raw paths remain private; no thresholds changed. PR #197 merged at 33a3a890, identical tested tree, with this tag blocker explicit. Release evidence documentation and non-publishing rehearsal are underway.

2026-10-12 bounded metadata diagnostic: preserved two incomplete full-inventory attempts (first phase timed out at 123.8 s and 181.1 s; neither reached its peer readings), then completed one correction-only Library experiment in 76.1 s using the verified current native binary with cache off. Order: directory/symlink inventory, fdu, pdu apparent, fdu, inventory. The pdu total was 155,670,462,753 B; fdu apparent readings were 155,640,904,953 and 155,640,925,321 B. Corrections were 29,547,802 and 29,548,090 B. Thus corrected pdu was inside both endpoint brackets (residuals +9,998/-10,370 with the earlier correction, +9,710/-10,658 with the later one). The old +29,232 B excess did not reproduce. Exactly four directories changed their correction sizes by +96, +128, +32 and +32 B, totaling +288 B; symlink totals were unchanged. This is insufficient to explain the historical discrepancy.

Error attribution from saved private identities: pdu reported 510 permission failures and one Interrupted; all 507 correction-inventory error paths were in pdu's 511-path set. The exact Interrupted path was permission denied (OS error 1) in both inventories and both detailed fdu error lists. All 64 detailed fdu paths appeared in pdu's error set, but 447 omitted fdu details prevent a full path-set comparison. Regular files were not inventoried in the completed run and live snapshots are not atomic, so this does not prove intervening churn or an engine counting defect. Historical aggregates lack contemporaneous per-path metadata needed for exact attribution. Evidence: durable private library-churn-diagnostic-disposition.json and library-churn-pdu-corrections-01; partial attempts and original scripts retained. No source changes, builds, tolerance changes, or further scans. Gate stays open. Next useful work is one scheduled, bounded per-path metadata capture on the implicated subtree when a residual reproduces, followed by exact identity/error reconciliation; a clean retry alone cannot close this issue.


2026-10-13 maintainer disposition (given in the Claude Code session finishing the 0.5.0 release): accepted as live-tree churn for 0.5.0 and not a tag blocker. Basis: four quieter trees agreed on all 52 readings in both five-subject runs; the Library is a live, permission-limited tree whose discrepancies moved between runs (GNU/BSD allocated in the first, pdu apparent +29,232 B in the repeat) and did not reproduce under focused brackets; the recurring Interrupted symptom also occurs on published 0.4.0. Retitled and lowered to P2: it stays open as a post-release macOS investigation of the residual. No thresholds or code were changed.
