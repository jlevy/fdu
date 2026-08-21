---
type: is
id: is-01m0hfsvwr9xm1gqqfk9wf0vkx
title: Add a --cache-clear=unreadable scope to reclaim stale-format snapshots
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-21T06:23:16.875Z
updated_at: 2026-08-21T06:23:16.875Z
---
Measured on a development machine 2026-08-20: the user cache held 63 MB across 52
entries, of which 26 were unreadable by the current binary (`--cache-status=all` reports
them as `unrecognized`). They are pre-release format churn, handled correctly as absent
rather than crashing, but nothing reclaims them.

`--cache-clear` takes `root` or `all`, so pruning dead entries also discards live ones. A
`--cache-clear=unreadable` scope is the cheapest partial answer and does not require
deciding the larger retention policy (Open Question 5 in the composable CLI spec).
