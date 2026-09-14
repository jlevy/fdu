---
type: is
id: is-01m2ety242k5f9vh469xm5pq2d
title: "Make the PR stack and #49 merge-ready together, with CI green on every PR"
kind: task
status: in_progress
priority: 0
version: 2
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T02:11:29.025Z
updated_at: 2026-09-14T03:41:05.804Z
---
The user's goal, set 2026-09-13: the whole stack merges together, ready to merge, with CI passing on every PR. This bead tracks the process; each fix has its own bead.

Stack: main <- #48 codex/opened-root-inventory-rewrite <- #50 claude/control-table-scale-fix <- #51 claude/one-shot-commit-cost <- #52 codex/streaming-performance-parity <- #54 claude/h86-linux-evidence-stage. It is formally linked as GitHub stack #53 by `gh stack link 48 50 51 52 54`. #49 claude/perf-floor-linux-2026-08-28 stands alone on main.

Steps:
1. Fix wave, in flight:
   - #48: FIX48-1/2/3/5/6 and fdu-k18s; fdu-s0xg, fdu-m5zj, fdu-eupp; fdu-2wlp, fdu-m6n3, fdu-qlxz.
   - #51: fdu-wjdk.
   - #52: fdu-8yv0, fdu-tilu, fdu-z0vu.
   - A read-only triage of the older P0-P2 beads decides what else is live.
2. One propagation pass, by merge only, #48 -> #50 -> #51 -> #52 -> #54, with CI green at each step.
3. Readiness pass:
   - Mark #52 ready for review.
   - Add a "Merge readiness" section to each description.
   - Simulate the merge in both orders.
   - Run make check and make cross-lint on the combined tree.
4. The user merges bottom-up with "Create a merge commit", never rebase-merge, or with gh stack merge.

Acceptance: every PR is MERGEABLE/CLEAN with all checks green at its final head; make check passes on the combined tree; every remaining item is an open stack-followup bead.

## Notes

2026-09-13 (night) checkpoint: all six PRs are green (19/19) and MERGEABLE/CLEAN.
- Heads: #48 0a2e341, #49 6e018a0, #50 aa397f7, #51 0edd917, #52 a79dd6d, #54 71a2632.
- #52 is marked ready for review.
- The final propagation merged #48 into #50, #51, #52 and #54. It resolved a plan-table conflict and the opened.rs signature conflicts. It also fixed a compile break inside merge fee0f41 (b803b8e used entry.children_complete; #52 moved that field behind Entry::directory()) and the watch-scope docs (a79dd6d).
- Nothing was compiled locally, because free disk was 3.4-4.7 GiB. Stacked CI carried all of it; each PR's CI builds it together with the PRs below it.

Still in flight:
- A small follow-up fixer on #48: fdu-ujsa, fdu-bqan, fdu-224p, fdu-kaog, fdu-k7lj.
- Lifecycle fixes fdu-dkr0/jxuq/lfiy/8jp0 exist only as an unpushed local checkpoint, a74137f in worktree agent-a365bdbfabb1cd605. They are blocked on disk space and need a build, tests, and a re-record of the opened-root goldens.
- fdu-uzzv (P1) has not started.

Readiness still owed: make check and make cross-lint on the combined tree (blocked on disk), the merge simulation with #49, and a "Merge readiness" section in each description.
