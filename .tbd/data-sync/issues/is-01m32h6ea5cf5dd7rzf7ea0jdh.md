---
type: is
id: is-01m32h6ea5cf5dd7rzf7ea0jdh
title: "PR #99 review pass: match Git ignore edges and enforce test preconditions"
kind: task
status: closed
priority: 1
version: 6
labels: []
dependencies: []
parent_id: is-01m32h5emeefrv7vx5yrx70fga
child_order_hints:
  - is-01m32twy03j76e8cndxy0bftd2
  - is-01m32twymg6h17854zjfpjnq4v
created_at: 2026-09-21T17:46:06.533Z
updated_at: 2026-09-21T21:02:24.107Z
closed_at: 2026-09-21T21:02:24.107Z
close_reason: "Both comments are on the PR: senior review at https://github.com/jlevy/fdu/pull/99#issuecomment-5764971659 and the per-finding disposition at https://github.com/jlevy/fdu/pull/99#issuecomment-5767462066. R1 fixed (permission-bits preflight + AGENTS.md section), R2 fixed (wait helpers split by whether the watch is established; verified by mutation), the dead-arm suggestion fixed with R2. Commit bf8f6241 on codex/release-ignore-correctness; CI 19/19 green."
resolution: null
duplicate_of: null
---
Branch `codex/release-ignore-correctness`, based on main. Reviewer model: Fable.

State to reach, both as comments on the PR:
1. Senior review per `tbd shortcut review-github-pr`, in the artifact format (scope, verdict, numbered findings with severities and a concrete Fix, suggestions, false positives, CI status).
2. Per-finding disposition per `tbd shortcut address-pr-review` — each finding fixed, rebutted, or explicitly deferred.

Merged up to `main` at c7babf76 on 2026-09-21 before review, so the diff reviewed is the one that would merge.

Review scope is this layer only; a finding about lower-layer code belongs on the lower PR.

## Notes

Senior review posted 2026-09-21: https://github.com/jlevy/fdu/pull/99#issuecomment-5764971659
Verdict MERGE WITH NITS. Ignore semantics verified against git itself: 243 verdicts over 26 scenarios, 0 mismatches. Preconditions verified to fail when violated, by mutation. R1 Medium: after this lands, make check as root fails as ~13 scattered panics and AGENTS.md does not document the opt-out variables. R2 Low: a panic message offers an opt-out deliberately not honoured on that path. Addressing stage owed.
