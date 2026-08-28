---
type: is
id: is-01m133x3nkxc1ar53129gmgdkj
title: Real-backend watch tests fail ~2 runs in 3 inside the parallel lib binary
kind: bug
status: open
priority: 1
version: 6
labels: []
dependencies: []
created_at: 2026-08-28T02:41:40.011Z
updated_at: 2026-08-28T03:58:52.830Z
---
crates/fdu-core/tests/watch_session_integration.rs, an_idle_tree_yields_nothing_and_costs_nothing, fails roughly one run in three on a loaded machine. Observed while a rename-only change was in the tree; the same code passed on the next two runs, so it is timing, not a regression.

The test drains the seed write before asserting idleness:

    let settle = Instant::now() + Duration::from_secs(3);
    let mut quiet_polls = 0;
    while Instant::now() < settle && quiet_polls < 3 {
        if session.next_batch(...).expect('batch').is_none() { quiet_polls += 1 } else { quiet_polls = 0 }
    }

The loop has two exits and the test treats them as equivalent. Reaching quiet_polls >= 3 means the backend has gone quiet and the following assertion is meaningful. Exhausting the three-second deadline means the backend has NOT gone quiet, and the test proceeds to assert four consecutive empty polls anyway, so the still-pending seed batch lands in the assertion. That is precisely the flake the test's own comment says it is guarding against: 'a single None can simply mean the backend has not delivered the seed event yet, and breaking on it lets that late batch land in the assertion below'. The guard covers the single-None case but not the deadline case.

The failure message compounds it. 'an idle tree must produce no batches' names the contract the test intends to check, so a load-induced miss reads as a genuine polling-implementation defect and will send the next reader after a bug that is not there.

Fix by distinguishing the exits rather than lengthening the deadline. If the settle loop ends without sustained quiet, the precondition was never established: skip or retry with a clear message saying the backend never went quiet, and do not assert the idleness contract. When it does go quiet, assert as now. Consider reporting the batch that arrived so a real defect is still legible.

This matters beyond local runs: CI runners are shared and loaded, so the same window exists there, and a test that fails for an unrelated reason trains readers to re-run rather than read.

## Notes

## Reproduced and correctly diagnosed 2026-08-27

Two earlier diagnoses were wrong and are recorded so nobody repeats them.

**Not external machine load.** The failures first appeared at load average 221 on ten
cores, which looked sufficient. They recur unchanged at load 74 with `fseventsd` back to
2.8% CPU.

**Not CPU contention.** `a_new_directory_also_escalates_for_a_relist` run on its own
passes in 0.83 s idle and 0.45 s with eight cores saturated by `yes`. Starving the CPU
does not reproduce it at all.

**It is concurrency inside the test binary.** Running the exact failing target,
`cargo test -p fdu-core --all-features --lib`, three times in a row with nothing else
running:

| Run | Result | Wall time |
| --- | --- | --- |
| 1 | 535 passed | 53.96 s |
| 2 | 2 failed | 49.77 s |
| 3 | 3 failed | 92.77 s |

Roughly one run in three passes, and the same binary's runtime varies by a factor of two.
The failing tests are the ones that wait on real FSEvents delivery, while five hundred
sibling tests in the same binary create, write, and delete temporary files on the same
volume. FSEvents is volume-wide, so that churn is delivered into the same stream the
watch tests are filtering, and macOS coalesces under pressure. The tests are not isolated
from their siblings' filesystem activity, and no timeout value fixes that.

Affected: `a_new_directory_also_escalates_for_a_relist`,
`created_files_arrive_as_verified_upserts`, `deleted_files_arrive_as_removes`.

Pre-existing. All three are present in `origin/main`.

## The fix the plan already prescribes

The PR #47 test reuse audit says of the real watch cases: "Keep a minimal create/remove
delivery and idle-no-work platform smoke. Move gaps, overflow, filters, budgets, state,
ordering, and shutdown to the scripted deterministic sessions."

`a_new_directory_also_escalates_for_a_relist` drives a watch-setup *race* through a real
backend. A race is exactly what a deterministic scripted observer exists to express, and
exactly what a shared event stream cannot be relied on to reproduce. Move it, and keep
only minimal create and remove delivery against the real backend.

Two supporting changes:

- `wait_for` returns whatever it accumulated when its deadline expires, so the caller
  cannot distinguish "the backend never delivered" from "the backend delivered the wrong
  ops", and reports the second. Give it a distinguishable result so a delivery timeout
  says so.
- Real-backend tests should not share a binary with hundreds of filesystem-mutating
  siblings. Consider moving them to their own integration target so they are serialized
  against that churn.

Raised to P1: this blocks `make check`, the required handoff gate, about two runs in
three on macOS.
