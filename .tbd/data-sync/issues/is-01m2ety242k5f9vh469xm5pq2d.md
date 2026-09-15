---
type: is
id: is-01m2ety242k5f9vh469xm5pq2d
title: "Make the PR stack and #49 merge-ready together, with CI green on every PR"
kind: task
status: closed
priority: 0
version: 6
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T02:11:29.025Z
updated_at: 2026-09-15T17:13:03.929Z
closed_at: 2026-09-15T17:13:03.926Z
close_reason: Superseded by fdu-gjc2; the first stack merged 2026-09-14 (7cc5554, dda7e6a), and the follow-up stack merged 2026-09-15 (5f03062, 8856b4d, 3373134).
resolution: null
duplicate_of: null
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

2026-09-13 22:15 PDT checkpoint:
- All six PRs are 19/19 green and CLEAN: #48 d48b8f8, #49 6e018a0, #50 6e94e6b, #51 5d60aba, #52 ddd2b7d, #54 e77b1d9.
- Every PR is reviewed through its head. Delta reviews: #48 5194005473 (engine) and 5194007815 (surfaces), #49 5193948340, #51 5193989647, #52 5193989948, #54 5193949483. All say safe to merge.
- Findings from the delta reviews are filed as beads: fdu-2q40, fdu-k2l5, fdu-a7un, fdu-c5kn, fdu-tp2p, fdu-ayzg, fdu-cvx1, fdu-2o2r, plus a note on fdu-hw7f.
- Each PR description now opens with a Merge readiness section.
Still owed, all blocked on host disk (3.5 GiB free):
- the lifecycle fixes, checkpoint a74137f in worktree agent-a365bdbfabb1cd605;
- fdu-uzzv;
- make check and cross-lint on the combined tree.

2026-09-14 DECISION (user): merge only after checks. Once the user empties Trash, run make check (and cross-lint) on the combined tree plus a quick interleaved timing of `fdu PATH` against main, on one tree with no .gitignore files and one with many. If the stack is within 10% of main on both, merge; the strict 3% quiet-host gate stays open on fdu-lj4h. If it is slower than that, stop and bring the user the numbers and a profile. The lifecycle Lows (a74137f), fdu-uzzv, fdu-2q40 and fdu-k2l5 go into one follow-up PR on main.

2026-09-14 MERGED. Pre-merge checks, both passed:
- Speed, fdu --cache off, 15 interleaved pairs, busy host. Stack/main median pair ratio: 0.962 on ~/.rustup/toolchains (72.5k entries, 0 .gitignore); 0.971 on ~/wrk/github/metabrowser (128.5k entries, 48); 0.994 on metabrowser-release-perf (580k entries, 286). Totals were identical on all three trees.
- make check (UV_PYTHON=3.12) and make cross-lint passed on the combined tree 8b35e28. The first run failed only because this host's uv picked a free-threaded 3.14 for python-smoke; see the bead filed for that.
- Merge simulation in both orders: clean, same tree.
Merged: stack #53 via gh stack merge --merge as 7cc5554, then #49 as dda7e6a. main's tree equals the checked tree 8b35e28. All evidence SHAs, including the perf/h86-* tags, are reachable from main.
Follow-up PRs on main: the lifecycle Lows, fdu-uzzv, fdu-2q40 and fdu-k2l5, plus the decisions recorded on fdu-agb6, fdu-l89e, fdu-8w5k, fdu-r3j4 and fdu-jilk.
