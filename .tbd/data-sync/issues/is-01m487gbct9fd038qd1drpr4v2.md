---
type: is
id: is-01m487gbct9fd038qd1drpr4v2
title: Land the history driver in the harness (benchmarks.realtree.history) with tests and a cell drift check
kind: task
status: open
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-10-06T09:07:53.874Z
updated_at: 2026-10-06T09:27:25.437Z
---
The 2026-10-05 history cells on #169 were produced by a scratch driver around benchmarks.realtree.compare_tools. Bring it into explorations/benchmarks/realtree/history.py (branch claude/perf-index-driver, landing through #176), reading index-suite.json, with tests. It must: map --analyze and --cache off for each build era; resolve job ids to probe modes through measure.JOBS (e.g. warm-revalidate -> revalidate); write one cell per job with component, job, component_digest, manifest_version and platform; and time every job whole-process. Add a check that each committed cell's run artifact reproduces its summary medians. Raised as T3 on #169 and B2/C2 on #175.
