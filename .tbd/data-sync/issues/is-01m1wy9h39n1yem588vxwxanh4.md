---
type: is
id: is-01m1wy9h39n1yem588vxwxanh4
title: Make allocation regression guards accept reductions in work
kind: bug
status: in_progress
priority: 2
version: 6
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - review
  - maintainability
dependencies:
  - type: blocks
    target: is-01m1dtr903vj783j9ajaxfnczf
  - type: blocks
    target: is-01m1x444e4rnksjs8v8p37padv
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T03:23:50.760Z
updated_at: 2026-09-07T05:31:46.761Z
---
PR 52 final review R2, head 5d7b86f: crates/fdu/tests/detached_performance_invariants.rs:175-191 rejects allocation growth below limit minus one allocation per entry, in addition to its upper bound. A future allocation reduction can therefore fail make check on one or more platforms. Keep upper-bound regression checks monotonic in improvement; test the checker independently using known over-budget inputs and show lower or zero growth is accepted. Keep exact zero-work assertions for unnecessary streaming effects, but do not encode a minimum amount of work or freeze a storage representation.

## Notes

Committed-test candidate reproduces the old failure for an improvement from growth 10671 to 8591 over 2080 entries. The guard is now upper-bound-only, accepts reduced/zero/negative growth and the exact ceiling, and rejects ceiling+1. The arithmetic checks run in the single allocator test before enabling instrumentation to avoid concurrent test traffic. Focused test passes; full handoff gate and CI pending.
