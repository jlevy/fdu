---
type: is
id: is-01m3pc4384h1zyw0hdjyzp3p7h
title: Report Linux native listings in ScanBackendDiagnostics (H169 Part C; public API change, 0.3.0)
kind: task
status: closed
priority: 3
version: 6
labels: []
dependencies: []
child_order_hints:
  - is-01m3qfdxw5sxhvpthzh1z40dmw
created_at: 2026-09-29T10:42:15.428Z
updated_at: 2026-09-30T05:17:09.080Z
closed_at: 2026-09-30T05:17:09.080Z
close_reason: "Done in d92b9ba5: bda4ade1 cherry-picked (-x) onto claude/stability-fixes. Conflicts were the diagnostics test only; resolved by merging HEAD's assertions with the ported triplet, and the single-worker scan now asserts attempts == successes + fallbacks and dirs_read == successes + portable reads (the serial walk lists natively here). Hooks added to list_directory too, so every route that lists through the reader counts native listings, with a declined directory counted as a fallback and then a portable attempt. CHANGELOG Breaking bullet for the three ScanBackendDiagnostics fields; platform-tuning.md's section rewritten to document the three backend JSON keys and their arithmetic. scan diagnostics tests, cli_exit, clippy, fmt, docs-format, and the realtree harness test (test_linux_scan_counts_native_listings_beside_portable_ones, 104 tests) pass; no CLI golden carries the trace, so none changed. Full make check re-run after."
resolution: null
duplicate_of: null
---
H169's native reader (perf/h169-linux-reader) has a Part C commit (bda4ade1) that adds three public fields to ScanBackendDiagnostics (linux_dents_attempts/successes/fallbacks) plus the cli_exit prefix and the measure.py dirs_read arithmetic. ScanBackendDiagnostics is public (lib.rs: pub mod scan) with all-public fields and no #[non_exhaustive], so adding fields breaks struct-literal construction and fails cargo-semver-checks for a 0.2.x patch. Deferred by the overnight loop per the 0.2.2 plan's no-public-API-change rule; the maintainer decides for 0.3.0 (consider #[non_exhaustive] at the same time). The only harness job requiring scan diagnostics is macOS-only adaptive-scan-index.

## Notes

2026-09-29: the next release is 0.3.0 (R161-1: Counts gained public fields and the maintainer accepted a breaking release), so this public ScanBackendDiagnostics change could ship in the same release instead of waiting. Branch perf/h169-linux-reader holds it.

2026-09-30: maintainer approved the recommendations: fdu-8f6k (Counts #[non_exhaustive]) and fdu-q7hf (reader diagnostics fields, port of bda4ade1) ship in 0.3.0 on claude/stability-fixes; fdu-4nue's output is by design (surface architecture: machine List materializes every row) and the bead is now the performance follow-up for that machine path.
