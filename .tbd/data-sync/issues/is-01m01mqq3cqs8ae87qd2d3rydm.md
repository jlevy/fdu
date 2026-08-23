---
type: is
id: is-01m01mqq3cqs8ae87qd2d3rydm
title: "H86: consumer representation as one structural experiment"
kind: epic
status: open
priority: 1
version: 11
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
labels:
  - perf
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
updated_at: 2026-08-23T01:52:37.321Z
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
