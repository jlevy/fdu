---
type: is
id: is-01m00v04evejf8jx9m44jzvakx
title: "PR #22 review R2: make counting allocator API sound"
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m00tzk6myk9ba0110gv86kdz
created_at: 2026-08-14T19:11:51.258Z
updated_at: 2026-08-14T19:24:42.405Z
closed_at: 2026-08-14T19:24:42.404Z
close_reason: Fixed with private Sinks fields, unsafe certified construction and complete GlobalAlloc callback safety contract, non-panicking fdu sinks, guarded TLS re-entry fallback, unit tests, and a passing construction doctest.
---
Blocker. PR #22 review R2. crates/perfkit/src/alloc.rs:54-104. Safe constructors accept arbitrary callbacks although GlobalAlloc requires callbacks never unwind or re-enter allocation. Enforce and document the unsafe boundary.

## Notes

Sinks now have private fields; custom construction is unsafe with a full non-unwind/non-allocation/non-lock/reentrancy/TLS teardown contract. Safe constructors accept only certified Sinks.
