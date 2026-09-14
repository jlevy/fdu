---
type: is
id: is-01m01mqq3cqs8ae87qd2d3rydm
title: "H86: consumer representation as one structural experiment"
kind: epic
status: in_progress
priority: 1
version: 22
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-performance-campaign-2.md
delegate: codex@spud10.local
labels:
  - perf
  - campaign-2
  - stack-followup
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
updated_at: 2026-09-14T01:49:45.723Z
started_at: 2026-09-01T15:11:35.154Z
---
One representation decision currently wearing seven hypothesis numbers: worker-local arena entries (fixed-width records), single name arena, children as sorted arena slices, batch-shaped observations carrying parent EntryId, and a one-pass bottom-up roll-up for the cold bootstrap. Measured ceiling on the 450k Linux rig: arena_spike.rs retains an index-shaped result in ~199ms / <=23MiB vs fdu tree view ~849ms / ~279MiB (dut 179ms), tallies exact. Gate with the differential harness (assert_same_image at every worker count), exp-022 precedent for one large structural verdict. Absorbs/supersedes the piecemeal forms in fdu-2ubt, fdu-prph (H19-22), fdu-weey (H60), fdu-fnfc/fdu-uv0s; composes with H85 (arenas make frees thread-local). Pre-registered signal: cold-scan-index wall down >=50% on the 450k Linux subject; peak RSS down >=60%; engine digests byte-identical at 1..N workers.

## Notes

2026-09-02 Linux evidence stage (exp-103; first recorded as exp-102, renumbered 2026-09-13 because #52 owns exp-102), run on a 4-core KVM Linux VM against the 450,001-entry generated subject. Figures below are as corrected by PR #54 review 5192260482 (fdu-szu3).

Relative gates PASS against immediate control c6380f7 over twelve paired interleaved trials, zero invalid samples, exact engine digests at workers 1-4, no post-run tree drift: default-tree wall -31.70% [-34.31%, -29.15%] with paired peak RSS -35.05%; cold-scan-index wall -18.16% [-24.25%, -13.72%] with paired peak RSS -49.16%; opened-discovery -10.73% [-13.97%, -8.24%] against a +3% noninferiority bound. Candidate wall p95/median <= 1.109 (1.137 across all recorded metrics), inside 1.5. Candidate max/min is UNVERIFIED: the fdu arms' run JSON lived only on the VM and was never committed, so no per-trial sample survives (field and durable run JSON tracked as fdu-c4jr).

Floor gates FAIL on the index tier, which the campaign-2 plan measures with default-tree. parfloor stat gives a 316.4 ms parallel syscall floor; arena_spike under the preregistered low-churn warm-steady cell gives 362.8 ms / 30.5 MiB. Candidate default-tree is 2.60x the syscall floor (gate 1.4x) and 6.59x spike RSS (gate 3x); cold-scan-index, supporting, is 4.86x and 5.03x. Control was 3.76x/10.28x and 6.02x/9.96x, so H86 moved these a long way without reaching them.

The escape hatch does not apply: arena_spike max/min 1.204 and parfloor 1.391 are both under 2.0, so the ratios reject rather than abstain. The margins survive the worst floor samples: default-tree 1.94x against the slowest parfloor sample, and peak RSS above 5x against the largest spike sample.

Residual: parfloor 316 ms vs arena_spike 363 ms is about 15% of wall; default-tree 822 ms is 2.60x the floor in total and 1.60x above it. That points at consumer-side work, and the earlier strace census agrees, but neither floor cell recorded CPU and the candidate default-tree spends about 64% of its CPU in the kernel, so this cell does not locate the residual.

Qualifications: the measured commits 5d7b86f/c6380f7 predate #52's restack (equivalents f972250/a74ac2a) and are tagged perf/h86-linux-candidate and perf/h86-linux-control. Their binaries predate the gitignore performance builds (1a39be9), the controls-off default-tree probe (64c6e61), and the point-lookup preflight (ad52469). Deviations from the preregistration: the aggregate <=1.25x gate was not evaluated, the subject is not the 450,463-entry primary one, and fdu's worker count was neither pinned nor recorded. The next floor cell should pin fdu --threads to the floor tools' worker count, record per-sample CPU for both floor tools, and commit its run JSON.

Caveat: exploratory stage, uncontrolled shared KVM host, and the schema has no host-pressure field. NOT a quiet-host verdict. The Linux floor claim and this epic remain open. Evidence: docs/project/experiments/exp-103-*.md and docs/project/research/research-2026-09-02-linux-floor-cell-for-h86.md
