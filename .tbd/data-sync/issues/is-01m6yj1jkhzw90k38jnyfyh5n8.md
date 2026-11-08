---
type: is
id: is-01m6yj1jkhzw90k38jnyfyh5n8
title: "PR #208 C1: tryscript 0.3.1 also changes code for zod 4's API"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m6yj18ch6wszs8sv3gx41ncf
created_at: 2026-11-08T21:46:45.233Z
updated_at: 2026-11-08T21:51:40.038Z
closed_at: 2026-11-08T21:51:40.038Z
close_reason: "Fixed by correction, not rewrite: PR comment 6726737054 now carries a marked correction that tryscript 0.3.1 also adapts its bundle to zod 4's API (two z.record calls, the union-error walker); the c8c32306 commit message keeps the original wording, since history is not rewritten."
resolution: null
duplicate_of: null
---
Low. package.json:23, package-lock.json:210-227; commit c8c32306 message and comment 6726737054. https://github.com/jlevy/fdu/pull/208#issuecomment-6727541578
