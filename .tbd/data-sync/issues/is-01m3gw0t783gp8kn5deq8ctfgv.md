---
type: is
id: is-01m3gw0t783gp8kn5deq8ctfgv
title: "H152: profile and exact-oracle the current content-query path"
kind: task
status: in_progress
priority: 1
version: 2
spec_path: docs/project/guides/performance-loop.md
delegate: codex@spud10.local
labels:
  - performance
  - experiment
dependencies: []
parent_id: is-01m3gvqwswcvwe38v0pp58sny0
hold: null
hold_until: null
created_at: 2026-09-27T07:24:38.503Z
updated_at: 2026-09-27T07:24:45.909Z
started_at: 2026-09-27T07:24:45.906Z
---
Determine whether repeated per-view classification and content lookup in four-view content-query leaves a removable stage large enough to clear the 3% paired wall gate on current main. First make report equality observable outside the component timer; then capture a current-head caller-tree profile on a deciding dense subject. This determination makes no speed claim. Stop without compiling an engine change if repeated resolution does not name enough headroom. Proposed experiment id exp-158; H153 is reserved only if H152 supports the code experiment.
