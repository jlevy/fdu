---
type: is
id: is-01m2eeeykapyy2pg6zjj9qxcbt
title: "Address review: PR #52 — one-shot parity without weakening streaming"
kind: task
status: closed
priority: 1
version: 20
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - stack-followup
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
  - is-01m2esgqj36xxaf7zxrysnnftp
  - is-01m2esgqx2v4wpb5v3fytaqgtz
  - is-01m2et31c6a5zryh6mw57f7v0s
  - is-01m2et31pte5v3c039ny9bjdjh
  - is-01m2ew97f3ay7c2rp9ztb7b8et
created_at: 2026-09-13T22:33:30.985Z
updated_at: 2026-09-14T02:35:03.518Z
closed_at: 2026-09-14T02:18:01.045Z
close_reason: "All eleven PR #52 review findings are dispositioned through 8640758. They first ran in CI together with #51 and #48 in merge 753e10f. That merge failed the opened allocation-slope guard; the growth was attributed to #48 3b6de62 (one allocation per unrecognized file) and fixed without raising any ceiling in c0511e9 on #52 and a080391 on #48. CI is green on #52 at c0511e9 (19/19). The disposition map is posted at https://github.com/jlevy/fdu/pull/52#issuecomment-5658101791. Post-merge verification follow-ups stay open under this parent: fdu-tilu, fdu-61vv, fdu-8yv0 (FIX52-1), and fdu-z0vu (FIX52-2)."
resolution: null
duplicate_of: null
---
Formal review 5192264318 on PR #52 (https://github.com/jlevy/fdu/pull/52#pullrequestreview-5192264318) at head afbb2ee. Findings: BUILD-1 and BUILD-2 (High), PERF-1 and PERF-2 (Medium), BUILD-3 and PERF-3 through PERF-8 (Low). Scope is this PR's own findings; the carried COMMIT-2, COMMIT-3, and READ-1 are fixed at their origin PRs and arrive with the later stack propagation. PERF-2 is covered by existing fdu-x16g (#54 renumbered its artifact to exp-103 in 86d2a6a). No local Rust builds on this host (disk below the build floor); CI is the gate.

## Notes

All eleven findings have code or documentation dispositions on PR #52 through 8640758 (BUILD-3 correction 8640758 follows e3b5103). PERF-2 is tracked on fdu-x16g; PERF-7's re-capture with binary hashes on fdu-0q6w; the head-engine requirement for final checks on fdu-lj4h. Verification so far: realtree harness 229 tests, provenance tests, admission checker tests, docs format, ledger and report drift checks all pass locally; cargo check and clippy -D warnings pass for fdu-core --all-targets --features gitignore,watch; an independent read-only review found nothing that would fail CI. Still open: no Rust test has executed and CI has not run, because GitHub runs no pull_request workflow while #52 conflicts with #51's moved base. The coordinator asked for #51 to be merged in, but the permission system denied the merge preview (git merge-tree) and a cargo clippy run on the fdu crate, so the merge was not attempted. Close this bead after the merge lands, CI is green, and the disposition map is posted.
