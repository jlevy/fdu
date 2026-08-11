---
type: is
id: is-01kzs4gd4ebz6b6r06zet3wmpc
title: watch-stream benchmark runner for time-sampled jobs
kind: task
status: open
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-11T19:24:06.157Z
updated_at: 2026-08-11T21:20:36.769Z
---
benchmarks registers a watch-stream job in its vocabulary but has no runner. Every existing job times a command to completion; a stream job has to sample over a window instead - events delivered, latency from filesystem change to emitted record, and CPU while idle (which should be zero). Needs a harness shape decision before implementation. Not blocking: no watch performance claim currently rests on it.
