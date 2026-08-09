# Plan: fdu Phase 1 — Core Engine, CLI, and Benchmarks

**Date:** 2026-08-08

**Status:** Active

**Background:**
[research-2026-08-06-file-rollup-engine.md](../../research/research-2026-08-06-file-rollup-engine.md)
— the twelve-tool survey and the architecture this plan builds.

## Working on This

Phase-0 work is on branch `claude/fdu-phase-0-scaffold`, open as
[PR #1](https://github.com/jlevy/fdu/pull/1), with `main` holding only the initial
commit. CI is green across Linux, macOS, and Windows.

Beads live on the `tbd-sync` branch and are visible from any clone (`tbd list`).
`make check` is the handoff gate; `AGENTS.md` carries the conventions worth not
rediscovering.

## Where This Stands

Phase 0 is complete: the repository exists, the architecture is expressed in code, and
the whole pipeline runs end to end.
What that means concretely, because “scaffold” is otherwise an unhelpful word:

**Built and tested.**

| Piece | Module | State |
| --- | --- | --- |
| Observation/commit contract | `types.rs` | Conditional producer observations; only effective accepted ops become clocked `AppliedDelta` |
| In-memory index | `index.rs` | Parent-pointer arena, generation/revision-safe arbitration, per-directory roll-ups, O(depth) apply, bounded feed |
| Roll-up reducers | `index.rs` | Counts, apparent and allocated bytes, pre-epoch-safe newest mtime, per-extension tallies — all hierarchical |
| Walk and reconcile | `scan.rs` | Scope-safe applying full/subtree reconciliation with explicit freshness and bounded producer batches; correct and portable, **not fast** |
| Snapshot | `snapshot.rs` | Flat format v2 bootstrap; bounded streaming load, payload checksum, semantic scope, owner-only concurrent replacement |
| Watch layer | `watch.rs` | notify-backed adapter plus clock-stable re-verifying apply/reconcile driver; not started by `open()` or Python |
| CLI | `cli.rs` | Human tree, schema-v2 JSON, exact kinds/errors, partial exit status, `NO_COLOR` |
| Python bindings | `fdu-py` | Bulk API, retained scan scope, freshness/errors, GIL release, watch-independent installed-wheel smoke |
| CI | `.github/workflows/ci.yml` | SHA-pinned Actions; locked three-OS tests, MSRV, docs, audit, and wheel smoke |

The workspace suite includes adversarial ordering and ABA arbitration, cache-scope,
non-UTF-8 identity, snapshot integrity/resource/permission bounds, concurrent
replacement, invalidation-retry, partial-exit, and installed-wheel tests.
Clippy pedantic is clean with `unsafe_code = "deny"` on the core crate; the
`--no-default-features` library path is built and tested in CI rather than assumed.

**Deliberately not built.** The walker is `read_dir` + `symlink_metadata`. Nothing in
this repository is fast yet, and **no performance claim should be made until the syscall
layer lands and the benchmark gate passes**. The snapshot format is a flat uncompressed
image whose reader is bounded and streaming but whose writer still materializes the
image. `open()` blocks until warm reconciliation finishes.
`IndexHandle` supports an explicit concurrent reader/reconciler model, but no server or
Python watcher integration is implied yet.
Watch queue bounds and permanent backend-failure marking remain part of the existing
watch-hardening bead.
These pieces settle the contract, cache lifecycle, and CI matrix before the syscall and
packed-layout work that is harder to change later.

Findings from implementation and the final branch-wide review, worth recording because
they were not in the research:

- Snapshot parsing now bounds the whole image, count, and path fields before allocation
  and rebuilds records incrementally.
  “Corrupt equals empty” is not a policy you get by intending it; resource bounds and
  sparse/oversized regression tests are part of it.
- The watcher deadlocked on shutdown because the worker thread was joined before the
  notify watcher was dropped, and dropping it is what closes the channel the worker
  waits on. Ordering is now explicit and commented.
  This is the kind of thing a feature flag hides until someone turns it on.
- A verified watch sample is not necessarily current by the time a consumer drains its
  queue. The applying driver now re-stats against a clock-stable index boundary and
  rejects a watcher/index root mismatch before consuming an observation.
- Operational batch size is still untrusted allocation input.
  Zero and oversized values now fail before allocation, and filesystem-boundary mode
  fails explicitly where the platform cannot supply device identity.
- The Python artifact had accidentally enabled the optional watch feature despite
  exposing no watcher.
  The wheel now proves the watch layer is deletable, and Windows cache discovery uses
  its native local-app-data location.
- Zero cannot be both the max-reducer identity and a valid epoch timestamp.
  Newest-mtime reduction now uses file presence as its identity and preserves negative
  timestamps.

## What Phase 1 Delivers

Goal 1 met and *demonstrated*: the fastest walker available that also returns full
detailed stats, with a cache that makes warm runs near-instant, benchmarked honestly
against dut and gdu.
Plus the CLI as a finished product surface, and the type-rule dialect defined early
enough that plugins never need two rule languages.

Phase 1 explicitly excludes content-tier metrics (words, sentences, paragraphs) and a
durable cross-restart delta journal.
A bounded process-local `AppliedDelta` feed exists; persisting or compacting it remains
separate work until the stat tier is solid.

## Sequencing

Risk spikes first, because three separate design decisions rest on numbers nobody has
measured. The plan is deliberately ordered so that the load-bearing assumption is tested
before the format that depends on it is frozen.

```text
      ┌─ SPIKE: revalidation cost at 500k ─┐
      ├─ SPIKE: snapshot load candidates ──┤
      ├─ SPIKE: metric-vector refcount ────┼──→ engine work ──→ benchmark gate ──→ Goal 1 met
      └─ SPIKE: tag-don't-prune gitignore ─┘         │
                                                     ├──→ CLI polish
                                                     ├──→ type-rule dialect
                                                     └──→ watch hardening ──→ metabrowser seam
```

### Stage A: Risk spikes

The whole design rests on “a parallel stat sweep of 500k unchanged files is fast enough
to feel instant.” If that is false, the cache tiering changes shape and everything
downstream of it is wasted work.
Measure first.

- **Revalidation cost curve.** Generate a 500k-entry corpus; measure a parallel stat
  sweep, with and without the directory-mtime shortcut, cold and warm page cache.
- **Snapshot load candidates.** Time flat-read versus block-compressed-with-tail-index
  for open latency *and* first-directory-listing latency.
  Open latency is the one that matters; a monolithic decode can win on throughput and
  still lose the product.
- **Metric-vector refcount.** Prototype dut’s atomic `unsearched_children` roll-up
  generalized from two `u64`s to a reducer vector, and confirm the barrier-free property
  survives generalization.
- **Tag-don’t-prune gitignore.** Confirm `GitignoreBuilder` used standalone can tag
  every entry at acceptable cost — this is the fix for the ~1.5 s gitignore parse that
  dominates metabrowser’s walker today.

### Stage B: The engine

- **Syscall walk layer.** Raw `getdents64` into a large reused per-thread buffer,
  dirfd-relative `statx` with a narrow field mask, `d_type` stat-avoidance, an LRU cache
  of open directory fds, work-stealing distribution, I/O threads capped around 8.
  Portable fallback retained for non-Linux.
- **Packed records.** Parent-pointer tree with name-only storage, optional attributes
  behind a flags word, interned device IDs, per-thread arenas.
  Target ncdu 2’s budget: ~25–32 bytes per file, with retained per-path diagnostics
  bounded separately rather than allowed to grow with every failed entry.
- **Reducer registry.** Turn the fixed `RollUp` struct into registered reducers with
  declared invertibility and an explicit aggregate-overflow policy, so a new metric is a
  registration rather than an engine change.
- **Block snapshot format.** Compressed blocks, tail index, `(block << k) | offset`
  references delta-encoded within a block, sibling groups contiguous, front-coded names,
  pre-computed roll-ups stored per directory.
- **Revalidation.** Add the directory-mtime shortcut and parallel sweep while preserving
  the current conditional, applying stream.
  Callers using `IndexHandle` can already serve stale-and-labeled between batches;
  conservative `open()` remains blocking.
- **Hardlink policy.** Pick a deterministic, incrementally maintainable rule.
  dut’s shared/unique split is the most informative, and none of the surveyed tools
  attempt to keep it correct under incremental updates — so this needs design, not just
  a choice.

### Stage C: Product surfaces

- **CLI polish** as scheduled work, not cosmetics, including structured raw path
  identity for partial errors and the corresponding lossless Python path surface.
- **Type-rule dialect**, a compatible superset of metabrowser’s `[[kind]]` predicates,
  compiled at build time.
- **Watch hardening**: cookie-paired renames applied without I/O on inotify, file-id
  stitching elsewhere, tuned backend selection (native for local filesystems, polling
  for NFS/FUSE/CIFS), periodic reconciliation for kqueue, and marking entries where
  watching failed instead of silently not watching.
  Bound raw and verified-observation queues; backpressure or a dropped hint must
  escalate to reconciliation.

### Stage D: Proof

- **Benchmark harness**, reporting cold and warm separately *and* raw-walk versus
  with-stats separately.
  Anything less compares different jobs: bfs and dut discard most metadata while fdu
  retains a full inventory.
- **Publishing**: crates.io and PyPI, abi3 wheels, re-verify name availability
  immediately before first publish since availability is a race.

## Exit Criteria

Phase 1 is done when all of these hold, and not before:

1. Cold scan within ~1.5x of dut on the same corpus, with full stats retained.
2. Warm re-run (snapshot load + revalidation) well under 1 s for 500k entries.
3. Memory within ~25–32 bytes per file record.
4. `fdu --help` is complete enough that an agent needs no other documentation, and the
   JSON schema is versioned and stable.
5. The benchmark harness reports the full cold/warm x raw/with-stats matrix, and the
   README’s performance claims cite it.
6. Goals 6 and 7 are ratified or amended — they already shape the architecture, so
   leaving them unsigned means building on an unratified premise.

## Open Questions

Carried from the research, still unanswered, each one a decision someone has to make:

1. **Hardlink attribution** under incremental updates — no prior art to copy.
2. **io_uring**: phase-1 complexity or a later accelerator behind a feature flag?
   Large machinery for a Linux-only win.
3. **DFS/BFS traversal order** — worth a runtime switch, and can warm/cold state be
   detected rather than configured?
4. **Content probe bounds** for type recognition (first 8 KiB?), and whether sniffing is
   on-demand-only at first so the walk stays stat-pure.
5. **How much classification belongs engine-side** versus in Python plugins.
   Proposal: the engine yields type verdicts from compiled rules; adapter-level sniffing
   stays in plugins. Validate against real manifests.
6. **Journal compaction and `since(clock)` across restart** — nice for SSE resume,
   simpler without.
7. **Non-invertible reducer cost under churn** — measure the pathological case (repeated
   deletes of the current max in a 100k-entry directory) before committing to which
   metrics are watch-maintained versus revalidation-only.
8. **Watcher ownership in metabrowser**: does `fdu::watch` replace `watch_backends.py`
   outright, or does metabrowser keep its watcher and push hints through
   `ingest_events()`? The open part is sequencing and the acceptance test for dropping
   the Python watcher.

## Note for Metabrowser

One research finding applies to metabrowser today, independent of whether fdu ever
ships: `watchfiles` maps notify’s event model down to `(change, path)` and drops the
`Rescan` flag on the way, so after a burst large enough to overflow kernel queues — a
`git checkout`, an `npm install` — the Python inventory can silently diverge until
restart. That is tracked separately in the metabrowser repository.

## Beads

Epic: **fdu-qfz6** — fdu phase 1: fastest walker with full stats, proven by benchmark.
Phase 0 is recorded and closed as **fdu-v178**.

Final phase-0 review follow-up: **fdu-vdi9**.

| Bead | Work |
| --- | --- |
| fdu-xktk | Reverify queued watch samples at a matching, clock-stable index root |
| fdu-52oq | Bound scan batching and fail closed on unsupported filesystem scope |
| fdu-3wpe | Keep Python independent of watch and discover the native Windows cache |
| fdu-x7jc | Preserve pre-epoch newest-mtime values |

| Stage | Bead | Work |
| --- | --- | --- |
| A | fdu-p2i1 | Spike: revalidation cost curve at 500k entries |
| A | fdu-1vd0 | Spike: snapshot format candidates, open vs first-listing latency |
| A | fdu-gdrv | Spike: metric-vector atomic-refcount roll-up |
| A | fdu-p35d | Spike: gitignore tag-don’t-prune via the `ignore` matcher |
| B | fdu-atqk | Walk layer: raw `getdents64` and dirfd-relative `statx` |
| B | fdu-aky1 | Walk layer: work-stealing parallelism and batched distribution |
| B | fdu-1gbl | Packed entry records: hit the 25–32 bytes per file budget |
| B | fdu-a6dz | Reducer registry: metrics as registrations, not engine changes |
| B | fdu-xihx | Block snapshot format: compressed blocks, tail index, lazy listing |
| B | fdu-wbis | Revalidation: directory-mtime shortcut and parallel sweep |
| B | fdu-579b | Hardlink attribution policy that survives incremental updates |
| B | fdu-r27g | Index concurrency: single-writer `RwLock`, escalate on measurement |
| C | fdu-oqoy | CLI human polish is product work, not cosmetics |
| C | fdu-jej9 | CLI agent surface: stable JSON schema, exit codes, help completeness |
| C | fdu-v4lc | Type-rule dialect: declarative rules compiled at build time |
| C | fdu-lka2 | Watch hardening: rename stitching, backend selection, kqueue sweep |
| D | fdu-ywu0 | Benchmark harness: cold/warm x raw-walk/with-stats vs dut and gdu |
| D | fdu-9cf0 | Publishing: crates.io, PyPI abi3 wheels, name re-verification gate |
| — | fdu-odx6 | Ratify proposed goals 6 and 7 |

Tracked but deliberately outside phase 1:

| Bead | Work |
| --- | --- |
| fdu-3n8c | Content-tier metrics: line, word, sentence, paragraph counts |
| fdu-3dtq | Cache coherency B: snapshot + append-only delta journal |
| fdu-ktka | io_uring accelerator for `openat`, `close`, and `statx` |
| fdu-p02b | Metabrowser integration: replace the Python walker and inventory hot path |

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
