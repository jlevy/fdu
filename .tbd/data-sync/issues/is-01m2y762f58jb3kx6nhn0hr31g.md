---
type: is
id: is-01m2y762f58jb3kx6nhn0hr31g
title: "H143: leftover after H111 floor/RSS fail"
kind: task
status: closed
priority: 1
version: 6
spec_path: docs/project/specs/active/plan-2026-09-19-linux-parallel-validation.md
labels:
  - linux
  - H143
dependencies: []
parent_id: is-01m2y4f4g34vdbgxf0jcvt8dw3
created_at: 2026-09-20T01:34:10.917Z
updated_at: 2026-09-20T04:39:16.847Z
closed_at: 2026-09-20T04:39:16.847Z
close_reason: "exp-142: same leftover identity on quiet linux-450k cold-scan-index; walk 94.7-94.8%; finish 4.3% is H86 consume; do not restart H86"
---
After H111 failed on a virtualized Linux host (exp-141), leftover is still the Darwin composite: getdents64+statx walk (H140) plus retained-index RSS above arena_spike. Not a rewrite of H19-H22/H60/H7. Not a restart of H86. A new named mechanism that can close the 1.4x index or 3x RSS gate on 450k, or a bare-metal remeasure of H111.

## Notes

PREDICT (2026-09-20, Linux KVM, exp-142)

Hypothesis: H143 leftover after H111 fail.
Job: same-binary 12-pair cold-scan-index on reconstructible linux-450k (450,001 entries).
Attribution: FDU_COUNTERS=1 hits on the same tree.
Determination: walk still >=90% of index component and leftover is getdents64+statx plus retained-index RSS, or a userspace stage >=3% that is not H86/H71.
Do not compile a walk trim. Do not restart H86. Do not retry H71.
Quiet first. Do not lower the 25% bar.
Host: 4-core KVM Xeon, 16 GiB, ext4, virtualized. Same class as exp-103/141.
