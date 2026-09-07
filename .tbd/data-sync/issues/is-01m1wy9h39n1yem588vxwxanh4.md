---
type: is
id: is-01m1wy9h39n1yem588vxwxanh4
title: Make allocation regression guards accept reductions in work
kind: bug
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels:
  - review
  - maintainability
dependencies: []
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
created_at: 2026-09-07T03:23:50.760Z
updated_at: 2026-09-07T03:29:18.922Z
---
PR 52 final review R2, head 5d7b86f: crates/fdu/tests/detached_performance_invariants.rs:175-191 rejects allocation growth below limit minus one allocation per entry, in addition to its upper bound. A future allocation reduction can therefore fail make check on one or more platforms. Keep upper-bound regression checks monotonic in improvement; test the checker independently using known over-budget inputs and show lower or zero growth is accepted. Keep exact zero-work assertions for unnecessary streaming effects, but do not encode a minimum amount of work or freeze a storage representation.

## Notes

Added a review-only Rust regression test calling the existing helper with recorded detached growth 10671 minus 2080 allocations for 2080 added entries, keeping ceiling 6 per entry. It fails: growth 8591, restored 10671, limit 12480. This directly proves an improvement of one allocation per entry fails the guard. No production branch edits.
