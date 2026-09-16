---
type: is
id: is-01m2nrqeqa5pf3j4d548wygd0a
title: "Release rehearsal: preserve watch_rule nanoseconds on Windows"
kind: bug
status: closed
priority: 1
version: 4
delegate: codex@spud10
labels:
  - release
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-16T18:47:36.425Z
updated_at: 2026-09-16T19:30:50.461Z
closed_at: 2026-09-16T19:30:50.458Z
close_reason: "Fixed by PR #73 (65aa773): exact integer nanoseconds now bypass platform SystemTime; local make check/cross-lint and all 19 PR CI jobs, including both Windows wheel smoke jobs, passed."
resolution: null
duplicate_of: null
---
Release rehearsal run 35135991670 failed in the Windows wheel public smoke test. Python watch_rule accepts exact i64 nanoseconds, but the native binding converts them through Windows SystemTime, whose 100 ns FILETIME granularity truncates the final digits before RFC 3339 formatting. Preserve the integer timestamp through formatting, cover non-100-ns-aligned and pre-epoch values, merge the fix, and rerun release.yml to a green immutable-evidence audit.

## Notes

Fix committed as dd6cd27 and opened as PR #73. Root cause was the Python binding's exact i64 nanoseconds round-tripping through Windows SystemTime/FILETIME at 100 ns precision. The fix formats integer nanoseconds directly and applies the same path to Recent-view timestamps. Local validation passed: focused regression, make python-smoke, full make check, and make cross-lint for macOS and Windows. Awaiting GitHub CI, then merge and rerun release.yml.
