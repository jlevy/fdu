---
type: is
id: is-01m01mqq3cqs8ae87qd2d3rydm
title: "H86: consumer representation as one structural experiment"
kind: epic
status: open
priority: 1
version: 6
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
dependencies: []
child_order_hints:
  - is-01m00ft85qkve3wbq52c7wjjs6
  - is-01kzwk20wzb7qcahfa3hq6mn4f
  - is-01kzwkryrdy9nfs1bx79c3eyen
  - is-01kzzj0bqfxfgxakh7a0xhanqd
  - is-01kzzj0c367rtcr2vxb8wrkz2w
created_at: 2026-08-15T02:41:38.411Z
updated_at: 2026-08-15T02:42:43.950Z
---
One representation decision currently wearing seven hypothesis numbers: worker-local arena entries (fixed-width records), single name arena, children as sorted arena slices, batch-shaped observations carrying parent EntryId, and a one-pass bottom-up roll-up for the cold bootstrap. Measured ceiling on the 450k Linux rig: arena_spike.rs retains an index-shaped result in ~199ms / <=23MiB vs fdu tree view ~849ms / ~279MiB (dut 179ms), tallies exact. Gate with the differential harness (assert_same_image at every worker count), exp-022 precedent for one large structural verdict. Absorbs/supersedes the piecemeal forms in fdu-2ubt, fdu-prph (H19-22), fdu-weey (H60), fdu-fnfc/fdu-uv0s; composes with H85 (arenas make frees thread-local). Pre-registered signal: cold-scan-index wall down >=50% on the 450k Linux subject; peak RSS down >=60%; engine digests byte-identical at 1..N workers.
