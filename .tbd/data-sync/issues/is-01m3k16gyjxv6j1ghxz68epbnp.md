---
type: is
id: is-01m3k16gyjxv6j1ghxz68epbnp
title: Merge current main and prepare bounded macOS performance plan
kind: task
status: in_progress
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-27-macos-performance-rerun.md
delegate: codex@spud10.local
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T03:33:37.360Z
updated_at: 2026-09-28T03:56:01.517Z
started_at: 2026-09-28T03:34:15.645Z
---
Merge origin/main into performance branch using merge-upstream workflow. Keep builds/caches on external spud-ext1 and benchmark subjects internal. Audit previous wall-clock overhead, reuse verified builds, predeclare compact measurement plan without changing acceptance rules, rerun current H153 and end-to-end/peer macOS workloads as appropriate, record provenance and results, validate once at final handoff, push and watch CI.

## Notes

Preparation committed and pushed as 326b014b: merged origin/main 02ab4cf5; focused formatting, 139 query tests, Code-renderer tests, and 100 CLI tests passed with external build output. Published macOS rerun plan; no timing, corpus generation, or release benchmark builds ran. Official measurements remain explicitly on hold at user request. CI run 36374594870 has all applicable completed checks passing; Windows Python 3.12 wheel job 108777784020 remains building without a reported failure. Preparation bead remains open only for final CI confirmation; do not start benchmarks.
