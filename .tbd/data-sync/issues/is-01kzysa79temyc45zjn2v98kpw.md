---
type: is
id: is-01kzysa79temyc45zjn2v98kpw
title: Content sidecar load is the layer-3 warm cost on Linux
kind: task
status: open
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-08-14T00:03:55.833Z
updated_at: 2026-08-21T16:37:26.128Z
---
The content sidecar load costs about 370 ms for 14,542 files, roughly 25 microseconds per file, against about 3 microseconds per record for the metadata snapshot. It dominates every warm content run: with a sidecar hit, all three analysis profiles converge on the same warm floor regardless of how much analysis they avoided. Same class of problem as H78 for the metadata snapshot and probably wants the same answer, a layout usable without rebuilding per-record state. Measured in a virtualized-warm Linux regime; see research-2026-08-13-linux-three-tier-baseline.md.

## Notes

2026-08-21 (Linux session): **re-screened after exp-064 landed; the number in this bead is stale.**

This bead sized the sidecar load at about 370 ms for 14,542 files, roughly 25 us per file,
and observed that all three analysis profiles converge on the same warm floor on a sidecar
hit. exp-064 removed a large part of that floor, and it was not where this bead implies.

On a 15,977-file tree the `content-cache-hit` component went **404.1 ms -> 267.6 ms**, i.e.
about 25.3 us -> 16.8 us per file, from two changes inside the load path rather than from a
new layout:

- `ContentIndex::merge_ancestors` was 43.73% of the profile -- a `BTreeMap<PathBuf, _>`
  descent per ancestor per file (`fdu-cq7t`).
- the type cascade could not exit early and ran twice per file (`fdu-9dcj`).

What this changes about the bead's thesis: the convergence on a common warm floor was
mostly **not** "record rebuild cost" of the kind H78 describes for the metadata snapshot.
It was per-file work in the apply path, and a third of it was one hash-versus-tree decision.
So the analogy to H78 is weaker than recorded, and a layout change should not be scoped from
the 370 ms figure.

What is left, from the post-exp-064 caller tree:

- `files: BTreeMap<PathBuf, FileAnalysis>`, whose `remove` was 11.09% through
  `apply_analysis` -- still a path-ordered tree on the hot path, and the obvious next
  increment of the same idea.
- `with_flags`, 4.42% of the pre-change profile, walking path components per file for the
  vendored and documentation flags.
- The structural version of `fdu-cq7t`: key roll-ups by `EntryId` and defer to one bottom-up
  pass, the `fdu-91ts` shape. This is the one that would actually test this bead's
  layout thesis.

Re-measure before scoping anything here; the profile this bead was written against no longer
exists.
