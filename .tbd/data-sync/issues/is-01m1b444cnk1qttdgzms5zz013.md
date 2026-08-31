---
type: is
id: is-01m1b444cnk1qttdgzms5zz013
title: "PR #48 branch is 3.6-10x slower than main: allocator churn, not I/O"
kind: bug
status: in_progress
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - performance
  - regression
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-31T05:19:25.577Z
updated_at: 2026-08-31T11:14:17.637Z
---
The opened-root-inventory-rewrite branch has an unreported whole-scan performance regression against main that is larger and broader than the control-table cap this epic started from. It affects trees with NO .gitignore files, so it is not control-file I/O.

Measured, 5 runs each, warm, medians, same host, release builds, 'fdu --color never TREE':

| Tree | .gitignore files | main (b75bf85) | branch tip (4ce1539) | installed (27aeed0) | slowdown |
|---|---|---|---|---|---|
| ~/.rustup/toolchains | 0 | 0.44 s | 1.58 s | 1.57 s | 3.6x |
| ~/wrk/github/thinking-scratchpad | 34 | 0.21 s | 1.82 s | 1.78 s | 8.7x |
| ~/wrk/github/metabrowser | 304 | 1.08 s | 10.81 s | 10.64 s | 10.0x |

A clean release build of the branch tip reproduces the stale installed binary almost exactly, so this is in the code, not in one bad binary, and 20+ commits after 27aeed0 nothing has fixed it.

FDU_COUNTERS=1 on ~/.rustup/toolchains (zero .gitignore, isolating the baseline regression) shows the cause is allocation, not work:

IDENTICAL between main and branch:
  directory opens 3775; entries enumerated 119367; metadata stats 119368;
  file opens 0; bytes read 0; upserts applied 119367; roll-up merges 1111246;
  index entries allocated 119367

DIVERGENT:
  allocations     983,424 -> 4,171,260   (4.24x)
  reallocations   138,299 -> 2,785,222   (20.1x)
  frees           983,418 -> 4,171,244   (4.24x)
  bytes allocated 182,126,386 -> 676,493,704 (3.71x)
  page faults     4,563 -> 7,124         (1.56x)

Same syscalls, same index work, ~4.2x the allocations and ~20x the reallocations. Normalised: about 26.7 extra allocations and 22 extra reallocations PER ENTRY (119,367 entries).

A 20x realloc ratio is the signature of a buffer grown by repeated push without reserve, on a per-entry path. Strong suspects on this branch, not yet confirmed by bisect: '13fe8b4 feat: every path has a portable name' and 'c0fb6de refactor: make a portable path a type, not a String'. Both introduce per-entry path construction.

This also explains the field reports better than the cap does: the agent's 1m17s on ~/wrk and 3m37s on ~ were the branch's regression, not fdu-versus-dust. Against main, fdu beats dust; against this branch, dust wins comfortably.

Acceptance: bisect the branch to the commit that introduces the allocation growth; per-entry allocations and reallocations return to main's order; the three trees above land within noise of main; a counters-based regression check exists so the next such change is caught before merge.

## Notes

## Fixes landed on PR #51 (claude/one-shot-commit-cost, stacked on #50)

Commits e8ec821 (allocation fixes) and 897d8fe (the fdu-etfj gate). make check and
cross-lint green; 613 tests pass.

Final medians, release, interleaved vs main b75bf85, five runs:

    toolchains   main 0.37  tip 1.58   fixed 0.70
    metabrowser  main 1.32  tip 11.49  fixed 3.92

Counters (control-free tree): 2.23M allocs / 422k reallocs, from 4.17M / 2.79M
(main: 983k / 138k).

## What remains open on this bead

The effective-change stream and per-op prepare clone: ~6 allocs/entry the one-shot
lifecycle pays for ApplyOutcome/Commit consumers it cannot have. This is the remaining
~1.9x wall gap and it is design work -- lifecycle-gate the commit pipeline's effect
recording the way serving indexes are already gated -- not another local patch.

Acceptance updated: the counters-based regression guard (pin allocs/entry on a fixture
tree in make check) is still unbuilt and should land with the design fix.
