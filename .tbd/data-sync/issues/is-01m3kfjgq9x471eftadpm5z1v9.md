---
type: is
id: is-01m3kfjgq9x471eftadpm5z1v9
title: "Final gate on the stack top: make check and make cross-lint, CI green including parity re-record"
kind: task
status: closed
priority: 0
version: 5
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjj22b63eyenyc9ys3b28
  - type: blocks
    target: is-01m3kfjkha31pmqt9kkssk7znr
parent_id: is-01m3kfj7vjnh8patz58z2p2bh1
created_at: 2026-09-28T07:44:50.407Z
updated_at: 2026-09-28T12:36:08.463Z
closed_at: 2026-09-28T12:36:08.462Z
close_reason: "make check + make cross-lint passed on #148 head 48812041 (fresh recompiles incl. MSRV 1.85); CI green on all 11 stack heads."
resolution: null
duplicate_of: null
---
Run once on the final top layer with the worktree's own ./target (fdu-dfbu): make check, make cross-lint (targets installed per AGENTS.md). Then CI on every stack PR green; the version bump needs the Linux parity diff re-recorded by CI. Report results honestly; fix bottom-up if anything fails.

## Notes

2026-09-28: gate requeued on #148 head 48812041 (after the admission fix); first attempt at 2a9339e3 was stopped while still waiting for the timing lock.
