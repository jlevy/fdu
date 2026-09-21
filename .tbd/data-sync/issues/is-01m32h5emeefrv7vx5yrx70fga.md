---
type: is
id: is-01m32h5emeefrv7vx5yrx70fga
title: Systematic two-comment review pass over every open PR (2026-09-21)
kind: epic
status: open
priority: 0
version: 11
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
updated_at: 2026-09-21T20:57:50.627Z
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

## Notes

STATE 2026-09-21 ~20:45 UTC.

Merged to main (11a6dc31): #108 (two gate fixes + correctness runbook) and #104 (the four #91/#92 review leftovers).

Step 1 complete and independently verified, twice, after concurrent pushes by parallel agents: all seven open-PR branches contain merged main, carry the #107 supply-chain fix, and are free of the inert H138 guard. Verified by `git merge-base --is-ancestor origin/main origin/<branch>` against origin, not by a script's own report — the first cascade script printed "pushed" for branches it had not updated, which is the same defect class as everything else found this session.

Step 2 complete: senior review posted on all eight PRs (#94 #96 #97 #98 #99 #103 #104 #105 #108).

Step 3: dispositions posted for #94, #97, #104, #108. In flight for #96+#103 (paired, because #103 implements #96's spec) and #98+#99.

Also in flight: the evidence validator on claude/evidence-validator, committed locally as 36048330 "perf: validate every experiment record against its own measurements", gating before push.

## Open blockers on unmerged PRs

- #98 R1/R2 (both Blocker, both reachable from an ordinary `fdu C:\`): a zero FILETIME — what FAT/exFAT report for ChangeTime — computes below i64::MIN and becomes a hard I/O error; and the hand-rolled CreateFileW lost std's ERROR_SHARING_VIOLATION fallback, so pagefile.sys and hiberfil.sys (10-40 GB) drop out of totals.
- #103 R1 (Blocker): --format paths escapes backslash, which on Windows is the separator, so it prints paths that do not exist — and the new cli-axes golden uses [JSON_SEP] inside TEXT lines, so it matches the doubled separator instead of failing on it. That is why the Windows round went green.
- #96 R1 (Blocker): subtree size and age have no defined meaning over an incomplete subtree, so a directory whose recent activity sits in an unwalked descendant reads as old and matches --modified-before 30d — in a workflow the plan itself ends with a deletion. Needs a product decision (unknown, or a labelled lower bound), not an agent's invention.

## Still owed beyond the dispositions

- `make check` has never run on #94, #96, #97, #98, #99, #103 or #105. CI is green on all of them, but CI is not the gate this project trusts; that is the whole point of fdu-vjf2. About 25 minutes each on four contended CPUs.
- #98, #99 and #103 are drafts marked "do not merge or release yet" — undrafting is a release decision.
- Merging in stack order once green: #94, then #96 and #97, then #103 and #105.
- macOS phase (fdu-q098) cannot be done from this container at all.

## Coordination hazard, seen repeatedly

Parallel agents are pushing to these same branches. They performed #97's and #105's merge-downs before this session could, so two redundant merge commits were discarded rather than layered. One of them silently swapped `blind(CachePolicy::Off)` for `OpenConfig::default()` (which is Auto) in a cache test; #97's addressing agent caught and re-pinned it.
