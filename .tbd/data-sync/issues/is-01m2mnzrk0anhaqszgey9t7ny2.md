---
type: is
id: is-01m2mnzrk0anhaqszgey9t7ny2
title: The fdu-transient-summary benchmark contract no longer reaches the transient tier
kind: bug
status: open
priority: 2
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-16T08:40:28.503Z
updated_at: 2026-09-16T08:40:28.503Z
---
explorations/benchmarks/realtree/compare_tools.py: the fdu-transient-summary contract's argv is '--cache off --view summary --format json --color never ROOT'. Since #65 a default run observes .gitignore, so execution::plan_report's summary_is_sufficient is false and that request takes RetainedState::FullIndex, the same plan as fdu-index-summary. The two contracts now measure the same work while declaring different work classes, and the README's 'five-tally exact summary' row describes a tier the argv no longer reaches. Either add --no-gitignore to the transient contract's argv, or retire the contract and say the tier is reached only with that flag. Found while re-measuring the README headline for fdu-y5xr.
