---
type: is
id: is-01m3p2sknbpqmbzj2rc6t51cyb
title: "Q0: re-baseline the fc-v49 host (A/A, four-arm baselines, perf-floor, tool cells, per-thread callgrind)"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T07:59:14.603Z
updated_at: 2026-09-29T16:28:57.469Z
closed_at: 2026-09-29T16:28:57.469Z
close_reason: "Q0 baselines exp-175-177; final 20-pair standing on ebc06c78: level with pdu default and diskus on linux-v6.12 and node-modules-dense (from 2.4x pdu default at Q0); end to end exp-194/195: linux-v6.12 default tree -39.00%, node-modules-dense -9.75%."
resolution: null
duplicate_of: null
---
Overnight plan Q0 (plan-2026-09-29-linux-overnight-performance-loop.md). Quiet A/A on linux-v6.12 default-tree + aggregate-summary (noise estimate); quiet four-arm baselines of e5a71c8a (controls on/--no-controls) on linux-v6.12, node-modules-dense, linux-balanced-1m; make perf-floor on both real subjects; fdu-default-tree tool cell vs pdu and diskus on both real subjects (part of fdu-m3r6); callgrind per thread on the profiling probe, default-tree, both real subjects, controls on and off. Record as baseline experiments from exp-175.

## Notes

Cells recorded as exp-175 (linux-v6.12), exp-176 (node-modules-dense), exp-177 (linux-balanced-1m), with tool cells in exp-175/176 evidence. Side-by-side callgrind/strace profiling of fdu vs pdu/dut/diskus in progress. perf-floor not yet run.
