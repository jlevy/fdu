---
type: is
id: is-01m4kjcpw6x6zpv87c9z4m277d
title: Attribute one root's 5% slower summary at the several-roots head on a dense tree
kind: bug
status: open
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-10T18:49:47.654Z
updated_at: 2026-10-13T19:29:16.267Z
started_at: 2026-10-11T05:28:43.852Z
---
exp-220 (Linux, quiet, two-arm, 40 pairs): aggregate-summary on node-modules-dense is slower at 0d6011da (#192's head) than at 23d11b19 (#192's base), paired median +5.28% [+2.27%, +7.62%]; linux-v6.12 +0.92% [-2.75%, +5.38%]. Under H195's registered rule the dense cell is inconclusive, not a regression: the interval lies above zero but crosses the +3% margin, so non-inferiority is neither shown nor refuted (review A1 on PR #194).

The figure depends on the estimator, about +3.5% to +5.3% (docs/project/experiments/evidence/exp-220/order-split.txt): mean +3.53% [+0.31%, +6.90%], Hodges-Lehmann +3.97%, marginal medians -0.68% (90.84 against 90.22 ms); 27 of 40 pairs positive, from -18.87% to +31.94%. Split by which arm ran first it is +6.71% and +2.93% by median (arm about +4.8%, order about +1.9%) but +7.36% and -0.31% by mean (arm about +3.5%, order about +3.8%). It is the only two-build cell of the session whose estimators spread past the 3% margin.

Corroboration: exp-217's three-arm cell run-b1-nmd-retained, both compared arms in outer slots, puts 0d6011da's summary +5.45% [+1.99%, +7.20%] above v0.4.0's on node-modules-dense (20 pairs); on linux-v6.12 (run-b2-lnx-retained) -1.47% [-5.61%, +6.98%].

Counted work does not explain it: instructions +0.3% (callgrind, two runs each: 282.1M/282.4M at 23d11b19 against 282.9M/283.4M at 0d6011da); statx/openat/getdents64 equal at v0.4.0 and 0d6011da (72,076/9,448/18,878; exp-217's trace did not cover 23d11b19's summary).

Also #192's own one-root path on the kernel tree (review A14; exp-217 callgrind.txt, one run each): linux-v6.12 default-tree 339.50M instructions at v0.4.0, 343.93M at 23d11b19, 352.59M at 0d6011da, with the consumer thread 230.38M to 232.66M (+1.0%) from 23d11b19 to 0d6011da. Whole-process counts move a few million between valgrind runs.

H195 predicted no per-entry cost for one root. Next: a two-arm rerun at more pairs with a second estimator beside the paired median (fdu-ee2u); a profile with hardware counters (perf stat cycles, cache and branch misses) on a host that exposes them, both binaries; check code layout (the probe's text grew 287 KB) and the summary route's thread and channel behaviour.

## Notes

2026-10-13 pre-tag reading, H198 (pre-registered 871bc6bd, recorded exp-222 on claude/serene-keller-dlvsbd at b502f4a0): two-arm, 100 pairs, quiet, 4-vCPU Firecracker guest (Xeon 2.8 GHz). The release candidate 1c42c91f against the age column's base 23d11b19, aggregate-summary on node-modules-dense: paired median -0.22% [-3.50%, +3.89%], mean +1.70% [-1.49%, +4.93%], Hodges-Lehmann +1.70%, order effect -0.21%. Inconclusive at +3% by H195's registered rule, so the tag proceeds; exp-220's +5.28% does not reproduce. Conditional cell (C), the several-roots head 0d6011da against the candidate: -3.77% [-7.44%, +0.91%], order effect +3.59%. Bead stays open for attribution on a host with hardware counters, as registered.
