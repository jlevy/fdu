---
type: is
id: is-01kzqn5vw0t83yh77s92f6njf9
title: "P3: watch-stream benchmark job"
kind: task
status: closed
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:36:57.727Z
updated_at: 2026-08-11T17:18:23.723Z
closed_at: 2026-08-11T17:18:23.723Z
close_reason: Python Index.watch() with GIL released across the wait, empty-batch ticks so the interpreter can always exit, close()/context-manager shutdown, and unsendable declared at the boundary; fdu-py enables watch while the fdu crate still builds cli-only. Benchmark job vocabulary extended with cli-summary, cli-files, watch-stream across schema.py, the JSON schema, and the CLI-job set. docs/project/guides/fdu-design-principles.md records the eleven principles as implemented, with both amendments and the testing hazards; AGENTS.md points at it. make check passes.
---
Register watch-stream as a named benchmark job per Principle 11 and the performance-evidence research's job-identity rule, alongside re-pointing cli-human and cli-json and adding cli-summary and cli-files as their surfaces land. The watch job's measured quantities are per-event latency (change to emitted record), steady-state idle cost (which should be ~zero CPU and zero syscalls - the efficiency contract made measurable), and cost under a burst. Record the exact argv in the manifest; flags are benchmark identity, so any later rename updates this job in the same change.
