---
type: is
id: is-01kzm5ee1czjcdqn35y96cnyef
title: Bound the watcher pipeline and make overload and shutdown fail safe
kind: bug
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - concurrency
  - watch
  - merge-blocker
dependencies:
  - type: blocks
    target: is-01kzm5eqahbmtm5gwhf6fmejwh
  - type: blocks
    target: is-01kzg4bfw0zmmztg25v9a0nkq4
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T21:04:17.963Z
updated_at: 2026-08-09T21:10:00.446Z
---
The current notify callback channel, verified-observation channel, and worker pending-path BTreeMap are all unbounded. Under sustained churn they permit unbounded memory growth; merely replacing send with blocking sync_channel would introduce shutdown deadlocks because Drop joins the worker while the observation receiver remains alive, and a notify callback must not block a backend thread. Simplify ownership as part of bounding it: the worker coalesces raw hints into bounded internal intents but performs no filesystem I/O. next_observation or apply_next verifies an intent synchronously on the consuming thread immediately before arbitration, avoiding both duplicate worker/apply stats and an uncancellable stat inside the joined worker. The backend callback uses a bounded nonblocking enqueue plus a sticky atomic overflow signal. Pending path cardinality, intent batch size, and output capacity are capped and validated; any overflow or coalescing loss collapses to one root WatchOverflow intent and never silently drops truth. Output uses nonblocking delivery and retains a bounded sticky root invalidation until capacity is available. Explicit cancellation wakes the coalescer, permits it to abandon undeliverable output during teardown, and joins it; worker panic and permanent stop are distinguishable from timeout. Keep the single-consumer transport private instead of exposing std Receiver, and document that consumer-side filesystem calls have ordinary filesystem latency while Watcher teardown itself never waits on one. Tests deterministically fill every stage, force concurrent drop, disconnect, panic/error, and continuous churn with barriers/channels and bounded deadlines. Prove bounded state, eventual root reconciliation, no silent loss, no duplicate verification on apply_next, and prompt worker join.
