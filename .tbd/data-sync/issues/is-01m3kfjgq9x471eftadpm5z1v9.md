---
type: is
id: is-01m3kfjgq9x471eftadpm5z1v9
title: "Final gate on the stack top: make check and make cross-lint, CI green including parity re-record"
kind: task
status: open
priority: 0
version: 3
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjj22b63eyenyc9ys3b28
  - type: blocks
    target: is-01m3kfjkha31pmqt9kkssk7znr
parent_id: is-01m3kfj7vjnh8patz58z2p2bh1
created_at: 2026-09-28T07:44:50.407Z
updated_at: 2026-09-28T07:45:10.642Z
---
Run once on the final top layer with the worktree's own ./target (fdu-dfbu): make check, make cross-lint (targets installed per AGENTS.md). Then CI on every stack PR green; the version bump needs the Linux parity diff re-recorded by CI. Report results honestly; fix bottom-up if anything fails.
