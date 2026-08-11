---
type: is
id: is-01kzqn5atxakb84p364hjfhg1p
title: "P3: watch golden tests with injected changes"
kind: task
status: closed
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn66p0pmck4yg6pexhww2z
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:36:40.284Z
updated_at: 2026-08-11T21:11:27.907Z
closed_at: 2026-08-11T21:11:27.906Z
close_reason: "Watch stream goldened via the watch-capture Node helper: causal sequencing (each step waits for the record its own action caused, matched on op and path past a cursor), loud timeout on a missing record, named patterns for clock and filesystem-dependent sizes. 71 golden blocks total."
---
Streaming goldens are the hard case in golden-testing-guidelines terms, because the session is inherently timing-bearing: the stable fields are the record sequence, op kinds, paths, kinds, and schema tags, while clock values, timestamps, mtimes, and sizes-in-flight are unstable and need named patterns. Design the scenarios so ordering is deterministic: inject changes with explicit sync points rather than sleeps, one change class per block (create, modify, delete, rename), and assert the resulting record sequence exactly. Cover: initial report then a streamed change; --modified-since now --watch producing an empty initial listing then a tail; an explicit invalidate record with its reason; SIGINT exiting 0 with the final save intact. Verify the process makes no filesystem calls while idle (the efficiency contract) - if tryscript cannot express that, put it in a Rust integration test instead and note the split. Run npx tryscript@latest docs first to confirm the current syntax for interactive/streaming blocks and whether a timeout or terminator idiom fits better than a fixed-duration run.

## Notes

Reopened 2026-08-11: the spec's Testing Strategy asks for 'watch streaming with injected changes' as golden coverage, and that is still missing, so it stays open rather than being closed on a workaround. The obstacle is real but solvable: tryscript compares one command's completed output and a watch process never exits, so the golden needs a bounded capture command. Portable design: a Node helper (tryscript already requires Node) that spawns fdu --watch, applies a scripted sequence of filesystem changes, collects the fdu.stream/1 records, terminates the child, and prints the normalized stream to stdout. That turns watching into a command that exits, and the existing golden discipline applies unchanged - named patterns for timestamps and paths, pinned env, byte-stable integers. Until it lands, coverage is crates/fdu/tests/watch_session.rs (event semantics, selection filtering) and crates/fdu/tests/watch_persistence.rs (save survives SIGKILL), plus the bounded parts already goldened: the --help contract and the scope-validation rejections.
