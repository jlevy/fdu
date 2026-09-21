---
type: is
id: is-01m32twvdtzccksj9ema19j93r
title: "PR #98 review R1: zero FILETIME becomes an I/O error, so FAT/exFAT volumes fail outright"
kind: bug
status: closed
priority: 1
version: 3
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m32h6dpd97fr5f8db831dn3y
hold: null
hold_until: null
created_at: 2026-09-21T20:35:38.042Z
updated_at: 2026-09-21T20:57:25.185Z
started_at: 2026-09-21T20:36:00.223Z
closed_at: 2026-09-21T20:57:25.185Z
close_reason: "Fixed in 650b6b08 on codex/release-windows-validity: windows_time_to_unix_ns yields 0 for 0 ticks (unavailable, per the Attrs contract) and saturates any other out-of-range value like the Unix compose_ns; it no longer returns an error, so Observed::attrs is infallible. Unit test covers 0, 1 tick, i64::MIN, i64::MAX, and the epoch; the same function extracted verbatim was run on Linux, and the module's tests passed in the Windows CI Test job. Not verified: an actual FAT/exFAT volume scan, which needs a Windows host."
resolution: null
duplicate_of: null
---
Blocker from https://github.com/jlevy/fdu/pull/98#issuecomment-5764966314. crates/fdu-core/src/scan/windows_metadata.rs:142-148 windows_time_to_unix_ns errors on ticks == 0, which FAT/exFAT report for ChangeTime; fdu E:\ hard-errors and a FAT subtree under NTFS is dropped with complete: false. Fix: 0 ticks yields 0 ns per the Attrs contract (engine_contract.rs:91-96); nonzero out-of-range saturates like the Unix compose_ns. Add the 0 case to the unit test at :154-165.
