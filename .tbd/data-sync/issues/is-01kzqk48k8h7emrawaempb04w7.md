---
type: is
id: is-01kzqk48k8h7emrawaempb04w7
title: "PR #3 review R1: make parallel scan bounded and cancellation-safe"
kind: bug
status: closed
priority: 1
version: 4
labels:
  - pr-review
dependencies:
  - type: blocks
    target: is-01kzqk493tkcy6nwws6vf9md7f
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T05:01:08.071Z
updated_at: 2026-08-11T05:05:19.992Z
closed_at: 2026-08-11T05:05:19.986Z
close_reason: Fixed with a bounded per-worker observation queue, RAII claim release, queue-wide abort, panic guard, cancellation-aware full-channel sends, and timeout regression tests for disconnect, abort, panic, backpressure, and normal completion.
---
FDU-PR3-R1. crates/fdu/src/scan.rs around DirectoryQueue and walk_worker. A claimed batch leaks outstanding work on send failure or panic, so peers can hang; the observation channel is unbounded. Add RAII claim release, queue-wide abort/wakeup, cancellation-aware bounded sends, and deterministic failure/slow-consumer/normal tests. Review: https://github.com/jlevy/fdu/pull/3#issuecomment-5249058288; prior thread: https://github.com/jlevy/fdu/pull/3#discussion_r3754432430.
