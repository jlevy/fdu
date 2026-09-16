---
type: is
id: is-01m2nrqeqa5pf3j4d548wygd0a
title: "Release rehearsal: preserve watch_rule nanoseconds on Windows"
kind: bug
status: in_progress
priority: 1
version: 2
delegate: codex@spud10
labels:
  - release
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-16T18:47:36.425Z
updated_at: 2026-09-16T18:47:48.590Z
---
Release rehearsal run 35135991670 failed in the Windows wheel public smoke test. Python watch_rule accepts exact i64 nanoseconds, but the native binding converts them through Windows SystemTime, whose 100 ns FILETIME granularity truncates the final digits before RFC 3339 formatting. Preserve the integer timestamp through formatting, cover non-100-ns-aligned and pre-epoch values, merge the fix, and rerun release.yml to a green immutable-evidence audit.
