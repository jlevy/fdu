---
type: is
id: is-01m4ppd9j28xdn96t9pw9ngeve
title: Explain live Library peer discrepancies before tagging 0.5.0
kind: bug
status: in_progress
priority: 1
version: 2
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m4pfhrrvbd19h3wwrqte3zan
hold: null
hold_until: null
created_at: 2026-10-11T23:57:44.386Z
updated_at: 2026-10-11T23:58:18.321Z
started_at: 2026-10-11T23:58:18.320Z
---
The required five-tree macOS peer pass at candidate d0862487 passed the repository, Rust toolchains, Applications, and Linux source tree, but failed four checks on the live home Library. GNU du -l was about 5.9 MiB higher than bracketed fdu allocated readings, localized to Application Support (6,205,440 unexplained bytes at the top level); BSD du had a smaller allocated discrepancy; one fdu reading reported Interrupted. The tree was about 68.9 GiB allocated and 144.9 GiB apparent with 511 denied or unstatted entries. Preserve raw private readings and investigate live churn versus a counting bug. Do not claim peer QA passed or tag while unexplained. All 15 narrower automated stability stages had passed beforehand.
