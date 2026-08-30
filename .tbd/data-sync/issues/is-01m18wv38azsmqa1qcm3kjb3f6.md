---
type: is
id: is-01m18wv38azsmqa1qcm3kjb3f6
title: MetaBrowser pre-push runs a wall-clock budget that measures the machine
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-30T08:33:40.617Z
updated_at: 2026-08-30T08:33:40.617Z
---
`tests/test_folder_treemap_layout_js.py::test_treemap_layout_behavior` asserts that
`layoutTree` places 800 cells within 16ms of wall clock, in a `node` subprocess, and the
lefthook `pre-push` hook runs the full suite. So a push is refused whenever the developer's
machine is busy, for a reason that has nothing to do with the change being pushed.

Observed: the push of the canonical-path alignment was blocked by

    budget: layoutTree emitted 800 cells in 298701.48ms (spec budget 16ms)

with load averages 8.51 / 14.73 / 23.98 from an unrelated job on the same machine. The
same test had passed minutes earlier in `make verify` (1634 passed) and passed again in
0.38s once the machine was idle. Not a regression, and 18,000x over budget is not a
measurement of the code.

fdu states the rule this violates: "None of this is in `make check`. A timing gate on a
shared CI runner measures the runner." A timing gate in a pre-push hook measures the
developer's laptop, which is worse -- it is shared with everything else they are running.

Options, roughly in order of preference:

- assert the algorithm's shape rather than its wall clock: cell count, no overlap, bounded
  recursion depth, work proportional to input
- keep the timing check but move it out of `pre-push` into a place where a slow result is
  reported rather than blocking
- raise the budget far enough that only a real regression trips it, and say in the failure
  message that load is the usual cause

The last is the weakest: a budget loose enough to survive load is too loose to catch the
2x regression a 16ms budget was presumably chosen to catch.
