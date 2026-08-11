---
type: is
id: is-01kzqn5atxakb84p364hjfhg1p
title: "P3: watch golden tests with injected changes"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn66p0pmck4yg6pexhww2z
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:36:40.284Z
updated_at: 2026-08-11T17:08:48.099Z
closed_at: 2026-08-11T17:08:48.099Z
close_reason: Session composes IndexHandle+Watcher+Query; --watch loop with fdu.stream/1 tagged change records, dirty-gated aggregate repaint, and event-driven detection throughout. Selection filters the stream, with removals filtered only by path and escalations never filtered. --watch + --scan-depth is a usage error that teaches scope-vs-selection. watch joins default features while cli-only and no-default-features still build. 5 integration tests against real filesystem events, including the idle-cost contract.
---
Streaming goldens are the hard case in golden-testing-guidelines terms, because the session is inherently timing-bearing: the stable fields are the record sequence, op kinds, paths, kinds, and schema tags, while clock values, timestamps, mtimes, and sizes-in-flight are unstable and need named patterns. Design the scenarios so ordering is deterministic: inject changes with explicit sync points rather than sleeps, one change class per block (create, modify, delete, rename), and assert the resulting record sequence exactly. Cover: initial report then a streamed change; --modified-since now --watch producing an empty initial listing then a tail; an explicit invalidate record with its reason; SIGINT exiting 0 with the final save intact. Verify the process makes no filesystem calls while idle (the efficiency contract) - if tryscript cannot express that, put it in a Rust integration test instead and note the split. Run npx tryscript@latest docs first to confirm the current syntax for interactive/streaming blocks and whether a timeout or terminator idiom fits better than a fixed-duration run.
