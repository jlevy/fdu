---
type: is
id: is-01m01mqq3cqs8ae87qd2d3rydm
title: "H86: consumer representation as one structural experiment"
kind: epic
status: in_progress
priority: 1
version: 15
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
delegate: codex@spud10.local
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
hold: null
hold_until: null
created_at: 2026-08-15T02:41:38.411Z
updated_at: 2026-09-01T15:19:29.943Z
started_at: 2026-09-01T15:11:35.154Z
---
One representation decision currently wearing seven hypothesis numbers: worker-local arena entries (fixed-width records), single name arena, children as sorted arena slices, batch-shaped observations carrying parent EntryId, and a one-pass bottom-up roll-up for the cold bootstrap. Measured ceiling on the 450k Linux rig: arena_spike.rs retains an index-shaped result in ~199ms / <=23MiB vs fdu tree view ~849ms / ~279MiB (dut 179ms), tallies exact. Gate with the differential harness (assert_same_image at every worker count), exp-022 precedent for one large structural verdict. Absorbs/supersedes the piecemeal forms in fdu-2ubt, fdu-prph (H19-22), fdu-weey (H60), fdu-fnfc/fdu-uv0s; composes with H85 (arenas make frees thread-local). Pre-registered signal: cold-scan-index wall down >=50% on the 450k Linux subject; peak RSS down >=60%; engine digests byte-identical at 1..N workers.

## Notes

2026-09-01 preregistration fixed in docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md#h86-preregistration-one-decision-two-evidence-stages. One private controls-disabled detached cold-bootstrap route: worker-owned compact records, numeric parent slots, one name arena, sorted child slices, one bottom-up roll-up, and one-time promotion before any later exact public mutation. Opened/public scan, refresh, observation, reconciliation, and controls-enabled scans retain the current causal exact path. Darwin stage: immutable immediate control c6380f7646524b51dbfcfec7e2efac49bf89d34b, historical parity control b75bf85a33edd9fe65d97df9395072797e54426e, required 113,794-entry MetaBrowser subject, >=12 quiet paired trials, default-tree >=3% with CI below zero, cold same direction, RSS <=0.80x, historical parity/allocation gates, opened <=+3% and <=1.05x allocations with zero arena uses. Linux stage remains required to close H86. arena_spike uses preregistered low-churn warm-steady preparation (three immediate spike warmups, no intervening full-index/memory churn); report p95/median and max/min. Candidate must be <=1.5 and <=2.0; if prepared spike max/min >2.0 its floor ratios remain unresolved rather than selecting a post-hoc mode.
