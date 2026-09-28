---
type: is
id: is-01m3k16gyjxv6j1ghxz68epbnp
title: Merge current main and rerun macOS performance with bounded workflow cost
kind: task
status: in_progress
priority: 1
version: 2
delegate: codex@spud10.local
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T03:33:37.360Z
updated_at: 2026-09-28T03:34:15.646Z
started_at: 2026-09-28T03:34:15.645Z
---
Merge origin/main into performance branch using merge-upstream workflow. Keep builds/caches on external spud-ext1 and benchmark subjects internal. Audit previous wall-clock overhead, reuse verified builds, predeclare compact measurement plan without changing acceptance rules, rerun current H153 and end-to-end/peer macOS workloads as appropriate, record provenance and results, validate once at final handoff, push and watch CI.
