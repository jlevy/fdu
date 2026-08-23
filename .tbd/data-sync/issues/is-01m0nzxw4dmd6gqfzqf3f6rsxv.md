---
type: is
id: is-01m0nzxw4dmd6gqfzqf3f6rsxv
title: "PR #42 review R7: lib-only dependency guard passes silently when cargo tree fails"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:03.148Z
updated_at: 2026-08-23T00:39:55.212Z
closed_at: 2026-08-23T00:39:55.212Z
close_reason: "Fixed. The guard captures cargo tree's output and exits on its failure before searching it. Verified red-green: it fires on -p fdu, and a failing cargo tree now fails the target where the old pipeline form returned 0."
---
Makefile:238. A pipeline's status is grep's, so a failing cargo tree yields empty input, grep returns 1, and ! inverts it to success. Capture the output first, then test it.
