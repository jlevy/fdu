# Feature: Cache Layers and Defaults

**Date:** 2026-08-15

**Author:** fdu project

**Status:** Draft

## Overview

Make the cache participate when it pays and stay out of the way when it does not, so
that simple usage is at parity with a scalar peer like `dust`, richer work is fast the
second time, and an interactive consumer can paint instantly from a snapshot on a tree
too large to walk interactively.

The three product requirements are layers over one engine, not three modes a user must
select:

1. **Parity.** One-shot totals are as fast as fdu can answer them, with no state
   retained and nothing written.
2. **Cached.** Work that is expensive to recompute — content analysis above all — is
   fast on the second run.
3. **Progressive.** When a full walk would take minutes, an approximate answer appears
   immediately from the snapshot, labelled as approximate, converging as verification
   completes, with FSEvents narrowing the revalidation on macOS.

Requirement 3 is already specified in
[progressive results](plan-2026-08-11-fdu-progressive-results.md) and tracked under epic
`fdu-wpa0`; this plan does not restate it.
What is missing, and what this plan covers, is the cost model underneath all three:
which requests should touch the snapshot at all.

## Goals

- A one-shot unfiltered metadata summary pays for a walk and nothing else
- The snapshot is read or written when it avoids work, and skipped when it does not
- The rule is derived from measurement, not from which policy flag was passed
- A user who wants a reusable snapshot can still get one without knowing engine
  internals
- Human and machine renderings agree about whether a cache exists

## Non-Goals

- The progressive session API, lazy open, and provenance composition, which are
  [their own plan](plan-2026-08-11-fdu-progressive-results.md)
- The FSEvents journal, which is
  [its own plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
- Changing what any policy *means* once selected; `--cache off`, `refresh`, `read-only`,
  and `only` keep their documented contracts
- Any claim about macOS or APFS timing; every number below is Linux/ext4 and is
  inherited, not proven, elsewhere

## Background

A field report put fdu at roughly twice the wall time of `dust` on macOS. The
[adaptive-worker campaign](../../reports/report-2026-08-15-adaptive-worker-gap-closure.md)
falsified the leading hypothesis — worker under-scaling — and found fdu *faster* than
`dust` in every measured cell.
It did not reproduce the symptom, because every measurement in that campaign, and in the
tool comparisons before it, used `--cache off`. A default invocation does not.

Measured during review on Linux/ext4 over `/usr` (84,539 entries), nine interleaved
paired trials, warm operating-system cache:

| Arm | Median | µs/entry | Versus parity |
| --- | ---: | ---: | ---: |
| `--cache off --view summary` | 71 ms | 0.84 | baseline |
| `--cache off --view tree` | 132 ms | 1.56 | +93% |
| `--cache auto --view summary`, cold | 214 ms | 2.53 | +204% |
| `--cache auto --view summary`, warm | 161 ms | 1.90 | +131% |
| `fdu PATH` (default), cold | 230 ms | 2.72 | +222% |
| `fdu PATH` (default), warm | 162 ms | 1.92 | +126% |
| `--cache only --view summary` | 81 ms | 0.96 | +18% |

Index construction accounts for +61 ms and the snapshot write for +82 ms.

The mechanism matters more than the totals.
A warm `--cache auto` run performs 84,539 metadata stats: revalidation walks every entry
regardless of what the snapshot holds.
For a metadata query the snapshot therefore avoids no filesystem work and is purely
additive, and even the no-scan `only` tier loses warm because deserialisation costs
about what a warm walk costs.

Where the snapshot does pay, both confirmed on the same host:

| Case | Without cache | With cache | Change |
| --- | ---: | ---: | ---: |
| `--analyze code`, warm | 639 ms | 325 ms | −49% |
| Metadata, **cold** OS cache, via `only` | 277 ms | 118 ms | −57% |

That is the whole cost model: **a snapshot earns its keep when it avoids expensive work
— re-reading file bodies, or a cold filesystem walk — and not when it merely mirrors a
walk that still has to happen.**

## Design

### Approach

Derive snapshot participation from what the request costs, not from the policy flag
alone. Two independent decisions:

- **Plan.** Whether the run retains an index at all.
- **Persistence.** Whether a retained index is written back.

### Components

`execution::plan_report` decides the plan.
`lib::open_with_pending_save` and `lib::spawn_save` decide persistence.
`cli::write_cache_status` renders what exists.

### API Changes

None to the public library or `fdu.report/1`. The summary result is byte-identical; only
the work done to produce it changes.
`CachePolicy` keeps all five variants and their meanings.

## Implementation Plan

### Phase 1: Plan selection (landed with this plan)

- [x] Stop gating the compact transient tier on the cache being unavailable.
  An unfiltered metadata summary with no analysis takes the transient tier under `off`,
  `auto`, and `read-only`; `only` and `refresh` still retain the index because their
  contracts are about the snapshot itself rather than the cheapest exact answer.
- [x] Make human and machine `--cache-status` agree: a request whose every candidate is
  unrecognized reports no snapshots in both renderings, rather than the text form saying
  “No cached snapshots.”
  while JSON describes a file that does not exist.
- [x] Update the CLI help, agent skill, and README, which all told users to pass
  `--cache off` to reach a path that is now the default.

Measured effect: `fdu --view summary PATH` went from 161 ms to 71 ms on the subject
above, and no longer leaves a snapshot behind.

### Phase 2: Persistence, and the workflow Phase 1 breaks

Phase 1 has a consequence that must be resolved before it ships as a default: because a
summary run no longer writes a snapshot, a subsequent `--cache only` on the same tree
has nothing to read and fails.
The `summary` then `only` sequence is a real workflow, and the golden CLI contracts
encode it.

- [ ] Decide and implement how an ordinary run populates a snapshot for later `only`
  reads. The candidate is a persistence gate on the cold-scan path: persist when analysis
  was requested, when the policy is `refresh`, or when the index is large enough that a
  future cold read saves meaningful time.
  Gate on **entry count**, not wall time — entry count is a deterministic property of
  the tree, so behaviour stays reproducible and paired benchmarking is not made
  ambiguous.
- [ ] Choose the threshold from Apple Silicon/APFS measurement.
  The only value this review’s data supports is roughly 250,000 entries, a cold walk of
  about a second on the measured host, and it is Linux/ext4 evidence.
- [ ] Update the affected golden contracts deliberately: `cli-lifecycle` uses a summary
  run as its “a report leaves a snapshot behind” vehicle, and `cli-axes` sequences
  `--cache only` after one.

## Testing Strategy

Unit tests in `execution` cover plan selection per policy, including that a present
snapshot does not force the index for a metadata summary and that `only` and `refresh`
still retain it. The existing `compact_summary_matches_the_indexed_summary_exactly` test
is the exactness guard and must keep passing unchanged: this plan changes cost, never
answers.

Golden CLI contracts cover the user-visible half — which commands leave a snapshot, and
what `--cache-status` reports in both renderings.
`watch_persistence` covers the warm-start save path and needed its setup moved off
`summary` for the same reason Phase 2 describes.

Timing claims come from the performance harness under
[the performance loop](../../guides/performance-loop.md), paired and interleaved, and
are scoped to the regime that produced them.

## Rollout Plan

Phase 1 is a behaviour change with no schema change, so it rolls out with the next
release once Phase 2 resolves the `only` workflow.
Phase 1 must not ship alone: it makes the fast path fast while removing the ordinary way
a snapshot comes to exist.

## Open Questions

- What entry-count threshold does Apple Silicon/APFS support, and does the mechanism —
  revalidation walking the tree regardless — hold there as it does on ext4?
- Should `--cache auto` read an existing fresh snapshot for a metadata summary once
  FSEvents can narrow revalidation to changed subtrees?
  Today a full revalidating walk makes that worthless; scoped revalidation may invert
  it.
- Does the progressive UI want a tier that prefers the snapshot and degrades to a scan,
  rather than the strict `only` that errors when none exists?
  Tracked as `fdu-wu6w`.

## References

- [Progressive results](plan-2026-08-11-fdu-progressive-results.md), epic `fdu-wpa0`
- [FSEvents-scoped revalidation](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
- [Adaptive-worker gap closure](../../reports/report-2026-08-15-adaptive-worker-gap-closure.md)
- [Platform tuning](../../guides/platform-tuning.md), for the rule on inherited
  constants

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
