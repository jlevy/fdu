---
type: is
id: is-01m00v05b6kdm7ds5aaer4nzzn
title: "PR #22 review R5: handle macOS 32-bit counter wrapping"
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m00tzk6myk9ba0110gv86kdz
created_at: 2026-08-14T19:11:52.165Z
updated_at: 2026-08-14T19:25:03.128Z
closed_at: 2026-08-14T19:25:03.127Z
close_reason: Fixed by preserving i32 proc_taskinfo counter bits as u32 and using 32-bit wrapping deltas per underlying Mach/Unix/fault/page-in counter. Boundary regression passes across the signed boundary and u32 wrap.
---
High. PR #22 review R5. crates/perfkit/src/process.rs:169-202. Signed proc_taskinfo counters are converted to zero after wrap. Preserve source-width bits and compute wrapping deltas with boundary tests.

## Notes

Preserved proc_taskinfo signed fields as u32 bit patterns and compute source-width wrapping deltas for Unix/Mach syscalls, faults, and page-ins. Boundary regression covers signed and unsigned wrap.
