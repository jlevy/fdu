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
  (`fdu-tk1b` remains the standing thread-policy bead); H146 `fdu-jkzd`; H85 screen
  `fdu-h967`; H147 keep `fdu-2wyr`
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

2. **H145 — opened-discovery leftover.** **Same leftover identity** (exp-145,
   uncontrolled). Journal clones 5,772 / 5,772; 438,021 live roll-up merges; opened
   component ~2.75× first-pass (both jobs pay `statx`). Quiet start 0.151/core did not
   hold (3/30 invalid at 0.251). No smallest skippable userspace cut ≥3%. Do not port
   `macos_bulk`. Do not apply H115 rebuild to progressive commits.
   Bead: `fdu-40pl` (close).

3. **H84 — adaptive unlock / thread-policy screen (`fdu-tk1b`).** **Confirmed silent**
   (exp-146, uncontrolled named jobs).
   ~2 µs/entry vs 30 µs; expansions 0; start 4 / reserve 8. Named-job `--threads 8` is
   not a 3% win (aggregate +1.75% on `linux-v6.12`; **+7.12% quiet regression on
   `/usr`**, exp-149; index +0.25%). `--no-controls` aggregate is a warm sign (−5.42%
   quiet on `linux-v6.12`; **−10.06% quiet on nominated `/usr`**, exp-148) and is not a
   shipped constant. Do not change `PORTABLE` to `measured`. Do not lower the unlock
   threshold. Bead: `fdu-4cni` (close).
   `fdu-zk2r` (close). `fdu-tk1b` stays open for bare metal.

4. **H146 — first-run leftover after H140.** **Same leftover identity** (exp-147,
   quiet). Walk 93% of first-run component.
   Isolated save ~24 ms (~5%) is ≥3% and not skippable.
   Load/core 0.082–0.119 held.
   Do not retry H100. Do not load a snapshot on `fdu PATH`. Bead: `fdu-jkzd` (close).

5. **H85 / H147 — transient batch recycle.** H86 detached arenas did not consume
   `RetainedState::Summary`. **H85 rejected** against its 20% bar (exp-150). **H147
   accepted** (exp-151, quiet `linux-v6.12` `--no-controls` aggregate −4.98%
   [−5.92%, −4.33%]; RSS flat; default gitignore-on placebo +0.91%). Engine kept
   (`5c6e6394`). Do not retry H85’s 20% bar.
   Do not restart H86.

6. **H72 — `d_type` skip on transient summary (`fdu-ueab`).** Previous measure was −1.4%
   on a 6.4%-directory tree.
   Nominated `/usr` is 22% directories plus symlinks.
   Both arms `--no-controls`. Do not skip directory `statx` when `one_filesystem` is on.
   Accept only if reconstructible `linux-v6.12` clears 3%. `/usr` screens transfer.
   Do not compile a walk trim (H71). Experiment **exp-152**.

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

- Linux cache-hit leftover after H125–H133: no new ≥3% userspace cut (H144 / exp-144)
- Linux opened-discovery leftover: same as Darwin H127; no new ≥3% cut (H145 / exp-145)
- H84 unlock is silent; named-job `--threads 8` is not a 3% win.
  `--no-controls` aggregate is a warm sign on `linux-v6.12` (exp-146) and nominated
  `/usr` (exp-148), not a shipped `PORTABLE` constant
- Linux first-run leftover is still the walk; snapshot write ~24 ms is ≥3% and not
  skippable (H146 / exp-147)
- H85 recycle missed its 20% mimalloc bar (exp-150); the same patch cleared 3% as H147
  (exp-151) on reconstructible `linux-v6.12`

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
