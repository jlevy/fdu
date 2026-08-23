---
type: is
id: is-01m0pqhr8tefw2bcxh49csmr6n
title: "PR #38 review R1: perf-record does not require --tree-provenance"
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m0pqh0yf7etx8dywann7tnx3
created_at: 2026-08-23T07:14:51.801Z
updated_at: 2026-08-23T07:34:37.035Z
closed_at: 2026-08-23T07:34:37.033Z
close_reason: "Fixed: --tree-provenance is required and rejects an empty value; from_run raises when tree_reconstructible is set without a recipe. Two tests."
---
performance-loop.md:220 says every experiment must record tree_provenance, but explorations/benchmarks/realtree/record.py gives --tree-provenance a default of "" and accepts --tree-reconstructible with no provenance, storing a contradiction. Make --tree-provenance required and reject reconstructible-without-provenance. Recorder-level only; the ledger-level cutover enforcement stays in fdu-1xlb.
