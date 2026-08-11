---
type: is
id: is-01kzqn5vw0t83yh77s92f6njf9
title: "P3: watch-stream benchmark job"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:36:57.727Z
updated_at: 2026-08-11T05:36:57.727Z
---
Register watch-stream as a named benchmark job per Principle 11 and the performance-evidence research's job-identity rule, alongside re-pointing cli-human and cli-json and adding cli-summary and cli-files as their surfaces land. The watch job's measured quantities are per-event latency (change to emitted record), steady-state idle cost (which should be ~zero CPU and zero syscalls - the efficiency contract made measurable), and cost under a burst. Record the exact argv in the manifest; flags are benchmark identity, so any later rename updates this job in the same change.
