---
type: is
id: is-01m3q1k1swt258x2wsca9hwrpe
title: Pass AT_NO_AUTOMOUNT on the routes the Linux native reader does not cover (musl, serial walk, reconciliation)
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-09-29-linux-parity-0.2.2.md
labels: []
dependencies: []
parent_id: is-01m3nbs9kfe7ygc6jx23j1byzt
created_at: 2026-09-29T16:57:25.563Z
updated_at: 2026-09-29T16:57:25.563Z
---
fdu-puk7 was closed when H169 phase 1 merged (217861c1; exp-185, exp-186): the Linux-native reader passes AT_NO_AUTOMOUNT on every per-entry statx, which, in its close reason's words, closes the automount issue on the Linux glibc walk path, while musl and the serial/reconcile routes still use std. Those routes are musl builds (the reader is behind cfg(all(target_os = "linux", target_env = "gnu"))), the serial walk (--threads 1, which exp-185 used as its portable placebo), reconciliation, and any directory the reader declines to the portable read_dir path. They stat through std without AT_NO_AUTOMOUNT, so a walk that reaches an unmounted autofs trigger directory can still mount it (see fdu-puk7's description for the kernel and std evidence). Pass AT_NO_AUTOMOUNT on each of them, for example a native statx or fstatat behind the Linux gate, with a test per route and identical answers. Not a performance item.
