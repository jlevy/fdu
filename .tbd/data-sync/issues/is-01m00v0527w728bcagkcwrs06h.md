---
type: is
id: is-01m00v0527w728bcagkcwrs06h
title: "PR #22 review R4: model process counter capabilities per platform"
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m00tzk6myk9ba0110gv86kdz
created_at: 2026-08-14T19:11:51.878Z
updated_at: 2026-08-14T19:25:02.850Z
closed_at: 2026-08-14T19:25:02.849Z
close_reason: Fixed with a shared Snapshot/delta/render model, explicit LinuxProc/MacOsProcTaskInfo sources, per-metric Option values, capability-specific collectors, omission tests, and native macOS output that contains no Linux-only zero rows.
---
High. PR #22 review R4. crates/perfkit/src/process.rs:47-77,167-176,221-234. macOS marks the whole snapshot available and renders unsupported Linux-only metrics as zero. Share the model but expose and render only supported metrics.

## Notes

Replaced all-or-nothing available plus zero fields with per-metric Options and explicit Source. macOS render emits only total syscalls, page faults, and page-ins; Linux retains procfs read/write/byte/fault metrics.
