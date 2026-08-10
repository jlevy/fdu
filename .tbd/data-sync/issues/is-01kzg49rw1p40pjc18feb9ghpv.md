---
type: is
id: is-01kzg49rw1p40pjc18feb9ghpv
title: "Walk layer: raw getdents64 and dirfd-relative statx"
kind: feature
status: open
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4ak7v8z2a7s41rsms8jcb
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
  - type: blocks
    target: is-01kzg49s5s1gst3526wx73q9rf
parent_id: is-01kzg48ekn4sm0azybr010qgmn
child_order_hints:
  - is-01kzmzmcszb269mrex4hzdcp3y
created_at: 2026-08-08T07:27:18.913Z
updated_at: 2026-08-10T22:13:58.900Z
---
Replace the portable read_dir + symlink_metadata walker. Goal 1 is not met, and must not be claimed, until this lands and the benchmark gate passes.

Techniques, all proven in dut and bfs:
- Raw getdents64 into a large reused per-thread buffer (bfs: 64 KiB inline; dut: 1 MB scratch), not libc readdir.
- Eagerly issue a second getdents into leftover buffer space to detect EOF without a later syscall (bfs).
- openat-family dirfd-relative traversal throughout, O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC.
- statx with a narrow field mask, AT_SYMLINK_NOFOLLOW, AT_NO_AUTOMOUNT, and AT_STATX_DONT_SYNC on network mounts.
- d_type from the dirent to skip stat entirely when type is all that is needed.
- LRU cache of open directory fds sized from RLIMIT_NOFILE, pinning roots and in-progress dirs.

Note ncdu 2 deliberately uses fstatat, not statx, for older-kernel compatibility: keep a portable fallback and do not make statx a hard requirement. Non-Linux keeps the portable path.
