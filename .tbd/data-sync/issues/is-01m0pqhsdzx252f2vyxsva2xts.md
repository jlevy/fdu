---
type: is
id: is-01m0pqhsdzx252f2vyxsva2xts
title: "PR #38 review R4: the generated ledger markdown is not drift-checked"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m0pqh0yf7etx8dywann7tnx3
created_at: 2026-08-23T07:14:52.990Z
updated_at: 2026-08-23T07:34:37.921Z
closed_at: 2026-08-23T07:34:37.920Z
close_reason: "Fixed: perf-ledger-check regenerates into a scratch file, formats it the way perf-ledger does, and diffs. In make check and in CI."
---
AGENTS.md says make check fails if either generated file has drifted. check runs perf-report-check (timeline.json + index.html) but summary.py has no --check mode and perf-ledger is not in the gate, so report-2026-08-10-fdu-performance-experiments.md can drift silently.
