---
type: is
id: is-01m01mqq3cqs8ae87qd2d3rydm
title: "H86: consumer representation as one structural experiment"
kind: epic
status: in_progress
priority: 1
version: 20
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
updated_at: 2026-09-02T15:44:55.506Z
started_at: 2026-09-01T15:11:35.154Z
---
One representation decision currently wearing seven hypothesis numbers: worker-local arena entries (fixed-width records), single name arena, children as sorted arena slices, batch-shaped observations carrying parent EntryId, and a one-pass bottom-up roll-up for the cold bootstrap. Measured ceiling on the 450k Linux rig: arena_spike.rs retains an index-shaped result in ~199ms / <=23MiB vs fdu tree view ~849ms / ~279MiB (dut 179ms), tallies exact. Gate with the differential harness (assert_same_image at every worker count), exp-022 precedent for one large structural verdict. Absorbs/supersedes the piecemeal forms in fdu-2ubt, fdu-prph (H19-22), fdu-weey (H60), fdu-fnfc/fdu-uv0s; composes with H85 (arenas make frees thread-local). Pre-registered signal: cold-scan-index wall down >=50% on the 450k Linux subject; peak RSS down >=60%; engine digests byte-identical at 1..N workers.

## Notes

2026-09-02 Linux evidence stage (exp-102), run on a 4-core KVM Linux VM against the 450,001-entry generated subject.

Relative gates PASS against immediate control c6380f7 over twelve paired interleaved trials, zero invalid samples, exact engine digests at workers 1-4, no post-run tree drift: cold-scan-index wall -18.16% [-24.25%, -13.72%] with peak RSS -49.4%; default-tree wall -31.70% [-34.31%, -29.15%] with peak RSS -35.9%; opened-discovery -10.73% [-13.97%, -8.24%] against a +3% noninferiority bound. Candidate p95/median <= 1.109 and max/min <= 1.324, inside the 1.5/2.0 bounds.

Floor gates FAIL. parfloor stat gives a 316.4 ms parallel syscall floor; arena_spike under the preregistered low-churn warm-steady cell gives 362.8 ms / 30.5 MiB. Candidate cold-scan-index is 4.86x the syscall floor (gate 1.4x) and 5.03x spike RSS (gate 3x); default-tree is 2.60x and 6.59x. Control was 6.02x/9.96x and 3.76x/10.28x, so H86 moved these a long way without reaching them.

The escape hatch does not apply: arena_spike max/min 1.204 and parfloor 1.391 are both well under 2.0, so the ratios are resolved and reject rather than abstain.

Reusable mechanism: parfloor 316 ms vs arena_spike 363 ms means an index-shaped retained result costs only ~15% over raw parallel enumeration, so the residual 2.6x on default-tree is consumer-side and not in the syscall layer.

Caveat: exploratory stage, uncontrolled shared KVM host. Sufficient to reject a floor ratio against same-session denominators; NOT a quiet-host verdict. The Linux floor claim and this epic remain open. Evidence: docs/project/experiments/exp-102-*.md and docs/project/research/research-2026-09-02-linux-floor-cell-for-h86.md
