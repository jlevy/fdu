---
type: is
id: is-01m3kfjkha31pmqt9kkssk7znr
title: Clean up tonight's scratch worktrees and data after landing
kind: chore
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m3kfj7vjnh8patz58z2p2bh1
created_at: 2026-09-28T07:44:53.288Z
updated_at: 2026-09-28T07:44:53.288Z
---
After the stack merges: trash worktrees under /Volumes/spud-ext1/agent-scratch/fdu-prog/ (change-sources-review, watch-rename-scope, macos-rerun, macos-rerun-builds, release-0.2.0, docs-polish, regression-review if any), git worktree prune, trash /Volumes/spud-ext1/agent-scratch/fdu-fsevents-review-20260927 except anything unported, check for unpushed work and live processes first, and remind the user to empty the Trash. Keep the prior agent's preserved baselines (fdu-fsevents-replay-01a0e000) unless the user says otherwise.
