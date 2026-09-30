---
type: is
id: is-01m3pc4384h1zyw0hdjyzp3p7h
title: Report Linux native listings in ScanBackendDiagnostics (H169 Part C; public API change, 0.3.0)
kind: task
status: open
priority: 3
version: 4
labels: []
dependencies: []
child_order_hints:
  - is-01m3qfdxw5sxhvpthzh1z40dmw
created_at: 2026-09-29T10:42:15.428Z
updated_at: 2026-09-29T22:00:18.090Z
---
H169's native reader (perf/h169-linux-reader) has a Part C commit (bda4ade1) that adds three public fields to ScanBackendDiagnostics (linux_dents_attempts/successes/fallbacks) plus the cli_exit prefix and the measure.py dirs_read arithmetic. ScanBackendDiagnostics is public (lib.rs: pub mod scan) with all-public fields and no #[non_exhaustive], so adding fields breaks struct-literal construction and fails cargo-semver-checks for a 0.2.x patch. Deferred by the overnight loop per the 0.2.2 plan's no-public-API-change rule; the maintainer decides for 0.3.0 (consider #[non_exhaustive] at the same time). The only harness job requiring scan diagnostics is macOS-only adaptive-scan-index.

## Notes

2026-09-29: the next release is 0.3.0 (R161-1: Counts gained public fields and the maintainer accepted a breaking release), so this public ScanBackendDiagnostics change could ship in the same release instead of waiting. Branch perf/h169-linux-reader holds it.
