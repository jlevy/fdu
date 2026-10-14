---
type: is
id: is-01m4xn8gwp26wpnjtp7jnghfh0
title: Audit native lifecycle policy and run full handoff gate
kind: task
status: in_progress
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-14T16:52:20.500Z
updated_at: 2026-10-14T16:58:15.881Z
started_at: 2026-10-14T16:52:53.661Z
---

## Notes

Astra policy/code audit found three focused cleanups: unsupported opened projections silently become Diagnostics; Index total/rollup retrieve metrics and provenance in separate native calls; rollup adapter builds a full report just for freshness. Sol implementing native atomic rollup replies, explicit projection rejection, typed freshness access and adapter fix with regressions. Also correcting standalone README summary fence. Full make check paused for source edits after structural checks, fmt and clippy passed; full rerun required on final source.
