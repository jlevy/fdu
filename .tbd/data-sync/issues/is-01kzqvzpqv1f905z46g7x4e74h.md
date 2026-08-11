---
type: is
id: is-01kzqvzpqv1f905z46g7x4e74h
title: Make evidence argv test platform-neutral
kind: bug
status: closed
priority: 2
version: 2
labels:
  - ci
  - windows
dependencies: []
parent_id: is-01kzqk2ct4s2qjv9e2z17fvywr
created_at: 2026-08-11T07:35:55.899Z
updated_at: 2026-08-11T07:36:47.136Z
closed_at: 2026-08-11T07:36:47.136Z
close_reason: Made the argv expansion assertion derive expected paths from pathlib; focused test, all 70 evidence-contract tests, and the full make check gate pass.
---
Windows CI normalizes pathlib paths to backslashes, but ArgumentExpansionTests hard-coded POSIX strings. Assert against str(Path) values so the locked evidence contract is portable across the CI matrix.
