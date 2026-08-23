# Feature: fdu for Interactive Clients — the Metabrowser Integration Contract

**Date:** 2026-08-23

**Author:** fdu project

**Status:** Draft

## Overview

The README stakes a claim: persistent per-directory roll-ups with incremental
revalidation are “what a live file browser actually needs.”
Metabrowser is that browser — a local Starlette server with a live web front end,
already fast for a Python implementation, already independently converged on fdu’s
architecture, and already naming a native walker as the way past its measured floor.
As fdu approaches its first release, working this integration end to end is the best
instrument available for finding where the API is still shaped for the CLI rather than
for a program.

This spec records what that exercise found.
Most of the contract is already served: the typed Python API walks the reference corpus
roughly 29× faster than the walker it would replace, revalidates an unchanged tree in
about 100 ms, answers per-directory queries in microseconds, and delivers verified watch
batches at whatever throttle the caller asks for.
Three gaps are real, and each generalizes past metabrowser to any embedded consumer — a
TUI, an IDE panel, an agent serving repeated queries:

1. **Partitioned tallies.** Metabrowser renders two numbers for every directory —
   everything, and everything not gitignored — and fdu can maintain only one.
   Selection-time filtering is measured two to three orders of magnitude too slow to
   serve it per request.
   This is the one genuinely new engine capability this spec proposes.
2. **An embedder-grade watch contract.** The watch layer’s event-driven core is right,
   but a server that lives on an event loop needs an async story, a network-filesystem
   fallback, a way to ingest hints from a foreign watcher, and a per-batch statement of
   which roll-ups changed.
3. **The streaming session**, so a cold open serves requests while the walk runs.
   [The progressive-results plan](plan-2026-08-11-fdu-progressive-results.md) owns that
   design; this spec adds only the integration-facing shape it must land with.

The rest is polish that working the contract surfaced: classification identity in
listings, a machine-readable truncation remainder, walk telemetry for the client’s own
performance loop, and documentation of thread affinity.

## Goals

- A metabrowser build can serve its default UI — dual tallies, live updates, resumable
  change feed, bounded subtree roll-ups — with its walker, inventory aggregation, and
  watcher replaced by fdu, and with no per-request re-aggregation in Python.
- Every capability lands engine-first, then CLI and Python present it, preserving the
  parity contract: nothing here is reachable by flag but not by typed call, or the
  reverse.
- The first-release Python API is cleaner for every embedded consumer, not shaped to one
  client: each addition names the question it answers, and metabrowser-specific policy
  stays out of the core.
- The claimed speedup is demonstrated end to end by a committed, testable example, not
  asserted.

## Non-Goals

- Metabrowser-side work.
  The adoption sequence, its feature flag, and its wire models belong to that
  repository; this spec covers the fdu side of the seam.
- The session and provenance design, owned by
  [progressive results](plan-2026-08-11-fdu-progressive-results.md), and the FSEvents
  journal, owned by [its own plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md).
  This spec consumes both and re-states neither.
- Arbitrary user-defined tag vocabularies.
  Two rules — gitignore and visibility — have a consumer today; a tag registry is
  speculation and fails the axis test.
- Content-analysis changes.
  The content tier is untouched.

## Background

### The consumer, and what it measures

Metabrowser holds a process-wide inventory of the served tree: one `FsEntry` per path,
per-directory subtree aggregates maintained on every change, a BFS boot walker feeding
it, a `watchfiles`-based watcher keeping it live, and an SSE bus streaming batched
change events to the browser with a resumable cursor.
Every directory row carries four aggregate values — total and gitignore-filtered files
and bytes — plus recursive newest-mtime and per-extension tallies for roll-up views.

Its own load-time review (2026-08-22) identifies the floor three architectural facts
impose: the index is amnesiac, so every boot re-walks everything and the second open
costs what the first did; the walk is sequential interpreted Python behind
`asyncio.to_thread`; and reads are O(N) passes over a heap of dataclasses.
On a 241k-file tree that is ~2.2 s before the first row and ~50 s of attached scanning.
Its hypothesis register names the exit: a native parallel walker (H40, “expected: the
cold scan from 50 s to a few seconds”), a columnar index (H41), and pricing the
gitignore matcher separately from the walk (H39). The caps `max_files=500_000` and
`max_depth=20` — and the `truncated` status they force — exist because a Python walker
cannot go further, as
[the interactive-browser research](../../research/research-2026-08-11-interactive-browser-use-case.md)
already observed.

Since that research was written, metabrowser also shipped the shared file-type taxonomy
(its 2026-08-13 plan, in that repository): it hosts the reference
`recommended-file-types.toml`, fdu compiles the same registry, and both serve
`file-type-breakdown-v1`. The classification seam this spec relies on is therefore
already aligned by contract.

### Orientation measurements, this host

Measured while working this spec, on one 120,001-entry generated corpus (`balanced`
recipe: 105,000 files, 15,001 directories), virtualized Linux, warm page cache, no
`.gitignore` patterns present — a regime that flatters the Python side, since its
matcher had nothing to do.
Single trials through the public Python APIs of both projects: orientation numbers for
this integration, not ledger claims, and not comparable across regimes.

| Moment | metabrowser today | fdu Python API |
| --- | ---: | ---: |
| Cold walk + index, no cache | 5.764 s | 0.201 s |
| Revalidate unchanged tree | re-walks: same 5.764 s | 0.106 s |
| Load from snapshot, no filesystem | not possible | 0.090 s |
| Immediate children of root, with roll-ups | — | 0.32 ms |
| Subtree roll-up query | — | 0.06 ms |
| Bounded tree report | — | 1.03 ms |
| Watch delivery, `interval=0.05` | — | 51 ms steady state |
| Summary, selection-filtered | — | 122 ms |

Both engines returned identical byte totals for the tree.
The last row is the design signal: an unfiltered summary reads pre-computed roll-up
state in 0.29 ms, while any selection filter pays a full re-aggregating traversal —
about 1 µs per retained entry.
That tier is priced for occasional filtered reports, not for serving every directory
listing twice per navigation on a millions-of-entries tree.

### What is already settled

- **Tag, don’t prune, and the matcher is cheap enough.** The gitignore spike (bead
  `fdu-p35d`, closed) measured the `ignore` crate’s compiled matcher at 0.39–0.51 µs per
  entry with 100 patterns and 1.50–1.76 µs with 1,000, and recorded the design decision:
  an optional compiled tagging mode, never hidden in the default metadata walk.
  This spec is that mode’s specification.
- **The type-rule dialect is compatible by construction** (bead `fdu-v4lc`, closed): the
  compiled registry is the one metabrowser hosts.
- **Sessions, provenance, lazy open** are specified in progressive results, with the
  session (`fdu-e86o`), its Python mirror (`fdu-a0j0`), attention-following verification
  (`fdu-1mwt`), and lazy warm open (`fdu-hd96`) already tracked there.
- **The integration bead** (`fdu-p02b`) left one question deliberately open — does
  `fdu::watch` replace the client’s watcher outright, or does the client keep its
  watcher and push hints — and this spec resolves it: both, sequenced, on one delta
  contract (see the watch contract below).

## Design

### The seam

| Metabrowser keeps | fdu replaces |
| --- | --- |
| SSE bus, wire models, HTTP surface | boot walker and rewalk paths |
| Plugin discovery and renderers | inventory aggregates and their eviction machinery |
| Active tracker, git history views | navigation tally passes over the index |
| IgnoreMode policy choice (which plane a request reads) | gitignore evaluation and per-entry tagging |
| Watcher *only* on filesystems fdu’s backends cannot serve | watcher and event verification elsewhere |

The client’s `write_token`/generation arbitration disappears rather than being ported:
the index’s clocked delta contract is the same guard, held in one place.

### The contract, requirement by requirement

| Metabrowser needs | fdu today | Verdict |
| --- | --- | --- |
| Boot walk that serves requests ~500 ms in | `open()`/`scan()` block until complete | **Session** (progressive results; shape below) |
| Per-directory recursive tallies, live | `Index.rollup()`, `merge_upward` | Ready |
| Dual all/unignored values on every row | one plane only; selection re-aggregates at ~1 µs/entry | **Gap: partitioned tallies** |
| Hidden-file policy with an allowlist | no visibility concept | **Gap: second tag rule** |
| Children listing with per-child roll-ups | `Index.children()` | Ready |
| Bounded subtree tree with omission accounting | `TreeNode.truncated` is a bare bool | Polish: remainder aggregate |
| Per-extension tallies per directory | `RollUp.by_extension` | Ready |
| Recency queries (top-N by mtime) | `files` view, `sort=mtime` | Ready |
| Per-entry type identity (kind, family, logical ext) | classified internally, not exposed in listings | Polish: expose |
| `file-type-breakdown-v1` envelope | same registry, `types`/`families` views | Adapter in the example |
| Live change feed, verified, coalesced | `Index.watch()` → typed batches | Ready |
| Event-loop (asyncio) consumption | blocking iterator, thread-affine | **Gap: async adapter** |
| Resumable cursor (SSE `Last-Event-ID`) | `since(clock)`, `ChangeSet.truncated` | Ready — document the mapping |
| “Which roll-ups went stale” per batch | client re-derives from paths | **Gap: dirty set** |
| NFS/FUSE watching (polling fallback) | native backends only | **Gap: poll backend** |
| Foreign-watcher hints | `refresh()` is whole-root only in Python | **Gap: scoped refresh** |
| Second open instant | snapshot load + revalidation (0.09 s + 0.11 s at 120k) | Ready now; instant at millions via lazy open (progressive results) |
| Index status for progress UI | `Status` after the fact | Session exposes it mid-walk |
| Client’s own perf loop instrumentation | counters and `PerformanceSummary` are CLI-only | Polish: expose |

### Partitioned tallies

The browser’s default listing shows, for every directory, values *as filtered by
gitignore* next to values for everything.
Serving that from selection costs a re-aggregating traversal per request per plane —
measured at ~1 µs/entry, which is seconds per navigation at home-folder scale.
Metabrowser maintains both aggregates on every mutation instead, and fdu must offer the
same: this is pre-computed roll-up state, the thing the index exists to hold.

**Tags are observations; planes are maintained aggregates.**

- `ScanOptions` (and the corresponding CLI scope axis) gains an explicit, off-by-default
  tag configuration naming rules from a small fixed vocabulary: `gitignore` (the
  `ignore`-crate compiled matcher, correct negation semantics — which retires the whole
  class of the hand-rolled prefixing bug metabrowser’s review calls F3) and `hidden`
  (dot-prefix, with a configurable allowlist so a client policy like “`.logs` and
  `.state` stay visible” is configuration, not a fork).
  Each entry carries its tag bits; tagging happens during the walk at the spike’s
  measured per-entry cost, and the watch layer re-tags on change.
  An observed change to a governing `.gitignore` escalates to subtree invalidation —
  re-tagging a subtree is exactly what `InvalidateSubtree` already expresses.
- For each enabled tag, every directory’s roll-up maintains one additional **plane**:
  the same fields it keeps today (files, dirs, bytes, allocated, newest mtime,
  per-extension tallies) restricted to entries not carrying the tag.
  Planes ride the existing reducer path; they multiply its cost, which is why they are
  opt-in and explicit.
- Tag rules change what stored records mean, so the enabled rule set — allowlist
  included — versions the snapshot fingerprint, exactly as bucketing rules do.
  A snapshot recorded under different rules is absent, never reinterpreted.
- Selection gains one axis value: `plane` (Python `Selection(plane=...)`, CLI
  `--plane`), defaulting to `all`. Any view can then answer “as the browser shows it”
  from pre-computed state, and the two-tier cost rule is preserved: an unfiltered
  single-plane request stays a roll-up read; combining `plane` with other filters
  re-aggregates as any filter does today.
  `RollUp`, `Child`, and `TreeNode` carry per-plane values and per-entry tag bits, so
  one `children()` call serves the dual-value listing.
- The partition rule holds per plane: for every enabled tag, tagged plus untagged equals
  the whole, asserted by test, so no entry can fall out of a tally.

What this deliberately is not: a general predicate store.
A tag is a named, versioned, compiled rule the engine owns end to end — that is what
keeps the cache honest and the cost stated.

### The embedder watch contract

The watch layer’s semantics are already what a live client needs: event-driven
detection, stat-verified samples, coalescing, explicit escalation, and a logical clock.
Four additions make it embeddable rather than merely correct:

1. **An async adapter.** `Watch` stays a thread-affine blocking iterator — that is the
   honest native shape — but the package ships and documents the event-loop handoff (a
   worker thread feeding an `asyncio` queue, yielding the same typed batches), so every
   asyncio consumer does not reinvent it around the GIL. The thread-affinity contract
   gets documented either way.
2. **A per-batch dirty set.** The engine already knows which ancestors `merge_upward`
   touched; the batch says so — the set of paths whose roll-ups changed since the last
   batch. That is precisely the client’s “projection invalidate” signal, computed where
   the knowledge lives instead of re-derived from change paths in Python.
3. **A polling backend.** `WatchOptions` gains an explicit backend choice (native/poll,
   with the poll interval stated), because network and FUSE filesystems drop native
   events silently, and metabrowser already knows this and downgrades.
   Whether the *engine* should probe the mount table and choose automatically is an open
   question below; an explicit knob is the honest first step and is what the client’s
   existing detection can drive.
4. **Scoped refresh.** `Index.refresh(path=...)` exposes the subtree reconciliation the
   engine already performs internally.
   This is the hint-ingestion primitive: a client keeping its own watcher for an exotic
   mount pushes each hint as a scoped refresh, and every mutation still flows through
   the one delta contract.
   This resolves `fdu-p02b`’s open question — `fdu::watch` where its backends serve the
   filesystem, client hints through scoped refresh where they do not, and the acceptance
   test for dropping the Python watcher is the cross-engine fixture in the testing
   strategy.

The resumable-cursor mapping needs no new machinery — `since(clock)` with
`ChangeSet.truncated` already models SSE resume including the “gap too large, resync”
case — so it becomes documentation with a tested example rather than API.

### The session surface

Progressive results owns the design (`Session::start` / read-anytime / cancel;
breadth-first monotone lower bounds; per-path freshness; `prioritize`). Working the
integration adds three requirements to land with it, not after it:

- **Progress is part of the surface**: entries applied so far, roll-up clock, and
  completeness, readable mid-walk, because the client renders a crawl-status UI from
  exactly that today.
- **The async shape ships with the sync one**, same adapter policy as watch.
- **A session hands off to a watch** without a gap: the clock at which the walk
  completed is the clock the watch resumes from, so a client can open, stream the fill,
  then follow changes, with no window where a mutation is neither in the walk nor in the
  feed.

### Classification identity in listings

`children()` and `files`-view rows gain the compiled registry’s verdict — type id,
family, and logical extension — as plain metadata fields.
The registry already runs during classification for the type views; exposing its
per-entry identity costs display only, reads no content, and is what lets the client
drop its own classifier while keeping its wire models.
The registry’s identity (schema version, revision, fingerprint) becomes readable from
Python, since the client stamps exactly that into its envelopes today.

### Instrumentation for the client’s own loop

Metabrowser runs its own measured performance loop.
`Report` and session/watch surfaces expose the walk telemetry the CLI footer already
computes — files and bytes walked, cache tier, fresh versus cached analysis — as typed
values (the envelope keeps excluding them; they are execution telemetry, delivered
beside the report, not query data).
The `FDU_COUNTERS` mechanism stays an environment concern and is documented for Python
consumers rather than wrapped.

### Polish

- `TreeNode` bounds state a remainder: when children are omitted, the node carries the
  aggregate of what was dropped (dirs, files, bytes per plane), machine-readable —
  “truncate freely, never silently” applied to the one place it is still a bare bool.
  A treemap’s “other” cell is this value.
- Document that `children()` on a pathologically wide directory returns everything, and
  that the bounded alternative is the tree view; whether `children()` needs its own
  bound is an open question.
- `WatchOptions.interval` default (2.0 s) is tuned for terminal repaints; embedder
  documentation states that live UIs set it near their frame budget (measured: 51 ms
  end-to-end at `interval=0.05`).

## Implementation Plan

Tracked under epic `fdu-u7vo`; each item names its bead.

### Phase 1: Partitioned tallies

- [ ] Tag rules in the engine: compiled gitignore and hidden-with-allowlist matchers,
  entry tag bits, opt-in `ScanOptions` configuration, snapshot fingerprint coverage;
  planes through the reducer path — per-plane roll-up state, `merge_upward`, refresh and
  watch re-tagging, `.gitignore`-edit escalation; partition-sum property tests and
  fingerprint invalidation (`fdu-mvt3`)
- [ ] Surfaces: `--tags`/`--plane` on the CLI, `Selection.plane` and per-plane
  `RollUp`/`Child` values in Python, tagged-fixture goldens in every format, parity
  rows, and plane-equals-all equivalence when no entry is tagged (`fdu-7rwf`)

### Phase 2: The embedder watch contract

- [ ] Per-batch dirty roll-up set, engine through Python (`fdu-mz1a`)
- [ ] `Index.refresh(path=...)` scoped reconciliation in the Python surface (`fdu-fh0k`)
- [ ] Polling backend selection in `WatchOptions`, with its interval stated (`fdu-rhu3`)
- [ ] The asyncio adapter and the thread-affinity documentation, with a tested
  SSE-resume example mapping `since`/`truncated` to `Last-Event-ID`/resync (`fdu-97pb`)

### Phase 3: Session integration shape

(after the progressive-results session beads land the core; `fdu-4o0m`, blocked by
`fdu-e86o` and `fdu-a0j0`)

- [ ] Mid-walk progress surface: entries applied, clock, completeness
- [ ] Async session adapter, same policy as watch
- [ ] Session-to-watch clock handoff, tested for the no-gap property

### Phase 4: Adoption proof

- [ ] Classification identity in `children()` and files rows; registry identity readable
  from Python (`fdu-16l7`)
- [ ] Walk telemetry as typed values beside report/session/watch results (`fdu-tib6`)
- [ ] `TreeNode` remainder aggregates (`fdu-knyw`)
- [ ] Reference embedder example under `crates/fdu-py/examples/` — boot, serve dual
  tallies, stream changes with dirty sets, resume from a cursor — plus the cross-engine
  agreement fixture comparing fdu against the metabrowser walker’s semantics (symlinks
  as leaves, hidden allowlist, gitignore negations), differences documented or
  eliminated (`fdu-vfyw`)

## Testing Strategy

Partition sums are the load-bearing property: for every enabled tag, plane plus
complement equals the untagged totals, by property test across scan, refresh, and watch
mutations. Golden sessions add a tagged fixture exercising `--tags`/`--plane` in every
format, and the parity harness replays them against Python as it does every axis.
Watch additions get the same treatment the layer already has — dirty sets asserted
against independently computed ancestor sets; scoped refresh asserted equivalent to full
refresh on the touched subtree; the async adapter driven by the existing watch tests
through the event loop.
The reference embedder is executed in CI like the other examples, and the cross-engine
fixture is the acceptance test `fdu-p02b` asked for.
Performance claims follow the loop: browser-moment jobs (cold with tags, warm unchanged,
plane query, watch steady state) join the benchmark manifests, and any constant tuned
here records its regime.

## Open Questions

- Are two maintained planes (all, unignored) the shipped set, or does the client’s
  show-all mode justify a third (visible)?
  Per-plane cost is real; measure the reducer overhead before enabling more than one
  extra plane by default in the tagged mode.
- Should the engine probe mount tables to choose the watch backend, or stay explicit and
  let clients own detection?
  Explicit ships first; probing is additive.
- Does `children()` need its own bound, or is the tree view’s remainder enough for wide
  directories?
- Free-threaded CPython (3.14t) wheels: metabrowser’s H48 expects real parallelism in
  Python; fdu’s abi3 wheel does not cover the t-ABI today.
  Out of scope here, tracked as a release-matrix question.

## References

- [Design principles](../architecture/fdu-design-principles.md) — the axis rules, cache
  honesty, and truncation contracts this spec extends
- [Surface architecture](../architecture/fdu-surface-architecture.md) — the parity
  harness every addition here must clear
- [Interactive browser research](../../research/research-2026-08-11-interactive-browser-use-case.md)
  — the three deadlines and the four-consumer comparison
- [Progressive results plan](plan-2026-08-11-fdu-progressive-results.md) — sessions,
  provenance, lazy open
- [FSEvents-scoped revalidation plan](plan-2026-08-10-fdu-fsevents-scoped-revalidation.md)
  — the convergence half for warm opens
- Beads: `fdu-p02b` (the integration), `fdu-p35d` (the gitignore spike verdict),
  `fdu-v4lc` (the shared type-rule dialect), `fdu-e86o`/`fdu-a0j0`/`fdu-1mwt` (the
  session), `fdu-hd96` (lazy open priority)
- Metabrowser: its load-time performance review (2026-08-22) and hypothesis register
  (H39–H41, H47–H48), and its shared file-type taxonomy plan (2026-08-13), in that
  repository

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
