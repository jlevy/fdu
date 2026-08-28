---
type: is
id: is-01m133x3nkxc1ar53129gmgdkj
title: Real-backend watch tests fail ~2 runs in 3 inside the parallel lib binary
kind: bug
status: open
priority: 1
version: 7
labels: []
dependencies: []
created_at: 2026-08-28T02:41:40.011Z
updated_at: 2026-08-28T07:00:09.803Z
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

## Host evidence, 2026-08-27 — first hypothesis with observational support

Four earlier hypotheses were tested and killed. Recording them so nobody re-runs them:
external CPU load (test passes in 0.45 s with eight cores saturated), volume-wide
filesystem churn (eight aggressive churn loops with instrumentation on the raw notify
callback produced zero missed events), pairwise binary concurrency
(`watch_persistence` alongside `watch_session`, 3/3 pass), and serializing watchers
(contradicted by `watch_session` running five real watchers in parallel and passing 5/5).

What the failure actually looks like, measured rather than inferred:

- the test receives *nothing*, not late events. `wait_for` reports `Saw 0 op(s): []`
  after a full sixty seconds;
- it is per-process, not per-test. One run failed all five `watch_session` tests at once,
  so that binary's delivery was dead for its whole life;
- it is not macOS as a platform. CI's `Test (macos-latest)` job has passed on every run
  of this branch, on fresh VMs, with these same tests.

`log show --predicate 'process == "fseventsd"'` on this host shows why that last row is
the important one:

- `fseventsd` is at 4.75 GB RSS and still climbing. It was 2.8 GB earlier in the same
  session, then 3.8 GB. That is far outside normal for this daemon;
- it is repeatedly logging `client_buffer_flush: sending client(...) USER DROPPED event
  to pid 12137`, and pid 12137 is Apple's own `CacheDelete.framework/deleted`, which has
  been a slow consumer for over four hours. The daemon is buffering and dropping for a
  stuck system client;
- 346 stream registrations in two hours, on fifteen days of uptime, from the many
  watcher-heavy tools this machine runs.

Hypothesis: a degraded local `fseventsd` sometimes gives a newly registered stream no
delivery at all. It explains every observation above, including why no synthetic load
reproduced it — the trigger is daemon state at registration time, not load this process
generates — and why `origin/main` fails here while both CI platforms are green.

**Falsifier, and it must be run before this is believed.** After a reboot,
`make rust-test` should pass repeatedly on this host. If it still fails against a healthy
daemon, this hypothesis is wrong like the other four and the investigation resumes.

## Containment that does not depend on the host

Independent of the cause, a test must not report a product defect when the host's event
service gave it nothing. The repository already has the pattern:
`test_support::permission_bits_are_enforced` skips fixtures the host cannot provide.

Probe delivery before asserting it. Write a file this test causes, and wait for that
file's own event. If the probe never arrives, the host event service is unavailable:
skip visibly, naming the host, and never evaluate the product assertion. If the probe
arrives, the stream works and every existing assertion runs at full strength.

Residual exposure: the `cli-watch` golden helpers cannot skip, because tryscript has no
skip semantics. A dead stream still fails `test-golden` there, though now with a
correctly attributed "timed out waiting for the repaint separator" rather than a claim
about product behavior.
