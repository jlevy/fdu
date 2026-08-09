---
type: is
id: is-01kzjcpgq1p1jt8k5xetdv8psc
title: Make atomic snapshot replacement safe under concurrent writers
kind: bug
status: closed
priority: 1
version: 3
labels:
  - persistence
  - concurrency
dependencies: []
parent_id: is-01kzjceqsx74x63350s7hjb8q0
created_at: 2026-08-09T04:32:34.016Z
updated_at: 2026-08-09T05:10:37.414Z
closed_at: 2026-08-09T05:10:37.414Z
close_reason: "Implemented in commit 5014b13 with focused regression tests; local make check and all PR #1 checks pass on Linux, macOS, Windows, MSRV 1.85, dependency policy, docs, and Python wheel smoke."
---
Snapshot saves in one process reuse a PID-only sibling temp name and open it with truncation. Concurrent saves can clobber each other's in-flight image. Reserve unique temp files with create_new, retain atomic rename semantics, clean them on failure, and add concurrency-focused regression coverage.
