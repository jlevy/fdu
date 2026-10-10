---
type: is
id: is-01m4kwnfxn50c9j5csbpmpchec
title: "Several-root summary: keep H72's symlink skip in identified mode"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
created_at: 2026-10-10T21:49:21.205Z
updated_at: 2026-10-10T21:49:21.205Z
---
Review E6 on PR #192 (jlevy/fdu#192). SinkMode::TransientFoldIdentified turns H72 off for directories and symlinks together (scan.rs), but FoldAliases reads only directories' (dev, ino). Directories need the stat (a d_ino cannot stand in at a mount point); symlinks do not. A symlink-heavy tree (H72's /usr) pays one statx per symlink under 'fdu --view summary a b' on Linux. Split the H72 property into directory and symlink halves, keep the symlink skip in identified mode, and measure it (pre-register the hypothesis per the performance loop).
