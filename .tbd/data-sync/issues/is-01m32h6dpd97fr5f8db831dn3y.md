---
type: is
id: is-01m32h6dpd97fr5f8db831dn3y
title: "PR #98 review pass: validate Windows caches with change time and file identity"
kind: task
status: closed
priority: 1
version: 12
labels: []
dependencies: []
parent_id: is-01m32h5emeefrv7vx5yrx70fga
child_order_hints:
  - is-01m32twvdtzccksj9ema19j93r
  - is-01m32tww1snf1n9w7dnqkv98s5
  - is-01m32twwnydz2mzfcr2aggmq7p
  - is-01m32twxbcecbbxabqwq19dz44
  - is-01m32wbb44bq2n90767gz6yn4n
  - is-01m32wbbwban5x8k14zwshwr9f
  - is-01m32wbcf7yb2afmpdrs2t3d9h
  - is-01m32wbd25xjxnmqp90nx64hys
created_at: 2026-09-21T17:46:05.901Z
updated_at: 2026-09-21T21:19:13.341Z
closed_at: 2026-09-21T21:19:13.341Z
close_reason: "Both comments are on the PR: senior review at https://github.com/jlevy/fdu/pull/98#issuecomment-5764966314 and the per-finding disposition at https://github.com/jlevy/fdu/pull/98#issuecomment-5767664651. R1, R2 (Blockers), R3 and R4 fixed in 650b6b08 + edd6625e + b04c957c on codex/release-windows-validity; the four suggestions deferred as fdu-1zrz, fdu-m3x5, fdu-hrjz, fdu-twry (still open under this bead). CI 19/19 green on b04c957c. FAT/exFAT, locked-file and ReFS behaviour remain unverified on a real Windows host, as the comment states."
resolution: null
duplicate_of: null
---
Branch `codex/release-windows-validity`, based on main. Reviewer model: Fable.

State to reach, both as comments on the PR:
1. Senior review per `tbd shortcut review-github-pr`, in the artifact format (scope, verdict, numbered findings with severities and a concrete Fix, suggestions, false positives, CI status).
2. Per-finding disposition per `tbd shortcut address-pr-review` — each finding fixed, rebutted, or explicitly deferred.

Merged up to `main` at c7babf76 on 2026-09-21 before review, so the diff reviewed is the one that would merge.

Review scope is this layer only; a finding about lower-layer code belongs on the lower PR.

## Notes

Senior review posted 2026-09-21: https://github.com/jlevy/fdu/pull/98#issuecomment-5764966314
Verdict CHANGES NEEDED. R1 Blocker: zero FILETIME becomes an I/O error, so FAT/exFAT volumes fail outright (windows_metadata.rs:142-148). R2 Blocker: the locked/denied-file fallback std provided is gone, so fdu C:\ drops pagefile/hiberfil and reports partial. R3 Medium: the 64-bit file index is not unique on ReFS (Dev Drive). R4 Medium: the MSRV job is ubuntu-only so it cannot see this module. Addressing stage owed.
