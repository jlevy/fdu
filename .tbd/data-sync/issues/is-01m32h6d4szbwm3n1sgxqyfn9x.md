---
type: is
id: is-01m32h6d4szbwm3n1sgxqyfn9x
title: "PR #97 review pass: Linux leftover records, H147/H72 keeps, H148 PGO screen"
kind: task
status: closed
priority: 1
version: 10
labels: []
dependencies: []
parent_id: is-01m32h5emeefrv7vx5yrx70fga
child_order_hints:
  - is-01m32k0mjek79ygqhjk44q3tq7
  - is-01m32k0n64ggazv88j5j85pc04
  - is-01m32k0nrvh4fq4ngjykf20kmm
  - is-01m32k0pcemcwmbka07n7t6ncr
  - is-01m32k0pzxny6wp90q82wfz9ta
  - is-01m32k0qk5mdfh3vbby8zhaab7
  - is-01m32k0r5ranxnzq9ya6s6d202
created_at: 2026-09-21T17:46:05.337Z
updated_at: 2026-09-21T19:12:17.110Z
closed_at: 2026-09-21T19:12:17.109Z
close_reason: "Both states reached on PR #97: senior review posted as comment 5765288334; per-finding disposition posted as comment 5765812977 (second pass, head ac891f7a). R1-R6 all fixed (R3 SinkMode fixed here after the first pass deferred it), S1-S4 applied, S5 declined. make check exit 0 and CI 19 pass / 2 skip on ac891f7a. Children fdu-kgkn, fdu-2rxi, fdu-aksh, fdu-efhi, fdu-7twk, fdu-cgoi, fdu-47f1 closed."
resolution: null
duplicate_of: null
---
Branch `cursor/linux-perf-iterate-de1b`, based on #94. Reviewer model: Fable.

State to reach, both as comments on the PR:
1. Senior review per `tbd shortcut review-github-pr`, in the artifact format (scope, verdict, numbered findings with severities and a concrete Fix, suggestions, false positives, CI status).
2. Per-finding disposition per `tbd shortcut address-pr-review` — each finding fixed, rebutted, or explicitly deferred.

Merged up to `main` at c7babf76 on 2026-09-21 before review, so the diff reviewed is the one that would merge.

Review scope is this layer only; a finding about lower-layer code belongs on the lower PR.
