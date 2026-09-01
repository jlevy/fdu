---
type: is
id: is-01m01mqq3cqs8ae87qd2d3rydm
title: "H86: consumer representation as one structural experiment"
kind: epic
status: in_progress
priority: 1
version: 17
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
updated_at: 2026-09-01T16:55:01.355Z
started_at: 2026-09-01T15:11:35.154Z
---
One representation decision currently wearing seven hypothesis numbers: worker-local arena entries (fixed-width records), single name arena, children as sorted arena slices, batch-shaped observations carrying parent EntryId, and a one-pass bottom-up roll-up for the cold bootstrap. Measured ceiling on the 450k Linux rig: arena_spike.rs retains an index-shaped result in ~199ms / <=23MiB vs fdu tree view ~849ms / ~279MiB (dut 179ms), tallies exact. Gate with the differential harness (assert_same_image at every worker count), exp-022 precedent for one large structural verdict. Absorbs/supersedes the piecemeal forms in fdu-2ubt, fdu-prph (H19-22), fdu-weey (H60), fdu-fnfc/fdu-uv0s; composes with H85 (arenas make frees thread-local). Pre-registered signal: cold-scan-index wall down >=50% on the 450k Linux subject; peak RSS down >=60%; engine digests byte-identical at 1..N workers.

## Notes

2026-09-01 disk pressure is resolved and validation resumed. The pipelined directory-group builder now supports controls by carrying the verified fixed-path control operation with each complete directory and installing it before siblings become visible. A controls-rich 12-pair uncontrolled exploratory screen versus c6380f7 improved cold-scan-index wall 33.55% (95% interval -36.41% to -33.14%), component 47.43%, allocations 6,024,294 to 987,134, reallocations 601,749 to 104,398, allocated bytes 491,242,604 to 133,101,059, and peak RSS 25.88%. Scanner projection/preparation/reduction counters are zero on the private route and the exact digest matches. Differential worker counts 1..4, controls limits, non-file controls, first public mutation, 92 all-feature scanner tests, and 83 no-default-feature scanner tests pass. The work also fixed a pre-existing specialized-baseline bug that rejected the documented ControlRemove for a non-file .gitignore. A fresh historical run puts cold construction at component parity (331.2 ms candidate vs 332.0 ms pre-rewrite), but default-tree remains about 8% slower by noisy paired median and RSS about 19% higher. Full compact retained layout, promotion, default-tree gate, quiet Darwin verdict, and Linux floor remain open.
