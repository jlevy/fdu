---
type: is
id: is-01m487gbct9fd038qd1drpr4v2
title: Land the history driver in the harness (benchmarks.realtree.history) with tests and a cell drift check
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-10-06T09:07:53.874Z
updated_at: 2026-10-06T09:07:53.874Z
---
The 2026-10-05 history cells on #169 were produced by a scratch driver around benchmarks.realtree.compare_tools. Bring it into explorations/benchmarks/realtree/history.py (branch claude/perf-index-driver, landing through #176), reading index-suite.json, with tests, and a check that each committed cell's run artifact reproduces its summary medians. Raised as T3 in review T on #169.
