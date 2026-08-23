---
type: is
id: is-01m0p07hd8xnhr1pk9r48mvw4v
title: "PR #42 R18: interpreter path spelled literally in four places"
kind: task
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:27:19.847Z
updated_at: 2026-08-23T00:57:54.160Z
closed_at: 2026-08-23T00:57:54.160Z
close_reason: "Fixed in 4e34ce3; addressed the review on PR #42, verified through make check and make release-rehearse."
---
Makefile:182,187,190 and ci.yml:213, each hard-coding POSIX bin/.
