---
type: is
id: is-01m3rkxyn2ywq4kz0kxbdh6rra
title: "H185/H72 skip admits d_type kinds in a search-denied directory: folded tree and summary differ from the full index (R163-1)"
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3qgck5yzpd603akhhkpw7t9
hold: null
hold_until: null
created_at: 2026-09-30T07:37:11.586Z
updated_at: 2026-09-30T09:56:21.231Z
started_at: 2026-09-30T07:37:27.893Z
closed_at: 2026-09-30T09:56:21.231Z
close_reason: Same finding (R163-1), tracked and closed as fdu-wigy.
resolution: duplicate
duplicate_of: is-01m3rk2qqhbhtwmgjane46hpqm
---
A directory that is readable but not searchable (mode 0400) lists, and every child's stat fails with EACCES. The full index reports each child as an error and holds no entry. Under the skip policy (H72 on the summary route since 0.2.1 on every Unix, H185 on the folded default tree on this branch) the native reader and the portable path admitted a DT_DIR child as a Dir entry (queued, then reported again when it failed to open) and a DT_LNK child as a Symlink entry with no error at all, so dirs was one higher and the error list one shorter than the full index's. Fix: per listing, stat every child until one stat succeeds (which proves the parent searchable), and only then take directory and symlink kinds from the listing. Covered by the transient-versus-indexed differentials (boundary tree and summary control case, non-root) and by native-reader unit tests.
