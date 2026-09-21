---
type: is
id: is-01m31jnypzjgvz7qsrhe2qhpn4
title: "PR #104's H138 allocation guard is inert and the test binary is racy"
kind: bug
status: closed
priority: 0
version: 2
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T08:52:48.991Z
updated_at: 2026-09-21T16:50:47.593Z
closed_at: 2026-09-21T16:50:47.593Z
close_reason: "Fixed in e38afecc on cursor/review-leftovers-de1b (PR #104). Guard now compares the pair against the two views measured separately, cancelling fixed overhead and aggregation: sharing saves 2054, never-share saves 3. Verified never-share fails, always-share fails the sibling bound, 40/40 parallel runs pass. The race was fixed separately on the same branch in 6ffbabcb/3565f222."
resolution: null
duplicate_of: null
---
PR #104 restores an H138 sharing guard under `fdu-iajs`, stating "an H138 sharing allocation guard added so flipping `row_consumers > 1` now fails". That claim is false. Two defects, both verified by execution and found independently by two reviews.

## 1. The guard cannot fail (severe)

`crates/fdu-core/tests/query_allocations.rs:113-121` asserts `both < types.saturating_mul(2)`.

Mutating `crates/fdu-core/src/query/query_report.rs:1028` from `row_consumers > 1` to never-share still passes. Measured: `types=5134`, `both=10265`, bound `10268` — it misses the lost walk by 3 allocations. One review saw 30/30 passes under the flip; I reproduced it independently.

The algebra says why. With F = fixed per-report overhead (about 3), W = one `every_entry` walk (about 2051), A = per-view aggregation (about 3080): `types = F+W+A`, shared `both = F+W+2A`, unshared `both = F+2W+2A`, bound `2*types = 2F+2W+2A`. The bound double-counts F, so unshared passes whenever F > 0. Aggregation and fixed overhead are both larger than the walk, so a ratio bound can never see the walk go missing.

This is exactly the failure `fdu-iajs` exists to close: the optimization remained untested, and CI went green proving nothing about H138.

## 2. The test binary is racy (severe)

`counters::enable` writes a process-global `AtomicBool` (`counters.rs:22,807`) while `thread_snapshot` reads a thread-local. libtest runs the file's two tests on parallel threads in one process, so whichever finishes first calls `enable(false)` mid-measurement in the other. `reset` clears only the calling thread's local, so interference can only truncate a count, never inflate it.

Observed truncated readings: 486, 966, 1099, 2053, 2059, 2062, 3779, 3866, 4852, 6726 against a clean 5134. One review measured 5 failures in 40 default-mode runs; another 9 in 60. The 966 in PR #105's CI failure is below the physical floor — `every_entry` does a join and a clone per entry, so 1024 files cannot cost under 2048 allocations.

This is what made PR #105 appear to fail: same engine, opposite outcomes on two runs.

## Fix

Serialize the counter window with a file-level `Mutex` held across each test, add a truncation floor so a raced reading fails honestly, and replace the ratio bound with one that cancels F and A: measure `[Families]` too and assert `types + families - both >= FILES`. Shared saves 2054; never-share saves 3. Both reviews derived this form independently.
