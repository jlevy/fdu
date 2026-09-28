---
type: is
id: is-01m3mcx3b2zhmeyf7cs036w8hk
title: Scrub host names from tbd bead records and stop recording them
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T16:17:25.857Z
updated_at: 2026-09-28T16:17:25.857Z
---
tbd start records delegate as <agent>@<host>; ~230 records on the public tbd-sync branch carry this machine's host name. User approved removal 2026-09-28. Stop new ones (claim with tbd start --as <name> / configure identity), scrub current records with a forward commit, and decide separately on rewriting tbd-sync history (other replicas would re-push old history).
