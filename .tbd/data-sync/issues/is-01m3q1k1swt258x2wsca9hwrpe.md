---
type: is
id: is-01m3q1k1swt258x2wsca9hwrpe
title: Pass AT_NO_AUTOMOUNT on the routes the Linux native reader does not cover (musl, serial walk, reconciliation)
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T16:57:25.563Z
updated_at: 2026-09-29T23:38:16.091Z
closed_at: 2026-09-29T23:38:16.091Z
close_reason: "Fixed in #161 (17874dd6, docs 6d9730d8): on glibc every listing route (serial walk, revalidate, reconcile, opened discovery) lists through the native reader, whose statx passes AT_NO_AUTOMOUNT; per-path stats go through the same wrapper; the walk root alone is resolved (root_device reads the device from an opened descriptor, so --one-filesystem bounds the walk to the filesystem the listing mounts). musl needs no code: std uses fstatat there (verified in 1.85.0 and 1.97.1 std source). strace: zero unflagged tree-entry stats on every route. Non-regression screen exp-196 (H184): serial walk -5.14%, reconciliation -3.56%, default tree +0.42% [-2.67%, +4.11%], opened-discovery wall +1.19% [+0.56%, +3.51%] with its discovery component flat. make check passes at 45943211."
resolution: null
duplicate_of: null
---
fdu-puk7 was closed when H169 phase 1 merged (217861c1; exp-185, exp-186): the Linux-native reader passes AT_NO_AUTOMOUNT on every per-entry statx, which, in its close reason's words, closes the automount issue on the Linux glibc walk path, while musl and the serial/reconcile routes still use std. Those routes are musl builds (the reader is behind cfg(all(target_os = "linux", target_env = "gnu"))), the serial walk (--threads 1, which exp-185 used as its portable placebo), reconciliation, and any directory the reader declines to the portable read_dir path. They stat through std without AT_NO_AUTOMOUNT, so a walk that reaches an unmounted autofs trigger directory can still mount it (see fdu-puk7's description for the kernel and std evidence). Pass AT_NO_AUTOMOUNT on each of them, for example a native statx or fstatat behind the Linux gate, with a test per route and identical answers. Not a performance item.

## Notes

2026-09-29, #161 review R161-2: the native reader's AT_NO_AUTOMOUNT makes the answer route-dependent on autofs trees (--threads 1, musl, the root's own stat and declined directories still mount the trigger). Decision: #161 documents the exception in CHANGELOG [Unreleased] and the platform review; this bead is a pre-0.2.2 release item. Before tagging 0.2.2, either land it (on Linux, observe_dir_entry and the root stat can use fstatat(AT_SYMLINK_NOFOLLOW), which the kernel treats as AT_NO_AUTOMOUNT, behind the existing gate) or keep the CHANGELOG statement in the release notes.
