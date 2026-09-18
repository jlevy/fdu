---
type: is
id: is-01m2mnzrk0anhaqszgey9t7ny2
title: The fdu-transient-summary benchmark contract no longer reaches the transient tier
kind: bug
status: closed
priority: 2
version: 4
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-16T08:40:28.503Z
updated_at: 2026-09-18T03:07:28.158Z
closed_at: 2026-09-18T03:07:28.158Z
close_reason: "Implemented on PR #87: SIGINT, registry READMEs, caret pins, version stamp/LF, 0.2 API note, 200ms interval, transient-summary --no-gitignore, python-smoke --python, JSON 2^53."
---
explorations/benchmarks/realtree/compare_tools.py: the fdu-transient-summary contract's argv is '--cache off --view summary --format json --color never ROOT'. Since #65 a default run observes .gitignore, so execution::plan_report's summary_is_sufficient is false and that request takes RetainedState::FullIndex, the same plan as fdu-index-summary. The two contracts now measure the same work while declaring different work classes, and the README's 'five-tally exact summary' row describes a tier the argv no longer reaches. Either add --no-gitignore to the transient contract's argv, or retire the contract and say the tier is reached only with that flag. Found while re-measuring the README headline for fdu-y5xr.

## Notes

Related harness gaps found by the doc-drift audit at 16efcd0 ("Documents that should exist" item 2), recorded here rather than as a duplicate bead:

- `PERF_TOOL_CONTRACT` defaults to `fdu-transient-summary` (Makefile:545), so `make perf-compare-tools` with no override records the mislabelled work class.
- The probe job `aggregate-summary` (explorations/benchmarks/realtree/measure.py:350) runs `summary --root {root}` with no `--no-controls`, so it takes the indexed plan. Its description (measure.py:353-357) says so, but the job name still reads as the aggregate tier.
- The transient tier is reachable in `measure` only as `--variant "name=PATH --no-controls"` (__main__.py:492-506). `make perf-compare` cannot add the flag to the candidate variant (Makefile:514), so a Make-driven aggregate round measures the indexed plan.

The docs PR for root, architecture, and guides describes the harness as it is and points here: docs/project/guides/performance-loop.md ("The aggregate tier", "Comparing against other tools"), performance-loop-runbook.md (MEASURE), and explorations/benchmarks/README.md now anchor on `fdu-index-summary`. When this bead lands, update those three places.

Also affected (found 2026-09-16 while updating plans, PR #71): make perf-floor's aggregate instrument, explorations/benchmarks floor.py, does not pass --no-controls either. Since PR #65 an unfiltered summary observes .gitignore and falls closed to the full index, so floor.py's 'aggregate' measures the full-index summary, not the aggregate tier, the same gap as the aggregate-summary job. Fix both together.
