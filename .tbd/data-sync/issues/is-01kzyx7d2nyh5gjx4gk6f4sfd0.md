---
type: is
id: is-01kzyx7d2nyh5gjx4gk6f4sfd0
title: Clarify content coverage and analysis size limit
kind: bug
status: closed
priority: 1
version: 4
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-14T01:12:17.748Z
updated_at: 2026-08-14T01:45:22.491Z
closed_at: 2026-08-14T01:45:22.490Z
close_reason: Full-file analysis, non-fatal expected coverage, grouped help, documentation, benchmark contracts, tests, and cleanup guidance are complete; full handoff gate passes.
---
Distinguish expected content coverage from operational failure, remove the per-file analysis read limit, read eligible text through EOF, group CLI help, and align README, Python, benchmark, design, feature-spec, decision-record, and performance-loop documentation.

## Notes

Implemented full-file analysis with a 17 MiB generated C regression; invalid UTF-8, binary, and unsupported analyzers remain explicit non-fatal coverage; only I/O, changed-during-read, and stale commits are partial. Removed --max-file-size and Python max_file_size. Updated benchmark contracts, help groups, golden coverage, and generated-corpus cleanup guidance. Verified common CLI compositions manually. CARGO_INCREMENTAL=0 make check passes, including 343 all-feature Rust tests, 97 CLI goldens, 64 benchmark tests, 69 real-tree harness tests, no-default/MSRV matrices, docs, Python wheel/smoke/concurrency, supply-chain checks, and audits.
