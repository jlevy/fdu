---
type: is
id: is-01m2esgsrjvv641ntm9mc8zfxk
title: "After #49 and #52 both land, verify the floor's index tier is still like-for-like"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2ebb3axt3ktj0rqkj0pw7bt
created_at: 2026-09-14T01:46:45.905Z
updated_at: 2026-09-14T01:46:45.905Z
---
Semantic check needed once PR #49 and PR #52 are both on main. Neither PR's CI can see this, because each was verified without the other.

**The interaction.**
- #49's FLOOR-5 fix (3728368): `make perf-floor` depends on `perf-probe-release` and passes that binary to `floor.py` with `--probe "$(PERF_RELEASE)"`, rather than building its own `--no-default-features` probe.
- #52's PERF-5 and fdu-by5y: `perf-probe-release` builds `perf_probe` with `--features gitignore`, so shipped control semantics are compiled in.
- After both land, the floor's probe has control observation compiled in. Before, #49 measured a probe built without `gitignore`.

**Why the ratios could stop being like-for-like.**
- The aggregate tier (`summary`) is a one-shot report. After #51 the planner forces `read_controls: false`, so it should still read no control files.
- The index tier (`scan-index`, the cold index scan) does not go through the report planner. It appears to take `ScanConfig` defaults, where `read_controls` defaults on; merge 753e10f says `--no-controls` stopped mattering only for `default-tree` and `summary`. Unconfirmed.
- The index tier may therefore start opening, parsing, and retaining every `.gitignore` under the subject. `parfloor` and `arena_spike` never read file contents. The ×floor index ratio would then include control-file I/O the denominators do not do, and #49's 2.82–2.93× figures (measured without it) would not be comparable with post-merge runs.

On `/usr` the effect may be small (few `.gitignore` files). On a source-tree subject it is not.

**To do after the combined merge.**
1. Confirm which probe modes observe controls: run `perf_probe scan-index` and `summary` on a fixture with `.gitignore` files, and check the retained control count or counters.
2. Decide what the floor's index tier should measure: the shipped `open`/scan default (controls on, then record it in the scoreboard as a regime difference from the floor), or pass `--no-controls` to match the floor's work.
3. Record the choice in `floor.py`'s docstring and the scoreboard JSON.
4. Re-run the floor on at least one subject with control files to show the effect.

**Also at merge time.** #49 and #52 edit the same `Makefile` lines around `perf-probe-release` and `.PHONY` (#49's description; fdu-266y). Resolve so `perf-floor` keeps depending on `perf-probe-release` and inherits `--features gitignore` deliberately, not by accident.

Reviews: https://github.com/jlevy/fdu/pull/49#pullrequestreview-5192251516 and https://github.com/jlevy/fdu/pull/52#pullrequestreview-5192264318
