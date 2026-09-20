# Feature: Linux PGO Screen after #97

**Date:** 2026-09-20

**Author:** fdu project

**Status:** Active.
Stacked on [#97](https://github.com/jlevy/fdu/pull/97) at `937a8917`.
This block screens profile-guided optimization on the current Linux release probe.
It is not an engine source change and not a leftover compile.

## Overview

[#97](https://github.com/jlevy/fdu/pull/97) recorded the leftover queue after #94 and
kept two Linux-specific engine cuts: H147 transient batch recycle and H72 `d_type` skip.
The leftover compileable queue on that 4-core VM is exhausted: H71 stays refuted here,
H85’s 20% bar is not lowered, and `PORTABLE` thread constants are not shipped.

[Campaign-2](plan-2026-08-23-fdu-performance-campaign-2.md) still queues a one-afternoon
PGO screen (`fdu-pdne`). H93 was the first registry meaning; that id was later reused,
so this block mints **H148**. The screen records a number.
It does not change the release profile unless both named jobs clear 3% with intervals
below zero.

## Goals

- Build an instrumented release `perf_probe`, train it on reconstructible `linux-v6.12`,
  and rebuild with the merged profile
- Pair the PGO binary against the same-source fat-LTO control on `cold-scan-index` and
  `warm-revalidate`
- Record the cell whether it clears or misses
- Adopt into release builds only if both jobs clear; otherwise leave `Cargo.toml` alone

## Non-Goals

- Pushing to #91, #92, #94, or #97
- Merging this branch to `main` or onto #97 unless asked
- Restarting H86, retrying H71, retrying H85’s 20% bar, or retrying H72 on a 6%
  directory source tree
- Shipping a `PORTABLE` thread constant
- Checking a PGO profile into the repository
- Raising the README 200K files/s or 4M cached lines/s
- A capability that exists only on the command line

## Standing

- **Branch:** `cursor/linux-pgo-screen-de1b`
- **Base:** `cursor/linux-perf-iterate-de1b`
  ([#97](https://github.com/jlevy/fdu/pull/97) at `937a8917`)
- **Protocol:** [performance-loop.md](../../guides/performance-loop.md)
- **Linux standing:**
  [Linux Standing](../../guides/performance-loop-runbook.md#linux-standing-2026-09-20)
- **Registry:**
  [Current engine (0.1.0)](../../guides/performance-loop.md#current-engine-010)
- **Beads:** epic `fdu-c1zi`; H148 screen `fdu-fg0q`; `fdu-pdne` remains the standing
  PGO bead
- **First experiment id:** exp-154
- **Hypothesis id:** H148 (do not reuse H93)
- **Quiet:** `PERF_HOST_REGIME=quiet` first.
  Do not lower the 25% busy bar.
  Label **uncontrolled** if the gate fails or the final snapshot exceeds 25%.

Read [First Principles](../../architecture/fdu-design-principles.md#first-principles)
before choosing a default, an ordering, an output shape, or a bound.
`make check` is the handoff gate after any engine patch.
This screen should not need one.

## What #97 Already Settled

Do not redo these as new leftover identities.

| Id | Linux result |
| --- | --- |
| H144 / exp-144 | Cache-hit leftover **same**; already-landed restore |
| H145 / exp-145 | Opened-discovery leftover **same**; journal clones |
| H84 / exp-146 | Unlock silent; named-job `--threads 8` not a 3% win |
| H146 / exp-147 | First-run leftover **same**; snapshot write not skippable |
| H85 / exp-150 | Recycle **rejected** against the 20% mimalloc bar |
| H147 / exp-151 | Same recycle **accepted** as the 3% keep |
| H72 / exp-152 | `d_type` skip **rejected** on `linux-v6.12` (−1.63%) |
| H72 / exp-153 | `d_type` skip **accepted** on nominated `/usr` (−9.01%) |

## Next Up

1. **H148 — PGO screen (`fdu-pdne`).** Same source as #97 HEAD. Control is the current
   fat-LTO / `codegen-units=1` release probe.
   Candidate is that probe rebuilt with `-Cprofile-use` after training `scan-index`,
   `revalidate`, `summary`, and `summary --no-controls` on reconstructible
   `linux-v6.12`. Accept only if **both** `cold-scan-index` and `warm-revalidate` clear
   3% wall with intervals entirely below zero and peak RSS no worse.
   A miss on either job rejects the screen.
   Do not adopt a one-job win into release.

Cold-scan-index is still walk-bound on this host (~95% `getdents64`+`statx`). A miss
there is the expected leftover, not a reason to lower the bar.

## Subjects

- **linux-v6.12**: reconstructible deciding subject (92,474 entries).
  Prefer this.
- Training and the pair use the same tree.
  That is the claim: a profile of these jobs on this subject.
- No RAM disk.
- Record host, virtualization, filesystem, cache state, and busy% on every cell.

## Testing Strategy

Record every cell with `make perf-record`, then `make perf-ledger` and
`make perf-report`. Quiet confirmatory of a Linux accept is allowed only if the gate
holds the whole pair.
Exact oracles stay as for the job under test.

## Rollout Plan

Work only on `cursor/linux-pgo-screen-de1b`, base `#97`. Do not push to #91, #92, #94,
or #97. No merge unless asked.
No force-push.

If #97 moves, rebase this branch onto it and keep the H148 meaning.

## Open Questions

- Whether PGO clears 3% on Linux `cold-scan-index` after the walk leftover (H140)
- Whether PGO clears 3% on Linux `warm-revalidate`
- Whether a miss still leaves a userspace-only job (transient `--no-controls` summary)
  worth a screening row in the same artifact

## References

- [Linux performance iteration](plan-2026-09-20-linux-performance-iteration.md) —
  leftover queue and H147/H72 on #97
- [Campaign-2](plan-2026-08-23-fdu-performance-campaign-2.md) — `fdu-pdne` remaining
  queue
- [Strategy review](../../reports/report-2026-08-23-research-loop-strategy-review.md) —
  screen records a number; adoption is a separate decision
- [The loop guide registry](../../guides/performance-loop.md#current-engine-010)
- [First Principles](../../architecture/fdu-design-principles.md#first-principles)
- Standing bead: `fdu-pdne`
- This stack: epic `fdu-c1zi`; H148 screen `fdu-fg0q`

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
