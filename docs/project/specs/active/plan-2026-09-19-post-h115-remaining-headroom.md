# Feature: Post-H115 Remaining Headroom

**Date:** 2026-09-19

**Author:** fdu project

**Status:** Active. Overnight H116–H120 is done.
This file is the remaining unaddressed-hypothesis queue after that overnight: H107
(ignore-is-the-walk only) and H111 (not this host).
H122 is confirmed (exp-118, tighter leftover exp-122). H123 is confirmed (exp-119). H121
is confirmed (exp-120): apply no longer dominates.
H124 is rejected (exp-121). H125 is accepted (exp-124): restore-count completeness.
H126 is confirmed (exp-125): completeness walk gone; no new userspace cut.
H127 is confirmed (exp-126): opened-discovery ~8.8× first-pass; no smallest cut.
H128 is confirmed (exp-127): file-heavy `default-tree` walk still the job.
H129 is accepted (exp-128): restore omits classify (−13.11% wall).
H130 is confirmed (exp-129): restore classify gone; `path_of` 11.85%; no engine patch.
H131 is accepted (exp-130): restore DFS parent-path join (−4.07% wall).
H132 is confirmed (exp-131): restore-walk `path_of` gone; snapshot `path_of` 9.89%. H133
is accepted (exp-132): skip unused snapshot path reconstruction (−6.37% wall).
H134 is confirmed (exp-133): snapshot `path_of` gone; no new ≥3% userspace cut.
H135 is confirmed (exp-134): first-pass leftover after H124 is still file I/O; no new
≥3% userspace cut. H136 is confirmed (exp-135): first-run leftover after H128 is still
the walk; snapshot write ~45 ms is ≥3% and not skippable.
H113 is superseded by H125.
[The runbook standing](../../guides/performance-loop-runbook.md#current-standing-2026-09-18)
keeps an abbreviated next-up in that order; this file is the source of truth for the
full rows. The loop guide registry remains the full hypothesis text.

## Overview

H115 accepted a restore-only bottom-up content roll-up (−9.69% `content-cache-hit` wall
on deciding-scale metabrowser).
H120 accepted streaming sidecar parse-into-apply (−10.13% peak RSS; wall non-inferior).
H125 accepted restore-count completeness (−8.03% wall on top of those).
H129 accepted restore-without-classify (−13.11% wall on top of H125). H131 accepted the
restore DFS parent-path join (−4.07% wall on top of H129). H132 confirmed the leftover
after that join. H133 accepted the unused snapshot path skip (−6.37% wall on top of
H131). H134 confirmed the leftover after that skip.
Those remain the standing content-hit bests.
H135 confirmed the first-pass leftover after H124 (still file I/O).

This block names the directions the overnight did **not** address, as first-class
hypotheses, and keeps the overnight verdicts as history so they are not re-queued.

It is a planning block, not a measurement cell.
It does not start a pair.
H113 is superseded. Do not retry the file-count shortcut.

## Goals

- Name only hypotheses that are plausible at the 3% wall bar (or a structural ceiling)
  on a named job and subject, and that can be wrong
- Keep H121–H136 registered in
  [the loop guide](../../guides/performance-loop.md#current-engine-010); keep H107 and
  H111 open with honest status.
  H124 is rejected (exp-121). H125 is accepted (exp-124). H126 is confirmed (exp-125).
  H129 is accepted (exp-128). H130 is confirmed (exp-129). H131 is accepted (exp-130).
  H132 is confirmed (exp-131). H133 is accepted (exp-132). H134 is confirmed (exp-133).
  H135 is confirmed (exp-134). H136 is confirmed (exp-135). H113 is superseded.
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
| Second completeness walk | 13% of `content_open` | H125 accepted (exp-124); H126 confirmed gone (exp-125) |
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
   H126 re-profiled after H125: completeness gone; mix unchanged; no new cut.
   Snapshot parse remains H78/H92.
3. **First-pass analyze.** H118 rejected insert-then-rebuild.
   H119 screened walk-overlap (`fdu::scan` 0.13%). Leftover I/O was H124: admit fewer
   files, or read-ahead on the ones already admitted.
   Rejected (exp-121). H135 confirmed the leftover after that reject: still file I/O
   (`read` 58.87%, `__open` 16.09%); no skippable ≥3% userspace cut.
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
| H126 | After H125, leftover profile names whether completeness is gone and whether a new ≥3% userspace cut remains | `content-cache-hit` | Confirmed (exp-125). Completeness gone. No new cut. |
| H127 | First-pass walk I/O versus opened-discovery I/O on file-heavy metabrowser | `cold-scan-index` vs `opened-discovery` | Confirmed (exp-126). Opened ~8.8× first-pass. No smallest cut. |
| H128 | File-heavy `default-tree` leftover after H122 still has the walk as the job | `default-tree` | Confirmed (exp-127). Walk 92.9%. No new cut. |
| H129 | Cache-only restore omits classify (walk and apply self-check); HashMap stays | `content-cache-hit` | Accepted (exp-128). Wall −13.11%. Engine kept (`6887a864`). |
| H130 | After H129, leftover names whether restore classify is gone and whether a new ≥3% userspace cut remains | `content-cache-hit` | Confirmed (exp-129). Classify 0. `path_of` 11.85%. No engine patch. |
| H131 | Restore DFS joins the parent path instead of `path_of` per file; HashMap stays | `content-cache-hit` | Accepted (exp-130). Wall −4.07%. Engine kept (`7840ce9b`). |
| H132 | After H131, leftover names whether restore-walk `path_of` is gone and whether a new ≥3% userspace cut remains | `content-cache-hit` | Confirmed (exp-131). Restore-walk `path_of` 0. Snapshot `path_of` 9.89% discarded on one-shot `serving=None`. No engine patch. |
| H133 | Skip `path_of` in `insert_loaded_child` when serving is off | `content-cache-hit` | Accepted (exp-132). Wall −6.37%. Engine kept (`143a1c73`). |
| H134 | After H133, leftover names whether snapshot `path_of` is gone and whether a new ≥3% userspace cut remains | `content-cache-hit` | Confirmed (exp-133). Snapshot `path_of` 0. No new cut. No engine patch. |
| H135 | After H124, leftover names whether first-pass apply/classify is a ≥3% userspace cut | `content-basic` | Confirmed (exp-134). Leftover still file I/O. No new cut. No engine patch. |
| H136 | After H128, leftover names whether first-run snapshot write/render is a skippable ≥3% cut | `default-tree-first` | Confirmed (exp-135). Walk still the job. Write ~45 ms, not skippable. No engine patch. |
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

H126 is confirmed (exp-125): completeness 1 sample / 15,296 (0.007% of `content_open`).
`load_content` 63.5%; snapshot 36.4%; first `analysis_candidates` walk 15.7%. Restore
mix unchanged (candidates ~48%, apply ~43%). No new userspace cut at record time.
H129 later took the classify half of that walk.
Do not retry H116.

H129 is accepted (exp-128): wall −13.11% [−20.22%, −12.67%] on frozen
`metabrowser-clone`. Restore-without-classify kept (`6887a864`). HashMap and `path_of`
stay. Quiet that tick 31.53%.

H130 is confirmed (exp-129): restore classify 0 of `content_open`. Completeness still 0.
Snapshot 43.3%. `path_of` 11.85%. No engine patch.
Quiet that tick 34.97%.

H131 is accepted (exp-130): wall −4.07% [−4.54%, −3.28%] on frozen `metabrowser-clone`.
Restore DFS parent-path join kept (`7840ce9b`). Public `path_of` stays.
Quiet that tick 27.23%.

H132 is confirmed (exp-131): restore-walk `path_of` 0 of `content_open`. Completeness
still 0. Snapshot 42.78%. Remaining `path_of` 9.89%, all under snapshot
`insert_loaded_child`, discarded because one-shot load has `serving = None`. No engine
patch. Quiet this tick refused after a 24.38% pre-check.

H127 is confirmed (exp-126): opened-discovery component 2,761 ms versus first-pass 315
ms (~8.8×) on frozen `metabrowser-clone`. Same 11,517 dir opens.
First-pass 1.952 `getattrlistbulk`/dir.
Opened uses `read_dir`+`fstatat`; 11,524 journal clones; 1.12M live roll-up merges.
No smallest cut. Opened roots run no analyzers.

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
  H116 was the attempt to drop the HashMap; H129 later skipped classify only and
  accepted)
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
  mix was re-checked after H125 as H126; H129’s leftover was H130; H131’s leftover was
  H132; H133 took the unused snapshot `path_of` skip; H134 confirmed that leftover (no
  new cut). H135 confirmed the first-pass leftover after H124 (still file I/O; no new
  cut). Do not invent a skip.
- Uncontrolled is allowed on leftover profiles and H107 when quiet fails.
  Label it. Do not lower the 25% busy bar.
- H111 is not in this host (no Linux runner).
- Do not re-queue H113 file-count, H116, H118, H119 walk-overlap, H114, H109,
  snapshot-load without a policy change, or the H86 / `fdu-jxhk` rewrite.

## Implementation Plan

### Remaining Queue (Source of Truth)

Take these in order.
Overnight H116–H120 is history, not a retry list.

1. **H136** (`fdu-w9jb`). **Confirmed** (exp-135). First-run leftover after H128 is
   still the walk (83–88% of `default-tree-first` component).
   Isolated `snapshot-save` 45.3 ms (~11–16% of first-run).
   Write is ≥3% and not skippable.
   No engine patch. Quiet this tick 75.4%; pair initial 46.59% final 88.11%. **H135**
   (`fdu-hh0t`). **Confirmed** (exp-134). First-pass leftover after H124 is still file
   I/O (`read` 58.87%, `__open` 16.09% of process).
   `classify_with` 2.02%. `commit_record` 0.59%. `merge_ancestors` 0.43%. No skippable
   ≥3% userspace cut. No engine patch.
   Quiet that tick 56.4%; pair initial 35.75% final 82.48%. **H134** (`fdu-03pr`).
   **Confirmed** (exp-133). Snapshot `path_of` gone (0 of `content_open`). Completeness
   still 0. Restore-walk `path_of` still 0. Classify still 0. Snapshot 37.94%. Remaining
   leftover is already-landed restore work and already-rejected stages.
   No engine patch. Quiet that tick 28.07%; pair 29.36%. **H133** (`fdu-7m91`).
   **Accepted** (exp-132). Skip unused snapshot `path_of` when serving is off.
   Wall −6.37% [−18.23%, −5.66%] on frozen `metabrowser-clone`. Engine kept
   (`143a1c73`). Quiet that tick 27.87%. **H132** (`fdu-8z5i`). **Confirmed** (exp-131).
   Restore-walk `path_of` gone (0 of `content_open`). Snapshot `path_of`
   (`insert_loaded_child`) 9.89%, discarded because one-shot load has `serving = None`.
   Completeness still 0. Snapshot 42.78%. No engine patch.
   Quiet that tick refused after a 24.38% pre-check.
   **H131** (`fdu-1dxc`). **Accepted** (exp-130). Restore DFS joins the parent path.
   Wall −4.07% [−4.54%, −3.28%] on frozen `metabrowser-clone`. Engine kept (`7840ce9b`).
   Public `path_of` stays.
   Quiet that tick 27.23%. **H130** (`fdu-ajbw`). **Confirmed** (exp-129). Restore
   classify gone. Completeness still 0. Snapshot 43.3%. `path_of` 11.85% of
   `content_open`. No engine patch.
   Quiet that tick 34.97%. **H129** (`fdu-qjjh`). **Accepted** (exp-128). Restore omits
   classify. Wall −13.11% [−20.22%, −12.67%] on frozen `metabrowser-clone`. Engine kept
   (`6887a864`). HashMap stays.
   Quiet that tick 31.53%. **H125** (`fdu-wd4q`). **Accepted** (exp-124). Restore-count
   completeness. Wall −8.03% [−10.79%, −7.79%]. Engine kept (`be8d4d69`). H113
   superseded. File-count not compiled.
   exp-113 unused. Do not retry H113. **H126** (`fdu-16jh`). **Confirmed** (exp-125).
   Completeness walk gone.
   First candidates walk was 15.7%; H129 took the classify half.
   Do not retry H116. **H127** (`fdu-v12n`). **Confirmed** (exp-126). Opened-discovery
   ~8.8× first-pass. `read_dir`+`fstatat` versus `getattrlistbulk`; journal clones
   remain. Opened roots run no analyzers.
   No smallest cut. **H128** (`fdu-0wym`). **Confirmed** (exp-127). File-heavy
   `default-tree` walk 92.9% of component.
   1.952 `getattrlistbulk`/dir.
   Snapshot not loaded.
   No new cut.

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
H126 confirmed (exp-125; completeness gone; no new cut), H127 confirmed (exp-126;
opened-discovery ~8.8× first-pass; no smallest cut), H128 confirmed (exp-127; file-heavy
`default-tree` walk still the job), H129 accepted (exp-128; restore-without-classify),
H130 confirmed (exp-129; classify gone; `path_of` 11.85%; no engine patch), H131
accepted (exp-130; restore DFS parent-path join), H132 confirmed (exp-131; restore-walk
`path_of` gone; snapshot `path_of` 9.89%), H133 accepted (exp-132; unused snapshot path
skip), H134 confirmed (exp-133; snapshot `path_of` gone; no new cut), H135 confirmed
(exp-134; first-pass leftover still file I/O; no new cut), H136 confirmed (exp-135;
first-run walk still the job; snapshot write not skippable), H107 skipped (ignore does
not skip descent), H122 confirmed (exp-118 + leftover exp-122), H123 confirmed, H121
confirmed (no apply cut), H124 rejected (exp-121).

## Testing Strategy

H125 is recorded (exp-124). Incomplete-sidecar fail-closed stays.
H126 is recorded (exp-125). Completeness walk gone; no new cut.
H127 is recorded (exp-126). Opened-discovery ~8.8× first-pass; no smallest cut.
H128 is recorded (exp-127). File-heavy `default-tree` walk still the job.
H129 is recorded (exp-128). Restore-without-classify kept.
H130 is recorded (exp-129). Restore classify gone; no engine patch.
H131 is recorded (exp-130). Restore DFS parent-path join kept.
H132 is recorded (exp-131). Restore-walk `path_of` gone; no engine patch.
H133 is recorded (exp-132). Unused snapshot path skip kept.
H134 is recorded (exp-133). Snapshot `path_of` gone; no new cut.
H135 is recorded (exp-134). First-pass leftover still file I/O; no new cut.
H136 is recorded (exp-135). First-run leftover still the walk; snapshot write not
skippable. H124 is recorded (exp-121). H122 (exp-118 + leftover exp-122), H123, and H121
are recorded determinations.
Exact oracles and content digest stay as for exp-108–135. H123 kept one-shot
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
- Whether the completeness walk is gone after H125 and whether a new ≥3% userspace cut
  remains (H126). **Closed:** exp-125. Completeness 0.007% of `content_open`. No new
  cut. Do not retry H116.
- Whether first-pass walk I/O and opened-discovery I/O differ by a named leftover on
  file-heavy metabrowser (H127). **Closed:** exp-126. Opened ~8.8× first-pass.
  `read_dir`+`fstatat` versus `getattrlistbulk`; journal clones remain.
  No smallest cut.
- Whether file-heavy `default-tree` leftover after H122 still has the walk as the job
  (H128). **Closed:** exp-127. Walk 92.9% of component.
  Snapshot not loaded.
  1.952 `getattrlistbulk`/dir.
  No new cut.
- Whether cache-only restore can omit classify (walk and apply self-check) while keeping
  the HashMap (H129). **Closed:** exp-128. Wall −13.11% [−20.22%, −12.67%]. Engine kept
  (`6887a864`). `path_of` remains.
- Whether restore classify is gone after H129 and whether a new ≥3% userspace cut
  remains (H130). **Closed:** exp-129. Classify 0. Completeness 0. `path_of` 11.85%.
  Snapshot 43.3%. No engine patch.
- Whether the restore DFS can join the parent path it already holds instead of `path_of`
  per file (H131). **Closed:** exp-130. Wall −4.07% [−4.54%, −3.28%]. Engine kept
  (`7840ce9b`). Digest identical.
  Public `path_of` stays.
- Whether restore-walk `path_of` is gone after H131 and whether a new ≥3% userspace cut
  remains (H132). **Closed:** exp-131. Restore-walk `path_of` 0. Snapshot `path_of`
  9.89% of `content_open`, discarded because one-shot load has `serving = None`. No
  engine patch.
- Whether one-shot snapshot load can skip `path_of` when serving is off (H133).
  **Closed:** exp-132. Wall −6.37% [−18.23%, −5.66%]. Engine kept (`143a1c73`). Digest
  identical. Public `path_of` stays.
  Opened-root serving insert stays.
- Whether snapshot `path_of` is gone after H133 and whether a new ≥3% userspace cut
  remains (H134). **Closed:** exp-133. Snapshot `path_of` 0. Completeness 0. Remaining
  leftover is already-landed restore work and already-rejected stages.
  No engine patch.
- Whether first-pass apply/classify after H124 is a ≥3% userspace cut (H135).
  **Closed:** exp-134. Leftover still file I/O (`read` 58.87%, `__open` 16.09%).
  `classify_with` 2.02%. No engine patch.
- Whether first-run snapshot write/render after H128 is a skippable ≥3% cut (H136).
  **Closed:** exp-135. Walk still 83–88% of first-run.
  Isolated save 45.3 ms, not skippable.
  No engine patch.

## References

- [The loop guide registry](../../guides/performance-loop.md#current-engine-010) —
  H107–H136
- [The runbook standing](../../guides/performance-loop-runbook.md#current-standing-2026-09-18)
- [Campaign 2](plan-2026-08-23-fdu-performance-campaign-2.md) — floor-anchored strategy
- [First Principles](../../architecture/fdu-design-principles.md#first-principles)
- [Engine architecture](../../architecture/fdu-engine-architecture.md) — one-shot vs
  opened
- [The instrumentation playbook](../../guides/performance-instrumentation-playbook.md)
- exp-107 through exp-135; H115 engine at `7798fdc1`; H120 streaming restore; H125
  restore-count at `be8d4d69`; H129 restore-without-classify at `6887a864`; H131
  parent-path join at `7840ce9b`; H133 unused snapshot path skip at `143a1c73`
- Beads: overnight epic `fdu-e9ow` (closed); remaining-queue epic `fdu-8ya1`; H121
  `fdu-vf4b`; H122 `fdu-ytg5`; H123 `fdu-rum0`; H124 `fdu-i39y`; H125 `fdu-wd4q`; H126
  `fdu-16jh`; H127 `fdu-v12n`; H128 `fdu-0wym`; H129 `fdu-qjjh`; H130 `fdu-ajbw`; H131
  `fdu-1dxc`; H132 `fdu-8z5i`; H133 `fdu-7m91`; H134 `fdu-03pr`; H135 `fdu-hh0t`; H136
  `fdu-w9jb`; H113 quiet `fdu-rfr6` (superseded); H107 `fdu-jcfn`; H111 `fdu-jekg`;
  sidecar parent `fdu-78q6`; EntryId composite `fdu-jxhk` (do not restart)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
