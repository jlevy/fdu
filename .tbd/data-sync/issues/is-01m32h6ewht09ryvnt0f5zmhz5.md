---
type: is
id: is-01m32h6ewht09ryvnt0f5zmhz5
title: "PR #103 review pass: directory filters and list presentation formats"
kind: task
status: in_progress
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m32h5emeefrv7vx5yrx70fga
created_at: 2026-09-21T17:46:07.120Z
updated_at: 2026-09-21T17:50:21.873Z
---
Branch `codex/directory-rollup-query`, based on #96. Reviewer model: Fable.

State to reach, both as comments on the PR:
1. Senior review per `tbd shortcut review-github-pr`, in the artifact format (scope, verdict, numbered findings with severities and a concrete Fix, suggestions, false positives, CI status).
2. Per-finding disposition per `tbd shortcut address-pr-review` — each finding fixed, rebutted, or explicitly deferred.

Merged up to `main` at c7babf76 on 2026-09-21 before review, so the diff reviewed is the one that would merge.

Review scope is this layer only; a finding about lower-layer code belongs on the lower PR.

## Notes

Senior review posted 2026-09-21: https://github.com/jlevy/fdu/pull/103#issuecomment-5764980291
Verdict CHANGES NEEDED. R1 Blocker: --format paths and --long escape backslash, so every nested path on Windows prints a doubled separator — and the new cli-axes golden uses [JSON_SEP] inside TEXT lines, so it matches the doubled separator instead of failing on it. That is why the Windows round went green. R2 High: size/time predicates without --kind now cover whole subtrees, so --modified-since in the default view changed meaning, undocumented. R3 Medium: the work-budget test cannot distinguish any pricing formula from any other. R4/R5 Low. Engine logic otherwise sound; three new guards verified by mutation; every stale consumer found. Addressing stage owed.
