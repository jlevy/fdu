---
type: is
id: is-01m4fkm0d134gx8fy5mq1a3b4j
title: "PR #189 C1: ARGS given to make release-stability leaks into the gates' sub-makes via MAKEFLAGS"
kind: bug
status: in_progress
priority: 1
version: 2
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m3tr05r13ydjt9hza3nksarb
hold: null
hold_until: null
created_at: 2026-10-09T05:54:17.632Z
updated_at: 2026-10-09T05:54:18.689Z
started_at: 2026-10-09T05:54:18.688Z
---
Coordinator-found running make release-stability for real at 2a643cfd: gate-semver-check.log: 'semver_check.py: error: unrecognized arguments: --target-dir ... --min-free-gb 5 --label ...'. make exports command-line variables (ARGS) and MAKEFLAGS/MAKELEVEL to recipes; stability_pass.py passes os.environ to every step. Fix: run every step without make's exported state.
