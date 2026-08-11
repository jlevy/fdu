---
type: is
id: is-01kzs5q3c12hy8w68w8djh2nyp
title: "Bugbot review of PR #5: six findings, five fixed one rebutted"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-11T19:45:14.112Z
updated_at: 2026-08-11T19:45:14.112Z
---
Cursor Bugbot reviewed PR #5 across three commits. Dispositions: R1 Python report mislabels cache source - FIXED (source now carried on PyIndex; report_dict also gained the source field it was missing versus CLI machine output). R2 path globs ignore backslashes - REBUTTED, already fixed in an earlier commit than the one reviewed; glob.rs splits on Path::components(), and the remaining split on '/' is over the user-written pattern, where forward slashes are correct. R3 modified filters silently dropped when system_time_to_nanos returns None - FIXED in both CLI and Python, now a usage error with a golden. R4 watch settle loop exits too early - FIXED, requires three consecutive quiet polls instead of one. R5 watch idle skips cache save - FIXED, real bug in the persistence work: a dirty batch throttled below the interval followed by an idle tree was never persisted, which is the most likely way a watch session actually ends. R6 watch test misses update proof - FIXED and the most valuable finding: open() writes a snapshot before the watch loop starts, so the test passed without the feature it existed to pin. It now fingerprints the initial snapshot and requires a rewrite, and it failed as expected before the fix.
