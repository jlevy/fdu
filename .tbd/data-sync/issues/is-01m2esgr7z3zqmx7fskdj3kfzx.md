---
type: is
id: is-01m2esgr7z3zqmx7fskdj3kfzx
title: Linux quiet gate may count the benchmark's own workers in the one-minute load average
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-14T01:46:44.351Z
updated_at: 2026-09-14T01:46:44.351Z
---
Suspected defect in the quiet gate PR #49 now relies on, recorded by the #49 fixer. **Unverified.**

**Mechanism.** FLOOR-3's fix (bd644bf) holds every floor trial to `measure._host_pressure_snapshot` and `measure._host_pressure_reasons` before and after (`explorations/benchmarks/realtree/floor.py:534-538` on claude/perf-floor-linux-2026-08-28). Any breaching trial is invalid, and any invalid measured trial downgrades the whole document to `uncontrolled`. On Linux the quiet test is the one-minute load average: `load_1m_per_cpu <= QUIET_MAX_LOAD_PER_CPU` (0.25), in both `_host_regime` and `_host_pressure_reasons` (`measure.py:77, 506-515, 728-736`). The floor runs every instrument with a fixed pool of N workers (FLOOR-1), N defaulting to the process CPU count. That pool's runnable threads enter the one-minute load average and decay over about a minute. So once a run is under way, the benchmark's own previous trials can hold load/core above 0.25, and later trials breach on pressure the harness created.

**The code already makes this argument for macOS.** `measure._darwin_cpu_busy_pct`'s docstring (`measure.py:605-613`) says a fixed-N benchmark "creates runnable workers by design" and that load average "cannot distinguish later background pressure from the benchmark's own previous samples". That is why macOS uses a one-second Mach CPU-tick delta instead. Linux has no equivalent path.

**Expected symptom.** A long `make perf-floor TRIALS=30` on an idle Linux host downgrades itself to `uncontrolled` with "quiet-host load/core exceeded 0.250 before the sample" reasons. The 180 s `--quiet-wait` only applies before each subject.

**To do.**
1. Verify on a Linux host. Run a 30-trial floor on an idle machine and record how many trials breach, and at which ordinals.
2. If confirmed, give Linux an instantaneous CPU-busy measurement: a `/proc/stat` delta over a short boundary interval after the child exits, mirroring `_darwin_cpu_busy_pct`. Gate on it, and keep load averages as recorded context. This also affects `measure.py`'s own quiet regime for `perf-compare` on Linux.
3. Add a test with a synthetic snapshot sequence showing a self-induced load tail does not invalidate a trial.

Review: https://github.com/jlevy/fdu/pull/49#pullrequestreview-5192251516. Disposition: https://github.com/jlevy/fdu/pull/49#issuecomment-5656540687
