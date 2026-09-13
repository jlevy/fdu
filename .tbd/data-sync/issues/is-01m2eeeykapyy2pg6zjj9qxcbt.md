---
type: is
id: is-01m2eeeykapyy2pg6zjj9qxcbt
title: "Address review: PR #52 — one-shot parity without weakening streaming"
kind: task
status: in_progress
priority: 1
version: 12
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
child_order_hints:
  - is-01m2eef5mdpzhcg5f4d1qe939h
  - is-01m2eefb7t5e7nxf2hdx13tfjc
  - is-01m2eefpxxpbmsjrk83ntwncw2
  - is-01m2eefw0awdfhkmw5sz1ax7k3
  - is-01m2eefz2462p26jk460h4abms
  - is-01m2eeg2eph6a2prwv1zgct9y5
  - is-01m2eeg5j9pvsn2vq5sjwyj653
  - is-01m2eeg8xtq4sp9p1083p6zn0p
  - is-01m2eegcapyzyrx7qngtzdhmtm
  - is-01m2eegfhjtav56tk3amg3bp4q
created_at: 2026-09-13T22:33:30.985Z
updated_at: 2026-09-13T22:35:46.552Z
---
Formal review 5192264318 on PR #52 (https://github.com/jlevy/fdu/pull/52#pullrequestreview-5192264318) at head afbb2ee. Findings: BUILD-1 and BUILD-2 (High), PERF-1 and PERF-2 (Medium), BUILD-3 and PERF-3 through PERF-8 (Low). Scope is this PR's own findings; the carried COMMIT-2, COMMIT-3, and READ-1 are fixed at their origin PRs and arrive with the later stack propagation. PERF-2 is covered by existing fdu-x16g (#54 renumbered its artifact to exp-103 in 86d2a6a). No local Rust builds on this host (disk below the build floor); CI is the gate.
