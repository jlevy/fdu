---
type: is
id: is-01m3p2d207jww7gcsp8r0k84fz
title: Check whether fdu's Linux walk triggers autofs automounts (std statx without AT_NO_AUTOMOUNT)
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-09-29T07:52:23.303Z
updated_at: 2026-09-29T12:20:25.565Z
closed_at: 2026-09-29T12:20:25.565Z
close_reason: "H169 phase 1 accepted and merged (217861c1): exp-185 and exp-186. The native reader passes AT_NO_AUTOMOUNT on every per-entry statx, which closes the automount issue on the Linux glibc walk path; musl and the serial/reconcile routes still use std (noted in the reader's docs)."
resolution: null
duplicate_of: null
---
Found while reading peers (dut main.c:604 and bfs pass AT_NO_AUTOMOUNT on every statx). Kernel v6.12 fs/stat.c: the statx syscall sets LOOKUP_AUTOMOUNT unless AT_NO_AUTOMOUNT is given (getname_statx_lookup_flags, line 240), while vfs_fstatat (lstat/fstatat) forces AT_NO_AUTOMOUNT (line 328). Rust std's DirEntry::metadata and symlink_metadata use statx on Linux; if std omits AT_NO_AUTOMOUNT (verify in library/std/src/sys/fs/unix.rs with rust-src for the pinned 1.97.1 and the 1.85 MSRV), a walk that lands on an autofs trigger directory (e.g. /net, autofs /home) mounts it, which can hang on NFS and changes what a du reports. Check: std source; a reproduction with an autofs map if the host allows; what fdu should do (a native statx with AT_NO_AUTOMOUNT behind the Linux gate, or fstatat). Not a performance item.
