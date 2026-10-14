---
type: is
id: is-01m4xn8gwp26wpnjtp7jnghfh0
title: Audit native lifecycle policy and run full handoff gate
kind: task
status: in_progress
priority: 2
version: 4
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-14T16:52:20.500Z
updated_at: 2026-10-14T17:16:04.774Z
started_at: 2026-10-14T16:52:53.661Z
---

## Notes

Implemented and independently reviewed three focused fixes: unsupported opened projections raise before native dispatch; native total/rollup include provenance under one borrow; typed Index.freshness avoids report construction in adapter. Added deterministic regression tests and corrected self-contained README fence. Targeted suite passes 68 tests after replacing an invalid root-provenance timestamp assumption with a direct no-second-read guard; removal case demonstrates mixed-state failure. Astra final review has no findings. Full make check running on final source with external worktree-owned target/env/cache paths; formatter/structural checks/clippy pass so far. Existing GIL follow-up includes implicit last-reference teardown; added optional cached-index verification roadmap bead fdu-l9jh. No performance measurements or claims.
