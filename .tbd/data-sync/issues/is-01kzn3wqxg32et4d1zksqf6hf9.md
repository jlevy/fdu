---
type: is
id: is-01kzn3wqxg32et4d1zksqf6hf9
title: Use fresh file identity in the Windows corpus oracle
kind: bug
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - windows
dependencies: []
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-10T05:56:24.111Z
updated_at: 2026-08-10T06:06:28.074Z
closed_at: 2026-08-10T06:06:28.074Z
close_reason: All local handoff gates and the complete Linux, macOS, Windows, and installed-wheel CI matrix pass with dedicated regressions and documented evidence.
---
Windows os.DirEntry.stat() deliberately returns zero for st_ino, st_dev, and st_nlink. The independent oracle therefore collapses every regular file into one apparent hardlink group. Use a fresh non-following path stat where directory-entry identity is incomplete or non-authoritative, add an emulated zero-identity regression that distinguishes real hardlinks from unrelated files, improve mismatch diagnostics, and prove all corpus/probe jobs on Windows.
