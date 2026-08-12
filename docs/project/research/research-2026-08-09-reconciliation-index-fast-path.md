# Research: Reconciliation Index Fast-Path Evidence

**Date:** 2026-08-09

**Status:** Implemented and validated

## Decision

Keep two related reductions in unchanged reconciliation work:

1. capture each known child’s state and generation-safe identity directly from the
   coherent child iteration instead of joining its path and looking it up repeatedly;
2. when reconciliation has an exclusive `&mut Index`, count an exact metadata match as
   unchanged without allocating or applying an upsert that is guaranteed to be a no-op.

Shared `IndexHandle` reconciliation retains conditional observations and ABA arbitration
across its read/write lock boundaries.
No state mutation bypasses `Delta`; the exclusive shortcut removes only an operation
that would not mutate state or advance the clock.

On one exact 100,000-entry corpus, all nine alternating before/after pairs improved.
The median component duration fell from 714.231 ms to 575.499 ms, and the median paired
change was -18.15%.

## Trigger

The first validated unchanged-revalidation curve for the portable engine measured:

| Required descendants | Component duration | External wall time | Peak RSS |
| ---: | ---: | ---: | ---: |
| 10,000 | 72.258 ms | 112.419 ms | 8.2 MB |
| 100,000 | 725.023 ms | 1.096 s | 53.6 MB |
| 500,000 | 8.186 s | 10.092 s | 254.9 MB |
| 1,000,000 | 62.906 s | 66.897 s | 494.1 MB |

All four exploratory samples passed the exact engine oracle.
The uncontrolled local curve is not a product benchmark, but it decisively shows that
the current 500k target is not met and that redundant per-entry index work is worth
removing before the syscall walker and bounded parallel sweep land.

## Method

- Before source: clean commit `7addd113a78709bfa2eaac9236008afdda0d5816`
- Before probe SHA-256:
  `1b7955a10482caaa62bf1ea2aa1f7cf4c6eebc84412ad355457ed788803ffca7`
- After probe SHA-256:
  `5584ac632e16b3feb2777089822da5923375e47fb36d8c16f75c29de39ec9241`
- Build:
  `cargo build --locked --release -p fdu --example perf_probe --no-default-features`
- Toolchain: Rust 1.97.1, target `aarch64-apple-darwin`
- Host class: Apple M1 Pro, 10 logical CPUs, Darwin 25.5.0, APFS
- Corpus: `balanced`, 100,000 required descendants
- Observed manifest SHA-256:
  `8fb3c65db602b2b56714da03880dd75cbf54a24a336f8657d52d03dff7a5d800`
- Per-run engine digest:
  `6707f0465fbee821a4001c99855cc8549fddcafb5b63a13f750364c60aaa4bd3`
- Snapshot: one compatible snapshot, read-only across both binaries
- Filesystem-cache state: uncontrolled; no cold or verified-warm label is implied
- Samples: one unrecorded warmup per binary, then nine alternating pairs with pair order
  reversed each time
- Acceptance: successful process, complete reconciliation, no effective mutation,
  exactly 100,000 unchanged entries, and exact engine-digest equality in every sample

## Results

| Measure | Before | After | Change |
| --- | ---: | ---: | ---: |
| Median component duration | 714.231 ms | 575.499 ms | -19.42% |
| Paired median change | — | — | -18.15% |
| Best/worst paired change | — | — | -23.32% / -14.68% |

Raw component durations, in milliseconds:

| Revision | Samples |
| --- | --- |
| Before | 750.516, 699.016, 734.987, 714.231, 687.591, 714.460, 697.091, 681.316, 733.096 |
| After | 575.499, 572.178, 627.060, 551.278, 585.170, 556.354, 578.617, 568.442, 582.249 |

Paired changes were -23.32%, -18.15%, -14.68%, -22.82%, -14.90%, -22.13%, -17.00%,
-16.57%, and -20.58%.

## Correctness and Concurrency Review

- Present-child expectations constructed from the child `EntryId` are identical to
  individual path-based expectations; a regression test locks that equivalence.
- Exclusive reconciliation owns `&mut Index`, so another index producer cannot change
  the captured state before the no-op decision.
- Shared reconciliation still emits and applies conditional observations, preserving
  generation, revision, structural-revision, and absence-guard arbitration.
- Added tests prove that both paths report the same unchanged count, publish no delta,
  and do not advance the index clock for an unchanged tree.
- Existing addition, edit, removal, kind-change, partial-error, stale-arbitration,
  invalidation, scope, and concurrent-reader tests continue to own changed paths.
- The optimization introduces no dependency, unsafe code, format/API change, or new
  mutation path.

## Limitations and Follow-Up

This is one local host, one topology, one scale, and an uncontrolled filesystem cache.
The improvement is causal for the localized index work but is not evidence that the 500k
product target is met.
Even a linear projection of the improved 100k result remains well above one second at
500k, while the first observed tail was substantially worse.

The next steps remain the safe directory-enumeration shortcut, Linux dirfd-relative
syscall walker, bounded parallel stat sweep, repeated 500k evidence, and claim-grade
host/build manifests.
Large repeated trials also require the verified base-corpus clone work tracked by
`fdu-6wu0`; the first 1M setup spent far longer creating and verifying the corpus than
running smaller engine samples.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
