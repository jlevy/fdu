---
type: is
id: is-01m4ppd9j28xdn96t9pw9ngeve
title: Explain live Library peer discrepancies on macOS (post-0.5.0)
kind: bug
status: in_progress
priority: 2
version: 7
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-11T23:57:44.386Z
updated_at: 2026-10-13T18:00:58.172Z
started_at: 2026-10-11T23:58:18.320Z
---
The required five-tree macOS peer pass at candidate d0862487 passed the repository, Rust toolchains, Applications, and Linux source tree, but failed four checks on the live home Library. GNU du -l was about 5.9 MiB higher than bracketed fdu allocated readings, localized to Application Support (6,205,440 unexplained bytes at the top level); BSD du had a smaller allocated discrepancy; one fdu reading reported Interrupted. The tree was about 68.9 GiB allocated and 144.9 GiB apparent with 511 denied or unstatted entries. Preserve raw private readings and investigate live churn versus a counting bug. Do not claim peer QA passed or tag while unexplained. All 15 narrower automated stability stages had passed beforehand.

## Notes

2026-10-13 maintainer disposition (in the Claude Code session finishing the 0.5.0 release): accepted as live-tree churn for 0.5.0 and not a tag blocker. Basis: four quieter trees agreed on all 52 readings in both five-subject runs; the Library is a live, permission-limited tree whose discrepancies moved between runs (GNU/BSD allocated in the first, pdu apparent +29,232 B in the repeat) and did not reproduce under focused brackets; the recurring Interrupted symptom also occurs on published 0.4.0. The bead stays open as a post-release macOS investigation of the residual; no thresholds or code were changed.
