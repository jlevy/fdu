---
type: is
id: is-01m2kfrpsxsmxpf7j25pq43a3t
title: "PR #63 review PR63-CACHE-1: CachePolicy::Only after a limit change says only 'different scan scope'"
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-15T21:32:31.420Z
updated_at: 2026-09-16T06:08:30.495Z
closed_at: 2026-09-16T06:08:30.494Z
close_reason: "621eb77: a refused cache-only open keeps the snapshot's limits beside its scope and names each limit that differs with both values, plus the two remedies. Verified fixed by PR #63 delta review 5218970886 at 9105768."
resolution: null
duplicate_of: null
---
PR #63 review PR63-CACHE-1 at 1fd71a9: lib.rs:422-425. Name the changed limits and the remedy (rerun with the old values, or use auto). Test an Only case.
