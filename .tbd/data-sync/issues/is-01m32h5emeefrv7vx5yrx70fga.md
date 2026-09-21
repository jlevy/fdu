---
type: is
id: is-01m32h5emeefrv7vx5yrx70fga
title: Systematic two-comment review pass over every open PR (2026-09-21)
kind: epic
status: open
priority: 0
version: 10
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
child_order_hints:
  - is-01m32h6bvgxbks1d5t5q5p0k44
  - is-01m32h6cjrkz58ce2t05m3n2v7
  - is-01m32h6d4szbwm3n1sgxqyfn9x
  - is-01m32h6dpd97fr5f8db831dn3y
  - is-01m32h6ea5cf5dd7rzf7ea0jdh
  - is-01m32h6ewht09ryvnt0f5zmhz5
  - is-01m32h6fe9mfa8b5exwsdcb62h
  - is-01m32h6g0hprgkrq1m6e8rx0f6
  - is-01m32h6gjn35c13axesr7bp753
created_at: 2026-09-21T17:45:34.093Z
updated_at: 2026-09-21T17:46:08.853Z
---
Every open PR gets the same treatment, in this order, so that two comments on a PR mean it has been fully covered:

1. Up to date with `main` first. Done 2026-09-21: `main` at `c7babf76` (after #107 merged) was merged into all nine branches bottom-up through the stack — level 1 onto `main`, then #96 and #97 onto #94, then #103 onto #96 and #105 onto #97. All nine merged clean and were pushed. Merge, never rebase: committed evidence cites SHAs, and rebasing has orphaned measured commits in this repository before.
2. A senior review per `tbd shortcut review-github-pr`, posted as a comment in the artifact format from `tbd shortcut pr-review-workflows`: scope, summary and textual verdict, numbered findings with stable IDs and severities and a concrete Fix, suggestions, false positives, CI status.
3. A separate agent addresses that review per `tbd shortcut address-pr-review`, tracking each finding as a bead and fixing, rebutting or explicitly deferring it, then posting the per-finding disposition as the second comment.

Reviewing and addressing stay decoupled on purpose; the published review is the handoff.

Model policy for this pass: Opus for regular work and for reviews of plan or documentation layers; Fable wherever a subtle bug could hide — engine code, cache and identity logic, allocation and performance guards, and evidence integrity, all of which have produced wrong-but-green results in this repository already.

Stack, with review scope being the layer only:

    main
    ├── #94  perf/campaign-linux-2026-09-19
    │   ├── #96  codex/directory-query-plan  ──> #103 codex/directory-rollup-query
    │   └── #97  cursor/linux-perf-iterate   ──> #105 cursor/linux-sidecar-load
    ├── #98  codex/release-windows-validity
    ├── #99  codex/release-ignore-correctness
    ├── #104 cursor/review-leftovers-de1b
    └── #108 claude/gate-integrity

A finding about lower-layer code belongs on that lower PR, and a blocker on a lower layer blocks everything above it.
