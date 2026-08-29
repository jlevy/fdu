---
type: is
id: is-01m01mqq3cqs8ae87qd2d3rydm
title: "H86: consumer representation as one structural experiment"
kind: epic
status: open
priority: 1
version: 13
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - perf
  - campaign-2
dependencies:
  - type: blocks
    target: is-01kzzbbjxb78m4rde2gb10kmjk
  - type: blocks
    target: is-01kzxsmcabr3shfgh9644tbdtg
  - type: blocks
    target: is-01kzzj1137r8kjyv0rwfc6ya70
child_order_hints:
  - is-01m00ft85qkve3wbq52c7wjjs6
  - is-01kzwk20wzb7qcahfa3hq6mn4f
  - is-01kzwkryrdy9nfs1bx79c3eyen
  - is-01kzzj0bqfxfgxakh7a0xhanqd
  - is-01kzzj0c367rtcr2vxb8wrkz2w
created_at: 2026-08-15T02:41:38.411Z
updated_at: 2026-08-28T15:32:15.748Z
---
One representation decision currently wearing seven hypothesis numbers: worker-local arena entries (fixed-width records), single name arena, children as sorted arena slices, batch-shaped observations carrying parent EntryId, and a one-pass bottom-up roll-up for the cold bootstrap. Measured ceiling on the 450k Linux rig: arena_spike.rs retains an index-shaped result in ~199ms / <=23MiB vs fdu tree view ~849ms / ~279MiB (dut 179ms), tallies exact. Gate with the differential harness (assert_same_image at every worker count), exp-022 precedent for one large structural verdict. Absorbs/supersedes the piecemeal forms in fdu-2ubt, fdu-prph (H19-22), fdu-weey (H60), fdu-fnfc/fdu-uv0s; composes with H85 (arenas make frees thread-local). Pre-registered signal: cold-scan-index wall down >=50% on the 450k Linux subject; peak RSS down >=60%; engine digests byte-identical at 1..N workers.

## Notes

Campaign 2 Phase B, the centerpiece. Floor-anchored targets supersede the piecemeal
predictions (recorded 2026-08-23):

- index tier <= 1.4x the parallel syscall floor on the primary Linux subject
  (arena_spike measures 1.06x; the band between is the priced contract cost --
  arbitration, progressive publication, error provenance -- stated, not hidden)
- aggregate tier <= 1.25x on the nominated REAL subjects; the ~15-point real-tree tax
  is in scope, so generated-tree evidence understates the win
- p95/median wall spread <= 1.5x where the index tier shows 3.27x today
- peak RSS <= 3x arena_spike's
- assert_same_image at every worker count; >= 1 real tree in the accept set
- one experiment (exp-022 precedent), not seven gated increments

Scope notes: absorbs fdu-0pzh (channel) and H89's headroom (intern from batch context);
decides fdu-refc (S6) as layout; fdu-h7sw and fdu-sk7v re-screen after landing; then
re-pose snapshot economics (Phase D) on the new representation. macOS validation per
the exp-054 pattern before any macOS claim.

Instrument first: fdu-5yjk (diagnostics for the FullIndex plan) and fdu-9ydj
(un-contaminated attribution) before profiling this work.

2026-08-28 (Linux session, from the fdu-33ri scoreboard): TWO PRE-REGISTERED TARGETS NEED A MODE NAMED.

make perf-floor measured arena_spike as BIMODAL on a 76k-entry real subject (/usr): a ~63 ms mode and a ~150 ms mode, selected by how much memory the preceding process churned. On a 48k subject (/opt) it is unimodal and 1.09x floor, reproducing the recorded 1.06x ceiling.

H86 pre-registers "peak RSS <= 3x arena_spike" and "p95/median wall spread <= 1.5x". Both are stated against an arena_spike number that, on a subject of this size, is not a single number. Before the structural experiment runs, those two targets should say which mode they are measured against, or the accept can be won or lost by which hump the ceiling run happened to land in.

Worth noting for the harness generally: p95/median reads a calm 1.16 on that same bimodal distribution, because both humps are individually narrow. max/min reads 4.25. The tail statistic the loop already records does not see this; floor.py reports spread and flags >=2x.

Also relevant to this epic's instrument-first note: the index tier's spawn wall exceeds its own component timer by 149 ms against a 110 ms component, so 57% of a spawn-timed index number is the probe's oracle rather than the engine (see fdu-4xtm).
