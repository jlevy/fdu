---
type: is
id: is-01m2gahg0pdyfxr9yxy0wce5e7
title: "Flaky on ubuntu CI: a_killed_watch_still_leaves_a_warm_cache missed the second snapshot rewrite"
kind: bug
status: closed
priority: 3
version: 5
labels:
  - stack-followup
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-14T16:03:28.917Z
updated_at: 2026-09-30T10:02:03.551Z
closed_at: 2026-09-30T10:02:03.550Z
close_reason: "Root cause found; the code that had it is gone, so no change beyond the evidence already landed. The failing run (CI run 34864857746, job 104046096716, 2026-09-14, at 50c7ec2) took 30.26 s: establish_watch returned in about 0.26 s because its baseline was read before the open's startup snapshot existed, so that snapshot passed for the warm-up rewrite, and second.txt was then written while fdu --watch stood between its joined startup save and its watcher's root listing. At that commit the CLI kept persistence itself, dirty_since_save |= batch.dirty, set only from loop batches, and Session::new's initial handoff (the full reconciliation and the capture drain) reported its discoveries to nobody. second.txt was inserted by the handoff's reconciliation (no inotify event when written before registration; a drained event that verified to no change when after), so no batch was ever dirty, no save ran, and the deadline passed: a lost-save bug, not a test race. Fixed by 430abbe3, which moved persistence into the engine: Session::finish_initial_handoff sets Persistence { pending: dirty } from the handoff's own commits and next_batch ORs every commit in, pinned by watch_session::tests::handoff_changes_are_persisted_after_the_tree_goes_quiet (a file written after the open's scan and before the handoff is in the first due save). The test itself was hardened in c29e9a85 (baseline taken once a snapshot exists; the watcher's stderr and exit status in every failure). Verified on Linux/inotify at 62e7ce6e: 120/120 runs under an ext4 writeback storm and 100/100 pinned to two CPUs under four busy loops plus the storm, on top of the earlier 100/100. Deadlines and assertions unchanged."
resolution: null
duplicate_of: null
---
Seen on PR #57 (branch claude/contract-decisions, head 50c7ec2), CI run 34864857746, job 104046096716, Test (ubuntu-latest).

`a_killed_watch_still_leaves_a_warm_cache` failed at `crates/fdu/tests/watch_persistence.rs:177@50c7ec2` with "the watch loop never rewrote the snapshot after a change". The warm-up write had already been observed, since `establish_watch` passed. So a live, persisting `fdu --watch` then went 30 s after the `second.txt` write without its snapshot's (length, mtime) fingerprint changing.

A rerun of the failed job passed, and 8 of 8 local runs on macOS passed.

Why the PR is probably not the cause: it does not change the command-line watch path. `fdu --watch` already set `read_controls: false` explicitly before the PR, and the watch loop and save code are unchanged; the engine now only reads the retained control table through a crate-private accessor with the same semantics.

Suspects to check on Linux (inotify):
- the second event is coalesced into, or lost behind, the warm-up save;
- a save gated on freshness skips while a reconciliation from the warm-up is in flight;
- the (length, mtime) fingerprint misses a rewrite.

Reproduce with repeated runs on ubuntu before changing the test's deadline.

## Notes

2026-09-30 stability pass, second entry (c29e9a85b7a2f65434d3752572e8bcb2c81f4f65): still not reproduced; a_killed_watch_still_leaves_a_warm_cache passed 100 of 100 runs on Linux/inotify with the core and CLI test suites running on the same host. The test now self-diagnoses: WatchChild keeps the watcher's stderr as it arrives and every wait on the watcher reports whether it is still running or how it exited and what it printed, so the next CI failure will say whether fdu --watch exited (the test never checked) or a live loop stopped saving. The warm-up baseline is now taken once a snapshot exists, so the open's own startup write can no longer pass for the loop's warm-up save (Session::start joins the startup save before binding the watcher; the first throttled save follows an interval later). Deadlines and assertions unchanged. Left open: the CI failure's cause is still unknown; close it when a run with this test's evidence lands, or when it stops recurring.
