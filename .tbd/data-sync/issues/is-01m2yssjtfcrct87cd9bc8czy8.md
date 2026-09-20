---
type: is
id: is-01m2yssjtfcrct87cd9bc8czy8
title: "H72: skip directory and symlink statx on Linux transient summary"
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-20-linux-performance-iteration.md
labels: []
dependencies: []
parent_id: is-01m2ymtwf6fth3a3rk0nn4kw8d
created_at: 2026-09-20T06:59:24.623Z
updated_at: 2026-09-20T07:07:14.057Z
closed_at: 2026-09-20T07:07:14.057Z
close_reason: "H72 recorded: rejected on reconstructible linux-v6.12 (exp-152, -1.63%); accepted on nominated /usr (exp-153, -9.01%). Engine kept f841662c."
---
H72 (fdu-i2f3 standing). Transient RetainedState::Summary needs no directory or symlink attributes; d_type/file_type can skip their statx. Previous measure was -1.4% on a 6.4% directory tree. Nominated /usr is 22% dirs+symlinks (14325 dirs, 31646 symlinks / 208411). Both arms --no-controls (H107). Do not skip when one_filesystem is on (device identity). Accept only if reconstructible linux-v6.12 clears 3% with interval below zero; /usr is confirmatory/screening because its image fingerprint drifts. Do not lower the 3% bar. Do not compile a walk trim (H71). Next experiment exp-152.
