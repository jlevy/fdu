---
type: is
id: is-01m4ppd9j28xdn96t9pw9ngeve
title: Explain live Library peer discrepancies before tagging 0.5.0
kind: bug
status: in_progress
priority: 1
version: 3
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-11T23:57:44.386Z
updated_at: 2026-10-12T00:04:35.524Z
started_at: 2026-10-11T23:58:18.320Z
---
The required five-tree macOS peer pass at candidate d0862487 passed the repository, Rust toolchains, Applications, and Linux source tree, but failed four checks on the live home Library. GNU du -l was about 5.9 MiB higher than bracketed fdu allocated readings, localized to Application Support (6,205,440 unexplained bytes at the top level); BSD du had a smaller allocated discrepancy; one fdu reading reported Interrupted. The tree was about 68.9 GiB allocated and 144.9 GiB apparent with 511 denied or unstatted entries. Preserve raw private readings and investigate live churn versus a counting bug. Do not claim peer QA passed or tag while unexplained. All 15 narrower automated stability stages had passed beforehand.

## Notes

Initial private readings are preserved. The interrupted fdu reading had exactly the same set of 64 reported failing paths as both surrounding reads: 63 denied plus one Interrupted became 64 denied again, with 447 omitted throughout. Two focused fdu/GNU-du/fdu repeats over Application Support found no top-level child outside the bracketed fdu values; all three tools reported 13 denied directories in both repeats. The initial 6.2 MB discrepancy did not reproduce. A complete five-tree peer pass is rerunning with the same candidate, tools, and thresholds. Do not call the original discrepancy conclusively attributed to individual files or mark the bead complete before assessing that repeat.
