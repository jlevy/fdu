---
type: is
id: is-01m2xp6x0akn2wfj0p4rx7zp1k
title: "H125: cache-only completeness uses the candidate count restore already computed"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
delegate: unknown@spud10
labels:
  - performance
  - campaign-2
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
hold: null
hold_until: null
created_at: 2026-09-19T20:37:32.296Z
updated_at: 2026-09-19T20:45:08.147Z
started_at: 2026-09-19T20:37:38.219Z
closed_at: 2026-09-19T20:45:08.145Z
close_reason: "exp-124 accepted H125 restore-count completeness: wall -8.03% [-10.79%, -7.79%]; H113 file-count superseded; exp-113 unused"
resolution: null
duplicate_of: null
---
Pre-registered 2026-09-19 ~13:38 PT.

H125 / exp-124. Control = #92 HEAD af306146 (H115+H120 in; no engine speed patch).
Job: content-cache-hit wall on deciding-scale metabrowser-clone (frozen APFS clone).
Accept: wall down at least 3% with the 95% interval entirely below zero; content digest
identical; incomplete sidecar still refused.

Named mechanism: load_content_cache already walks analysis_candidates into the restore
HashMap. Capture that len before the apply drain. Cache-only completeness compares hits
to that stored count.

Not H113's file-count heuristic (Index::analysis_candidate_count / rollup.files).
Not H116 (does not drop the first candidates walk).
exp-113 stays reserved for H113 quiet.

What refutes: interval includes zero, digest changes, or incomplete sidecar is served.
Quiet H113 this tick refused at 45.48% busy / 1.599 load per core. File-count shortcut
not compiled. This cell is uncontrolled (new mechanism; H113 quiet exception does not
apply). Do not lower the 25% bar.
