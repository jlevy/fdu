---
type: is
id: is-01kzysa79temyc45zjn2v98kpw
title: Content sidecar load is the layer-3 warm cost on Linux
kind: task
status: in_progress
priority: 1
version: 13
delegate: unknown@spud10
labels:
  - campaign-2
  - macos-agenda
dependencies: []
hold: null
hold_until: null
created_at: 2026-08-14T00:03:55.833Z
updated_at: 2026-09-19T06:23:19.333Z
started_at: 2026-09-19T06:07:37.642Z
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

2026-09-14 (PR #58 review, corrected in bd8ed0b): four statements in the note above
overreach the exp-104 evidence. The record now says:

1. The ~8% exp-069 named covered two maps: the roll-up HashMap and the candidate map the
   sidecar loader builds (content_cache.rs:145-152, one insert and one remove per file).
   H103 took the roll-up half only. The loader half is untested and, at two full-path
   hashes per file, is expected to be smaller. The 21.06% oracle share was measured in
   exp-104 on the registry subject, not in exp-069's profile, so "read off a profile that
   still included the oracle" is withdrawn.
2. "Not instruction-bound" was measured on one virtualized 4-core Linux host, warm-steady;
   Apple Silicon and bare metal are unmeasured. High-IPC-removed / memory-stalled-remains
   is plausible, not measured (no IPC recorded).
3. "Halves with scale because the snapshot parse grows faster" is replaced by the
   arithmetic: about 2,000 instructions saved per entry on both subjects (1,950 and
   2,141), against the rest of the per-entry cost being about 2x on the kernel checkout.
   The subjects differ in shape as well as size, and only one was profiled.
4. Component is +0.07% [-0.77%, +1.00%] as a paired change; +0.28% was a ratio of
   medians.

fdu-cfpa is closed as not a defect: the shipped HashMap<PathBuf, _> finds a roll-up under
either separator spelling, because Path's Hash and Eq are component-wise.

2026-09-19 (H112 pre-registration, exp-109): H83 is not refuted for this architecture.
The cheap increments are dead (H102 file-map order landed; H103 Path::hash/SipHash
refuted on wall; H109 Path rewrite not justified). A format rewrite (H78 class) and
fdu-jxhk (EntryId roll-ups, one bottom-up pass) are not the smallest next step.

H112: on deciding-scale content-cache-hit, sidecar restore is not one cost. One named
stage among read, parse (integrity + decode), candidate install
(analysis_candidates + HashMap), and apply (apply_analysis / commit / merge_ancestors)
accounts for a majority of restore time and a wall ceiling of at least 3%.

Metric / accept: determination. A stage dominates if it is >=50% of restore phase time
and >=3% of claim-grade wall. Attachment is a 12-pair content-cache-hit of current best
(98da0c83) vs phase timers, FDU_COUNTERS unset. Quiet if the cell holds; else
uncontrolled. No RAM disk. Timers are not a speed claim; keep if counters-off wall is
non-inferior, revert if they regress wall or fail tests.

If one stage dominates, H83 stays open scoped to that stage. If none does, H83 as
"sidecar restore is the win" is too coarse for one increment.

2026-09-19 (exp-109 / H112): sidecar restore is not one cost. On deciding-scale
metabrowser-clone content-cache-hit (145,931 entries, 133,597 hits, digest
3b8cfa71… unchanged), apply dominates restore. Off-by-default phase timers
(median of 3 counters-on hits): read 0.8%, parse 8.5%, candidates 25.4%, apply
63.3%. Counters-off sample of load_content: apply_analysis 53.6%,
analysis_candidates 20.2%, candidate-map hash 8.6%. Claim-grade wall +0.31%
[-0.81%, +1.63%] vs current best at 98da0c83; non-inferior; timers kept (89
lines, no unsafe). Parse-speed is a dead end. Do not retry H103-class instruction
trims. Next smallest cut is H113 (duplicate analysis_candidates completeness
walk at lib.rs:598-602, 12.6% of content_open). Then this bead's structural
form / fdu-jxhk. Bead stays open.
