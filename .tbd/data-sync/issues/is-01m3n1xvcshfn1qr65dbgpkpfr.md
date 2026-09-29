---
type: is
id: is-01m3n1xvcshfn1qr65dbgpkpfr
title: "H169: dirfd-relative directory opens with raw getdents64 and fd-derived directory metadata"
kind: task
status: in_progress
priority: 2
version: 2
spec_path: docs/project/research/research-2026-09-28-pdu-and-the-linux-peer-gap.md
delegate: claude-code@vm
labels:
  - performance
  - linux
  - experiment
dependencies: []
parent_id: is-01m3mvdz2891yheyemx49gzm6j
hold: null
hold_until: null
created_at: 2026-09-28T22:24:50.585Z
updated_at: 2026-09-29T09:37:49.555Z
started_at: 2026-09-29T09:37:49.555Z
---
Collapse glibc opendir's fstat and the parent-side statx into one call per directory (125k fewer syscalls on balanced-1m). Linux only. Registry row in docs/project/guides/performance-loop.md; mechanism and pre-registration in the pdu brief.
