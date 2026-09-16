---
type: is
id: is-01m0hfsvwr9xm1gqqfk9wf0vkx
title: Add a --cache-clear=unreadable scope to reclaim stale-format snapshots
kind: task
status: open
priority: 3
version: 2
labels: []
dependencies: []
created_at: 2026-08-21T06:23:16.875Z
updated_at: 2026-09-16T00:14:31.443Z
---
Measured on a development machine 2026-08-20: the user cache held 63 MB across 52
entries, of which 26 were unreadable by the current binary (`--cache-status=all` reports
them as `unrecognized`). They are pre-release format churn, handled correctly as absent
rather than crashing, but nothing reclaims them.

`--cache-clear` takes `root` or `all`, so pruning dead entries also discards live ones. A
`--cache-clear=unreadable` scope is the cheapest partial answer and does not require
deciding the larger retention policy (Open Question 5 in the composable CLI spec).

## Notes

2026-09-15 SCOPE DECISION OWED, FROM THE REVIEW OF PR #67 (review 5216601111, head 816fcf7):
PR #67 makes a snapshot in a *newer* format stale, exactly like an older one, so `--cache-clear=all` from an older build deletes a newer build's snapshots.
- Accepted for 0.1.0: `=all` is by definition the whole directory and already removes every current snapshot, status lists each file as `stale (newer snapshot format N)` before anyone acts, no release has shipped, and the alternating-versions case is a developer's, whose newer build rewrites on its next scan.
- Owed here: when this bead designs a stale-only scope, decide explicitly whether `newer_format` belongs in it. The reviewer would exclude it, or name the scope by what it keeps: 'stale' inverts its plain meaning for a newer format, and a scope that deletes a newer build's caches while keeping the current ones is the one case that surprises someone.
- The states this bead now has to choose among are `current`, `stale` (with `stale_reason` of `older_format`, `newer_format`, `other_engine`, `unreadable`), `leftover`, and `unrecognized`.
