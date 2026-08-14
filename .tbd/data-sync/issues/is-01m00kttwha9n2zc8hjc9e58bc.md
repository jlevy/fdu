---
type: is
id: is-01m00kttwha9n2zc8hjc9e58bc
title: Use dhat-rs to attribute the eleven reallocations per entry
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T17:06:37.584Z
updated_at: 2026-08-14T17:06:37.584Z
---
Direct answer to fdu-zgxd, which counters localized but could not attribute. Per-layer counters report 11.0 reallocations and 15.4 allocations per entry on a 450k cold scan; they say which layer without saying which call site, because they do not sample stacks. dhat-rs (v0.3.3, June 2026) is a drop-in global allocator that records a stack trace per allocation and emits a viewer file, and it is the only allocation profiler that works on Linux, macOS and Windows alike - bytehound and heaptrack are Linux-only for collection. It composes with what already exists: perfkit's CountingAlloc is generic over the inner allocator precisely so another allocator can be wrapped, and dhat can sit inside it. Its testing mode also supports assertions of the form 'this path performs exactly N allocations', which would turn the per-entry allocation figures from an observation into a regression guard - the natural next step after fdu-wu0p established that allocation is producer-side. Expect real overhead during collection, so this is a deep-investigation tool run deliberately, not something to leave on. Deliverable is a call-site attribution of the reallocations and a decision on whether the cause is cheap to remove.
