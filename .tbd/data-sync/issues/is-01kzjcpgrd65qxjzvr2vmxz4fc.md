---
type: is
id: is-01kzjcpgrd65qxjzvr2vmxz4fc
title: Prune cached descendants when one-filesystem traversal hits a new mount boundary
kind: bug
status: closed
priority: 1
version: 3
labels:
  - correctness
  - revalidation
dependencies: []
parent_id: is-01kzjceqsx74x63350s7hjb8q0
created_at: 2026-08-09T04:32:34.060Z
updated_at: 2026-08-09T05:10:37.434Z
closed_at: 2026-08-09T05:10:37.434Z
close_reason: "Implemented in commit 5014b13 with focused regression tests; local make check and all PR #1 checks pass on Linux, macOS, Windows, MSRV 1.85, dependency policy, docs, and Python wheel smoke."
---
With one_filesystem enabled, warm reconciliation stops descending when a previously local directory becomes a cross-device mount, but leaves the snapshot's old descendants indexed. Emit conditional removals for known children whenever a directory is now outside traversal scope, and cover the decision with focused tests.
