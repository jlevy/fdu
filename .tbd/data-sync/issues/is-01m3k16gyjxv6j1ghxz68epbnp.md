---
type: is
id: is-01m3k16gyjxv6j1ghxz68epbnp
title: Merge current main and prepare bounded macOS performance plan
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-27-macos-performance-rerun.md
delegate: codex@spud10.local
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-09-28T03:33:37.360Z
updated_at: 2026-09-28T03:38:09.717Z
started_at: 2026-09-28T03:34:15.645Z
---
Merge origin/main into performance branch using merge-upstream workflow. Keep builds/caches on external spud-ext1 and benchmark subjects internal. Audit previous wall-clock overhead, reuse verified builds, predeclare compact measurement plan without changing acceptance rules, rerun current H153 and end-to-end/peer macOS workloads as appropriate, record provenance and results, validate once at final handoff, push and watch CI.

## Notes

User narrowed scope: do not run performance measurements yet. Finish merge from origin/main02ab4cf5, focused smoke validation, and a plan separating storage, workload costs, and final freeze/authorization. Full official timing deferred until more integrations selected.
