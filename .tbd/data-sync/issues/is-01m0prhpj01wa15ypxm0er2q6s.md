---
type: is
id: is-01m0prhpj01wa15ypxm0er2q6s
title: Walk telemetry as typed values in Python
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:32:18.623Z
updated_at: 2026-08-23T17:01:51.279Z
---
Expose the walk telemetry the CLI footer already computes (files and bytes walked, cache tier, fresh vs cached analysis) as typed values delivered beside report/session/watch results, never inside the versioned envelope. Embedded clients run measured loops of their own and need the same evidence.

## Notes

Mirrors PerformanceSummary (execution.rs:59: walked_files, walked_bytes, fresh_files, bytes_read, analysis_ns, cached_files, cached_bytes, source), delivered BESIDE the report exactly as performance_footer (cli.rs:1099) does for text, never inside the versioned envelope. FDU_SCAN_DIAGNOSTICS (cli.rs:39-40) is the precedent for structured run telemetry: a versioned payload on stderr behind a sentinel, tested in crates/fdu/tests/cli_exit.rs.
