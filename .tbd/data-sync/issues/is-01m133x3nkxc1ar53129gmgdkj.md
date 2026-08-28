---
type: is
id: is-01m133x3nkxc1ar53129gmgdkj
title: Real-filesystem watch tests fail under CPU load, and one has a specific settle defect
kind: bug
status: open
priority: 2
version: 3
labels: []
dependencies: []
created_at: 2026-08-28T02:41:40.011Z
updated_at: 2026-08-28T03:27:27.039Z
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

## Wider than one test, observed 2026-08-27

The flakiness is not confined to `an_idle_tree_yields_nothing_and_costs_nothing`. Running
the full core suite while a workspace build competed for CPU also failed three
`watch::tests` unit tests in one run:

- `a_new_directory_also_escalates_for_a_relist`
- `created_files_arrive_as_verified_upserts`
- `deleted_files_arrive_as_removes`

All 33 `watch::tests` then passed three times in isolation, and the full 558-test suite
passed cleanly once the machine was quiet. So these are load-sensitive real-filesystem
timing tests, not regressions.

Treat the settle defect described above as the specific, fixable instance, and this as the
pattern: a test that waits on a real backend needs to distinguish "the backend went quiet"
from "my deadline expired", and to say which happened when it fails. A failure message
naming the product contract, on a test that actually lost a race, sends the next reader
after a bug that is not there — which is what makes this worth fixing rather than
re-running.

Worth auditing the other real-backend watch tests for the same two-exits-one-meaning shape
while fixing the first.
