---
type: is
id: is-01m2gahg0pdyfxr9yxy0wce5e7
title: "Flaky on ubuntu CI: a_killed_watch_still_leaves_a_warm_cache missed the second snapshot rewrite"
kind: bug
status: open
priority: 3
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T16:03:28.917Z
updated_at: 2026-09-14T16:03:28.917Z
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
