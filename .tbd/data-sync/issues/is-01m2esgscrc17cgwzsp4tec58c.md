---
type: is
id: is-01m2esgscrc17cgwzsp4tec58c
title: "Consider --no-oracle for the floor's index tier once #52 lands"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-14T01:46:45.527Z
updated_at: 2026-09-14T01:46:45.527Z
---
Deferred suggestion from PR #49 review FLOOR-5 ("Consider `--no-oracle` for the index tier after #52"). fdu-266y, closed, recorded it as not applicable on #49's branch because the flag lives on the #52 stack. Recorded by the fixer as "once #52 lands".

**Why it matters.** PR #49 measured the index tier's spawn wall at 149 ms above its own 110 ms component: 57% of a spawn-timed index number is the probe's engine-digest oracle. Callgrind attributes 43.17% of instructions to `Sha256::digest`. The aggregate tier's `tallies` oracle costs about 3%, so the two tiers are unequally contaminated. `floor.py` already times the probe's internal `component_ns`, not the spawn wall, so the ×floor ratio is not directly skewed. But RSS, `harness_overhead_ns`, and any spawn-timed reading still include the oracle. fdu-4xtm (closed) added `--no-oracle` to `perf_probe` on the #52 stack.

**To do once #52 is on main.**
1. Check whether `perf_probe scan-index --no-oracle` still emits the tallies (`dirs`, `files`, `apparent_bytes`, `allocated_bytes`) that `floor.py`'s shared tally oracle needs. fdu-4xtm's close note says timing rejects attribution-only output, so the flag may drop exactly what the floor's correctness check reads.
2. If the tallies survive, pass `--no-oracle` to the index tier in `floor.py`, and record in the scoreboard JSON that the engine digest was not computed.
3. If they do not, either add a tallies-only oracle mode to the probe (five integers, as the aggregate tier has), or record why the index tier keeps its digest.
4. Re-run and report how far RSS and harness overhead fall.

Review: https://github.com/jlevy/fdu/pull/49#pullrequestreview-5192251516
