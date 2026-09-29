---
type: is
id: is-01m3pc4384h1zyw0hdjyzp3p7h
title: Report Linux native listings in ScanBackendDiagnostics (H169 Part C; public API change, 0.3.0)
kind: task
status: open
priority: 3
version: 2
labels: []
dependencies: []
created_at: 2026-09-29T10:42:15.428Z
updated_at: 2026-09-29T16:53:41.202Z
---
H169's native reader (perf/h169-linux-reader) has a Part C commit (bda4ade1) that adds three public fields to ScanBackendDiagnostics (linux_dents_attempts/successes/fallbacks) plus the cli_exit prefix and the measure.py dirs_read arithmetic. ScanBackendDiagnostics is public (lib.rs: pub mod scan) with all-public fields and no #[non_exhaustive], so adding fields breaks struct-literal construction and fails cargo-semver-checks for a 0.2.x patch. Deferred by the overnight loop per the 0.2.2 plan's no-public-API-change rule; the maintainer decides for 0.3.0 (consider #[non_exhaustive] at the same time). The only harness job requiring scan diagnostics is macOS-only adaptive-scan-index.

## Notes

Code location (2026-09-29): the Part C diagnostics fields are implemented in commit bda4ade1 on branch perf/h169-linux-reader, pushed to origin at the maintainer's request so it survives the container. Rebase or port it onto the 0.3.0 line when the public API change is taken up.
