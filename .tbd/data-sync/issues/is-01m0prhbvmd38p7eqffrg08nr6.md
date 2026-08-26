---
type: is
id: is-01m0prhbvmd38p7eqffrg08nr6
title: "Watch: explicit polling backend for network filesystems"
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:32:07.668Z
updated_at: 2026-08-26T07:01:50.826Z
closed_at: 2026-08-23T21:27:34.540Z
close_reason: WatchConfig.backend selects Native or Poll { interval }; the watcher boxes notify's recommended watcher or its poller behind one trait object, so only the source of raw events changes and coalescing, stat verification, and the delta path stay the same code. Reached from Python as WatchOptions(poll_interval=). A zero interval is rejected on both surfaces. Engine-side mount-table detection stays an open question, as the bead specified.
resolution: null
duplicate_of: null
---
WatchOptions gains an explicit backend choice (native/poll) with a stated poll interval, because network and FUSE filesystems drop native events silently. Engine-side mount-table auto-detection stays an open question; explicit selection ships first and the client's existing detection can drive it.
