---
type: is
id: is-01m1b444cnk1qttdgzms5zz013
title: "PR #48 branch is 3.6-10x slower than main: allocator churn, not I/O"
kind: bug
status: in_progress
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - performance
  - regression
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-31T05:19:25.577Z
updated_at: 2026-08-31T06:38:15.149Z
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

## ROOT CAUSE FOUND (bisect complete, mechanism identified)

First, a correction to this record: at 23:25 the global `/Users/levy/.cargo/bin/fdu` was
replaced with a MAIN build (gb75bf85a3) -- presumably this investigation's own comparison
step. Any measurement of "the installed binary" after 23:25 measured main. I lost an hour
to exactly this before checking `--version`; the regression then reproduced perfectly with
an explicitly built branch tip: 1.7-1.84s vs 0.42-0.54s on ~/.rustup/toolchains,
allocations 4.15M vs 982k. All conclusions below are from explicitly version-checked
binaries.

## The bisect: cumulative, not one commit

Debug-build oracle (signature survives profiles; counters deterministic):

    1dac219                              983k   clean
    6d18e87 make index commits exact    1.73M   +0.75M
    947cd49 route producers through     3.54M   +1.8M
    c5d1780 add opened-root journal     4.68M   +1.1M
    tip c853f7c                         4.15M

The named suspects 13fe8b4/c0fb6de (portable path) are EXONERATED by timeline: 27aeed0e
already carried the full signature and predates both.

## Mechanism at tip

The one-shot cold scan now routes every entry through the exact-commit pipeline:

1. Walker materializes `Op::Upsert { path: path.clone(), .. }` per entry (947cd49).
2. `Index::apply` -> `prepare_observation` -> `canonical_relative_path` per op:
   `normalize(path).into_iter().collect::<PathBuf>()` -- rebuilds every path
   component-by-component, a PathBuf grown by repeated push. That is the 20x
   reallocation signature. These paths come from fdu's own walker and are canonical by
   construction; they are re-canonicalized as untrusted input once per entry.
3. Commit machinery clones effective changes / impact paths again.

Journal EXONERATED at tip: patching `apply` to `journal: false` changed nothing
(4.165M vs 4.167M). c5d1780's historical +1.1M must have moved or been absorbed;
attribute per-mechanism shares at tip by profiling, not by that table.

## The design flaw (instance 2 of the PR #50 pattern)

One mandatory commit pipeline for every lifecycle. The one-shot CLI pays per-entry
validation, op materialization, and clone-per-change built for concurrent opened-root
mutation, with no consumer that can ever observe the difference. PR #50's control table
is instance 1 (state retained for a reader that does not exist); this is instance 2
(pipeline work for arbitration that cannot happen -- one-shot has exactly one producer
and zero concurrent readers). The codebase contains the correct pattern applied once:
serving indexes are `None` unless opened (`new_opened_with_...`), and the CLI never pays
for them.

Design-principles rules violated: "Speed changes are decided by measurement, never by
argument" -- the rework was filed as architecture, so nobody ran perf-compare, and the
floor doc's own budget ("one enumeration and one metadata read per entry, and nothing
else") was spent 3.6x by the spine while campaign-2 fought for 3%.

## Fix direction

- `canonical_relative_path`: check `path_is_relative_normal` first and pass the already-
  canonical path through untouched (move, not rebuild). Walker paths always take this
  lane; only genuinely non-normal input pays normalization. Kills the realloc storm.
- `apply` should consume the Observation (walker already `mem::take`s the batch) so
  prepare can move paths instead of cloning.
- Audit commit-side clones (effective changes, impact sets) for the one-shot lifecycle.
- Acceptance stands as written, plus: the counters-based regression check is CI-able
  because allocation counts are deterministic -- pin allocs/entry on a fixture tree in
  `make check`, so the next per-entry-path change is caught at review, not at benchmark.
