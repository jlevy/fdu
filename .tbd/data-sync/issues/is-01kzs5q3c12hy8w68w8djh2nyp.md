---
type: is
id: is-01kzs5q3c12hy8w68w8djh2nyp
title: "Complete Bugbot review of PR #5"
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-11T19:45:14.112Z
updated_at: 2026-08-12T16:49:06.921Z
closed_at: 2026-08-12T16:49:06.920Z
close_reason: All actionable review findings through the final Bugbot pass are implemented and verified. make check passes; make perf-ledger followed by the Flowmark check is clean; the dev version marker was proven clean-to-dirty-to-clean without moving HEAD.
---
Cursor Bugbot reviewed PR #5 across three commits. Dispositions: R1 Python report mislabels cache source - FIXED (source now carried on PyIndex; report_dict also gained the source field it was missing versus CLI machine output). R2 path globs ignore backslashes - REBUTTED, already fixed in an earlier commit than the one reviewed; glob.rs splits on Path::components(), and the remaining split on '/' is over the user-written pattern, where forward slashes are correct. R3 modified filters silently dropped when system_time_to_nanos returns None - FIXED in both CLI and Python, now a usage error with a golden. R4 watch settle loop exits too early - FIXED, requires three consecutive quiet polls instead of one. R5 watch idle skips cache save - FIXED, real bug in the persistence work: a dirty batch throttled below the interval followed by an idle tree was never persisted, which is the most likely way a watch session actually ends. R6 watch test misses update proof - FIXED and the most valuable finding: open() writes a snapshot before the watch loop starts, so the test passed without the feature it existed to pin. It now fingerprints the initial snapshot and requires a rewrite, and it failed as expected before the fix.

## Notes

Rounds 1-2: R1 Python cache-source provenance fixed; R2 separator-glob report rebutted because matching was already platform-neutral; R3 out-of-range modified bounds now fail rather than silently dropping; R4 watch settle requires three quiet polls; R5 idle watch persistence fixed; R6 persistence test now proves a rewrite; R7 failed/skipped saves retain the pending state, table-tested through the extracted state machine.\n\nFinal review, 2026-08-12: R8 checkout build versions had disabled Cargo's default package tracking; explicit recursive package tracking now refreshes .dirty for both edits and cleanups, proven on clean commit 8525e2b. R9 make perf-ledger could produce output that failed the Flowmark gate; regeneration now finishes with the standard pinned flowmark --auto . pass and was verified clean across all 15 experiment artifacts. The explicit-path/UI audit also found and repaired stale README, runbook, changelog, help, skill, design principles, and active-spec claims.
