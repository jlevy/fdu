---
type: is
id: is-01kzysa79temyc45zjn2v98kpw
title: Content sidecar load is the layer-3 warm cost on Linux
kind: task
status: open
priority: 1
version: 9
labels:
  - campaign-2
  - macos-agenda
dependencies: []
created_at: 2026-08-14T00:03:55.833Z
updated_at: 2026-09-14T15:38:24.160Z
---
The content sidecar load costs about 370 ms for 14,542 files, roughly 25 microseconds per file, against about 3 microseconds per record for the metadata snapshot. It dominates every warm content run: with a sidecar hit, all three analysis profiles converge on the same warm floor regardless of how much analysis they avoided. Same class of problem as H78 for the metadata snapshot and probably wants the same answer, a layout usable without rebuilding per-record state. Measured in a virtualized-warm Linux regime; see research-2026-08-13-linux-three-tier-baseline.md.

## Notes

exp-070 (2026-08-24): validated the separator fixes (6c7a099, f204abb) against exp-069's accepted binary. content-cache-hit -1.28% [-5.66%, +1.86%] -- inside the pre-registered non-inferiority margin, so exp-069's -31% still describes what ships. content-query +0.94% [+0.08%, +2.38%] excludes zero, but user CPU is flat-to-lower (11,115 -> 11,091 ms) and system CPU falls, so it reads as drift on a load-13 host rather than the mechanism; bounded below the margin either way. OPEN FOLLOW-UP: normalized() returns Cow<Path> and branches at runtime on a constant; a cfg(unix) identity form would be free by construction instead of free-if-the-optimizer-cooperates, retiring the question exp-070 could only bound. One line, its own round. Still ahead on this tier: Path::hash + SipHash on the rollups HashMap (~8% of the warm profile), then the structural form fdu-jxhk.
2026-09-14 (Linux session): the "Path::hash + SipHash on the rollups HashMap (~8% of the
warm profile)" item named above has been taken and REFUTED as H103/exp-104.

Candidate: rollups keyed by the byte-ordered PathKey exp-069 built, hashed with an
in-crate FxHash-style mixer (no dependency, no unsafe, 66 lines, no public signature
touched). Verified identical before timing: content digest and engine digest
byte-identical, 95,933 cache hits on both arms.

Result on a 102,318-entry linux kernel checkout, 40 interleaved pairs: wall
+0.05% [-0.82%, +0.86%], component +0.28%, RSS flat. REJECT -- and a tight interval
centred on zero, not an underpowered reading.

The mechanism is real and it does not matter. Instructions fell 1.69% on the kernel
subject and 3.18% on a 3,077-entry cargo-registry one, and neither wall nor CPU moved.
Two results transfer:

1. This tier is no longer instruction-bound. 219M instructions off a 12.9B run bought
   nothing, because SipHash rounds and component parsing are high-IPC while what remains
   is memory-stalled. Screen any later "fewer instructions" hypothesis on the warm
   content open against this result, not against the 3% bar in the abstract.
2. The saving halves with scale (3.18% -> 1.69% from 3k to 102k entries) because the
   snapshot parse grows faster than the roll-up hashing does. A screening subject
   overstates this class by about 2x.

Also corrected: the ~8% in H102's registry entry was the whole of merge_ancestors' map
work, read off a profile that still included the probe's own oracle digest at 21.06%. A
caller tree puts <Path as Hash>::hash at 3.98% of profile and 5.2% of engine.

The other named item here, a cfg(unix) identity form of normalized(), should NOT get a
measurement round of its own after this: it is a one-line instruction saving on the tier
that just measured zero for instruction savings, and exp-070 already bounded it inside
its non-inferiority margin. Do it as a clarity change or not at all.

What is left on this bead is the structural form (fdu-jxhk) and the layout/allocation
answer H78/H83 point at. That is where this tier's cost now is.
