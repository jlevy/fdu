---
type: is
id: is-01m2y5ryfmgvs5d748gbfnyq52
title: "PR #91 review R2: cover streamed sidecar rollback after a valid prefix"
kind: task
status: in_progress
priority: 2
version: 4
delegate: unknown@spud10
labels: []
dependencies: []
parent_id: is-01m2y58e5s7yzpstkcqmheymy5
hold: null
hold_until: null
created_at: 2026-09-20T01:09:32.276Z
updated_at: 2026-09-20T01:17:05.717Z
started_at: 2026-09-20T01:10:13.063Z
closed_at: 2026-09-20T01:15:24.291Z
close_reason: Added malformed-later-record and checksummed-trailing-bytes tests that assert a clean miss with neither files nor roll-ups.
resolution: null
duplicate_of: null
---

## Notes

PR91 R2 review https://github.com/jlevy/fdu/pull/91#pullrequestreview-5258707801; content_cache.rs187/226 streamed late-failure rollback tests. Implemented c2487f38; targeted tests pass, root reviewed, gate/push pending.
