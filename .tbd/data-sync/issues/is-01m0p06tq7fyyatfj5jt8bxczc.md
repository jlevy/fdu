---
type: is
id: is-01m0p06tq7fyyatfj5jt8bxczc
title: "PR #42 R7: lib-only dependency guard passes silently if cargo tree fails"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0p06qgww21s4fpdkj2bb6bf
created_at: 2026-08-23T00:26:56.614Z
updated_at: 2026-08-23T00:57:54.137Z
closed_at: 2026-08-23T00:57:54.137Z
close_reason: "Fixed in 4e34ce3; addressed the review on PR #42, verified through make check and make release-rehearse."
---
Makefile:238. Pipeline status is the last command, so a failing cargo tree yields empty input, grep returns 1, ! inverts to 0, guard reports success.
