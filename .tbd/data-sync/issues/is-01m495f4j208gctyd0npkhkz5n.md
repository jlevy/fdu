---
type: is
id: is-01m495f4j208gctyd0npkhkz5n
title: The --stale-ok perf line reports a rate on zero work
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m48xs24ec44m1prxyt7xany5
created_at: 2026-10-06T17:51:31.393Z
updated_at: 2026-10-06T17:51:31.393Z
---
With --stale-ok serving from a snapshot, the perf: line reports an entries-per-second rate although no walk ran. Report no rate (or say served from snapshot) when the work is zero. Found by the README senior review (2026-10-06).
