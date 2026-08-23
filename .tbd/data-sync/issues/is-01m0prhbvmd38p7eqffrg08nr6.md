---
type: is
id: is-01m0prhbvmd38p7eqffrg08nr6
title: "Watch: explicit polling backend for network filesystems"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:32:07.668Z
updated_at: 2026-08-23T07:32:07.668Z
---
WatchOptions gains an explicit backend choice (native/poll) with a stated poll interval, because network and FUSE filesystems drop native events silently. Engine-side mount-table auto-detection stays an open question; explicit selection ships first and the client's existing detection can drive it.
