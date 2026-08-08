---
type: is
id: is-01kzg4d2fb96erw3h1b5k0c6xy
title: "Cache coherency B: snapshot + append-only delta journal"
kind: feature
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-08T07:29:07.051Z
updated_at: 2026-08-08T07:29:07.051Z
---
Phase 1 ships option A: rewrite the whole snapshot on quiesce and at shutdown. Simple, write cost proportional to tree size, and a crash loses only warmth, never correctness — the next open revalidates.

Option B is the growth path and is cheap to reach because the journal record format IS the delta type that already exists: append applied deltas to a sidecar; on open, load snapshot, replay journal, then revalidate only what fingerprints say is stale; compact into a fresh snapshot when the journal exceeds a threshold.

Option C (persist just a dirty-set of subtree roots) is the degraded mode for a read-only cache directory.

Open questions to settle when this is built:
- Compact synchronously at quiesce, or in a background thread?
- Should since(clock) be servable ACROSS a process restart from the journal (nice for SSE resume), or only within one process lifetime (simpler)?

In all three options correctness never depends on the watcher: the revalidation sweep at open remains the backstop, the same guarantee git's fsmonitor gets by layering notifications OVER stat fingerprints rather than replacing them.
