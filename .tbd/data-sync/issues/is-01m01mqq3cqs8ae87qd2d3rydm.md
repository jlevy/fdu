---
type: is
id: is-01m01mqq3cqs8ae87qd2d3rydm
title: "H86: consumer representation as one structural experiment"
kind: epic
status: in_progress
priority: 1
version: 19
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
updated_at: 2026-09-01T19:30:51.473Z
started_at: 2026-09-01T15:11:35.154Z
---
One representation decision currently wearing seven hypothesis numbers: worker-local arena entries (fixed-width records), single name arena, children as sorted arena slices, batch-shaped observations carrying parent EntryId, and a one-pass bottom-up roll-up for the cold bootstrap. Measured ceiling on the 450k Linux rig: arena_spike.rs retains an index-shaped result in ~199ms / <=23MiB vs fdu tree view ~849ms / ~279MiB (dut 179ms), tallies exact. Gate with the differential harness (assert_same_image at every worker count), exp-022 precedent for one large structural verdict. Absorbs/supersedes the piecemeal forms in fdu-2ubt, fdu-prph (H19-22), fdu-weey (H60), fdu-fnfc/fdu-uv0s; composes with H85 (arenas make frees thread-local). Pre-registered signal: cold-scan-index wall down >=50% on the 450k Linux subject; peak RSS down >=60%; engine digests byte-identical at 1..N workers.

## Notes

2026-09-01 checkpoint: the pipelined directory-group builder handles controls and reaches practical historical cold-construction parity. Controls-rich wall improved 33.55% (95% interval -36.41% to -33.14%), component 47.43%, allocations 6.02M to 0.99M, allocated bytes 491 MB to 133 MB, and peak RSS 25.88%, with exact digests and zero scanner preparation/projection/reduction. Generic monomorphized orchestration is retained after exp-099 measured wall +0.16% (interval -1.46% to +0.81%); exp-098 records the rejected dynamic form. The valid historical run gives cold wall +0.93% (interval -5.63% to +3.83%) and component -0.39% (interval -3.02% to +4.04%); peak RSS remains about 17-22% higher. Negative-tested platform-calibrated allocation and zero-work guards pass. Commit 88304cb is green across the complete Ubuntu/macOS/Windows stacked-PR CI matrix in run 33549100437. Quiet-host strict noninferiority, retained-layout/RSS attribution, promotion, and Linux evidence remain open.
