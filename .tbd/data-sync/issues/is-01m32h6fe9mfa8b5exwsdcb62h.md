---
type: is
id: is-01m32h6fe9mfa8b5exwsdcb62h
title: "PR #104 review pass: restore #91/#92 review leftover guards and evidence honesty"
kind: task
status: in_progress
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m32h5emeefrv7vx5yrx70fga
created_at: 2026-09-21T17:46:07.689Z
updated_at: 2026-09-21T18:12:26.540Z
---
Branch `cursor/review-leftovers-de1b`, based on main. Reviewer model: Fable.

State to reach, both as comments on the PR:
1. Senior review per `tbd shortcut review-github-pr`, in the artifact format (scope, verdict, numbered findings with severities and a concrete Fix, suggestions, false positives, CI status).
2. Per-finding disposition per `tbd shortcut address-pr-review` — each finding fixed, rebutted, or explicitly deferred.

Merged up to `main` at c7babf76 on 2026-09-21 before review, so the diff reviewed is the one that would merge.

Review scope is this layer only; a finding about lower-layer code belongs on the lower PR.

## Notes

Senior review posted: https://github.com/jlevy/fdu/pull/104#issuecomment-5765152906 (approve; 0 Blocker, 0 High, 1 Medium, 2 Low).
Disposition posted: https://github.com/jlevy/fdu/pull/104#issuecomment-5765270437 — all three addressed in ba3efb28.

R1: the always-share detection rested on 8 allocations (bounded view 2055 correct, 3080 under always-share, bound FILES*3 = 3072). Replaced with a stated FILES*2 + FILES/2 = 2560, about 500 clearance each side. Both faults still caught by mutation; 20/20 parallel runs clean. Also corrected the neighbouring 'allocations < types' caption, which claimed to bound one full-tree walk but cannot detect always-share because types carries ~3000 aggregation allocations.
R2: the subtraction in the sharing guard closes a cost paid by every report, not one paid only by viewed reports; a never-share build spending FILES more per viewed report passes at a measured saving of 1102. Stated in the comment rather than implied.
R3: 'post-H112' named the wrong change — H112 is exp-109 itself, measured with the wide bucket; the narrow apply-only bucket arrived in e667b739. Corrected in content_cache.rs and performance-loop.md, and exp-120 now carries the erratum exp-124 already had.
