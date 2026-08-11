---
type: is
id: is-01kzqn4eh461jy13mvs25bmwvn
title: "P3: Session API composing IndexHandle, Watcher, and Query"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzqn4rdq9vy4qvcve073rfhf
  - type: blocks
    target: is-01kzqn5jbrqef88q43pdd0pa71
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:36:11.299Z
updated_at: 2026-08-11T17:08:48.066Z
closed_at: 2026-08-11T17:08:48.065Z
close_reason: Session composes IndexHandle+Watcher+Query; --watch loop with fdu.stream/1 tagged change records, dirty-gated aggregate repaint, and event-driven detection throughout. Selection filters the stream, with removals filtered only by path and escalations never filtered. --watch + --scan-depth is a usage error that teaches scope-vs-selection. watch joins default features while cli-only and no-default-features still build. 5 integration tests against real filesystem events, including the idle-cost contract.
---
A Session type owning IndexHandle plus watch::Watcher, yielding batches already filtered through the run's Selection, so the CLI loop is a thin consumer and watch is the same query repeated (Principle 9). Detection stays event-driven: the watcher binds notify's recommended_watcher (FSEvents on macOS, inotify on Linux, ReadDirectoryChangesW on Windows) - no polling loop is introduced anywhere in this work, and idle cost stays zero filesystem work. The existing coalescer semantics are preserved unchanged: kernel events are hints, coalesced per path with the settle window, then verified with exactly one fresh stat per coalesced path before becoming a delta. Drive apply_next on the consuming thread; never hold an index lock across filesystem I/O or user callbacks. Tests: deterministic apply/shutdown interleavings extending the existing watch test discipline; a quiet-tree test asserting no filesystem syscalls occur while idle.
