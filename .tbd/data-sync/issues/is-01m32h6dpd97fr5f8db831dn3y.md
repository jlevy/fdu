---
type: is
id: is-01m32h6dpd97fr5f8db831dn3y
title: "PR #98 review pass: validate Windows caches with change time and file identity"
kind: task
status: in_progress
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m32h5emeefrv7vx5yrx70fga
created_at: 2026-09-21T17:46:05.901Z
updated_at: 2026-09-21T17:50:20.904Z
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
