---
type: is
id: is-01m4kjcpw6x6zpv87c9z4m277d
title: Attribute one root's 5% slower summary at the several-roots head on a dense tree
kind: bug
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-10T18:49:47.654Z
updated_at: 2026-10-10T18:49:47.654Z
---
exp-220 (Linux, quiet, two-arm, 40 pairs): aggregate-summary on node-modules-dense is +5.28% [+2.27%, +7.62%] slower at 0d6011da (#192's head) than at 23d11b19 (#192's base), robust to run order (about +4.8% when pairs are split by which arm ran first); linux-v6.12 +0.92% [-2.75%, +5.38%]. Instruction counts differ by +0.3% (callgrind, two runs each: 282.1M/282.4M against 282.9M/283.4M) and statx/openat/getdents64 counts are equal (72,076/9,448/18,878), so the cause is not counted work. H195 predicted no per-entry cost for one root. Next: a profile with hardware counters (perf stat cycles, cache and branch misses) on a host that exposes them, both binaries; check code layout (the probe's text grew 287 KB) and the summary route's thread and channel behaviour.
