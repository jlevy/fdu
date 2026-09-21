# Feature: Linux Parallel Validation of the Darwin #92 Stack

**Date:** 2026-09-19

**Author:** fdu project

**Status:** Recorded.
Base `main` (`c7babf76`);
[#91](https://github.com/jlevy/fdu/pull/91)/[#92](https://github.com/jlevy/fdu/pull/92)
merged 2026-09-20. This is not a Linux floor pass: H111 failed on this virtualized host
(exp-141). Darwin measurement on #92 had stopped at `26480612` when this block started;
the stack later landed R1–R3 and `c441edf6` at `937f9445`. H139 is recorded (exp-138,
same, quiet). H140 is recorded (exp-139, same leftover identity).
H141 is recorded (exp-140, same, uncontrolled).
H143 is recorded (exp-142, same leftover identity).
H142 is recorded (exp-143, same leftover identity).
Do not mint those ids on the Darwin branch.

## Overview

[#91](https://github.com/jlevy/fdu/pull/91) holds H115 and H120.
[#92](https://github.com/jlevy/fdu/pull/92) stacks five more accepted engine changes and
a set of leftovers, all measured on Darwin, almost all **uncontrolled**. This block asks
what of that stack is the same on Linux, what is different, and whether H111’s
pre-registered Linux floor gates now pass on the current engine.

It is a measurement and recording block.
It does not restart H86. It does not invent a Darwin walk trim.
It does not raise the README 200K files/s or 4M cached lines/s.

## Goals

- Replicate the accepted #92 engine slice on Linux and record same or different
- Profile the leftovers that Darwin said were kernel I/O or already-rejected userspace,
  and say whether Linux names a different leftover
- Run H111 (`fdu-jekg`) on the 450k Linux subject with the gates already registered
- Keep every cell in the ledger, including quiet-gate skips and negatives

## Non-Goals

- Pushing to the deleted Darwin branches (#91 or `perf/campaign-next-2026-09-19`)
- Merging this branch to `main` unless asked (merge onto `main`, then #97)
- Force-pushing, or restarting the H86 structural rewrite (`fdu-xde5`)
- Retrying H113 file-count, H109 Path rewrite, H116, H118, H119, H124, H71 raw
  `getdents64` / io_uring, or a `macos_bulk` port
- Loading a snapshot on one-shot `fdu PATH` (H108)
- Persisting ignored bits (format change)
- A capability that exists only on the command line
- Treating a Darwin `default-tree` cell as H111

## Standing

- **Branch:** `perf/campaign-linux-2026-09-19`
  ([#94](https://github.com/jlevy/fdu/pull/94))
- **Base:** `main` (`c7babf76`); #91/#92 merged 2026-09-20 (`6e3d2937` / `a290aedc`).
  Engine tip of those merges is `937f9445` (including `c441edf6` single-view Cow borrow)
- **Protocol:** [performance-loop.md](../../guides/performance-loop.md)
- **Darwin standing:**
  [Current Standing](../../guides/performance-loop-runbook.md#current-standing-2026-09-18)
- **Linux standing:**
  [Linux Standing](../../guides/performance-loop-runbook.md#linux-standing-2026-09-20)
- **Registry:**
  [Current engine (0.1.0)](../../guides/performance-loop.md#current-engine-010)
- **Beads:** epic `fdu-v62p`; H139 `fdu-bt6x`; H140 `fdu-6hjg`; H141 `fdu-rmo2`; H142
  `fdu-2003` (closed); H111 `fdu-jekg` (closed); leftover H143 `fdu-kfl3` (closed);
  merge-ready `fdu-jh1i`
- **First experiment id:** exp-138 (exp-113 remains reserved unused)
- **Next free ids after this PR:** H144 / exp-144
- **Quiet:** `PERF_HOST_REGIME=quiet` first.
  Do not lower the 25% busy bar.
  Label **uncontrolled** if the gate fails or the final snapshot exceeds 25%.

Read [First Principles](../../architecture/fdu-design-principles.md#first-principles)
before choosing a default, an ordering, an output shape, or a bound.
`make check` is the handoff gate after any engine patch.
`--no-default-features` / lib-only is required.
`make cross-lint` after platform-gated edits.
Backtick type names in rustdoc (`clippy::doc_markdown` already failed this stack).

## What Darwin Already Settled

Do not redo these as Darwin cells.
Replicate or contrast them on Linux.

### Accepted engine (in the tree you start from)

| Id | Mechanism | Darwin job | Darwin wall | Commit |
| --- | --- | --- | ---: | --- |
| H115 / exp-112 | Restore-only bottom-up content roll-up | `content-cache-hit` | −9.69% [−26.02%, −7.13%] | on #91 (`7798fdc1`) |
| H120 / exp-117 | Streaming sidecar parse-into-apply | `content-cache-hit` RSS | peak RSS −10.13%; wall non-inferior | on #91 |
| H125 / exp-124 | Completeness from restore candidate count | `content-cache-hit` | −8.03% [−10.79%, −7.79%] | `be8d4d69` |
| H129 / exp-128 | Cache-only restore omits classify | `content-cache-hit` | −13.11% [−20.22%, −12.67%]; RSS −11.83% | `6887a864` |
| H131 / exp-130 | Restore DFS joins parent path | `content-cache-hit` | −4.07% [−4.54%, −3.28%] | `7840ce9b` |
| H133 / exp-132 | Skip unused snapshot `path_of` when `serving` is off | `content-cache-hit` | −6.37% [−18.23%, −5.66%] | `143a1c73` |
| H138 / exp-137 | One `every_entry` walk for unfiltered views | `content-query` | −18.76% [−22.86%, −13.69%] | `a5c98d59` |

H125–H133 were sequential pairs on frozen `metabrowser-clone` (145,931 entries / 133,708
cache hits, digest `3be19a3e…`). Compounded they are about −28% cache-hit wall versus
the post-H115/H120 control (~1,063 ms → ~778 ms on those cells).
Absolute walls are not comparable across uncontrolled hosts.
Do not mix Darwin milliseconds with Linux milliseconds and call that a delta.

### Determinations that should differ on Linux if the leftover is the kernel

| Id | Darwin leftover | Why Linux may disagree |
| --- | --- | --- |
| H122 / exp-118, exp-122 | Walk 96–97.5%; 1.403 `getattrlistbulk`/dir; `__open` + bulk | Linux floor is `getdents64` + per-entry `statx` (playbook: 2.00 `getdents64`/dir). H71 already refuted rearranging that layer on a scouting VM. |
| H128 / exp-127 | File-heavy `default-tree` still the walk (92.9%) | Same job, different syscall mix |
| H135 / exp-134 | First-pass `content-basic` still `read` / `__open` | Linux `openat` / `read` share may move; userspace classify still expected small |
| H136 / exp-135 | First-run still the walk; snapshot write ~45 ms, not skippable | Write cost is platform-specific; skippability is not |
| H127 / exp-126 | Opened-discovery ~8.8× first-pass; `read_dir`+`fstatat`; journal clones | Opened path is not `macos_bulk`; journal clones are engine work |

### Closed on both hosts unless a new mechanism appears

H108 (one-shot stays `cold scan`), H109 (Path rewrite), H113 (file-count), H114 (alloc
trim), H116 (HashMap drop), H118 (first-pass insert-rebuild), H119 (walk overlap), H121
(no apply-stage ≥50%), H123 (retained report is the product path), H124 (type/size /
read-ahead), H107 (no ignore-is-the-walk subject on the Darwin hunt; do not invent one).

## Recorded (Linux Only)

Taken in this order.
Minted ids stay reserved; do not reuse them on Darwin.

1. **H139 — cache-hit stack, same or different.** **Same** (exp-138, quiet).
   `content-cache-hit` wall −22.48% [−23.46%, −21.39%] on reconstructible `linux-v6.12`
   (92,474 entries). Peak RSS −10.24%. Digest identical.
   A clean metabrowser clone on this host is 916 entries (Darwin’s 146k tree was
   workspace state) and was not the subject.
   Paired at `a5c98d59`; not re-paired after R1–R3/`c441edf6`; expected below noise.
   Do not retry the cache-hit increments.
   Bead: `fdu-bt6x` (close).

2. **H140 — walk leftover.** **Same leftover identity** (exp-139, uncontrolled).
   Walk 95.7–96.1% of `default-tree` component on `linux-v6.12`. Leftover is the
   `getdents64`+`statx` floor (one open/dir, one stat/entry).
   Quiet attempt did not hold.
   Do not compile a walk trim.
   Do not retry H71. Bead: `fdu-6hjg` (close).

3. **H141 — content-query stack, same or different.** **Same** (exp-140, uncontrolled).
   `content-query` wall −17.60% [−18.07%, −17.17%] on `linux-v6.12`. Digest identical.
   Quiet start did not hold.
   Paired at `a5c98d59`; not re-paired after R1–R3/`c441edf6`; expected below noise.
   Do not retry H138. Bead: `fdu-rmo2` (close).

4. **H111 — Linux floor (`fdu-jekg`).** **Failed** (exp-141, uncontrolled, virtualized).
   450k index 1.78× vs 1.4×; RSS 5.20× vs 3×; aggregate on nominated reals 1.59× and
   1.86× vs 1.25×. p95/median passes.
   `arena_spike` on 450k spread 1.11 (ratios may reject).
   Leftover is the Darwin composite; minted as H143. Do not restart H86. Bead:
   `fdu-jekg` (close).

5. **H142 — first-pass analyze leftover.** **Same leftover identity** (exp-143,
   uncontrolled). 86,634 opens, 184,057 reads (~2.12/file) on `linux-v6.12`. Quiet start
   did not hold. Do not retry type/size or read-ahead.
   Bead: `fdu-2003` (close).

6. **H143 — leftover after H111 fail.** **Same leftover identity** (exp-142, quiet).
   Walk 94.7–94.8% of `linux-450k` `cold-scan-index` component.
   Leftover is `getdents64`+`statx` plus retained-index RSS / detached finish.
   No new cut. Do not restart H86. Do not retry H71. Bead: `fdu-kfl3` (close).

After this PR: **H144** if minted — a new named mechanism that can close the 1.4× index
or 3× RSS gate, or a bare-metal H111 remeasure.
Do not start that on this branch.
Do not restart H86. Do not retry H71.

## Subjects

- **Do not** use `system-private-frameworks`. That tree is Darwin-only.
- **linux-450k** (exp-103): 450,001 entries, reconstructible, required for H111.
- **metabrowser** if you can freeze a clone.
  Darwin digest on the APFS clone was `3be19a3e…`. A Linux checkout of the same commit
  is a different filesystem and may be a different entry count; record what you actually
  walked.
- rustup / cargo-registry are allowed for walk leftover if they are immutable for the
  pair. They were not ignore-is-the-walk subjects on Darwin.
- No RAM disk for ordinary claim-grade cells.
- Record host, virtualization, filesystem, cache state, and busy% on every cell.

## Same vs Different

A Linux cell is **same** when the named mechanism is still the leftover or the landed
patch still clears the same accept rule, even if the percentage differs.
A Linux cell is **different** when the leftover stage changes identity (for example
userspace ≥3% on a job Darwin said was kernel I/O) or a landed patch fails the accept
rule.

Do not quote Darwin files/s or milliseconds as a Linux product claim.
Do not raise README numbers from either host.

## Testing Strategy

Record every cell with `make perf-record`, then `make perf-ledger` and
`make perf-report`. `make check` after any engine patch.
Quiet confirmatory of a Linux accept is allowed only if the gate holds the whole pair.

Exact oracles and content digest stay as for the job under test.
H108 still requires one-shot `fdu PATH` to stay `cold scan`.

## Rollout Plan

Merge onto `main`, then [#97](https://github.com/jlevy/fdu/pull/97).
[#91](https://github.com/jlevy/fdu/pull/91) and
[#92](https://github.com/jlevy/fdu/pull/92) merged 2026-09-20. Do not push to the
deleted Darwin stacked branches.
No force-push.

H139–H143 meanings stay: leftover identity or accept-rule hold, not matching Darwin
percentages.

## Open Questions

- H139 closed: cache-hit stack is **same** on Linux (exp-138, quiet, −22.48%)
- H140 closed: walk leftover is **same** identity (exp-139; getdents64+statx floor)
- H141 closed: content-query stack is **same** on Linux (exp-140, uncontrolled, −17.60%)
- H111 closed: floor/RSS gates **fail** on this virtualized host (exp-141)
- H143 closed: leftover identity **same** (exp-142, quiet; walk floor + retained-index
  RSS). Bare-metal H111 remeasure remains possible and is not this cell.
- H142 closed: first-pass leftover is **same** identity (exp-143; file I/O)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
