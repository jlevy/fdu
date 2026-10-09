---
type: is
id: is-01m495ercwjpy2gbztb68mg5tn
title: 0.3.0 CHANGELOG mentions a --diagnostics flag the CLI does not have
kind: bug
status: closed
priority: 2
version: 5
labels: []
dependencies: []
parent_id: is-01m48xs24ec44m1prxyt7xany5
created_at: 2026-10-06T17:51:18.939Z
updated_at: 2026-10-09T02:05:25.055Z
closed_at: 2026-10-09T02:05:25.054Z
close_reason: "Merged in #185 (f1ff2c79): the 0.3.0 CHANGELOG entry names the FDU_SCAN_DIAGNOSTICS=1 trace."
resolution: null
duplicate_of: null
---
CHANGELOG.md's 0.3.0 section names a --diagnostics flag; fdu --help on main has no such flag. Correct the entry to the real flag or remove it. Found by the README senior review (2026-10-06).

## Notes

Fixed on claude/release-0.4.0-prep (237696eb, PR for fdu-k60m); close when it merges.
