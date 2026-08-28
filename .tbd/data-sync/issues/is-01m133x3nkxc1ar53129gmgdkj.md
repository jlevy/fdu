---
type: is
id: is-01m133x3nkxc1ar53129gmgdkj
title: Idle-watch test proceeds after its settle loop times out without reaching quiet
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-28T02:41:40.011Z
updated_at: 2026-08-28T02:41:40.011Z
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
