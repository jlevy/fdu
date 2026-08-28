---
type: is
id: is-01m133x3nkxc1ar53129gmgdkj
title: Real-backend watch tests raced their own watch registration (fixed)
kind: bug
status: closed
priority: 1
version: 9
labels: []
dependencies: []
created_at: 2026-08-28T02:41:40.011Z
updated_at: 2026-08-28T14:46:28.975Z
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

## Resolved 2026-08-28 — and the cause was none of the five hypotheses above

Every environmental diagnosis recorded earlier in this bead was wrong. Keeping them,
because the sequence is the useful part: machine load, CPU starvation, volume-wide
filesystem churn, cross-binary concurrency, and finally a degraded local `fseventsd`.
Each was plausible because the symptom is timing-shaped, and each was disproved by
measurement — CPU starvation least ambiguously, since the test passes in 0.45 s with
eight cores saturated.

The cause was a race the tests set up for themselves.

A watcher is not watching when its constructor returns. Registration is requested first
and takes effect a little later, and anything written inside that window produces no
event at all. The engine detects precisely this and answers
`InvalidateSubtree { path: "", reason: WatchSetupRace }`, meaning "I missed a window,
relist the root". Every one of these tests wrote its subject immediately after binding,
so each raced its own setup; when it lost, the engine replied correctly and the test
rejected the reply because it was waiting for one specific file's event.

What made it visible was changing `wait_for` to report what it saw rather than a
truncated list:

    delivered 2 op(s) in 60s but never the one awaited:
      [Upsert { path: "", kind: Dir, .. },
       InvalidateSubtree { path: "", reason: WatchSetupRace }]

fdu was correct in every failing run. No product code changed.

## The fix

Each test now proves its watch is live before measuring what it measures.

- Engine tests warm up with a write and wait for any delivery.
- Session tests rewrite a file that already exists and that their own selection admits.
  Rewriting rather than creating is load-bearing: it changes no file count, so a test may
  still assert totals, and it leaves nothing to clean up. A first attempt created and
  then deleted a warm-up and cost sixty seconds per test, because the engine coalesces
  that pair into no net change and the wait could never be satisfied — correct engine
  behavior finding a wrong test.
- Persistence tests replace fixed two- and three-second sleeps, which were guesses about
  registration latency, with a warm-up whose persisted snapshot proves the watcher runs.

`wait_for` now separates three outcomes that had been one: matched, arrived-but-wrong
(a real content disagreement, still fails), and nothing at all (the host's event service
gave this stream nothing, which is not evidence about fdu, so the caller skips by name
the way `permission_bits_are_enforced` already declines a host that cannot supply a
precondition).

Result: `make rust-test` passed four consecutive runs, from zero of three. The session
suite runs in three to five seconds rather than sixty to a hundred and twenty, because
the deadlines that were being spent are no longer reached. Landed in `19d3a76`.

## Not the cause, but still true and worth someone's attention

This host's `fseventsd` reached 4.75 GB RSS and was logging `USER DROPPED` events to a
stuck Apple `CacheDelete` client. That is a genuine oddity, it is unrelated to these
failures, and it will not fix itself without a reboot. Recorded here only so the
observation is not lost; it is not a reason to reopen this bead.
