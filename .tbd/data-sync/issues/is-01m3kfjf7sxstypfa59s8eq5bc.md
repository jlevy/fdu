---
type: is
id: is-01m3kfjf7sxstypfa59s8eq5bc
title: Prepare the 0.2.0 release layer
kind: task
status: in_progress
priority: 1
version: 4
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjgq9x471eftadpm5z1v9
parent_id: is-01m3kfj7vjnh8patz58z2p2bh1
created_at: 2026-09-28T07:44:48.885Z
updated_at: 2026-09-28T08:43:12.911Z
---
Release-prep agent on claude/release-0.2.0: version 0.1.0 -> 0.2.0 (crates, Cargo.lock, release tests, 34 golden lines, QA playbook), CHANGELOG [0.2.0] - 2026-09-28 with REG-1 additions (H153, H156, no-priming, Delivery struct literal), docs/project/release-notes/0.2.0.md, TODO.md past tense (REG-2), Plan::persists in the engine-architecture table (REG-6), and the documented parity re-record procedure. Lead fills macOS numbers and watch-fix entries after merging those layers.

## Notes

2026-09-28: release layer pushed as claude/release-0.2.0 @ 9c5c96ab (base 724ba597): version bump e1ade9c5, CHANGELOG [0.2.0] + release notes b3b2d190 (FILL(lead) placeholders for macOS numbers and the watch fix; release-test fails until filled), TODO/engine-doc 9c5c96ab. Verified: test-golden 210, cargo test -p fdu 126, local parity 57 deviations matched, release-test 72/73 (the placeholder). Lead: merge lower layers upward, move sibling [Unreleased] entries into [0.2.0], fix README fdu@0.1.0 example, retitle/close fdu-0gqc.
