---
type: is
id: is-01kzysa79temyc45zjn2v98kpw
title: Content sidecar load is the layer-3 warm cost on Linux
kind: task
status: open
priority: 1
version: 6
labels:
  - campaign-2
  - macos-agenda
dependencies: []
created_at: 2026-08-14T00:03:55.833Z
updated_at: 2026-08-23T10:46:03.777Z
---
The content sidecar load costs about 370 ms for 14,542 files, roughly 25 microseconds per file, against about 3 microseconds per record for the metadata snapshot. It dominates every warm content run: with a sidecar hit, all three analysis profiles converge on the same warm floor regardless of how much analysis they avoided. Same class of problem as H78 for the metadata snapshot and probably wants the same answer, a layout usable without rebuilding per-record state. Measured in a virtualized-warm Linux regime; see research-2026-08-13-linux-three-tier-baseline.md.

## Notes

exp-069 (H102) accepted 2026-08-23: ContentIndex::files keyed by path bytes -> content-cache-hit -31.00% [-31.43%, -27.80%] on metabrowser-clone (52k files), code-sloc/document cache hits -30%, content-query -67% (unpredicted, same mechanism via query_report.rs file lookups), cold jobs unchanged, digest identical, RSS flat. Remaining on this tier: Path::hash + SipHash on the rollups HashMap<PathBuf> and the loader's candidate HashMap (~8% of the warm profile) -- same byte-keyed treatment; then the structural form fdu-jxhk. The bead's original 25 us/file figure is now ~7.8 us/file (409 ms / 52,717) on this host.
