# Feature: Linux Performance Iteration after #94

**Date:** 2026-09-20

**Author:** fdu project

**Status:** Active.
Stacked on [#94](https://github.com/jlevy/fdu/pull/94) at `c234da2b`.
This block looks for a named Linux cut that still clears 3% wall after H139–H143. It is
not a restart of H86 and not a Linux floor pass.

## Overview

[#94](https://github.com/jlevy/fdu/pull/94) recorded that the Darwin #92 engine is the
same on Linux for cache-hit and content-query, and that the leftovers Darwin named are
still those leftovers: walk is `getdents64`+`statx`, first-pass analyze is file I/O, and
H111 fails on this virtualized host because of that walk plus retained-index RSS.

Those determinations close the *replication* block.
They do not close Linux performance.
This stacked block asks whether any remaining job still has a userspace stage ≥3% that
is not already rejected, and whether the inherited scan-thread policy (H84 / `fdu-tk1b`)
is leaving a measurable warm gap on this host.

Profile first. Compile only a named cut.
Record every cell, including negatives.

## Goals

- Profile the Linux leftovers #94 did not name: cache-hit restore, opened-discovery, and
  first-run snapshot write
- Measure H84 on this virtualized host: confirm the 30 µs adaptive unlock does not fire,
  and screen whether an explicit worker count moves a named job ≥3%
- Compile a cut only when a leftover names a userspace stage ≥3% that is not H86, H71,
  H124, or another closed id
- Keep every cell in the ledger

## Non-Goals

- Pushing to #91, #92, or #94
- Merging this branch to `main` or onto #94 unless asked
- Restarting the H86 structural rewrite (`fdu-xde5`)
- Compiling a walk trim, retrying H71, H118, H119, H124, H125, H129, H131, H133, or H138
- Shipping a thread-policy constant as evidence from a 4-core VM (H76 / H84 remain
  screening here; bare metal is `fdu-lf3v`)
- Loading a snapshot on one-shot `fdu PATH` (H108)
- Raising the README 200K files/s or 4M cached lines/s
- A capability that exists only on the command line

## Standing

- **Branch:** `cursor/linux-perf-iterate-de1b`
- **Base:** `perf/campaign-linux-2026-09-19`
  ([#94](https://github.com/jlevy/fdu/pull/94) at `c234da2b`)
- **Protocol:** [performance-loop.md](../../guides/performance-loop.md)
- **Linux standing:**
  [Linux Standing](../../guides/performance-loop-runbook.md#linux-standing-2026-09-20)
- **Registry:**
  [Current engine (0.1.0)](../../guides/performance-loop.md#current-engine-010)
- **Beads:** epic `fdu-hi1f`; H144 `fdu-5wzu`; H145 `fdu-40pl`; H84 screen `fdu-4cni`
  (`fdu-tk1b` remains the standing thread-policy bead); H146 `fdu-jkzd`
- **First experiment id:** exp-144
- **Quiet:** `PERF_HOST_REGIME=quiet` first.
  Do not lower the 25% busy bar.
  Label **uncontrolled** if the gate fails or the final snapshot exceeds 25%.

Read [First Principles](../../architecture/fdu-design-principles.md#first-principles)
before choosing a default, an ordering, an output shape, or a bound.
`make check` is the handoff gate after any engine patch.
`--no-default-features` / lib-only is required.
`make cross-lint` after platform-gated edits.

## What #94 Already Settled

Do not redo these as new leftover identities.

| Id | Linux result |
| --- | --- |
| H139 / exp-138 | Cache-hit stack **same**; wall −22.48% quiet versus #91 |
| H140 / exp-139 | Walk leftover **same**; `getdents64`+`statx` |
| H141 / exp-140 | Content-query stack **same**; wall −17.60% |
| H111 / exp-141 | Floor/RSS gates **fail** on this virtualized host |
| H143 / exp-142 | H111 leftover **same**; walk floor + retained-index RSS |
| H142 / exp-143 | First-pass leftover **same**; file I/O |

## Next Up (Linux Only)

Take these in order.
Mint the reserved id when the cell starts, not before.

1. **H144 — cache-hit leftover after the landed stack.** **Same leftover identity**
   (exp-144, quiet). Apply ~80–84 ms; parse and candidates ~27 ms each.
   No new userspace cut.
   Do not retry H125/H129/H131/H133. Bead: `fdu-5wzu` (close).

2. **H145 — opened-discovery leftover.** H127 analog.
   Darwin leftover was `read_dir`+`fstatat`, journal clones, and live roll-up merges; no
   smallest cut. Linux opened-discovery is not `macos_bulk`; name whether a userspace
   stage ≥3% remains. Do not port `macos_bulk`. Do not apply H115 rebuild to progressive
   commits unless the leftover names that.
   Bead: minted with the spec.

3. **H84 — adaptive unlock / thread-policy screen (`fdu-tk1b`).** Existing id.
   Predicted: the 30 µs APFS threshold never fires against the Linux warm floor (~1.5
   µs), so automatic stays at `available.clamp(1, 6)`. On this 4-core host that start is
   already 4. Confirm `adaptive_scale_ups` stays 0 and ns/entry stays far below 30 µs.
   Then screen explicit `--threads` on `aggregate-summary` and `cold-scan-index`. A warm
   pair that clears 3% is a sign, not a shipped constant.
   Do not treat a 4-core VM sweep as H76 queue-depth evidence.
   Do not change `PORTABLE` to `measured` from this host.

4. **H146 — first-run leftover after H140.** H136 analog.
   `default-tree-first` on `linux-v6.12`. Darwin leftover was still the walk; snapshot
   write ~45 ms and not skippable.
   Linux write cost may move; skippability is the question.
   Do not retry H100. Do not load a snapshot on `fdu PATH`.

5. **A named cut only if a leftover above names one.** That cell takes the next free id
   (H147 / exp-148 if the four determinations use exp-144–147). Accept rule: 3% wall,
   interval below zero, digest identical.
   Do not restart H86.

## Subjects

- **linux-v6.12**: reconstructible deciding subject (92,474 entries).
  Prefer this.
- **linux-450k**: generated 450,001 entries.
  Allowed for H84 index screen; cannot decide a real-tree accept.
- No RAM disk.
- Record host, virtualization, filesystem, cache state, and busy% on every cell.

## Same vs Different

A leftover is **same** when the named stage is still the leftover Darwin recorded, even
if the percentage differs.
A leftover is **different** when a userspace stage ≥3% appears that Darwin did not name,
or a landed patch fails the accept rule.

H84 is not a same/different replication.
It is a determination plus an optional screen.

Do not quote Darwin files/s or milliseconds as a Linux product claim.
Do not raise README numbers from either host.

## Testing Strategy

Record every cell with `make perf-record`, then `make perf-ledger` and
`make perf-report`. `make check` after any engine patch.
Quiet confirmatory of a Linux accept is allowed only if the gate holds the whole pair.

Exact oracles and content digest stay as for the job under test.
H108 still requires one-shot `fdu PATH` to stay `cold scan`.

## Rollout Plan

Work only on `cursor/linux-perf-iterate-de1b`, base `#94`. Do not push to #91, #92, or
#94. No merge unless asked.
No force-push.

If #94 moves, rebase this branch onto it and keep H144–H146 meanings.

## Open Questions

- Whether Linux cache-hit leftover after H125–H133 still has a userspace stage ≥3%
- Whether Linux opened-discovery leftover names a cut Darwin did not
- Whether H84’s unlock is silent here and whether a worker screen clears 3%
- Whether Linux first-run snapshot write is a skippable ≥3% cut

## References

- [Linux parallel validation](plan-2026-09-19-linux-parallel-validation.md) — H139–H143
  recorded on #94
- [The loop guide registry](../../guides/performance-loop.md#current-engine-010)
- [Linux standing](../../guides/performance-loop-runbook.md#linux-standing-2026-09-20)
- [Platform tuning](../../guides/platform-tuning.md) — inherited `PORTABLE` constants
- [First Principles](../../architecture/fdu-design-principles.md#first-principles)
- Beads: epic minted with this spec; H84 `fdu-tk1b`; H76 parent `fdu-0myw`

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
