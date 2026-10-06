---
type: is
id: is-01m495ervajgkm115aaz2gwh6a
title: Investigate one-off exit 2 'file changed during content analysis' on a fresh clone
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m48xs24ec44m1prxyt7xany5
created_at: 2026-10-06T17:51:19.401Z
updated_at: 2026-10-06T17:51:19.401Z
---
One fdu --analyze run on a fresh, idle local clone exited 2 with 'file changed during content analysis'; an immediate rerun was clean. Determine whether a file really changed (e.g. git maintenance writing into .git) or whether the change detection has a false positive (timestamp granularity, a racing stat). Found by the README senior review (2026-10-06).
