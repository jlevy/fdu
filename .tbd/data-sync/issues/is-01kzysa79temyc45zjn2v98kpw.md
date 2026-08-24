---
type: is
id: is-01kzysa79temyc45zjn2v98kpw
title: Content sidecar load is the layer-3 warm cost on Linux
kind: task
status: open
priority: 1
version: 8
labels:
  - campaign-2
  - macos-agenda
dependencies: []
created_at: 2026-08-14T00:03:55.833Z
updated_at: 2026-08-24T16:21:29.082Z
---
The content sidecar load costs about 370 ms for 14,542 files, roughly 25 microseconds per file, against about 3 microseconds per record for the metadata snapshot. It dominates every warm content run: with a sidecar hit, all three analysis profiles converge on the same warm floor regardless of how much analysis they avoided. Same class of problem as H78 for the metadata snapshot and probably wants the same answer, a layout usable without rebuilding per-record state. Measured in a virtualized-warm Linux regime; see research-2026-08-13-linux-three-tier-baseline.md.

## Notes

exp-070 (2026-08-24): validated the separator fixes (6c7a099, f204abb) against exp-069's accepted binary. content-cache-hit -1.28% [-5.66%, +1.86%] -- inside the pre-registered non-inferiority margin, so exp-069's -31% still describes what ships. content-query +0.94% [+0.08%, +2.38%] excludes zero, but user CPU is flat-to-lower (11,115 -> 11,091 ms) and system CPU falls, so it reads as drift on a load-13 host rather than the mechanism; bounded below the margin either way. OPEN FOLLOW-UP: normalized() returns Cow<Path> and branches at runtime on a constant; a cfg(unix) identity form would be free by construction instead of free-if-the-optimizer-cooperates, retiring the question exp-070 could only bound. One line, its own round. Still ahead on this tier: Path::hash + SipHash on the rollups HashMap (~8% of the warm profile), then the structural form fdu-jxhk.
