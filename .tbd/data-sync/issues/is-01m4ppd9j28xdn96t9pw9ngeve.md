---
type: is
id: is-01m4ppd9j28xdn96t9pw9ngeve
title: Explain live Library peer discrepancies before tagging 0.5.0
kind: bug
status: in_progress
priority: 1
version: 4
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-11T23:57:44.386Z
updated_at: 2026-10-12T00:23:50.337Z
started_at: 2026-10-11T23:58:18.320Z
---
The required five-tree macOS peer pass at candidate d0862487 passed the repository, Rust toolchains, Applications, and Linux source tree, but failed four checks on the live home Library. GNU du -l was about 5.9 MiB higher than bracketed fdu allocated readings, localized to Application Support (6,205,440 unexplained bytes at the top level); BSD du had a smaller allocated discrepancy; one fdu reading reported Interrupted. The tree was about 68.9 GiB allocated and 144.9 GiB apparent with 511 denied or unstatted entries. Preserve raw private readings and investigate live churn versus a counting bug. Do not claim peer QA passed or tag while unexplained. All 15 narrower automated stability stages had passed beforehand.

## Notes

The second five-tree run also failed: all 52 readings on four quieter trees remained exact, and the earlier GNU/BSD/top-level discrepancies cleared, but the Library pdu apparent total after directory/symlink correction exceeded the following fdu reading by 29,232 bytes. One Interrupted diagnostic recurred, again with the same refused-path set before and after. Both failed runs and focused allocation probes are preserved privately. No thresholds changed; neither live churn nor a counting defect is conclusively attributed. Release preparation #197 merged at 33a3a890 with the same tested source tree, explicitly leaving this as a tag blocker. A bounded published-0.4 versus candidate-0.5 metadata probe is collecting detailed error identity locally.
