---
type: is
id: is-01m4xk7r3cg0bdkmvrtbervbz9
title: Reclaim unused build artifacts and worktrees
kind: task
status: closed
priority: 2
version: 4
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-14T16:16:57.964Z
updated_at: 2026-10-14T16:33:07.598Z
started_at: 2026-10-14T16:17:56.849Z
closed_at: 2026-10-14T16:33:07.585Z
close_reason: Verified and removed the three explicitly approved unused build directories; staged obsolete pytest runs; preserved source, evidence, active work, and unpushed worktree commits.
resolution: null
duplicate_of: null
---

## Notes

Cleanup performed by Sol subagent under reclaim-macos-disk-space skill. User explicitly approved permanent deletion of exact /Volumes/spud-ext1/agent-scratch/fdu-restore-table-01a1383a/{target,uv-cache,venv}; verified recreatability, source matches, no live users and safe path identity. Removed 2257649664 allocated bytes; observed external free-space increase 1766932 KiB during action (concurrent cleanup also active). Preserved source.git, check.log, pr-body.txt and other siblings. Staged three obsolete internal pytest runs (440,715,716; 23007232 bytes) in Trash, newest717 kept; Trash unemptied. Secondary fdu worktree preserved because it has unpushed commits.
