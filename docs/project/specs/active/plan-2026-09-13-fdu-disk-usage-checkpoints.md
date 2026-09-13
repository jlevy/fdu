# Feature: Disk-Usage Checkpoints and Daily Deltas

**Date:** 2026-09-13

**Status:** Proposed.
Research and implementation plan; no new commands or persistence format ship with this
document.

## Objective

Save an inventory of a home folder or selected roots, return a day later, and identify
which directories gained or lost bytes without enumerating millions of unchanged files.
Comparing the same two saved checkpoints must return the same net changes.

The first inventory requires a scan.
Subsequent refreshes should verify changed scopes, update their ancestors, and read or
write only the persisted state they need.
The performance target covers the whole command, including loading and saving state.

## What Exists Today

Reviewed against `main` at `b75bf85` and the open engine stack through `afbb2ee` on
2026-09-13. The installed CLI identified itself as `0.1.0-dev+gb75bf85a3`, matching
`main`; it must not be confused with the newer checkout.

| Capability | Current behavior | Consequence for daily comparison |
| --- | --- | --- |
| Metadata inventory | Stores apparent and allocated bytes, fingerprints, and directory roll-ups | Reuse these facts and reducers |
| Default one-shot metadata report | Usually chooses a fresh scan instead of loading and revalidating the snapshot | A cache does not currently avoid the next traversal |
| `open()` with a compatible cache | Loads the image and reconciles every entry before returning | Reuse is still proportional to tree size |
| `--cache only` | Loads saved facts without filesystem verification; labels them stale | Useful for viewing an old inventory, not discovering changes |
| Snapshot persistence | One replaceable image per root; flat format v2 on `main` | No named historical baseline; loading materializes the full index |
| Native watch and `since(clock)` | Incremental updates and bounded process-local history | Does not recover a day of changes after process exit |
| FSEvents historical replay | Planned, with a recorded exploratory spike | No `journal` implementation or stored replay cursor yet |
| Partial scans | Report errors; incomplete scans do not replace the complete snapshot | `--allow-partial` does not make a denied home-folder scan cacheable |

Source: [execution planning](../../../../crates/fdu-core/src/execution.rs),
[open and cache policy](../../../../crates/fdu-core/src/lib.rs),
[snapshot format](../../../../crates/fdu-core/src/snapshot.rs), and
[cache guide](../../guides/cache-design.md).

[PR #48](https://github.com/jlevy/fdu/pull/48) adds the opened-root lifecycle, bounded
change polling, and verified multi-path refresh.
The stack through [PR #52](https://github.com/jlevy/fdu/pull/52) improves its one-shot
costs; at the reviewed head its CI passes but final performance acceptance remains open.
Its versioned change history is still process-local, and its v3 snapshot change does not
implement the FSEvents cursor described by the older plan.
Build this feature through the engine and mirror it in CLI and Python; do not make a
Python inventory replica or persist an opened handle’s session identity.

## User Workflow

The operations below define behavior, not committed command syntax.

1. **Capture a named baseline.** Scan the selected scope, show coverage and filesystem
   free space, and save immutable checkpoint A. Refuse a duplicate name unless the
   caller explicitly requests replacement.
2. **Refresh on the next visit.** Resume the working inventory from its last applied
   filesystem-event boundary.
   Reconcile changed scopes, then publish checkpoint B. If replay is unavailable or
   insufficient, run the ordinary scan and retain A.
3. **Compare A with B.** Show the largest positive and negative directory deltas,
   before/after bytes, file-count changes, and coverage.
   Expand a directory for the files or child directories responsible.
4. **Reuse or advance deliberately.** Re-reading A→B performs no refresh and produces
   the same answer. A separate refresh creates C; comparing A→C leaves A unchanged.
   Updating a rolling baseline is an explicit operation.

The default question is “which directories account for the largest storage changes?”
Rank by absolute signed allocated-byte change, with a deterministic path tie-breaker.
Keep increases and decreases visible, provide apparent-byte selection, and expose every
output bound.
Compute deltas before selecting the largest rows; comparing yesterday’s top
ten with today’s top ten loses directories that newly became large.

An immediately usable interim workflow is to save complete, dated directory reports:

```bash
fdu "$HOME/.cache" --one-filesystem --cache off \
  --view tree --depth all --limit all --format json > cache-before.json
```

Repeat to a different output path later and compare directory records by path, including
paths present in only one report.
Check `complete` and `errors` before subtraction.
Keep the reports outside the measured root.
Both sizes are already present in JSON. This supplies a baseline today, but both
captures still scan the selected tree and fdu does not yet provide the comparison
command.
`--modified-since` filters present files; it cannot recover old sizes or deleted
files. `--depth` and `--exclude` are report controls, not a promise to avoid scanning
descendants.

## Scope and Accounting

Start with independently refreshable roots such as `$HOME/.cache`, `$HOME/.codex`,
`$HOME/.claude`, `$HOME/Library/Caches`, and the workspace directory.
Add the home folder once the same contract is measured at that scale.
A home inventory and a nested root may be alternative views, but their totals must not
be added together.

Persist root and volume identity, symlink and filesystem-boundary policies, hidden-file
and exclusion rules, and the size-accounting version.
Comparisons require compatible scope.
Include hidden and Git-ignored build artifacts: those are often the growth being
investigated. Use one filesystem per inventory initially, with symlinks not followed.

Include Trash when measuring the whole home folder.
Moving a cache to Trash decreases its source directory and increases Trash; it does not
by itself free the blocks.
Exclude the monitor’s own managed store explicitly from the inventory and report its
bytes separately, so writing a checkpoint does not create an endless stream of
self-generated usage deltas.
This requires an actual engine scan-scope exclusion, not only a display filter.

Measure both apparent and allocated bytes.
Current roll-ups count file entries; they do not promise unique physical ownership of
APFS clones or hardlinks.
Keep the accounting policy stable across checkpoints; the deterministic hardlink policy
remains tracked in `fdu-579b`. Record `statvfs`/`df` free space alongside each
checkpoint. Changes in snapshots, sharing, purgeable data, open deleted files, and files
outside the scope can prevent a directory sum from matching that physical delta.

Permission loss is unknown state, not removal.
A disappearance is established only by a successful reconciliation of its containing
scope. For the first slice, retain the current complete-only snapshot rule and offer
smaller readable roots when a home scan is partial.
Persisting partial coverage requires a separately versioned format and unknown-subtree
semantics; it cannot be added by weakening the existing complete flag.

## Persistent State and Idempotence

Keep three identities separate:

| State | Lifetime and meaning |
| --- | --- |
| Working inventory | Latest reconciled entry facts and persisted directory aggregates |
| Applied FSEvents cursor | Per-volume progress used to discover work since the last refresh |
| Named checkpoint | Immutable comparison baseline with scope, coverage, capture interval, and durable revision identity |

A week-old baseline may be compared using a recently refreshed cursor.
The baseline’s age must not force replay from a week ago.
Persisted checkpoint IDs must be independent of `Clock` or `EngineVersion`, whose
identity and reset rules belong to a live process.

Replay events nominate work; they are not byte increments.
Coalesce overlapping scopes, observe current filesystem facts, and apply conditional
replacements or removals through the existing engine reducer path.
Duplicate observations must be no-ops.
When a file grows from 100 to 160 bytes, its contribution changes by 60, whether the
event is delivered once or three times.
A created-then-deleted file absent at both checkpoints contributes zero net change;
intermediate churn is a different feature.

Rename detection is optional attribution.
A move contributes a decrease at the old path and an increase at the new one, with net
zero at their shared ancestor when allocation is unchanged.
Inode matches alone do not prove a rename across long gaps because of reuse and
hardlinks. Compute ancestor deltas from the same facts without adding both a parent’s
recursive total and its children’s totals to a grand total.

Capture a filesystem-event fence before a full scan.
Publish the resulting inventory and that conservative cursor together.
For a scoped refresh, advance the cursor only through work reconciled and committed;
overlapping historical events remain safe to reobserve.
Cancellation, denied paths, or failed persistence must not publish a cursor past
unapplied work. Preserve hints arriving during refresh for the next pass.

Each committed working revision, its cursor, checkpoint references, and aggregate
changes need one recoverable publication boundary.
Define crash durability explicitly; atomic rename alone is only atomic visibility.
Concurrent refresh writers for the same root must serialize or conflict by revision.
Checkpoint reads remain immutable.

Named baselines are retained user state, even when their inventory blocks originated in
a cache. Ordinary cache eviction must not silently destroy them.
Share unchanged blocks between revisions or retain a compact change log; avoid one full
million-file copy per day.
Compaction preserves pinned checkpoints and atomically publishes its replacement.
Bound unpinned history by a stated retention policy; refuse an over-budget save rather
than silently evicting a named baseline.
Final policy values require measurements.

## Refresh Mechanism and Cost

Use the [FSEvents plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md) for macOS
change discovery and the existing bulk metadata reconciler for verification.
Spotlight may nominate additional paths or prioritize a preview.
Its asynchronous, selective index cannot establish complete cross-run additions,
deletions, or allocated usage.
The
[research update](../../research/research-2026-08-10-performance-frontier.md#daily-disk-usage-comparison-2026-09-13)
records the source and local evidence behind that decision.

```text
previous inventory + applied cursor
             | FSEvents replay
             v
      normalized dirty scopes
             | fresh metadata observations
             v
  engine updates facts and ancestor totals
             | atomic durable publication
             v
new inventory + applied cursor + checkpoint B
             |
             + checkpoint A --> signed directory/file deltas
```

Let N be retained entries, E replayed events, D entries enumerated in dirty scopes, and
A affected ancestors.
Whole-command latency includes:

```text
open needed state + replay(E) + reconcile(D) + update(A)
    + persist changed state + produce requested delta rows
```

The current flat loader and writer still cost O(N). A journal alone therefore cannot
deliver an end-to-end O(changes) claim.
Reuse the planned block format and durable mutation storage (`fdu-xihx`, `fdu-pdra`,
`fdu-3dtq`), including persisted roll-ups and lazy access.
A flat-format replay prototype is useful for measuring avoided traversal, but must
report its remaining load and rewrite cost.

Changed-directory enumeration can itself be large: one changed file in a directory of
100,000 siblings may require that directory to be relisted under the initial policy.
Replay cost also depends on cursor age and volume-wide event traffic, not only files
changed under the root.
Coalescing and bounds must expose these costs and choose the scan fallback when it is
cheaper. Queries over arbitrary old checkpoints may have to scan their retained change
range; fast stored comparisons and bounded top-change queries need their own measured
read path.

FSEvents results retain `JournalScoped` confidence.
A successful `HistoryDone`, matching UUID, or age bound does not prove complete history.
A requested fully verified result uses the scan path.
If a command’s work budget is exhausted, preserve its baseline and report incomplete
work; do not relabel the old answer as current.

## Delivery Slices and Acceptance

| Slice | Deliverable | Acceptance |
| --- | --- | --- |
| 1. Reproducible replay probe (`fdu-uwhl`) | Commit the probe, exact flags, cursor fences, and machine-readable results; compare with and without `FullHistory` | Deep append, deletion, move, restart, overlap, and controlled loss agree with an independent scan or produce a declared degraded result |
| 2. Checkpoint and comparison contract (`fdu-8ybz`) | Engine-native immutable baselines and signed net comparisons, initially using complete scanned state | Repeated A→B reads are identical; refresh never advances A; native, CLI, and Python surfaces agree |
| 3. Incremental macOS refresh | Cursor encoding/gates (`fdu-2cdv`), replay (`fdu-3tun`), scoped reconciliation (`fdu-rvje`) | Persist/restart/refresh tests and failure injection prove no lost cursor work or duplicate accounting |
| 4. Bounded persistent access | Block/lazy inventory and durable changes with checkpoint-aware compaction | Quiet refresh does not decode or rewrite all N entries; retained state stays within its declared budget |
| 5. Large-home workflow | Measure full capture, day-gap refresh, delta read, and fallback at realistic churn | Publish paired results against a fresh scan, with all work and coverage reported |

Slice 2 can provide useful comparisons before the replay optimization.
Slice 4 is needed before claiming fast whole-home refresh independent of inventory size.
Reconcile the format and lifecycle changes with the open PR stack before implementation;
do not turn this docs plan into an implicit dependency on an unmerged performance
result.

Use the existing [performance loop](../../guides/performance-loop.md) and predeclare
accept rules before trials.
Measure 100k, 1M, and multi-million-entry trees, including real package caches and a
home-folder corpus, with quiet, ordinary, and install/build churn.
Test gaps of 1 hour, 24 hours, 48 hours, and 7 days; repeat near the age-policy
boundary. A next-day check must not be described as incremental when the policy simply
forces it to scan.

Record first-result and completion latency, snapshot bytes read/written, entries
enumerated and verified, replayed events, dirty scopes, peak RSS, retained-store growth,
coverage, and fallback reason.
Seconds-scale day-gap completion is the product target, not an existing benchmark
result. Record the full-scan oracle separately from the timed candidate path, and use
immutable or quiesced subjects for exact equality; a live home scan is an operational
observation rather than a simultaneous filesystem snapshot.

Correctness cases include in-place append with unchanged directory mtimes,
allocated-only changes, file and subtree deletion, rename across sibling and monitored
roots, duplicate/overlapping events, hardlinks, sparse/cloned files, ignored and hidden
paths, permission loss and restoration, volume identity changes, event loss, concurrent
writers, and crashes before and after cursor publication.
Compaction and retention must preserve every pinned A→B result.
Exercise failure without materializing cloud-only files.
A resident observer is an optional later optimization; it does not replace the
cross-process, next-day acceptance test.

## References

- [Cache design](../../guides/cache-design.md)
- [FSEvents-scoped revalidation](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
- [Performance frontier](../../research/research-2026-08-10-performance-frontier.md)
- [Campaign 2, Phase D](plan-2026-08-23-fdu-performance-campaign-2.md#phase-d-the-warm-end-state-after-b-because-the-representation-decides-the-format)
- [Future persistence roadmap](../future/plan-2026-08-09-fdu-post-phase-1-roadmap.md)
- [Surface architecture](../../architecture/fdu-surface-architecture.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
