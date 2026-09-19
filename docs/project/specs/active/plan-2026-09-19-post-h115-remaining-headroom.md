# Feature: Post-H115 Remaining Headroom

**Date:** 2026-09-19

**Author:** fdu project

**Status:** Active. This plan is the 2026-09-19 creative block: new hypotheses after
H115, what was rejected as not significant, and the overnight queue.
[The runbook standing](../../guides/performance-loop-runbook.md#current-standing-2026-09-18)
points here for that queue and does not keep a second copy.
The loop guide registry remains the full hypothesis text.

## Overview

H115 accepted a restore-only bottom-up content roll-up (−9.69% `content-cache-hit` wall
on deciding-scale metabrowser).
That is tonight’s standing best.
This block asks what remaining time is still large enough to hunt, on four jobs, without
restarting a dead end or a person-gated rewrite.

It is a planning block, not a measurement cell.
Another agent may be in a quiet H113 gate on the same branch; this document does not
schedule a second uncontrolled H113 and does not start a pair.

## Goals

- Name only hypotheses that are plausible at the 3% wall bar (or a structural ceiling)
  on a named job and subject, and that can be wrong
- Register them as H116 and up in
  [the loop guide](../../guides/performance-loop.md#current-engine-010)
- Own the overnight pickup after H113’s quiet confirmatory: order, metric, subject,
  accept-rule sketch, why next, what refutes, bead
- Keep one source of truth for that queue (this file)

## Non-Goals

- Starting a measurement cell, keeping an engine experiment patch, or reverting H115
- Loading a metadata snapshot on `fdu PATH` as a “free win” (H108 / H9)
- Restarting the H86 structural rewrite, the `fdu-jxhk` EntryId composite, or H83 as a
  format rewrite
- Retrying H113 on an uncontrolled cell, H114 alloc trims, H109 Path rewrites, parse
  speed, or H103-shaped instruction cuts
- Linux H111 on a host that has no Linux runner tonight
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

New registry rows (full text in the loop guide):

| # | One-line claim | Job |
| --- | --- | --- |
| H116 | Cache-only restore can match sidecar records to the live index without a full `analysis_candidates` Vec+HashMap | `content-cache-hit` |
| H117 | An opened-root second report on Darwin is at least 3% faster than a metadata one-shot of the same request | opened retained read vs `default-tree` |
| H118 | First-pass `analyze_index` can use H115’s insert-then-rebuild instead of per-file `merge_ancestors` | `content-basic` component |
| H119 | First-pass analyze can overlap file I/O with the metadata walk instead of opening every file after the scan | `content-basic` wall / product `--analyze` |
| H120 | Streaming sidecar parse-into-apply (no full decoded-records Vec beside the files map) cuts peak RSS at least 10% | `content-cache-hit` RSS |

H116 is not H113. H113 only skipped the second `len()` walk after restore.
H116 deletes the HashMap build inside `load_content_cache` (exp-109: 25.4% of restore).
If H116 lands, that second walk may disappear with it; do not treat that as an H113
accept.

H117 is not a license to load a snapshot on `fdu PATH`. The instrument is a `fdu-core`
probe mode (or the Python `Index`) that opens once and reports twice.
`opened-discovery` is a cold first open; `warm-revalidate` loads a snapshot and
reconciles. Neither is this claim.
The probe mode is engine-first if it does not exist yet.

H119 forbids serializing workers (H79). The timed `content-basic` *component* excludes
the setup scan, so overlap is judged on process wall or a product `--analyze` job.

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
  H116 removes that classify on the hit path)
- Hardware CRC32C (`fdu-6kyn`) or PGO (`fdu-pdne`) as tonight’s wall hunt
- `searchfs` (H77) or Linux H111: person-gated / no Linux runner in this overnight setup
- Bounding the observation channel (H91) without a current-engine occupancy trace
- Directory-only transient tree (H66) as a cache-hit leftover

### Overnight Inclusion Rules

- H113 quiet confirmatory (`fdu-rfr6`, exp-113) runs first *if* a quiet host holds.
  If the start gate fails, skip it.
  Do not run another uncontrolled H113.
- Then take the queue below, in order.
  After an accept, re-screen the next row: H116 may eat H113 and part of H120; H118 may
  vanish into H119’s I/O.
- Uncontrolled is allowed on H116–H120 when quiet fails.
  Label it. Do not lower the 25% busy bar.
- H117’s effect, if real, should be several-fold and readable uncontrolled.
  Still do not claim quiet.
- H111 is not in this overnight setup (no Linux runner).
- Items that need a quiet cell and nothing else are labeled **needs quiet host**.
  Tonight that is only H113.

## Implementation Plan

### Overnight Queue (Source of Truth)

After whatever H113 quiet does:

1. **H116** (`fdu-kro6`). **Done — rejected** (exp-114, uncontrolled).
   Wall +8.70% [−19.33%, +63.90%]. Quiet gate 85.6%. User CPU −15.65%; RSS −11.27%.
   Engine reverted. Do not retry uncontrolled.

2. **H118** (`fdu-kyts`). **Done — rejected** (exp-115, uncontrolled).
   Component −2.60% [−12.00%, +23.86%]. Quiet gate 39.7%. User CPU −4.53%. File I/O hid
   the ancestor walk. Engine reverted.
   Do not retry uncontrolled.

3. **H119** (`fdu-9g54`). **Done — screened.** Walk overlap cannot clear 3%. Profile:
   `read` 59.06%, `__open` 17.44%, `fdu::scan` 0.13%. `openat` is the leftover and needs
   a new `unsafe` block; not tonight.
   No engine change. Do not retry walk-overlap.

4. **H117** (`fdu-7wiq`). Metric: opened-root second report wall vs one-shot
   `default-tree` / `fdu PATH` on the same unchanged Darwin tree
   (`system-private-frameworks` or `metabrowser-clone`). Accept: opened second read at
   least 3% faster (expected several-fold); one-shot footer stays `cold scan`. Why next:
   names the remaining metadata lever without violating H108. Refute: retained read
   within 3% of one-shot after warmup (retention is not the cost).
   First step if missing: a `fdu-core` probe mode, not a CLI flag.
   Quiet: not required if the gap is large.

5. **H120** (`fdu-y9n9`). Metric: `content-cache-hit` peak RSS (guard: wall
   non-inferior, interval not entirely above +3%). Subject: `metabrowser-clone`. Accept:
   peak RSS down at least 10%; digest identical.
   Why next: content-hit RSS is the product constraint on `--analyze` trees; H116 may
   already have taken the HashMap half.
   Refute: peak is the retained index plus content records, not the transient decode.
   Quiet: not required.

**Not tonight:** H111 (no Linux runner), H77 `searchfs` (person-gated), H86 / `fdu-jxhk`
rewrites, H107 unless the ignored share *is* the walk.

## Testing Strategy

Each queued hypothesis is a 12-pair interleaved `make perf-compare` against HEAD with
H115 in, `FDU_COUNTERS` unset for the claim-grade wall, no RAM disk.
Exact oracles and content digest stay as for exp-108–112. H116 and H117 must keep the
incomplete-sidecar and one-shot `cold scan` fail-closed behaviors.
Record every verdict, including skips at the quiet gate.

## Rollout Plan

Docs and beads on PR [#91](https://github.com/jlevy/fdu/pull/91) only.
No merge, no force-push, no second performance PR. Engine changes land only as the
experiment that tests the next row.

## Open Questions

- Whether H116’s index-by-path lookup is cheaper than today’s HashMap, or only cheaper
  than building `analysis_candidates` (the classify walk is the predicted win).
- Whether a first-pass `content-basic` component can clear 3% (H118) on a job whose wall
  is file I/O. Answered no on this uncontrolled cell (exp-115): −2.60%
  [−12.00%, +23.86%].
- Whether an opened-retained probe already exists under another name; none of
  `opened-discovery`, `warm-revalidate`, or `warm-snapshot-load` is that job.

## References

- [The loop guide registry](../../guides/performance-loop.md#current-engine-010) —
  H107–H120
- [The runbook standing](../../guides/performance-loop-runbook.md#current-standing-2026-09-18)
- [Campaign 2](plan-2026-08-23-fdu-performance-campaign-2.md) — floor-anchored strategy
- [First Principles](../../architecture/fdu-design-principles.md#first-principles)
- [Engine architecture](../../architecture/fdu-engine-architecture.md) — one-shot vs
  opened
- [The instrumentation playbook](../../guides/performance-instrumentation-playbook.md)
- exp-108 through exp-112; H115 engine at `7798fdc1`
- Beads: epic `fdu-e9ow`; H116 `fdu-kro6`; H118 `fdu-kyts`; H119 `fdu-9g54`; H117
  `fdu-7wiq`; H120 `fdu-y9n9`; creative pass `fdu-m3mw`; H113 quiet `fdu-rfr6`; sidecar
  parent `fdu-78q6`; EntryId composite `fdu-jxhk` (do not restart)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
