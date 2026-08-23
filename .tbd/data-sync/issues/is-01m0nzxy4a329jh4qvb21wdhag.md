---
type: is
id: is-01m0nzxy4a329jh4qvb21wdhag
title: "PR #42 review R12: surface architecture doc quotes a cargo tree guard that can never match"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:05.194Z
updated_at: 2026-08-23T00:39:56.840Z
closed_at: 2026-08-23T00:39:56.840Z
close_reason: Fixed. The doc quotes the guard as the Makefile runs it, and explains why --prefix none and the capture are load-bearing.
---
docs/project/architecture/fdu-surface-architecture.md:54 omits --prefix none, so cargo tree's tree glyphs defeat the ^ anchor and the documented check can never fire.
