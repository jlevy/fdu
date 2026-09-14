---
type: is
id: is-01m2esgrngd4vjkprfqa8xq2yg
title: Re-run the Linux floor scoreboard with the fixed harness; recorded numbers predate the FLOOR fixes
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-14T01:46:44.783Z
updated_at: 2026-09-14T01:47:11.279Z
---
Follow-up on PR #49, recorded by its fixer. The Linux half of fdu-33ri.

**What is stale.** `docs/project/reports/report-2026-08-28-linux-floor-scoreboard.md` and PR #49's description record ×floor numbers for `/usr` and `/opt`: aggregate 1.58× / 1.40×, index 2.82× / 2.93×, `arena_spike` 3.92× (bimodal) / 1.09×. These came from a 4 vCPU Linux container with the harness as first committed (1fa2309). The review found thirteen harness defects, all since fixed, and several change what those numbers mean:
- FLOOR-1: fdu ran its automatic pool, not the floor's thread count. It coincided only because the host had 4 cores.
- FLOOR-2: a fixed instrument rotation, so `arena_spike` always followed `parfloor-enum`. The bimodality attribution is untested.
- FLOOR-3 and FLOOR-7: quiet was checked once, not per trial.
- FLOOR-4 and FLOOR-8: a spread-flagged tier could read as closed.
- FLOOR-6: "entries" was dirs+files, not the `perf-subjects` definition, and `/opt`'s entry count under that definition was never recorded.

The report carries a "Review addendum 2026-09-13" saying what the first harness did not control, but no numbers from the fixed harness exist.

**To do.**
1. On a Linux host, re-run `make perf-floor` with the fixed harness (6e018a0 or later) on `/usr`, and on `/opt` or a nominated subject.
2. Record entry counts under the `perf-subjects` definition, the carryover-balanced schedule, per-trial quiet validity, and spread flags.
3. Replace or supplement the report's table, and say whether the aggregate reproduction (1.58× against the by-hand 1.59×) and the index tier's 2.8–2.9× still hold.
4. Re-test the predecessor hypothesis for `arena_spike`'s bimodality now that the schedule is balanced.

**Sequencing.**
- Check the load-average quiet gate first (fdu-hw7f), or a long run may downgrade itself to `uncontrolled`.
- If #52 has landed, also check fdu-w9ug (the index tier may read `.gitignore`) before trusting the index ratio.

Review: https://github.com/jlevy/fdu/pull/49#pullrequestreview-5192251516
