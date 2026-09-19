# Feature: Post-H115 Remaining Headroom

**Date:** 2026-09-19

**Author:** fdu project

**Status:** Active. Overnight H116–H120 is done.
This file is now the remaining unaddressed-hypothesis queue after that overnight: H113
(quiet), H121–H124, H107 (ignore-is-the-walk only), and H111 (not this host).
[The runbook standing](../../guides/performance-loop-runbook.md#current-standing-2026-09-18)
points here for next-up and does not keep a second copy of the rows.
The loop guide registry remains the full hypothesis text.

## Overview

H115 accepted a restore-only bottom-up content roll-up (−9.69% `content-cache-hit` wall
on deciding-scale metabrowser).
H120 accepted streaming sidecar parse-into-apply (−10.13% peak RSS; wall non-inferior).
Those are the standing content-hit bests.
This block names the directions the overnight did **not** address, as first-class
hypotheses, and keeps the overnight verdicts as history so they are not re-queued.

It is a planning block, not a measurement cell.
It does not start a pair.
H113 still needs a quiet host; morning is the intended cell.
Do not run another uncontrolled H113.

## Goals

- Name only hypotheses that are plausible at the 3% wall bar (or a structural ceiling)
  on a named job and subject, and that can be wrong
- Register remaining work as H121–H124 in
  [the loop guide](../../guides/performance-loop.md#current-engine-010); keep H113,
  H107, and H111 open with honest status
- Own next-up after the overnight: order, metric, subject, accept-rule sketch, why next,
  what refutes, bead
- Keep one source of truth for that queue (this file)

## Non-Goals

- Starting a measurement cell, keeping an engine experiment patch, or reverting H115
- Loading a metadata snapshot on `fdu PATH` as a “free win” (H108 / H9)
- Restarting the H86 structural rewrite, the `fdu-jxhk` EntryId composite, or H83 as a
  format rewrite
- Retrying H113 on an uncontrolled cell, H114 alloc trims, H109 Path rewrites, parse
  speed, or H103-shaped instruction cuts
- Linux H111 on this Darwin host (still open; not in this host)
- A capability that exists only on the command line
- Replacing the overnight macOS-agenda epic (`fdu-d4kg`) or rewriting campaign-2’s
  2026-08-23 Tier 1–3 list

Complexity bar: a queued change may add a private engine path that preserves goldens and
digests. It may not add a dependency, an `unsafe` block, a public query grammar, or a
second content identity rule.

## Background

After exp-112 the deciding-scale `content-cache-hit` standing best is 1,174.1 ms wall /
855.6 ms component / 389.8 MiB on `metabrowser-clone` (145,931 entries / 133,597 files).
exp-108/109 split that path before the accept:

| Stage | Share | Status |
| --- | --- | --- |
| Sidecar apply / ancestor merges | 63% of restore (sample 54% of `load_content`) | H115 took the named cut |
| Candidate install (`analysis_candidates` + HashMap) | 25% of restore / 20% of `load_content` | Still in the engine |
| Snapshot parse | 26% of engine | H78/H92; not a tonight increment |
| Second completeness walk | 13% of `content_open` | H113; quiet confirmatory only |
| Sidecar parse | 8.5% of restore | Dead for wall (H112) |
| `install_controls` | 7.5% of engine | Dead for a Path rewrite (H109) |

Metadata one-shot (`fdu PATH`) is still a cold walk (H108). First-pass `content-basic`
still scans, then opens every admitted file, then `apply_analysis` → `merge_ancestors`
per file. Cache-hit RSS is about 380 MiB on that tree, against about 90 MiB for a
metadata-only CLI run of similar size.

## Design

### Approach

Treat remaining time as four separate jobs, not one “make restore faster” leftover.

1. **Metadata one-shot.** The walk is the job.
   A snapshot cannot cheapen it.
   The product lever that respects serving policy is opened-root retention, not a CLI
   cache load.
2. **Content-cache-hit.** Apply’s named ancestor-merge cut landed.
   The next named restore stage is candidate install, then snapshot parse (already owned
   by H78/H92).
3. **First-pass analyze.** Incremental `commit` still walks ancestors per file.
   File I/O is a second pass after the walk.
   Those are different mechanisms.
4. **RSS.** The content-hit peak is large enough to be a product constraint on
   `--analyze` trees. Transient decode-plus-map copies are the increment that is not H86.

### Components

Overnight registry rows (settled; full text in the loop guide):

| # | One-line claim | Job | Status |
| --- | --- | --- | --- |
| H116 | Cache-only restore can match sidecar records to the live index without a full `analysis_candidates` Vec+HashMap | `content-cache-hit` | Rejected (exp-114). Do not retry uncontrolled. |
| H117 | An opened-root second report on Darwin is at least 3% faster than a metadata one-shot of the same request | opened retained read vs `default-tree` | Confirmed (exp-116). Probe only; see H123. |
| H118 | First-pass `analyze_index` can use H115’s insert-then-rebuild instead of per-file `merge_ancestors` | `content-basic` component | Rejected (exp-115). Do not retry uncontrolled. |
| H119 | First-pass analyze can overlap file I/O with the metadata walk instead of opening every file after the scan | `content-basic` wall / product `--analyze` | Screened. Do not retry walk-overlap. |
| H120 | Streaming sidecar parse-into-apply (no full decoded-records Vec beside the files map) cuts peak RSS at least 10% | `content-cache-hit` RSS | Accepted (exp-117). |

Remaining registry rows (open; full text in the loop guide):

| # | One-line claim | Job |
| --- | --- | --- |
| H113 | File-count completeness after H115 is a real quiet wall win | `content-cache-hit` |
| H122 | After the current engine, a deciding-scale metadata CLI/walk profile still shows the walk as the job | installed `fdu PATH` / `default-tree` |
| H107 | Default gitignore observation differs by ≥3% wall only where the ignored share *is* the walk | `default-tree` |
| H123 | A product opened-root or refresh path that retains the index is ≥3% faster than repeating a one-shot | opened retained read vs `fdu PATH` |
| H121 | After H115 and H120, a cache-hit restore re-profile names whether apply still dominates | `content-cache-hit` |
| H124 | Fewer first-pass opens (type/size gate) or read-ahead cuts analyze wall ≥3% | `content-basic` / product `--analyze` |
| H111 | H86’s remaining gap is the Linux floor and RSS claim | Linux 450k floor |

H113 still needs a quiet host.
Morning is the intended cell.
Do not run uncontrolled.

H122 is not H86 and not H111. H108’s instrumented pair put the detached walk at ~96% of
`fdu PATH` wall; overnight did not optimize that job.
Profile first.

H107 stays H107. Re-run only on a tree whose ignored share can be the walk.
Do not retry metabrowser (exp-106).

H123 is not a license to load a snapshot on `fdu PATH` (H108 / H9). H117 confirmed the
engine already has the cheaper retained read (`opened-second-report`). This row is the
product path that uses it.

H121 is a profile, not another alloc trim and not a retry of H116. H83 remains only if
apply still dominates after the mix is re-measured.

H124 is not H118 (apply shape) and not H119 (walk overlap / `openat`). Named mechanism:
admit fewer files, or read-ahead on the ones already admitted.

H111 is open and not in this host.

### API Changes

None. Any probe mode this block needs is a `fdu-core` example entry, not a command-line
flag.

### Rejected as Not Significant

These were considered against the post-H115 path and not registered:

- Another `ContentRollUp` or `PathBuf` alloc trim on apply (H114 just failed)
- Sorting `rebuild_rollups` by something other than `components().count()` (H103
  instruction shape; the pass is already O(files + dirs))
- Sidecar parse, CRC, or slicing-by-N (H112: parse is 8.5% of restore)
- Persisting content roll-ups or a mmap sidecar as *this* increment (H78/H83/H92 already
  own the format; not an overnight cut)
- Double-classify in `apply_analysis` as its own hypothesis (`fdu-926e` already exists;
  H116 was the attempt to drop that classify on the hit path and failed on wall)
- Hardware CRC32C (`fdu-6kyn`) or PGO (`fdu-pdne`) as tonight’s wall hunt
- `searchfs` (H77): person-gated
- Linux H111 on this Darwin host (still open; not in this host)
- Bounding the observation channel (H91) without a current-engine occupancy trace
- Directory-only transient tree (H66) as a cache-hit leftover

### Remaining Inclusion Rules

- H113 quiet confirmatory (`fdu-rfr6`, exp-113) is first *if* a quiet host holds this
  morning. If the start gate fails, skip it and take H122. Do not run another
  uncontrolled H113. Incomplete 2026-09-19 quiet cells are not a verdict.
  exp-113 unused.
- Then take the remaining queue below, in order.
  After an accept, re-screen the next row: H122 may name a cut that eats H107; H121 may
  retire H83 or leave it.
- Uncontrolled is allowed on H121–H124 and H107 when quiet fails.
  Label it. Do not lower the 25% busy bar.
  H113 is the exception: a failed quiet gate is a stop.
- H111 is not in this host (no Linux runner).
- Items that need a quiet cell and nothing else are labeled **needs quiet host**. That
  is only H113.
- Do not re-queue H116, H118, H119 walk-overlap, H114, H109, snapshot-load without a
  policy change, or the H86 / `fdu-jxhk` rewrite.

## Implementation Plan

### Remaining Queue (Source of Truth)

Take these in order.
Overnight H116–H120 is history, not a retry list.

1. **H113** (`fdu-rfr6`). **Open.
   Needs quiet host.** Morning is the intended cell.
   File-count completeness after H115. Metric: `content-cache-hit` wall on
   `metabrowser-clone`, ≥3% with the interval below zero; digest identical; incomplete
   sidecar refused. Control = HEAD with H115 and H120 in.
   New id **exp-113**. Do not run uncontrolled.
   What refutes: quiet interval includes zero, or the start gate fails (skip, not a
   reject). Why next: already instrumented; leftover 12.6% `content_open` walk from
   exp-109 may have changed after H115.

2. **H122** (`fdu-ytg5`). **Open.** Highest user-visible leverage.
   Deciding-scale installed-CLI / `default-tree` **profile** after the current engine
   (H115 + H120 in). Subject: `system-private-frameworks` or another immutable deciding
   tree. Determination: the metadata walk is still ≥90% of `fdu PATH` wall (H108
   instrumented: detached walk 1.292 s of 1.34 s, ~96%), and names the stage that owns
   the leftover (enumerate, stat, consume).
   Not a cut. Not H86. What refutes: walk share below 90%, or the leftover is a stage
   already owned by a rejected hypothesis.
   Why next: overnight optimized restore and RSS, not the default command.

3. **H107** (`fdu-jcfn`). **Open only where ignore *is* the walk.** Metric:
   `default-tree` wall, |median| ≥3% and interval excludes zero, either direction, on a
   tree whose ignored share can be the walk.
   Do not retry metabrowser (exp-106: +1.64% [−4.00%, +4.37%]). What refutes: another
   subject where exclusion and the control walk still cancel.
   Why next: H122 may name a tree where this is the walk.

4. **H123** (`fdu-rum0`). **Open.** Follow-on to H117 (probe only).
   A product opened-root or refresh path that retains the index is ≥3% faster than
   repeating one-shot `fdu PATH` for the same request.
   Subject: `system-private-frameworks` or `metabrowser-clone`. One-shot footer stays
   `cold scan`. Not a snapshot load on `fdu PATH`. What refutes: no product surface can
   retain and re-report without changing one-shot cache policy, or the product path
   misses 3%. Why next: H108 left the default CLI as a cold walk; H117 showed the engine
   already has the cheaper job.

5. **H121** (`fdu-vf4b`). **Open.** Post-H115+H120 cache-hit **re-profile**. Metric:
   same-subject `content-cache-hit` stage split (timers already in) on
   `metabrowser-clone`. Determination: a named restore stage is still ≥50% of restore
   and ≥3% of wall. Profile first.
   Then a named apply/install cut only if apply still dominates (that leftover is H83).
   Not another alloc trim.
   Not a retry of H116. What refutes: apply no longer dominates (H83 scoped down), or no
   stage clears the bar.
   Why next: two accepted restore changes landed after exp-109’s mix.

6. **H124** (`fdu-i39y`). **Open.** First-pass analyze I/O. Metric: `content-basic` wall
   or product `--analyze` wall, ≥3% with the interval below zero on deciding-scale
   metabrowser; digest identical; worker parallelism retained.
   Named mechanism: type/size gate (do not open files that cannot contribute) or
   read-ahead on admitted files.
   Not H118. Not H119 walk-overlap.
   Not `openat` (`unsafe`). What refutes: interval includes zero, or every admitted open
   is required for the requested metrics.
   Why next: H118/H119 showed apply and walk-overlap cannot move this job; `read` 59% /
   `__open` 17% is the leftover.

7. **H111** (`fdu-jekg`). **Open.
   Not in this host.** Linux floor stage of H86. No Linux runner on this Darwin campaign
   machine. Do not treat a Darwin cell as this claim.
   Do not restart the rewrite.

**Overnight history (do not re-queue):** H116 rejected, H118 rejected, H119 screened,
H117 confirmed as a probe, H120 accepted.

## Testing Strategy

H113, H121, H123, and H124 are 12-pair interleaved `make perf-compare` against HEAD with
H115 and H120 in, `FDU_COUNTERS` unset for the claim-grade wall, no RAM disk.
H122 and the H121 mix are profiles / determinations first; do not start a cut from a
guess. Exact oracles and content digest stay as for exp-108–117. H113 must keep
incomplete-sidecar fail-closed.
H123 must keep one-shot `cold scan`. Record every verdict, including skips at the quiet
gate.

## Rollout Plan

Docs and beads on PR [#91](https://github.com/jlevy/fdu/pull/91) only.
No merge, no force-push, no second performance PR. Engine changes land only as the
experiment that tests the next row.
Do not start a measurement cell from this registry pass.

## Open Questions

- Whether H115 + H120 changed the exp-109 restore mix enough that apply no longer
  dominates (H121). If it still does, H83 remains; if it does not, do not start an apply
  cut.
- Whether a Darwin deciding-scale `fdu PATH` profile (H122) names a leftover that is not
  already H86/H111 on Linux.
- Whether H117’s retained read can become a product path (H123) without loading a
  snapshot on one-shot `fdu PATH`.
- Whether a type/size gate or read-ahead (H124) can cut first-pass analyze wall after
  H118/H119.

## References

- [The loop guide registry](../../guides/performance-loop.md#current-engine-010) —
  H107–H124
- [The runbook standing](../../guides/performance-loop-runbook.md#current-standing-2026-09-18)
- [Campaign 2](plan-2026-08-23-fdu-performance-campaign-2.md) — floor-anchored strategy
- [First Principles](../../architecture/fdu-design-principles.md#first-principles)
- [Engine architecture](../../architecture/fdu-engine-architecture.md) — one-shot vs
  opened
- [The instrumentation playbook](../../guides/performance-instrumentation-playbook.md)
- exp-107 through exp-117; H115 engine at `7798fdc1`; H120 streaming restore
- Beads: overnight epic `fdu-e9ow` (closed); remaining-queue epic `fdu-8ya1`; H121
  `fdu-vf4b`; H122 `fdu-ytg5`; H123 `fdu-rum0`; H124 `fdu-i39y`; H113 quiet `fdu-rfr6`;
  H107 `fdu-jcfn`; H111 `fdu-jekg`; sidecar parent `fdu-78q6`; EntryId composite
  `fdu-jxhk` (do not restart)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
