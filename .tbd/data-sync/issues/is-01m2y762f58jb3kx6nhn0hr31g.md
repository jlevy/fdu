---
type: is
id: is-01m2y762f58jb3kx6nhn0hr31g
title: "H143: leftover after H111 floor/RSS fail"
kind: task
status: open
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-linux-parallel-validation.md
labels:
  - linux
  - H143
dependencies: []
parent_id: is-01m2y4f4g34vdbgxf0jcvt8dw3
created_at: 2026-09-20T01:34:10.917Z
updated_at: 2026-09-20T01:34:14.930Z
---
After H111 failed on a virtualized Linux host (exp-141), leftover is still the Darwin composite: getdents64+statx walk (H140) plus retained-index RSS above arena_spike. Not a rewrite of H19-H22/H60/H7. Not a restart of H86. A new named mechanism that can close the 1.4x index or 3x RSS gate on 450k, or a bare-metal remeasure of H111.
