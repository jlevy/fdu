---
type: is
id: is-01m2ety242k5f9vh469xm5pq2d
title: "Make the PR stack and #49 merge-ready together, with CI green on every PR"
kind: task
status: open
priority: 0
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T02:11:29.025Z
updated_at: 2026-09-14T02:11:29.025Z
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
