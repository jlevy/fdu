---
type: is
id: is-01m01mqq3cqs8ae87qd2d3rydm
title: "H86: consumer representation as one structural experiment"
kind: epic
status: in_progress
priority: 1
version: 16
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
updated_at: 2026-09-01T16:10:17.976Z
started_at: 2026-09-01T15:11:35.154Z
---
One representation decision currently wearing seven hypothesis numbers: worker-local arena entries (fixed-width records), single name arena, children as sorted arena slices, batch-shaped observations carrying parent EntryId, and a one-pass bottom-up roll-up for the cold bootstrap. Measured ceiling on the 450k Linux rig: arena_spike.rs retains an index-shaped result in ~199ms / <=23MiB vs fdu tree view ~849ms / ~279MiB (dut 179ms), tallies exact. Gate with the differential harness (assert_same_image at every worker count), exp-022 precedent for one large structural verdict. Absorbs/supersedes the piecemeal forms in fdu-2ubt, fdu-prph (H19-22), fdu-weey (H60), fdu-fnfc/fdu-uv0s; composes with H85 (arenas make frees thread-local). Pre-registered signal: cold-scan-index wall down >=50% on the 450k Linux subject; peak RSS down >=60%; engine digests byte-identical at 1..N workers.

## Notes

2026-09-01 preregistration is fixed in the campaign-2 H86 section. The first Darwin checkpoint now pipelines directory-shaped component records into a private controls-disabled builder while the shared filesystem walker continues; controls-enabled and every opened/public path fall back to the causal scanner reducer. Differential tests cover explicit worker counts 1..4, complete facts/totals/partitions/scope/freshness/state, and the first exact post-bootstrap mutation. The naive join-then-build spike destroyed pipeline overlap (~410 ms versus ~270 ms) and was rejected. Pipelining recovered the loss; merging direct contributions during the walk and borrowing completed directory rollups reduced the final pass to ~5.4 ms with counters enabled. On the 113,794-entry MetaBrowser subject, the latest 12-pair uncontrolled exploratory controls-disabled cold-scan-index result is +0.19% median with 95% CI [-2.96%, +1.78%] versus c6380f7: practical timing parity, not an acceptance verdict. Scoped counters versus c6380f7 fell from 1,107,052 to 1,037,490 allocations (-6.3%), 212,144 to 101,952 reallocations (-51.9%), 216,940,915 to 168,768,301 allocated bytes (-22.2%), and 1,217,448 to 129,013 roll-up merges (-89.4%). The full H86 compact retained layout/name arena/child slices, default-tree >=3%, RSS <=0.80x, opened non-regression, quiet Darwin verdict, historical parity, and Linux floor stage remain open. Validation is temporarily constrained by the host filesystem reaching ENOSPC; cargo check passed after the shared-walker refactor, while the test build could not write its incremental cache.
