# Feature: Post-H115 Remaining Headroom

**Date:** 2026-09-19

**Author:** fdu project

**Status:** Active. Overnight H116–H120 is done.
This file is the remaining unaddressed-hypothesis queue after that overnight: H107
(ignore-is-the-walk only) and H111 (not this host).
H122 is confirmed (exp-118, tighter leftover exp-122). H123 is confirmed (exp-119). H121
is confirmed (exp-120): apply no longer dominates.
H124 is rejected (exp-121). H125 is accepted (exp-124): restore-count completeness.
H113 is superseded by H125.
[The runbook standing](../../guides/performance-loop-runbook.md#current-standing-2026-09-18)
keeps an abbreviated next-up in that order; this file is the source of truth for the
full rows. The loop guide registry remains the full hypothesis text.

## Overview

H115 accepted a restore-only bottom-up content roll-up (−9.69% `content-cache-hit` wall
on deciding-scale metabrowser).
H120 accepted streaming sidecar parse-into-apply (−10.13% peak RSS; wall non-inferior).
H125 accepted restore-count completeness (−8.03% wall on top of those).
Those remain the standing content-hit bests, plus this increment.
This block names the directions the overnight did **not** address, as first-class
hypotheses, and keeps the overnight verdicts as history so they are not re-queued.

It is a planning block, not a measurement cell.
It does not start a pair.
H113 is superseded. Do not retry the file-count shortcut.

## Goals

- Name only hypotheses that are plausible at the 3% wall bar (or a structural ceiling)
  on a named job and subject, and that can be wrong
- Keep H121–H125 registered in
  [the loop guide](../../guides/performance-loop.md#current-engine-010); keep H107 and
  H111 open with honest status.
  H124 is rejected (exp-121). H125 is accepted (exp-124). H113 is superseded.
- Own next-up after the overnight: order, metric, subject, accept-rule sketch, why next,
  what refutes, bead
- Keep one source of truth for that queue (this file)

## Non-Goals

- Starting a measurement cell, keeping an engine experiment patch, or reverting H115 or
  H120
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

H115 (exp-112) is the standing wall-speed best on deciding-scale `content-cache-hit`:
−9.69% [−26.02%, −7.13%] (accept cell 1,174.1 ms / 855.6 ms / 389.8 MiB). H120 (exp-117)
is the standing content-hit RSS best: peak RSS −10.13% [−10.49%, −10.03%] (accept cell
1,103.2 ms / 812.5 ms / 339.4 MiB). Absolute walls are not comparable across
uncontrolled cells. Subject: `metabrowser-clone` (145,931 entries / 133,597 files).
exp-108/109 split that path before those accepts:

| Stage | Share (exp-109) | Status |
| --- | --- | --- |
| Sidecar apply / ancestor merges | 63% of restore (sample 54% of `load_content`) | H115 took the named cut |
| Candidate install (`analysis_candidates` + HashMap) | 25% of restore / 20% of `load_content` | H116 rejected on wall; still in the engine |
| Snapshot parse | 26% of engine | H78/H92; not this increment |
| Second completeness walk | 13% of `content_open` | H125 accepted (exp-124); H113 superseded |
| Sidecar parse | 8.5% of restore | Dead for wall (H112) |
| `install_controls` | 7.5% of engine | Dead for a Path rewrite (H109) |

Metadata one-shot (`fdu PATH`) is still a cold walk (H108). First-pass `content-basic`
still scans, then opens every admitted file, then `apply_analysis` → `merge_ancestors`
per file (H118 rejected insert-then-rebuild on that job).
Cache-hit RSS was about 380 MiB on that tree before H120; the H120 accept cell is 339.4
MiB, against about 90 MiB for a metadata-only CLI run of similar size.

## Design

### Approach

Treat remaining time as four separate jobs, not one “make restore faster” leftover.

1. **Metadata one-shot.** The walk is still the job (H108; instrumented ~96% of
   `fdu PATH` wall). A snapshot cannot cheapen it.
   H122 confirmed the walk is still the job.
   H123 confirmed the product retained report.
   Not a CLI cache load.
2. **Content-cache-hit.** H115 took the named ancestor-merge cut.
   H116 rejected dropping the candidate-install HashMap on wall.
   H120 took the decode-`Vec` RSS cut.
   H113 was the quiet file-count confirmatory; H125 took the restore-count skip.
   H121 re-profiled the mix (apply no longer dominates).
   Snapshot parse remains H78/H92.
3. **First-pass analyze.** H118 rejected insert-then-rebuild.
   H119 screened walk-overlap (`fdu::scan` 0.13%). Leftover I/O was H124: admit fewer
   files, or read-ahead on the ones already admitted.
   Rejected (exp-121).
4. **RSS.** H120 landed streaming restore.
   That is not a landing-page files/s claim.

### Components

Overnight registry rows (settled; full text in the loop guide):

| # | One-line claim | Job | Status |
| --- | --- | --- | --- |
| H116 | Cache-only restore can match sidecar records to the live index without a full `analysis_candidates` Vec+HashMap | `content-cache-hit` | Rejected (exp-114). Do not retry uncontrolled. |
| H117 | An opened-root second report on Darwin is at least 3% faster than a metadata one-shot of the same request | opened retained read vs `default-tree` | Confirmed (exp-116). Probe only; see H123. |
| H118 | First-pass `analyze_index` can use H115’s insert-then-rebuild instead of per-file `merge_ancestors` | `content-basic` component | Rejected (exp-115). Do not retry uncontrolled. |
| H119 | First-pass analyze can overlap file I/O with the metadata walk instead of opening every file after the scan | `content-basic` wall / product `--analyze` | Screened. Do not retry walk-overlap. |
| H120 | Streaming sidecar parse-into-apply (no full decoded-records Vec beside the files map) cuts peak RSS at least 10% | `content-cache-hit` RSS | Accepted (exp-117). |
| H122 | After the current engine, a deciding-scale metadata CLI/walk profile still shows the walk as the job | installed `fdu PATH` / `default-tree` | Confirmed (exp-118, leftover exp-122). Do not retry as a cut. |
| H123 | A product opened-root or refresh path that retains the index is ≥3% faster than repeating a one-shot | opened retained read vs `fdu PATH` | Confirmed (exp-119). Probe kept. Not a snapshot load. |
| H121 | After H115 and H120, a cache-hit restore re-profile names whether apply still dominates | `content-cache-hit` | Confirmed (exp-120). Apply 43%; candidates 48%; no stage ≥50%. No cut. |
| H124 | Fewer first-pass opens (type/size gate) or read-ahead cuts analyze wall ≥3% | `content-basic` / product `--analyze` | Rejected (exp-121). Do not retry type/size or a safe read-ahead. |
| H125 | Cache-only completeness uses the candidate count restore already computed | `content-cache-hit` | Accepted (exp-124). H113 superseded. |
| H113 | File-count completeness after H115 is a real quiet wall win | `content-cache-hit` | Superseded by H125. File-count not compiled. |

Remaining registry rows (open; full text in the loop guide):

| # | One-line claim | Job |
| --- | --- | --- |
| H107 | Default gitignore observation differs by ≥3% wall only where the ignored share *is* the walk | `default-tree` (skipped: ignore does not skip descent; no ignore-is-the-walk subject) |
| H111 | H86’s remaining gap is the Linux floor and RSS claim | Linux 450k floor |

H122 is confirmed (exp-118, leftover exp-122): walk 96.3–97.5% of deciding-scale
`default-tree` component.
Leftover is directory `__open` (55,256) plus `getattrlistbulk` at 1.403 calls per
directory (77,509). No userspace symbol ≥3%. No walk-cut id minted.
`dir_enumeration_calls` kept.
Not H86 and not H111. Do not retry as a snapshot load or consume trim.

H107 stays H107. Re-run only on a tree whose ignored share can be the walk.
Do not retry metabrowser (exp-106). The 2026-09-19 hunt (exp-122): rustup and frameworks
have no `.gitignore`; cargo-registry screens only; tbd and urollup keep the same
`dir_opens` with controls on and off.
`should_descend` does not consult ignore, so a large ignored subtree cannot be the walk.

H123 is confirmed (exp-119): product `Index.report()` / `query::report` second pass 1.7
ms versus one-shot `default-tree` 2,078.3 ms on `system-private-frameworks` (~1,222×).
Probe mode `index-second-report` kept.
Not a snapshot load on `fdu PATH` (H108 / H9). H117 remains the opened-root probe.

H121 is confirmed (exp-120): after H115+H120, apply is 42.7% of restore and candidates
47.6%. No stage is ≥50% of restore.
No apply cut. Do not retry H116. H83 is scoped down on apply/install.

H124 is rejected (exp-121): every admitted open is required for lines.
Path-binary already skipped.
Empty plus discovered-binary opens cannot reach 3% wall.
Read calls are already one data chunk per file.
Not H118. Not H119 walk-overlap.
No engine change.

H125 is accepted (exp-124): wall −8.03% [−10.79%, −7.79%] on frozen `metabrowser-clone`.
Restore-count completeness kept (`be8d4d69`). H113 superseded.
File-count shortcut not compiled.
exp-113 unused.

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
- Hardware CRC32C (`fdu-6kyn`) or PGO (`fdu-pdne`) as this increment’s wall hunt
- `searchfs` (H77): person-gated
- Linux H111 on this Darwin host (still open; not in this host)
- Bounding the observation channel (H91) without a current-engine occupancy trace
- Directory-only transient tree (H66) as a cache-hit leftover

### Remaining Inclusion Rules

- H113 is superseded (H125). Do not retry the file-count shortcut.
  exp-113 unused.
- Then take the remaining queue below, in order.
  After an accept, re-screen the next row: H122 may name a cut that eats H107; H121’s
  mix is stale after H125.
- Uncontrolled is allowed on leftover profiles and H107 when quiet fails.
  Label it. Do not lower the 25% busy bar.
- H111 is not in this host (no Linux runner).
- Do not re-queue H113 file-count, H116, H118, H119 walk-overlap, H114, H109,
  snapshot-load without a policy change, or the H86 / `fdu-jxhk` rewrite.

## Implementation Plan

### Remaining Queue (Source of Truth)

Take these in order.
Overnight H116–H120 is history, not a retry list.

1. **H125** (`fdu-wd4q`). **Accepted** (exp-124). Restore-count completeness.
   Wall −8.03% [−10.79%, −7.79%] on frozen `metabrowser-clone`. Engine kept
   (`be8d4d69`). H113 superseded.
   File-count not compiled.
   exp-113 unused. Do not retry H113.

2. **H122** (`fdu-ytg5`). **Confirmed** (exp-118, leftover exp-122). Highest
   user-visible leverage.
   Deciding-scale `default-tree` walk is 96.3–97.5% of instrumented component on
   `system-private-frameworks`. Leftover is directory `__open` plus `getattrlistbulk` at
   1.403 calls/dir including EOF. No userspace walk cut ≥3%. Not H86. Do not retry as a
   snapshot load.

3. **H107** (`fdu-jcfn`). **Skipped.** Hunt recorded in exp-122. No nominated
   ignore-is-the-walk subject.
   `rustup-toolchains` and `system-private-frameworks` have zero `.gitignore` files
   (depth≤6; exp-118 already recorded 0 control reads on frameworks).
   `metabrowser-clone` is the exp-106 refute.
   `cargo-registry-src` has crate-internal gitignores but is screening-only (~22k) and
   not a checkout whose ignored share is the walk.
   tbd and urollup keep the same `dir_opens` with controls on and off.
   Ignore does not skip descent.
   Do not retry metabrowser.
   Do not invent a subject.

4. **H123** (`fdu-rum0`). **Confirmed** (exp-119). Follow-on to H117 (probe only).
   Product `query::report` on a retained `Index` is 1.7 ms versus one-shot 2,078.3 ms
   (~1,222×) on `system-private-frameworks`. Probe kept.
   No serving-policy change and no CLI flag.
   One-shot footer stays `cold scan`. Not a snapshot load on `fdu PATH`.

5. **H121** (`fdu-vf4b`). **Confirmed** (exp-120). Post-H115+H120 cache-hit re-profile
   on a frozen APFS clone of `metabrowser-clone` (live path had concurrent writers).
   Apply 42.7% of restore / ~12.8% of wall; candidates 47.6% / ~14.2%. No stage ≥50% of
   restore. No apply cut.
   Do not retry H116.

6. **H124** (`fdu-i39y`). **Rejected** (exp-121). First-pass analyze I/O. Opens 125,686
   of 133,708 files (path-binary already skipped).
   Empty 0.58% of opens; discovered-binary 4.9%; combined skippable share under 1% of
   wall. Read calls ~2 per open.
   Same-binary wall −4.22% [−20.79%, +5.10%]. No engine change.
   Do not retry a type/size gate or a larger read chunk.
   `F_RDADVISE` is person-gated `unsafe`.

7. **H111** (`fdu-jekg`). Open.
   Not in this host. Linux floor stage of H86. No Linux runner on this Darwin campaign
   machine. Do not treat a Darwin cell as this claim.
   Darwin comparison from exp-122: 1 open/dir + 1.403 `getattrlistbulk`/dir including
   EOF versus the playbook’s Linux 2.00 `getdents64`/dir plus per-entry `statx`. Do not
   restart the rewrite.

**Overnight history (do not re-queue):** H116 rejected, H118 rejected, H119 screened,
H117 confirmed as a probe, H120 accepted.
**Stacked session (do not re-queue):** H113 superseded (quiet gates including 45.48%;
leftover 16% of `content_open` in exp-123; restore-count accepted as H125 / exp-124),
H107 skipped (ignore does not skip descent), H122 confirmed (exp-118 + leftover
exp-122), H123 confirmed, H121 confirmed (no apply cut), H124 rejected (exp-121).

## Testing Strategy

H125 is recorded (exp-124). Incomplete-sidecar fail-closed stays.
H124 is recorded (exp-121). H122 (exp-118 + leftover exp-122), H123, and H121 are
recorded determinations.
Exact oracles and content digest stay as for exp-108–124. H123 kept one-shot
`cold scan`. Record every verdict, including skips at the quiet gate.

## Rollout Plan

#91 review fixes landed at `e667b739`. Further measurement is on stacked
`perf/campaign-next-2026-09-19`, base `perf/campaign-quiet-2026-09-18`, not `main`. Do
not push to #91. No merge, no force-push.
Engine changes land only as the experiment that tests the next row.

## Open Questions

- Whether H115 + H120 changed the exp-109 restore mix enough that apply no longer
  dominates (H121). **Closed:** exp-120. Apply 43%; candidates 48%; no stage ≥50%. No
  apply cut.
- Whether a Darwin deciding-scale `fdu PATH` profile (H122) names a leftover that is not
  already H86/H111 on Linux.
  **Closed:** exp-118 named `__open` + `getattrlistbulk`; exp-122: 1.403 bulk calls/dir;
  no Darwin userspace walk cut ≥3%. No walk-cut id minted.
- Whether H117’s retained read can become a product path (H123) without loading a
  snapshot on one-shot `fdu PATH`. **Closed:** exp-119. Product `query::report` 1.7 ms
  versus one-shot 2,078.3 ms.
  Probe kept. Not a snapshot load.
- Whether a type/size gate or read-ahead (H124) can cut first-pass analyze wall after
  H118/H119. **Closed:** exp-121. Every admitted open is required for lines; read calls
  are already one data chunk per file.
  No engine change.
- Whether restore’s already-paid candidate count can skip the second completeness walk
  without H113’s file-count heuristic (H125). **Closed:** exp-124. Wall −8.03%
  [−10.79%, −7.79%]. Engine kept.
  H113 superseded.

## References

- [The loop guide registry](../../guides/performance-loop.md#current-engine-010) —
  H107–H125
- [The runbook standing](../../guides/performance-loop-runbook.md#current-standing-2026-09-18)
- [Campaign 2](plan-2026-08-23-fdu-performance-campaign-2.md) — floor-anchored strategy
- [First Principles](../../architecture/fdu-design-principles.md#first-principles)
- [Engine architecture](../../architecture/fdu-engine-architecture.md) — one-shot vs
  opened
- [The instrumentation playbook](../../guides/performance-instrumentation-playbook.md)
- exp-107 through exp-124; H115 engine at `7798fdc1`; H120 streaming restore; H125
  restore-count at `be8d4d69`
- Beads: overnight epic `fdu-e9ow` (closed); remaining-queue epic `fdu-8ya1`; H121
  `fdu-vf4b`; H122 `fdu-ytg5`; H123 `fdu-rum0`; H124 `fdu-i39y`; H125 `fdu-wd4q`; H113
  quiet `fdu-rfr6` (superseded); H107 `fdu-jcfn`; H111 `fdu-jekg`; sidecar parent
  `fdu-78q6`; EntryId composite `fdu-jxhk` (do not restart)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
