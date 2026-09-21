---
type: is
id: is-01m31hc3p7wv76jeq5dhgv3bd8
title: "HANDOFF: stabilization state, owed work, and order of operations (2026-09-21)"
kind: epic
status: open
priority: 0
version: 10
labels: []
dependencies: []
child_order_hints:
  - is-01m31hcmn5xyxv08q0a8gf4rcs
  - is-01m31fbmpgajer03wejhrby3qm
  - is-01m2y7cf9fsawdtq8p5grnr3nq
  - is-01m2zpkypg9xvwkfy54mah4gxn
  - is-01m2zpkwa48ecgw720rkgrs9wb
  - is-01m2zpkzxm97et6bdxeac7eh6w
  - is-01m2zpkxdtb9twaj4bxhegejc5
created_at: 2026-09-21T08:29:57.830Z
updated_at: 2026-09-21T08:32:10.447Z
---
Single entry point for the next agent. Read this first; it says what is true, what is owed, and what to do in what order.

## State as of 2026-09-21

`main` is at `a290aedc` and carries PRs #91 and #92 (merge commits `6e3d2937`, `a290aedc`). Both merge commits are 19/19 green. The merged tree is byte-identical to the `937f9445` that passed a full local `make check` and `make cross-lint` on macOS, verified by trial merge before merging. The path-independence **full matrix** was then dispatched against merged `main` (run 35545852613) and passed on ubuntu, macOS and Windows. That workflow never runs on push — only on schedule, `workflow_dispatch`, or a PR labeled `path-independence-full` — so dispatch it deliberately after engine changes land.

Open PRs: #94 (Linux validation, on `main`), #97 (on #94), #96 (directory plan, on #94), #103 (directory implementation, on #96, green, draft), #98 and #99 (correctness drafts, on `main`).

## The one thing that blocks everything else

`fdu-vjf2` / GitHub #106: `make check` stops at `supply-chain` on any clone with a git worktree under `.claude/worktrees/`. There is currently **no working local gate** on such a host, and the error names a hook file, so it reads like a real supply-chain violation rather than a scanning bug.

This is not theoretical. Four real defects in #103 reached CI undetected because of it: a stale cross-surface format contract (all platforms), a Windows-only path-separator test failure, a stale report-shape consumer, and drifted Linux parity recordings. Fix this first, or work on a host with no nested worktree. Do not add nested paths to the inventory, and do not delete other worktrees to make it pass — several hold uncommitted work.

## Owed on the merged work

Independent pre-merge verification of #91 and #92 (both returned MERGE; the R3 fail-open was reproduced against pre-fix code and each defence confirmed independently sufficient) left four follow-ups. None blocks anything; all are evidence or test-guard integrity:

- `fdu-y050` — exp-116's `verdict.change_pct` is a cross-job ratio (-99.9%) where the schema wants the paired effect (-2.2%); plus a false "engine digest unchanged" claim, inconsistent uncontrolled-regime labels, and exp-124/137 errata. Needs `make perf-ledger` then `make perf-report` in the same commit.
- `fdu-iajs` — the fix-consolidation merges dropped R3 name negatives and the H138 sharing guard. Flipping `row_consumers > 1` to never share currently passes every test.
- `fdu-2pct` — restore timers no longer span the whole sidecar load. **Do this before judging H121**, whose rule is a stage at >=50% of restore phase time; the baseline it would be compared against (exp-109's "apply 63.3%") was measured when the timer wrapped the whole loop.
- `fdu-e8u0` — the evidence report prints a `peak_rss_bytes` primary metric as milliseconds.

`fdu-y050` and `fdu-e8u0` were deliberately deferred past the merge: both need the same two generated files that #92 rewrites, so fixing them on #91 would have meant generated-file conflicts and a fresh CI round on both PRs for no change in the published result. One regeneration on merged `main` is equivalent and cheaper.

## Owed on #103

Green on all 19 CI checks at `f885d53e`, but no host has run `make check` or `make cross-lint` on it. Mark it ready once the gate runs somewhere clean. Verification bead is `fdu-arv8`; epic is `fdu-65x1`.

Decided, not open: the metadata default changed shape (`reports[0].tree` became `files` plus `bound`) while the schema string stays `fdu.report/7`. A consumer pinned to that version breaks silently, which is how `smoke.py` failed. This is intentional, documented in `docs/machine-output.md`, and accepted because fdu is pre-1.0 with no external consumers. Recorded so it is visible rather than implicit; do not "fix" it by bumping the version without asking.

## Order of work

1. `fdu-vjf2` — restore a working local gate.
2. `fdu-arv8` — full gate on #103, then mark ready.
3. `fdu-2pct` — before any H121 judgement.
4. `fdu-y050`, `fdu-e8u0` — one regeneration commit on `main`.
5. `fdu-iajs` — restore the dropped guards.
6. Linux work: see the child beads.

## Conventions worth knowing before you start

- Fix at the lowest branch where a defect originates; bring branches up by merge, never rebase or force-push. Committed evidence cites SHAs, and rebasing has already orphaned measured commits once in this repository.
- `tryscript run --update` writes what it saw, expanding named patterns into literals; `make check` has a portability check that refuses those.
- The parity artifact is recorded by CI on Linux, not locally — it holds platform-dependent values, so a local recording cannot falsify itself. Take it from the CI artifact.
- Stacked PRs cannot be merged with `gh pr merge`; use `PUT /repos/{owner}/{repo}/pulls/{n}/merge-async` with the body passed via `--input`, and poll for the result.

## Notes

IMPORTANT UPDATE, same day: parallel agents opened PRs covering most of the 'owed' list, so re-read before starting anything.

PR #104 (cursor/review-leftovers-de1b -> main) addresses ALL FOUR review follow-ups: fdu-2pct (apply timer starts at candidates.remove so the four restore rows sum to the sidecar load; clear_content documented; notes H121's exp-120 used the narrower post-H112 bucket while exp-109's 63.3% wrapped the whole loop), fdu-iajs (R3 name negatives restored, a valid-rename load control added, an H138 sharing allocation guard added so flipping row_consumers > 1 now fails, dead update_rollups = false branch removed), fdu-e8u0 (evidence table formats peak_rss_bytes as MiB; exp-117 reads 377.5 -> 339.4 MiB), and fdu-y050 (exp-116 change_pct is the paired -2.158% with the 1,630x figure kept in reason; commit corrected to 984e4618 and quoted so YAML does not read it as infinity; digest claims reduced to 'same shape'; uncontrolled labels aligned; dated errata added).

PR #105 (cursor/linux-sidecar-load-de1b -> cursor/linux-perf-iterate-de1b) covers fdu-2pct, fdu-78q6 and fdu-nszx: Linux sidecar restore mix after the leftover apply-timer expansion.

So the order of work in this bead's body is stale for items 3, 4 and 5. Review #104 and #105 rather than reimplementing them. Item 1 (fdu-vjf2, GitHub #106) and item 2 (fdu-arv8, the full gate on PR #103) remain untouched and are still the first things to do.
