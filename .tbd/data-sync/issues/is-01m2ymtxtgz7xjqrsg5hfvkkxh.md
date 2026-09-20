---
type: is
id: is-01m2ymtxtgz7xjqrsg5hfvkkxh
title: "H145: Linux opened-discovery leftover"
kind: task
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels:
  - linux
  - campaign-2
dependencies:
  - type: blocks
    target: is-01m2ymty2v5bt2d0s3h3s6mty8
parent_id: is-01m2ymtwf6fth3a3rk0nn4kw8d
created_at: 2026-09-20T05:32:45.776Z
updated_at: 2026-09-20T05:54:58.612Z
closed_at: 2026-09-20T05:54:58.612Z
close_reason: "Same leftover identity as Darwin H127 (exp-145, uncontrolled): 5772 journal clones, 438k live roll-up merges, opened 2.75x first-pass; no smallest userspace cut; do not port macos_bulk."
---
H127 analog on linux-v6.12. Darwin leftover was read_dir+fstatat, journal clones, live merges; no smallest cut. Name whether Linux has userspace ≥3%. Do not port macos_bulk.
